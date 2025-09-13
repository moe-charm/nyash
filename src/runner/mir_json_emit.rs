use serde_json::json;

/// Emit MIR JSON for Python harness/PyVM.
/// The JSON schema matches tools/llvmlite_harness.py expectations and is
/// intentionally minimal for initial scaffolding.
pub fn emit_mir_json_for_harness(
    module: &nyash_rust::mir::MirModule,
    path: &std::path::Path,
) -> Result<(), String> {
    use nyash_rust::mir::{MirInstruction as I, BinaryOp as B, CompareOp as C};
    let mut funs = Vec::new();
    for (name, f) in &module.functions {
        let mut blocks = Vec::new();
        let mut ids: Vec<_> = f.blocks.keys().copied().collect();
        ids.sort();
        for bid in ids {
            if let Some(bb) = f.blocks.get(&bid) {
                let mut insts = Vec::new();
                // PHI first（オプション）
                for inst in &bb.instructions {
                    if let I::Phi { dst, inputs } = inst {
                        let incoming: Vec<_> = inputs
                            .iter()
                            .map(|(b, v)| json!([v.as_u32(), b.as_u32()]))
                            .collect();
                        insts.push(json!({"op":"phi","dst": dst.as_u32(), "incoming": incoming}));
                    }
                }
                // Non-PHI
                for inst in &bb.instructions {
                    match inst {
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
                                nyash_rust::mir::ConstValue::Null | nyash_rust::mir::ConstValue::Void => {
                                    insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": "void", "value": 0}}));
                                }
                            }
                        }
                        I::BinOp { dst, op, lhs, rhs } => {
                            let op_s = match op { B::Add=>"+",B::Sub=>"-",B::Mul=>"*",B::Div=>"/",B::Mod=>"%",B::BitAnd=>"&",B::BitOr=>"|",B::BitXor=>"^",B::Shl=>"<<",B::Shr=>">>",B::And=>"&",B::Or=>"|"};
                            insts.push(json!({"op":"binop","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()}));
                        }
                        I::Compare { dst, op, lhs, rhs } => {
                            let op_s = match op { C::Lt=>"<", C::Le=>"<=", C::Gt=>">", C::Ge=>">=", C::Eq=>"==", C::Ne=>"!=" };
                            insts.push(json!({"op":"compare","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()}));
                        }
                        I::Call { dst, func, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"call","func": func.as_u32(), "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                        }
                        I::ExternCall { dst, iface_name, method_name, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            let func_name = if iface_name == "env.console" {
                                format!("nyash.console.{}", method_name)
                            } else { format!("{}.{}", iface_name, method_name) };
                            insts.push(json!({"op":"externcall","func": func_name, "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                        }
                        I::BoxCall { dst, box_val, method, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            // Minimal dst_type hints
                            let mut obj = json!({
                                "op":"boxcall","box": box_val.as_u32(), "method": method, "args": args_a, "dst": dst.map(|d| d.as_u32())
                            });
                            let m = method.as_str();
                            let dst_ty = if m == "substring" {
                                Some(json!({"kind":"handle","box_type":"StringBox"}))
                            } else if m == "length" || m == "lastIndexOf" {
                                Some(json!("i64"))
                            } else { None };
                            if let Some(t) = dst_ty { obj["dst_type"] = t; }
                            insts.push(obj);
                        }
                        I::NewBox { dst, box_type, args } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"newbox","type": box_type, "args": args_a, "dst": dst.as_u32()}));
                        }
                        I::Branch { condition, then_bb, else_bb } => {
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
