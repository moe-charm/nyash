//! HostCall-related lowering helpers split from core.rs (no behavior change)
use crate::mir::{MirFunction, ValueId};
use super::builder::IRBuilder;
use std::collections::HashMap;

pub fn lower_array_get(
    b: &mut dyn IRBuilder,
    param_index: &HashMap<ValueId, usize>,
    known_i64: &HashMap<ValueId, i64>,
    array: &ValueId,
    index: &ValueId,
) {
    if crate::jit::config::current().hostcall {
        let idx = known_i64.get(index).copied().unwrap_or(0);
        if let Some(pidx) = param_index.get(array).copied() {
            b.emit_param_i64(pidx);
            b.emit_const_i64(idx);
            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET_H, 2, true);
        } else {
            let arr_idx = -1;
            b.emit_const_i64(arr_idx);
            b.emit_const_i64(idx);
            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET, 2, true);
        }
    }
}

pub fn lower_array_set(
    b: &mut dyn IRBuilder,
    param_index: &HashMap<ValueId, usize>,
    known_i64: &HashMap<ValueId, i64>,
    array: &ValueId,
    index: &ValueId,
    value: &ValueId,
) {
    if crate::jit::config::current().hostcall {
        let idx = known_i64.get(index).copied().unwrap_or(0);
        let val = known_i64.get(value).copied().unwrap_or(0);
        if let Some(pidx) = param_index.get(array).copied() {
            b.emit_param_i64(pidx);
            b.emit_const_i64(idx);
            b.emit_const_i64(val);
            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET_H, 3, false);
        } else {
            let arr_idx = -1;
            b.emit_const_i64(arr_idx);
            b.emit_const_i64(idx);
            b.emit_const_i64(val);
            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET, 3, false);
        }
    }
}

pub fn lower_box_call(
    func: &MirFunction,
    b: &mut dyn IRBuilder,
    param_index: &HashMap<ValueId, usize>,
    known_i64: &HashMap<ValueId, i64>,
    known_f64: &HashMap<ValueId, f64>,
    float_box_values: &std::collections::HashSet<ValueId>,
    recv: &ValueId,
    method: &str,
    args: &Vec<ValueId>,
    dst: Option<ValueId>,
) {
        if !crate::jit::config::current().hostcall { return; }
        match method {
            "len" | "length" => {
                if let Some(pidx) = param_index.get(recv).copied() {
                    crate::jit::events::emit_lower(
                        serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_LEN_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]}),
                        "hostcall","<jit>"
                    );
                    b.emit_param_i64(pidx);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, dst.is_some());
                } else {
                    crate::jit::events::emit(
                        "hostcall","<jit>",None,None,
                        serde_json::json!({
                            "id": crate::jit::r#extern::collections::SYM_ARRAY_LEN,
                            "decision": "fallback", "reason": "receiver_not_param",
                            "argc": 1, "arg_types": ["I64"]
                        })
                    );
                    b.emit_const_i64(-1);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                }
            }
            "isEmpty" | "is_empty" => {
                crate::jit::events::emit_lower(
                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]}),
                    "hostcall","<jit>"
                );
                if let Some(pidx) = param_index.get(recv).copied() {
                    b.emit_param_i64(pidx);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, 1, dst.is_some());
                }
            }
            // math.* family (read-only)
            m if m.starts_with("math.") => {
                let sym = format!("nyash.{}", m);
                use crate::jit::hostcall_registry::{check_signature, ArgKind};
                let mut observed: Vec<ArgKind> = Vec::new();
                for (i, v) in args.iter().enumerate() {
                    let kind = if let Some(mt) = func.signature.params.get(i) {
                        match mt {
                            crate::mir::MirType::Float => ArgKind::F64,
                            crate::mir::MirType::Integer => ArgKind::I64,
                            crate::mir::MirType::Bool => ArgKind::I64,
                            crate::mir::MirType::String | crate::mir::MirType::Box(_) => ArgKind::Handle,
                            _ => {
                                if known_f64.contains_key(v) || float_box_values.contains(v) { ArgKind::F64 } else { ArgKind::I64 }
                            }
                        }
                    } else {
                        if known_f64.contains_key(v) || float_box_values.contains(v) { ArgKind::F64 } else { ArgKind::I64 }
                    };
                    observed.push(kind);
                }
                let arg_types: Vec<&'static str> = observed.iter().map(|k| match k { ArgKind::I64 => "I64", ArgKind::F64 => "F64", ArgKind::Handle => "Handle" }).collect();
                match check_signature(&sym, &observed) {
                    Ok(()) => {
                        crate::jit::events::emit_lower(
                            serde_json::json!({"id": sym, "decision":"allow", "reason":"sig_ok", "argc": observed.len(), "arg_types": arg_types}),
                            "hostcall","<jit>"
                        );
                        if crate::jit::config::current().native_f64 {
                            let (symbol, arity) = match method {
                                "math.sin" => ("nyash.math.sin_f64", 1),
                                "math.cos" => ("nyash.math.cos_f64", 1),
                                "math.abs" => ("nyash.math.abs_f64", 1),
                                "math.min" => ("nyash.math.min_f64", 2),
                                "math.max" => ("nyash.math.max_f64", 2),
                                _ => ("nyash.math.sin_f64", 1),
                            };
                            for i in 0..arity {
                                if let Some(v) = args.get(i) {
                                    if let Some(fv) = known_f64.get(v).copied() { b.emit_const_f64(fv); continue; }
                                    if let Some(iv) = known_i64.get(v).copied() { b.emit_const_f64(iv as f64); continue; }
                                    let mut emitted = false;
                                    'scan: for (_bb_id, bb) in func.blocks.iter() {
                                        for ins in bb.instructions.iter() {
                                            if let crate::mir::MirInstruction::NewBox { dst, box_type, args: nb_args } = ins {
                                                if *dst == *v && box_type == "FloatBox" {
                                                    if let Some(srcv) = nb_args.get(0) {
                                                        if let Some(fv) = known_f64.get(srcv).copied() { b.emit_const_f64(fv); emitted = true; break 'scan; }
                                                        if let Some(iv) = known_i64.get(srcv).copied() { b.emit_const_f64(iv as f64); emitted = true; break 'scan; }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !emitted { b.emit_const_f64(0.0); }
                                } else { b.emit_const_f64(0.0); }
                            }
                            let kinds: Vec<super::builder::ParamKind> = (0..arity).map(|_| super::builder::ParamKind::F64).collect();
                            b.emit_host_call_typed(symbol, &kinds, dst.is_some(), true);
                        }
                    }
                    Err(reason) => {
                        crate::jit::events::emit(
                            "hostcall","<jit>",None,None,
                            serde_json::json!({"id": sym, "decision":"fallback", "reason": reason, "argc": observed.len(), "arg_types": arg_types})
                        );
                    }
                }
            }
            // Map/String/Array read methods and limited mutating (whitelist)
            _ => {
                // Whitelist-driven
                let pol = crate::jit::policy::current();
                match method {
                    // String
                    "charCodeAt" => {
                        crate::jit::events::emit_lower(
                            serde_json::json!({"id": crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, "decision":"allow", "reason":"sig_ok", "argc":2, "arg_types":["Handle","I64"]}),
                            "hostcall","<jit>"
                        );
                        // recvはHandle (param) を期待。indexはknown_i64でcoerce
                        if let Some(pidx) = param_index.get(recv).copied() {
                            b.emit_param_i64(pidx);
                            let idx = args.get(0).and_then(|v| known_i64.get(v).copied()).unwrap_or(0);
                            b.emit_const_i64(idx);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, 2, dst.is_some());
                        }
                    }
                    // Map
                    "size" => {
                        crate::jit::events::emit_lower(
                            serde_json::json!({"id": crate::jit::r#extern::collections::SYM_MAP_SIZE_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]}),
                            "hostcall","<jit>"
                        );
                        if let Some(pidx) = param_index.get(recv).copied() {
                            b.emit_param_i64(pidx);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE_H, 1, dst.is_some());
                        } else {
                            // fallback: id-only (receiver not param)
                            crate::jit::events::emit_lower(
                                serde_json::json!({"id": crate::jit::r#extern::collections::SYM_MAP_SIZE, "decision":"fallback", "reason":"receiver_not_param", "argc":1, "arg_types":["I64"]}),
                                "hostcall","<jit>"
                            );
                            b.emit_const_i64(-1);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE, 1, dst.is_some());
                        }
                    }
                    "has" => {
                        // Decide on key kind via registry and known values
                        use crate::jit::hostcall_registry::{check_signature, ArgKind};
                        let canonical = "nyash.map.has".to_string();
                        let mut observed_kinds: Vec<ArgKind> = Vec::new();
                        observed_kinds.push(ArgKind::Handle);
                        let key_vid = args.get(0).copied();
                        let key_kind = if let Some(kv) = key_vid {
                            if let Some(mt) = func.signature.params.iter().find(|_| true) { // heuristic; signature may not align
                                match mt {
                                    crate::mir::MirType::Float => ArgKind::I64,
                                    crate::mir::MirType::Integer => ArgKind::I64,
                                    crate::mir::MirType::Bool => ArgKind::I64,
                                    crate::mir::MirType::String | crate::mir::MirType::Box(_) => ArgKind::Handle,
                                    _ => ArgKind::Handle,
                                }
                            } else if let Some(_) = known_i64.get(&kv) { ArgKind::I64 } else { ArgKind::Handle }
                        } else { ArgKind::Handle };
                        observed_kinds.push(key_kind);
                        let arg_types: Vec<&'static str> = observed_kinds.iter().map(|k| match k { ArgKind::I64 => "I64", ArgKind::F64 => "F64", ArgKind::Handle => "Handle" }).collect();
                        let _ = check_signature(&canonical, &observed_kinds);
                        // HH fast-path if key is a Handle and also a param; otherwise H/I64
                        if let Some(pidx) = param_index.get(recv).copied() {
                            b.emit_param_i64(pidx);
                            if let Some(kv) = key_vid {
                                match key_kind {
                                    ArgKind::I64 => {
                                        let kval = known_i64.get(&kv).copied().unwrap_or(0);
                                        b.emit_const_i64(kval);
                                        b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_GET_H, 2, dst.is_some());
                                    }
                                    ArgKind::Handle => {
                                        if let Some(kp) = param_index.get(&kv).copied() {
                                            b.emit_param_i64(kp);
                                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_GET_HH, 2, dst.is_some());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    // Array (mutating)
                    "push" | "set" => {
                        let wh = &pol.hostcall_whitelist;
                        let sym = if method == "push" { crate::jit::r#extern::collections::SYM_ARRAY_PUSH } else { crate::jit::r#extern::collections::SYM_ARRAY_SET };
                        if wh.iter().any(|s| s == sym) {
                            crate::jit::events::emit_lower(
                                serde_json::json!({"id": sym, "decision":"allow", "reason":"whitelist", "argc": args.len()}),
                                "hostcall","<jit>"
                            );
                            b.emit_host_call(sym, 2, false);
                        } else {
                            crate::jit::events::emit(
                                "hostcall","<jit>",None,None,
                                serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating", "argc": args.len()})
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
}
