use crate::mir::optimizer::MirOptimizer;
use crate::mir::optimizer_stats::OptimizationStats;
use crate::mir::{BarrierOp, MirModule, TypeOpKind, ValueId, WeakRefOp};

// PluginInvoke retired: no-op stubs removed

pub fn normalize_legacy_instructions(
    _opt: &mut MirOptimizer,
    module: &mut MirModule,
) -> OptimizationStats {
    use crate::mir::MirInstruction as I;
    let mut stats = OptimizationStats::new();
    let rw_dbg = crate::config::env::rewrite_debug();
    let rw_sp = crate::config::env::rewrite_safepoint();
    let rw_future = crate::config::env::rewrite_future();
    // ArrayGet/Set → BoxCall を常時ON（レガシー撤退のため）
    let _array_to_boxcall = true; // currently unused in this pass (kept for future slices)

    // Env gate for callable lowering (dev-opt): HAKO_CALLABLE_LOWERING=1
    let callable_lowering = std::env::var("HAKO_CALLABLE_LOWERING").ok().map(|v| v=="1" || v=="true" || v=="on").unwrap_or(false);
    // Helper: central whitelist (箱化). Keep a single source of truth for
    // safe method names per core box to avoid drift between passes.
    #[inline]
    fn is_safe_core_method(box_name: &str, method: &str) -> bool {
        match box_name {
            // Array — pure or well-understood ops only
            "ArrayBox" => matches!(
                method,
                "size" | "len" | "length" | "get" | "set" | "push" | "slice" | "join" | "contains" | "indexOf"
            ),
            // Map — core CRUD + keys/values
            "MapBox" => matches!(method, "size" | "len" | "has" | "get" | "set" | "delete" | "clear" | "keys" | "values"),
            // String — byte-semantics only（index/substring/char等）
            "StringBox" => matches!(method, "size" | "len" | "length" | "indexOf" | "lastIndexOf" | "substring" | "charAt" | "concat"),
            _ => false,
        }
    }

    for (_fname, function) in &mut module.functions {
        use std::collections::HashMap;
        // Precompute definition map, constant maps, and callable origins
        let mut def_map: HashMap<ValueId, (crate::mir::basic_block::BasicBlockId, usize)> = HashMap::new();
        let mut const_str: HashMap<ValueId, String> = HashMap::new();
        let mut const_int: HashMap<ValueId, i64> = HashMap::new();
        for (bb_id, block) in &function.blocks {
            for (i, inst) in block.instructions.iter().enumerate() {
                match inst {
                    I::Const { dst, value } => {
                        def_map.insert(*dst, (*bb_id, i));
                        if let crate::mir::ConstValue::String(s) = value { const_str.insert(*dst, s.clone()); }
                        if let crate::mir::ConstValue::Integer(v) = value { const_int.insert(*dst, *v); }
                    },
                    I::Call { dst, .. } => { if let Some(d)=dst { def_map.insert(*d, (*bb_id, i)); } },
                    I::BoxCall { dst, .. } => { if let Some(d)=dst { def_map.insert(*d, (*bb_id, i)); } },
                    I::TypeOp { dst, .. } => { def_map.insert(*dst, (*bb_id, i)); },
                    _ => {}
                }
            }
        }
        // Map: callable ValueId -> (box_name, recv, name_id, arity_id)
        let mut callable_info: HashMap<ValueId, (String, ValueId, ValueId, ValueId)> = HashMap::new();
        for (_bb, block) in &function.blocks {
            for inst in &block.instructions {
                match inst {
                    // methodRef captured via Method callee (whitelisted builtin boxes)
                    I::Call { dst: Some(d), callee: Some(crate::mir::definitions::call_unified::Callee::Method { box_name, method, receiver, .. }), args, .. } => {
                        if method=="methodRef" && args.len()==2 {
                            if let Some(recv)=receiver {
                                if box_name=="ArrayBox" || box_name=="MapBox" || box_name=="StringBox" {
                                    callable_info.insert(*d, (box_name.clone(), *recv, args[0], args[1]));
                                }
                            }
                        }
                    }
                    // methodRef constructed via ModuleFunction (BoxName.methodRef/2) — capture receiver from args[0]
                    I::Call { dst: Some(d), callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(name)), args, .. } => {
                        if args.len()==3 {
                            if name=="ArrayBox.methodRef/2" {
                                callable_info.insert(*d, ("ArrayBox".to_string(), args[0], args[1], args[2]));
                            } else if name=="MapBox.methodRef/2" {
                                callable_info.insert(*d, ("MapBox".to_string(), args[0], args[1], args[2]));
                            } else if name=="StringBox.methodRef/2" {
                                callable_info.insert(*d, ("StringBox".to_string(), args[0], args[1], args[2]));
                            }
                        }
                    }
                    I::BoxCall { dst: Some(d), box_val: recv, method, args, .. } => {
                        // Legacy BoxCall-based methodRef: keep ArrayBox-only (type unknown here)
                        if method=="methodRef" && args.len()==2 {
                            callable_info.insert(*d, ("ArrayBox".to_string(), *recv, args[0], args[1]));
                        }
                    }
                    _ => {}
                }
            }
        }


        for (bb_id, block) in &mut function.blocks {
            for inst in &mut block.instructions {
                match inst {
                    I::WeakNew { dst, box_val } => {
                        let d = *dst;
                        let v = *box_val;
                        *inst = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::New,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakLoad { dst, weak_ref } => {
                        let d = *dst;
                        let v = *weak_ref;
                        *inst = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::Load,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierRead { ptr } => {
                        let p = *ptr;
                        *inst = I::Barrier {
                            op: BarrierOp::Read,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierWrite { ptr } => {
                        let p = *ptr;
                        *inst = I::Barrier {
                            op: BarrierOp::Write,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Print { value, .. } => {
                        let v = *value;
                        *inst = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.console.log".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    
                    
                    I::Debug { .. } if !rw_dbg => {
                        *inst = I::Nop;
                    }
                    I::Safepoint if !rw_sp => {
                        *inst = I::Nop;
                    }
                    I::FutureNew { dst, value } if rw_future => {
                        let d = *dst;
                        let v = *value;
                        *inst = I::Call {
                            dst: Some(d),
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.new".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::FutureSet { future, value } if rw_future => {
                        let f = *future;
                        let v = *value;
                        *inst = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.set".to_string(),
                            )),
                            args: vec![f, v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    I::Await { dst, future } if rw_future => {
                        let d = *dst;
                        let f = *future;
                        *inst = I::Call {
                            dst: Some(d),
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.future.await".to_string(),
                            )),
                            args: vec![f],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                    }
                    _ => {}
                }
            }
            // ModuleFunction("<Box>.<method>/<arity>") → Method(box=<Box>, method, receiver=args[0])
            // Safe whitelist: ArrayBox, MapBox, StringBox の代表APIのみ
            for inst in &mut block.instructions {
                if let I::Call { dst, func: _, callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(name)), args, effects } = inst {
                    // Parse name like "MapBox.get/1"
                    let (box_name, method, arity_ok) = (|| {
                        if let Some((bn, rest)) = name.split_once('.') {
                            if let Some((m, ar)) = rest.split_once('/') {
                                // accept digits only
                                if ar.bytes().all(|b| b.is_ascii_digit()) { return (bn.to_string(), m.to_string(), true); }
                            }
                        }
                        (String::new(), String::new(), false)
                    })();
                    if !arity_ok { continue; }
                    // require at least 1 arg to extract receiver
                    if args.is_empty() { continue; }
                    // Whitelist of safe methods by box（箱化された判定に委譲）
                    let ok = is_safe_core_method(&box_name, method.as_str());
                    if !ok { continue; }
                    // Rewrite to Callee::Method and drop receiver from args
                    let recv = args[0];
                    let new_args: Vec<ValueId> = args.iter().cloned().skip(1).collect();
                    *inst = I::Call {
                        dst: *dst,
                        func: ValueId::new(0),
                        callee: Some(crate::mir::definitions::call_unified::Callee::Method {
                            box_name: box_name.clone(),
                            method: method.clone(),
                            receiver: Some(recv),
                            certainty: crate::mir::definitions::call_unified::TypeCertainty::Known,
                        }),
                        args: new_args,
                        effects: *effects,
                    };
                    stats.intrinsic_optimizations += 1;
                }
            }
            // Callable lowering (methodRef.call → direct) for arity==0 only
            if callable_lowering {
                for inst in &mut block.instructions {
                    if let I::BoxCall { dst, box_val, method, method_id: _, args, effects } = inst {
                        if method == "call" && args.len() == 1 {
                            if let Some((box_nm, recv, name_id, arity_id)) = callable_info.get(box_val).cloned() {
                                if let (Some(mname), Some(0)) = (const_str.get(&name_id).cloned(), const_int.get(&arity_id)) {
                                    *inst = I::Call { dst: *dst, func: ValueId::new(0), callee: Some(crate::mir::definitions::call_unified::Callee::Method { box_name: box_nm, method: mname, receiver: Some(recv), certainty: crate::mir::definitions::call_unified::TypeCertainty::Known }), args: vec![], effects: *effects };
                                    stats.intrinsic_optimizations += 1;
                                }
                            }
                        }
                    }
                }
            }

            // Callable lowering (arity>0) using static argv reconstruction from local ArrayBox push chain
            // Safe, conservative: only when argv is a locally constructed ArrayBox in the same block
            if callable_lowering {
                // enumerate with index to access defs and scan window
                for (idx, insn) in block.instructions.clone().into_iter().enumerate() {
                    if let I::BoxCall { dst, box_val, method, args, effects, .. } = insn.clone() {
                        if method == "call" && args.len() == 1 {
                            if let Some((box_nm, recv, name_id, arity_id)) = callable_info.get(&box_val).cloned() {
                                let mname_opt = const_str.get(&name_id).cloned();
                                let arity_opt = const_int.get(&arity_id).cloned();
                                if let (Some(mname), Some(n)) = (mname_opt, arity_opt) {
                                    if n > 0 {
                                        let argv_arr = args[0];
                                        if let Some((def_bb, def_i)) = def_map.get(&argv_arr).cloned() {
                                            if def_bb == *bb_id && def_i < idx {
                                                // Scan from def_i to idx and reconstruct push arguments
                                                let mut elems: Vec<ValueId> = Vec::new();
                                                let mut safe = true;
                                                for j in def_i..idx {
                                                    match &block.instructions[j] {
                                                        I::NewBox { dst: arr_dst, box_type, .. } => {
                                                            if *arr_dst == argv_arr {
                                                                // require ArrayBox constructor
                                                                if box_type != "ArrayBox" { safe = false; break; }
                                                            }
                                                        }
                                                        I::BoxCall { box_val: arr, method: m, args: a, .. } => {
                                                            if *arr == argv_arr {
                                                                if m == "push" && a.len() == 1 {
                                                                    elems.push(a[0]);
                                                                } else {
                                                                    safe = false; break;
                                                                }
                                                            }
                                                        }
                                                        _ => { /* ignore other ops conservatively */ }
                                                    }
                                                }
                                                if safe && (elems.len() as i64) == n {
                                                    // Lower to direct call with reconstructed argv
                                                    block.instructions[idx] = I::Call {
                                                        dst,
                                                        func: ValueId::new(0),
                                                        callee: Some(crate::mir::definitions::call_unified::Callee::Method {
                                                            box_name: box_nm,
                                                            method: mname,
                                                            receiver: Some(recv),
                                                            certainty: crate::mir::definitions::call_unified::TypeCertainty::Known,
                                                        }),
                                                        args: elems,
                                                        effects,
                                                    };
                                                    stats.intrinsic_optimizations += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Pass A: Extern("nyrt.*") → Method for core boxes
            // Safe subset only: string/array (size/len + common string ops) / map (size/keys/values)
            for inst in &mut block.instructions {
                if let I::Call { dst, func: _, callee: Some(crate::mir::definitions::call_unified::Callee::Extern(name)), args, effects } = inst {
                    let (box_name, method_opt) = match name.as_str() {
                        "nyrt.string.length" => ("StringBox", Some("size")),
                        "nyrt.string.indexOf" => ("StringBox", Some("indexOf")),
                        "nyrt.string.lastIndexOf" => ("StringBox", Some("lastIndexOf")),
                        "nyrt.string.substring" => ("StringBox", Some("substring")),
                        "nyrt.string.charAt" => ("StringBox", Some("charAt")),
                        "nyrt.string.replace" => ("StringBox", Some("replace")),
                        "nyrt.array.size"    => ("ArrayBox",  Some("size")),
                        "nyrt.map.size"      => ("MapBox",    Some("size")),
                        "nyrt.map.keys"      => ("MapBox",    Some("keys")),
                        "nyrt.map.values"    => ("MapBox",    Some("values")),
                        _ => ("", None),
                    };
                    if let Some(m) = method_opt {
                        if !args.is_empty() {
                            let recv = args[0];
                            let new_args: Vec<ValueId> = args.iter().cloned().skip(1).collect();
                            *inst = I::Call {
                                dst: *dst,
                                func: ValueId::new(0),
                                callee: Some(crate::mir::definitions::call_unified::Callee::Method {
                                    box_name: box_name.to_string(),
                                    method: m.to_string(),
                                    receiver: Some(recv),
                                    certainty: crate::mir::definitions::call_unified::TypeCertainty::Known,
                                }),
                                args: new_args,
                                effects: *effects,
                            };
                            stats.intrinsic_optimizations += 1;
                        }
                    }
                }
            }

            // Pass B: BoxCall → Method when receiver is a local NewBox(Array/Map/String) in the same block
            // Safe whitelist per box to avoid unintended semantics
            for (idx, insn) in block.instructions.clone().into_iter().enumerate() {
                if let I::BoxCall { dst, box_val, method, method_id: _mid, args, effects } = insn.clone() {
                    // Check receiver def site
                    if let Some((def_bb, def_i)) = def_map.get(&box_val).cloned() {
                        if def_bb == *bb_id && def_i < idx {
                            // Receiver must be a recent NewBox of a whitelisted core box
                            let mut recv_box_name: Option<&'static str> = None;
                            if let I::NewBox { dst: rdst, box_type, .. } = &block.instructions[def_i] {
                                if *rdst == box_val {
                                    match box_type.as_str() {
                                        "ArrayBox" | "MapBox" | "StringBox" => recv_box_name = Some(Box::leak(box_type.clone().into_boxed_str())),
                                        _ => {}
                                    }
                                }
                            }
                            if let Some(bx) = recv_box_name {
                                let ok = is_safe_core_method(bx, method.as_str());
                                if ok {
                                    block.instructions[idx] = I::Call {
                                        dst,
                                        func: ValueId::new(0),
                                        callee: Some(crate::mir::definitions::call_unified::Callee::Method {
                                            box_name: bx.to_string(),
                                            method: method.clone(),
                                            receiver: Some(box_val),
                                            certainty: crate::mir::definitions::call_unified::TypeCertainty::Known,
                                        }),
                                        args,
                                        effects,
                                    };
                                    stats.intrinsic_optimizations += 1;
                                }
                            }
                        }
                    }
                }
            }

                        // terminator rewrite (subset migrated as needed)
            if let Some(term) = &mut block.terminator {
                match term {
                    I::TypeCheck {
                        dst,
                        value,
                        expected_type,
                    } => {
                        let ty = crate::mir::MirType::Box(expected_type.clone());
                        *term = I::TypeOp {
                            dst: *dst,
                            op: TypeOpKind::Check,
                            value: *value,
                            ty,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Cast {
                        dst,
                        value,
                        target_type,
                    } => {
                        let ty = target_type.clone();
                        *term = I::TypeOp {
                            dst: *dst,
                            op: TypeOpKind::Cast,
                            value: *value,
                            ty,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakNew { dst, box_val } => {
                        let d = *dst;
                        let v = *box_val;
                        *term = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::New,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::WeakLoad { dst, weak_ref } => {
                        let d = *dst;
                        let v = *weak_ref;
                        *term = I::WeakRef {
                            dst: d,
                            op: WeakRefOp::Load,
                            value: v,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierRead { ptr } => {
                        let p = *ptr;
                        *term = I::Barrier {
                            op: BarrierOp::Read,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::BarrierWrite { ptr } => {
                        let p = *ptr;
                        *term = I::Barrier {
                            op: BarrierOp::Write,
                            ptr: p,
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    I::Print { value, .. } => {
                        let v = *value;
                        *term = I::Call {
                            dst: None,
                            func: ValueId::new(0),
                            callee: Some(crate::mir::definitions::call_unified::Callee::Extern(
                                "env.console.log".to_string(),
                            )),
                            args: vec![v],
                            effects: crate::mir::EffectMask::PURE.add(crate::mir::Effect::Io),
                        };
                        stats.intrinsic_optimizations += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    stats
}

pub fn normalize_ref_field_access(_opt: &mut MirOptimizer, _module: &mut MirModule) -> OptimizationStats {
    // RefGet/RefSet retired: no-op
    OptimizationStats::new()
}
