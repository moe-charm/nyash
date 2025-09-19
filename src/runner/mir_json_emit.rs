use serde_json::json;

/// Emit MIR JSON for Python harness/PyVM.
/// The JSON schema matches tools/llvmlite_harness.py expectations and is
/// intentionally minimal for initial scaffolding.
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
                        let incoming: Vec<_> = inputs
                            .iter()
                            .map(|(b, v)| json!([v.as_u32(), b.as_u32()]))
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
                                "op":"phi","dst": dst.as_u32(), "incoming": incoming,
                                "dst_type": {"kind":"handle","box_type":"StringBox"}
                            }));
                        } else {
                            insts.push(
                                json!({"op":"phi","dst": dst.as_u32(), "incoming": incoming}),
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
                            dst, func, args, ..
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"call","func": func.as_u32(), "args": args_a, "dst": dst.map(|d| d.as_u32())}));
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
    let root = json!({"functions": funs});
    std::fs::write(path, serde_json::to_string_pretty(&root).unwrap())
        .map_err(|e| format!("write mir json: {}", e))
}

/// Variant for the bin crate's local MIR type
pub fn emit_mir_json_for_harness_bin(
    module: &crate::mir::MirModule,
    path: &std::path::Path,
) -> Result<(), String> {
    use crate::mir::{BinaryOp as B, CompareOp as C, MirInstruction as I, MirType};
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
                        let incoming: Vec<_> = inputs
                            .iter()
                            .map(|(b, v)| json!([v.as_u32(), b.as_u32()]))
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
                                "op":"phi","dst": dst.as_u32(), "incoming": incoming,
                                "dst_type": {"kind":"handle","box_type":"StringBox"}
                            }));
                        } else {
                            insts.push(
                                json!({"op":"phi","dst": dst.as_u32(), "incoming": incoming}),
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
                        I::ExternCall {
                            dst,
                            iface_name,
                            method_name,
                            args,
                            ..
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            let mut obj = json!({
                                "op":"externcall","func": format!("{}.{}", iface_name, method_name), "args": args_a,
                                "dst": dst.map(|d| d.as_u32()),
                            });
                            if iface_name == "env.console" {
                                if dst.is_some() {
                                    obj["dst_type"] = json!("i64");
                                }
                            }
                            insts.push(obj);
                            if let Some(d) = dst.map(|v| v.as_u32()) { emitted_defs.insert(d); }
                        }
                        I::BoxCall {
                            dst,
                            box_val,
                            method,
                            args,
                            ..
                        } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
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
    let root = json!({"functions": funs});
    std::fs::write(path, serde_json::to_string_pretty(&root).unwrap())
        .map_err(|e| format!("write mir json: {}", e))
}
