use serde_json::json;
use crate::mir::definitions::call_unified::Callee;

/// Emit MIR JSON for Python harness/PyVM.
/// The JSON schema matches tools/llvmlite_harness.py expectations and is
/// intentionally minimal for initial scaffolding.
///
/// Phase 15.5: Supports both v0 (legacy separate ops) and v1 (unified mir_call) formats

/// Helper: Create JSON v1 root with schema information
/// Includes version, capabilities, metadata for advanced MIR features
fn create_json_v1_root(functions: serde_json::Value) -> serde_json::Value {
    json!({
        "schema_version": "1.0",
        "capabilities": [
            "unified_call",      // Phase 15.5: Unified MirCall support
            "phi",              // SSA Phi functions
            "effects",          // Effect tracking for optimization
            "callee_typing"     // Type-safe call target resolution
        ],
        "metadata": {
            "generator": "nyash-rust",
            "phase": "15.5",
            "build_time": "Phase 15.5 Development",
            "features": ["mir_call_unification", "json_v1_schema"]
        },
        "functions": functions
    })
}

/// Helper: Emit unified mir_call JSON (v1 format)
/// Supports all 6 Callee types in a single unified JSON structure
fn emit_unified_mir_call(
    dst: Option<u32>,
    callee: &Callee,
    args: &[u32],
    effects: &[&str],
) -> serde_json::Value {
    let mut call_obj = json!({
        "op": "mir_call",
        "dst": dst,
        "mir_call": {
            "args": args,
            "effects": effects,
            "flags": {}
        }
    });

    // Generate Callee-specific mir_call structure
    match callee {
        Callee::Global(name) => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "Global",
                "name": name
            });
        }
        Callee::ModuleFunction(name) => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "ModuleFunction",
                "name": name
            });
        }
        Callee::Method { box_name, method, receiver, certainty } => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "Method",
                "box_name": box_name,
                "method": method,
                "receiver": receiver.map(|v| v.as_u32()),
                "certainty": match certainty { crate::mir::definitions::call_unified::TypeCertainty::Known => "Known", crate::mir::definitions::call_unified::TypeCertainty::Union => "Union" }
            });
        }
        Callee::Constructor { box_type } => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "Constructor",
                "box_type": box_type
            });
        }
        Callee::Closure { params, captures, me_capture } => {
            let captures_json: Vec<_> = captures.iter()
                .map(|(name, vid)| json!([name, vid.as_u32()]))
                .collect();
            call_obj["mir_call"]["callee"] = json!({
                "type": "Closure",
                "params": params,
                "captures": captures_json,
                "me_capture": me_capture.map(|v| v.as_u32())
            });
        }
        Callee::Value(vid) => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "Value",
                "function_value": vid.as_u32()
            });
        }
        Callee::Extern(name) => {
            call_obj["mir_call"]["callee"] = json!({
                "type": "Extern",
                "name": name
            });
        }
    }

    call_obj
}

pub fn emit_mir_json_for_harness(
    module: &nyash_rust::mir::MirModule,
    path: &std::path::Path,
) -> Result<(), String> {
    use nyash_rust::mir::{BinaryOp as B, CompareOp as C, MirInstruction as I, MirType};
    let mut funs = Vec::new();
    for (name, f) in &module.functions {
        let mut blocks = Vec::new();
        let mut ids: Vec<_> = f.blocks.keys().copied().collect();
        ids.sort();
        for bid in ids {
            if let Some(bb) = f.blocks.get(&bid) {
                let mut insts = Vec::new();
                // Pre-scan: collect values defined anywhere in this block (to delay use-before-def copies)
                let mut block_defines: std::collections::HashSet<u32> = std::collections::HashSet::new();
                for inst in &bb.instructions {
                    match inst {
                        | I::UnaryOp { dst, .. }
                        | I::Const { dst, .. }
                        | I::BinOp { dst, .. }
                        | I::Compare { dst, .. }
                        | I::Call { dst: Some(dst), .. }
                        | I::ExternCall { dst: Some(dst), .. }
                        | I::BoxCall { dst: Some(dst), .. }
                        | I::NewBox { dst, .. }
                        | I::Phi { dst, .. } => {
                            block_defines.insert(dst.as_u32());
                        }
                        _ => {}
                    }
                }
                // Track which values have been emitted (to order copies after their sources)
                let mut emitted_defs: std::collections::HashSet<u32> = std::collections::HashSet::new();
                // PHI first（オプション）
                for inst in &bb.instructions {
                    if let I::Copy { dst, src } = inst {
                        // For copies whose source will be defined later in this block, delay emission
                        let s = src.as_u32();
                        if block_defines.contains(&s) && !emitted_defs.contains(&s) {
                            // delayed; will be emitted after non-PHI pass
                        } else {
                            insts.push(json!({"op":"copy","dst": dst.as_u32(), "src": src.as_u32()}));
                            emitted_defs.insert(dst.as_u32());
                        }
                        continue;
                    }
                    if let I::Phi { dst, inputs } = inst {
                        let values_objs: Vec<_> = inputs
                            .iter()
                            .map(|(b, v)| json!({"value": v.as_u32(), "block": b.as_u32()}))
                            .collect();
                        // dst_type hint: if all incoming values are String-ish, annotate result as String handle
                        let all_str =
                            inputs
                                .iter()
                                .all(|(_b, v)| match f.metadata.value_types.get(v) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                });
                        if all_str {
                            insts.push(json!({
                                "op":"phi","dst": dst.as_u32(), "values": values_objs,
                                "dst_type": {"kind":"handle","box_type":"StringBox"}
                            }));
                        } else {
                            insts.push(
                                json!({"op":"phi","dst": dst.as_u32(), "values": values_objs}),
                            );
                        }
                    }
                }
                // Non-PHI
                // Non-PHI
                let mut delayed_copies: Vec<(u32, u32)> = Vec::new();
                for inst in &bb.instructions {
                    match inst {
                        I::Copy { dst, src } => {
                            let d = dst.as_u32();
                            let s = src.as_u32();
                            if block_defines.contains(&s) && !emitted_defs.contains(&s) {
                                delayed_copies.push((d, s));
                            } else {
                                insts.push(json!({"op":"copy","dst": d, "src": s}));
                                emitted_defs.insert(d);
                            }
                        }
                        I::UnaryOp { dst, op, operand } => {
                            let kind = match op {
                                nyash_rust::mir::UnaryOp::Neg => "neg",
                                nyash_rust::mir::UnaryOp::Not => "not",
                                nyash_rust::mir::UnaryOp::BitNot => "bitnot",
                            };
                            insts.push(json!({"op":"unop","kind": kind, "src": operand.as_u32(), "dst": dst.as_u32()}));
                        }
                        I::Const { dst, value } => {
                            match value {
                                nyash_rust::mir::ConstValue::Integer(i) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "i64", "value": i}}));
                                }
                                nyash_rust::mir::ConstValue::Float(fv) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "f64", "value": fv}}));
                                }
                                nyash_rust::mir::ConstValue::Bool(b) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "i64", "value": if *b {1} else {0}}}));
                                }
                                nyash_rust::mir::ConstValue::String(s) => {
                                    // String constants are exported as StringBox handle by default
                                    insts.push(json!({
                                        "op":"const",
                                        "dst": dst.as_u32(),
                                        "value": {
                                            "type": {"kind":"handle","box_type":"StringBox"},
                                            "value": s
                                        }
                                    }));
                                }
                                nyash_rust::mir::ConstValue::Null
                                | nyash_rust::mir::ConstValue::Void => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "void", "value": 0}}));
                                }
                            }
                        }
                        I::TypeOp { dst, op, value, ty } => {
                            let op_s = match op {
                                nyash_rust::mir::TypeOpKind::Check => "check",
                                nyash_rust::mir::TypeOpKind::Cast => "cast",
                            };
                            let ty_s = match ty {
                                MirType::Integer => "Integer".to_string(),
                                MirType::Float => "Float".to_string(),
                                MirType::Bool => "Bool".to_string(),
                                MirType::String => "String".to_string(),
                                MirType::Void => "Void".to_string(),
                                MirType::Box(name) => name.clone(),
                                _ => "Unknown".to_string(),
                            };
                            insts.push(json!({
                                "op":"typeop",
                                "operation": op_s,
                                "src": value.as_u32(),
                                "dst": dst.as_u32(),
                                "target_type": ty_s,
                            }));
                            emitted_defs.insert(dst.as_u32());
                        }
                        I::BinOp { dst, op, lhs, rhs } => {
                            let op_s = match op {
                                B::Add => "+",
                                B::Sub => "-",
                                B::Mul => "*",
                                B::Div => "/",
                                B::Mod => "%",
                                B::BitAnd => "&",
                                B::BitOr => "|",
                                B::BitXor => "^",
                                B::Shl => "<<",
                                B::Shr => ">>",
                                B::And => "&",
                                B::Or => "|",
                            };
                            let mut obj = json!({"op":"binop","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()});
                            // dst_type hint for string concatenation: if either side is String-ish and op is '+', mark result as String handle
                            if matches!(op, B::Add) {
                                let lhs_is_str = match f.metadata.value_types.get(lhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                let rhs_is_str = match f.metadata.value_types.get(rhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                if lhs_is_str || rhs_is_str {
                                    obj["dst_type"] =
                                        json!({"kind":"handle","box_type":"StringBox"});
                                }
                            }
                            insts.push(obj);
                        }
                        I::Compare { dst, op, lhs, rhs } => {
                            let op_s = match op {
                                C::Lt => "<",
                                C::Le => "<=",
                                C::Gt => ">",
                                C::Ge => ">=",
                                C::Eq => "==",
                                C::Ne => "!=",
                            };
                            let mut obj = json!({"op":"compare","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()});
                            // cmp_kind hint for string equality
                            if matches!(op, C::Eq | C::Ne) {
                                let lhs_is_str = match f.metadata.value_types.get(lhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                let rhs_is_str = match f.metadata.value_types.get(rhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                if lhs_is_str && rhs_is_str {
                                    obj["cmp_kind"] = json!("string");
                                }
                            }
                            insts.push(obj);
                        }
                        I::Call {
                            dst, func, callee, args, effects, ..
                        } => {
                            // Phase 15.5: Unified Call support with environment variable control
                            let use_unified = match std::env::var("NYASH_MIR_UNIFIED_CALL").ok().as_deref().map(|s| s.to_ascii_lowercase()) {
                                Some(s) if s == "0" || s == "false" || s == "off" => false,
                                _ => true,
                            };

                            if use_unified && callee.is_some() {
                                // v1: Unified mir_call format
                                let effects_str: Vec<&str> = if effects.is_io() { vec!["IO"] } else { vec![] };
                                let args_u32: Vec<u32> = args.iter().map(|v| v.as_u32()).collect();
                                let unified_call = emit_unified_mir_call(
                                    dst.map(|v| v.as_u32()),
                                    callee.as_ref().unwrap(),
                                    &args_u32,
                                    &effects_str,
                                );
                                insts.push(unified_call);
                            } else {
                                // v0: Legacy call format (fallback)
                                let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                                insts.push(json!({"op":"call","func": func.as_u32(), "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                            }
                        }
                        I::ExternCall {
                            dst,
                            iface_name,
                            method_name,
                            args,
                            ..
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            let func_name = if iface_name == "env.console" {
                                format!("nyash.console.{}", method_name)
                            } else {
                                format!("{}.{}", iface_name, method_name)
                            };
                            let mut obj = json!({
                                "op": "externcall",
                                "func": func_name,
                                "args": args_a,
                                "dst": dst.map(|d| d.as_u32()),
                            });
                            // Minimal dst_type hints for known externs
                            if iface_name == "env.console" {
                                // console.* returns i64 status (ignored by user code)
                                if dst.is_some() {
                                    obj["dst_type"] = json!("i64");
                                }
                            }
                            insts.push(obj);
                        }
                        I::BoxCall {
                            dst,
                            box_val,
                            method,
                            args,
                            ..
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            // Minimal dst_type hints
                            let mut obj = json!({
                                "op":"boxcall","box": box_val.as_u32(), "method": method, "args": args_a, "dst": dst.map(|d| d.as_u32())
                            });
                            let m = method.as_str();
                            let dst_ty = if m == "substring"
                                || m == "dirname"
                                || m == "join"
                                || m == "read_all"
                                || m == "read"
                            {
                                Some(json!({"kind":"handle","box_type":"StringBox"}))
                            } else if m == "length" || m == "lastIndexOf" {
                                Some(json!("i64"))
                            } else {
                                None
                            };
                            if let Some(t) = dst_ty {
                                obj["dst_type"] = t;
                            }
                            insts.push(obj);
                            if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                        }
                        I::NewBox {
                            dst,
                            box_type,
                            args,
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"newbox","type": box_type, "args": args_a, "dst": dst.as_u32()}));
                            emitted_defs.insert(dst.as_u32());
                        }
                        I::Branch {
                            condition,
                            then_bb,
                            else_bb,
                        } => {
                            insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()}));
                        }
                        I::Jump { target } => {
                            insts.push(json!({"op":"jump","target": target.as_u32()}));
                        }
                        I::Return { value } => {
                            insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())}));
                        }
                        _ => { /* skip non-essential ops for initial harness */ }
                    }
                }
                // Emit delayed copies now (sources should be available)
                for (d, s) in delayed_copies {
                    insts.push(json!({"op":"copy","dst": d, "src": s}));
                }
                if let Some(term) = &bb.terminator {
                    match term {
                        I::Return { value } => insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())})),
                        I::Jump { target } => insts.push(json!({"op":"jump","target": target.as_u32()})),
                        I::Branch { condition, then_bb, else_bb } => insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()})),
                        _ => {}
                    }
                }
                blocks.push(json!({"id": bid.as_u32(), "instructions": insts}));
            }
        }
        // Export parameter value-ids so a VM can bind arguments
        let params: Vec<_> = f.params.iter().map(|v| v.as_u32()).collect();
        funs.push(json!({"name": name, "params": params, "blocks": blocks}));
    }

    // Phase 15.5: JSON v1 schema with environment variable control
    let force_v0 = std::env::var("NYASH_JSON_SCHEMA_V0").ok().as_deref() == Some("1")
        || std::env::var("NYASH_LLVM_DOWNGRADE_V1").ok().as_deref() == Some("1");
    let use_v1_schema = if force_v0 {
        false
    } else {
        std::env::var("NYASH_JSON_SCHEMA_V1").unwrap_or_default() == "1"
            || match std::env::var("NYASH_MIR_UNIFIED_CALL").ok().as_deref().map(|s| s.to_ascii_lowercase()) {
                Some(s) if s == "0" || s == "false" || s == "off" => false,
                _ => true,
            }
    };

    let root = if use_v1_schema {
        create_json_v1_root(json!(funs))
    } else {
        json!({"functions": funs})  // v0 legacy format
    };

    std::fs::write(path, serde_json::to_string_pretty(&root).unwrap())
        .map_err(|e| format!("write mir json: {}", e))
}

/// Variant for the bin crate's local MIR type
pub fn emit_mir_json_for_harness_bin(
    module: &crate::mir::MirModule,
    path: &std::path::Path,
) -> Result<(), String> {
    use crate::mir::{BinaryOp as B, CompareOp as C, MirInstruction as I, MirType};
    use crate::mir::definitions::call_unified::TypeCertainty;
    // Decide schema once for this emission
    let force_v0 = std::env::var("NYASH_JSON_SCHEMA_V0").ok().as_deref() == Some("1")
        || std::env::var("NYASH_LLVM_DOWNGRADE_V1").ok().as_deref() == Some("1");
    let use_v1_schema = if force_v0 { false } else { std::env::var("NYASH_JSON_SCHEMA_V1").ok().as_deref() == Some("1") };
    let mut funs = Vec::new();
    for (name, f) in &module.functions {
        let mut blocks = Vec::new();
        let mut ids: Vec<_> = f.blocks.keys().copied().collect();
        ids.sort();
        for bid in ids {
            if let Some(bb) = f.blocks.get(&bid) {
                let mut insts = Vec::new();
                // Pre-scan to collect values defined in this block
                let mut block_defines: std::collections::HashSet<u32> = std::collections::HashSet::new();
                for inst in &bb.instructions {
                    match inst {
                        I::Copy { dst, .. }
                        | I::Const { dst, .. }
                        | I::BinOp { dst, .. }
                        | I::Compare { dst, .. }
                        | I::Call { dst: Some(dst), .. }
                        | I::ExternCall { dst: Some(dst), .. }
                        | I::BoxCall { dst: Some(dst), .. }
                        | I::NewBox { dst, .. }
                        | I::Phi { dst, .. } => { block_defines.insert(dst.as_u32()); }
                        _ => {}
                    }
                }
                let mut emitted_defs: std::collections::HashSet<u32> = std::collections::HashSet::new();
                for inst in &bb.instructions {
                    if let I::Phi { dst, inputs } = inst {
                        let values_objs: Vec<_> = inputs
                            .iter()
                            .map(|(b, v)| json!({"value": v.as_u32(), "block": b.as_u32()}))
                            .collect();
                        let all_str =
                            inputs
                                .iter()
                                .all(|(_b, v)| match f.metadata.value_types.get(v) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                });
                        if all_str {
                            insts.push(json!({
                                "op":"phi","dst": dst.as_u32(), "values": values_objs,
                                "dst_type": {"kind":"handle","box_type":"StringBox"}
                            }));
                        } else {
                            insts.push(
                                json!({"op":"phi","dst": dst.as_u32(), "values": values_objs}),
                            );
                        }
                        emitted_defs.insert(dst.as_u32());
                    }
                }
                let mut delayed_copies: Vec<(u32, u32)> = Vec::new();
                for inst in &bb.instructions {
                    match inst {
                        I::Copy { dst, src } => {
                            let d = dst.as_u32(); let s = src.as_u32();
                            if block_defines.contains(&s) && !emitted_defs.contains(&s) {
                                delayed_copies.push((d, s));
                            } else {
                                insts.push(json!({"op":"copy","dst": d, "src": s}));
                                emitted_defs.insert(d);
                            }
                        }
                        I::Const { dst, value } => {
                            match value {
                                crate::mir::ConstValue::Integer(i) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "i64", "value": i}}));
                                }
                                crate::mir::ConstValue::Float(fv) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "f64", "value": fv}}));
                                }
                                crate::mir::ConstValue::Bool(b) => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "i64", "value": if *b {1} else {0}}}));
                                }
                                crate::mir::ConstValue::String(s) => {
                                    insts.push(json!({
                                        "op":"const",
                                        "dst": dst.as_u32(),
                                        "value": {
                                            "type": {"kind":"handle","box_type":"StringBox"},
                                            "value": s
                                        }
                                    }));
                                }
                                crate::mir::ConstValue::Null | crate::mir::ConstValue::Void => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "void", "value": 0}}));
                                }
                            }
                            emitted_defs.insert(dst.as_u32());
                        }
                        I::BinOp { dst, op, lhs, rhs } => {
                            let op_s = match op {
                                B::Add => "+",
                                B::Sub => "-",
                                B::Mul => "*",
                                B::Div => "/",
                                B::Mod => "%",
                                B::BitAnd => "&",
                                B::BitOr => "|",
                                B::BitXor => "^",
                                B::Shl => "<<",
                                B::Shr => ">>",
                                B::And => "&",
                                B::Or => "|",
                            };
                            let mut obj = json!({"op":"binop","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()});
                            if matches!(op, B::Add) {
                                let lhs_is_str = match f.metadata.value_types.get(lhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                let rhs_is_str = match f.metadata.value_types.get(rhs) {
                                    Some(MirType::String) => true,
                                    Some(MirType::Box(bt)) if bt == "StringBox" => true,
                                    _ => false,
                                };
                                if lhs_is_str || rhs_is_str {
                                    obj["dst_type"] =
                                        json!({"kind":"handle","box_type":"StringBox"});
                                }
                            }
                            insts.push(obj);
                            emitted_defs.insert(dst.as_u32());
                        }
                        I::Compare { dst, op, lhs, rhs } => {
                            let op_s = match op {
                                C::Eq => "==",
                                C::Ne => "!=",
                                C::Lt => "<",
                                C::Le => "<=",
                                C::Gt => ">",
                                C::Ge => ">=",
                            };
                            insts.push(json!({"op":"compare","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()}));
                            emitted_defs.insert(dst.as_u32());
                        }
                        I::Call { dst, func, callee, args, effects } => {
                            if use_v1_schema {
                                if let Some(c) = callee.as_ref() {
                                    let args_u32: Vec<u32> = args.iter().map(|v| v.as_u32()).collect();
                                    let eff_names = effects.effect_names();
                                    let eff_slices: Vec<&str> = eff_names.iter().map(|s| *s).collect();
                                    let obj = emit_unified_mir_call(
                                        dst.map(|v| v.as_u32()),
                                        c,
                                        &args_u32,
                                        &eff_slices,
                                    );
                                    insts.push(obj);
                                    if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                                    break;
                                }
                            }
                            // v0 or no callee → legacy call payload (func is a NameConst value id)
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"call","func": func.as_u32(), "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                            if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                        }
                        I::ExternCall {
                            dst,
                            iface_name,
                            method_name,
                            args,
                            effects,
                        } => {
                            if use_v1_schema {
                                let args_u32: Vec<u32> = args.iter().map(|v| v.as_u32()).collect();
                                let eff_names = effects.effect_names();
                                let eff_slices: Vec<&str> = eff_names.iter().map(|s| *s).collect();
                                let callee = Callee::Extern(format!("{}.{}", iface_name, method_name));
                                let obj = emit_unified_mir_call(dst.map(|v| v.as_u32()), &callee, &args_u32, &eff_slices);
                                insts.push(obj);
                                if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                            } else {
                                let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                                let mut obj = json!({
                                    "op":"externcall","func": format!("{}.{}", iface_name, method_name), "args": args_a,
                                    "dst": dst.map(|d| d.as_u32()),
                                });
                                if iface_name == "env.console" { if dst.is_some() { obj["dst_type"] = json!("i64"); } }
                                insts.push(obj);
                                if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                            }
                        }
                        I::BoxCall {
                            dst,
                            box_val,
                            method,
                            args,
                            effects, method_id: _
                        } => {
                            if use_v1_schema {
                                // Try to infer box name from metadata; fall back gracefully
                                let box_name = match f.metadata.value_types.get(&box_val) {
                                    Some(MirType::Box(bt)) => bt.clone(),
                                    Some(MirType::String) => "StringBox".to_string(),
                                    _ => "UnknownBox".to_string(),
                                };
                                let args_u32: Vec<u32> = args.iter().map(|v| v.as_u32()).collect();
                                let mut all_args = Vec::with_capacity(args_u32.len() + 1);
                                all_args.push(box_val.as_u32());
                                all_args.extend(args_u32.into_iter());
                                let eff_names = effects.effect_names();
                                let eff_slices: Vec<&str> = eff_names.iter().map(|s| *s).collect();
                                let callee = Callee::Method { box_name, method: method.clone(), receiver: Some(*box_val), certainty: TypeCertainty::Known };
                                let obj = emit_unified_mir_call(dst.map(|v| v.as_u32()), &callee, &all_args, &eff_slices);
                                insts.push(obj);
                                if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                            } else {
                                let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                                let mut obj = json!({
                                    "op":"boxcall","box": box_val.as_u32(), "method": method, "args": args_a, "dst": dst.map(|d| d.as_u32())
                                });
                                let m = method.as_str();
                                let dst_ty = if m == "substring" || m == "dirname" || m == "join" || m == "read_all" || m == "read" {
                                    Some(json!({"kind":"handle","box_type":"StringBox"}))
                                } else if m == "length" || m == "lastIndexOf" { Some(json!("i64")) } else { None };
                                if let Some(t) = dst_ty { obj["dst_type"] = t; }
                                insts.push(obj);
                                if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                            }
                        }
                        I::NewBox {
                            dst,
                            box_type,
                            args,
                        } => {
                            if use_v1_schema {
                                let args_u32: Vec<u32> = args.iter().map(|v| v.as_u32()).collect();
                                let callee = Callee::Constructor { box_type: box_type.clone() };
                                let obj = emit_unified_mir_call(Some(dst.as_u32()), &callee, &args_u32, &["alloc"]);
                                insts.push(obj);
                                emitted_defs.insert(dst.as_u32());
                            } else {
                                let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                                insts.push(json!({"op":"newbox","type": box_type, "args": args_a, "dst": dst.as_u32()}));
                                emitted_defs.insert(dst.as_u32());
                            }
                        }
                        I::Branch {
                            condition,
                            then_bb,
                            else_bb,
                        } => {
                            insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()}));
                        }
                        I::Jump { target } => {
                            insts.push(json!({"op":"jump","target": target.as_u32()}));
                        }
                        I::Return { value } => {
                            insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())}));
                        }
                        _ => {}
                    }
                }
                // Append delayed copies after their sources
                for (d, s) in delayed_copies { insts.push(json!({"op":"copy","dst": d, "src": s})); }
                if let Some(term) = &bb.terminator {
                    match term {
                    I::Return { value } => insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())})),
                    I::Jump { target } => insts.push(json!({"op":"jump","target": target.as_u32()})),
                    I::Branch { condition, then_bb, else_bb } => insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()})),
                    _ => {} }
                }
                blocks.push(json!({"id": bid.as_u32(), "instructions": insts}));
            }
        }
        let params: Vec<_> = f.params.iter().map(|v| v.as_u32()).collect();
        funs.push(json!({"name": name, "params": params, "blocks": blocks}));
    }
    // Optional: v1 schema wrapper (structure is largely compatible as we keep instruction objects unchanged here)
    let force_v0 = std::env::var("NYASH_JSON_SCHEMA_V0").ok().as_deref() == Some("1")
        || std::env::var("NYASH_LLVM_DOWNGRADE_V1").ok().as_deref() == Some("1");
    let use_v1_schema = if force_v0 { false } else { std::env::var("NYASH_JSON_SCHEMA_V1").ok().as_deref() == Some("1") };
    let root = if use_v1_schema {
        json!({
            "schema_version": "1.0",
            "capabilities": ["unified_call","phi","effects","callee_typing"],
            "metadata": {"generator":"nyash-rust","phase":"15.5","features":["mir_call_unification","json_v1_schema"]},
            "functions": funs
        })
    } else {
        json!({"functions": funs})
    };
    std::fs::write(path, serde_json::to_string_pretty(&root).unwrap())
        .map_err(|e| format!("write mir json: {}", e))
}
