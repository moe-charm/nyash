/*!
 * VM Value Operations
 *
 * Purpose: Arithmetic/logical/comparison helpers and boolean coercions
 * Responsibilities: execute_binary_op / execute_unary_op / execute_compare_op
 * Key APIs: execute_binary_op, execute_unary_op, execute_compare_op
 * Typical Callers: vm_instructions::{execute_binop, execute_unaryop, execute_compare}
 */

use crate::mir::{BinaryOp, CompareOp, UnaryOp};
use super::vm::{VM, VMError, VMValue};

impl VM {
    /// Try to view a BoxRef as a UTF-8 string using unified semantics
    fn try_boxref_to_string(&self, b: &dyn crate::box_trait::NyashBox) -> Option<String> {
        crate::runtime::semantics::coerce_to_string(b)
    }
    /// Execute binary operation
    pub(super) fn execute_binary_op(&self, op: &BinaryOp, left: &VMValue, right: &VMValue) -> Result<VMValue, VMError> {
        let debug_bin = std::env::var("NYASH_VM_DEBUG_BIN").ok().as_deref() == Some("1");
        if debug_bin { eprintln!("[VM] binop {:?} {:?} {:?}", op, left, right); }
        if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
            let lty = match left { VMValue::String(_) => "String", VMValue::Integer(_) => "Integer", VMValue::Float(_) => "Float", VMValue::Bool(_) => "Bool", _ => "Other" };
            let rty = match right { VMValue::String(_) => "String", VMValue::Integer(_) => "Integer", VMValue::Float(_) => "Float", VMValue::Bool(_) => "Bool", _ => "Other" };
            match *op {
                BinaryOp::Add => {
                    let strat = crate::grammar::engine::get().add_coercion_strategy();
                    let rule = crate::grammar::engine::get().decide_add_result(lty, rty);
                    eprintln!("[GRAMMAR-DIFF][VM] add.coercion_strategy={} left={} right={} rule={:?}", strat, lty, rty, rule);
                }
                BinaryOp::Sub => {
                    let strat = crate::grammar::engine::get().sub_coercion_strategy();
                    let rule = crate::grammar::engine::get().decide_sub_result(lty, rty);
                    eprintln!("[GRAMMAR-DIFF][VM] sub.coercion_strategy={} left={} right={} rule={:?}", strat, lty, rty, rule);
                }
                BinaryOp::Mul => {
                    let strat = crate::grammar::engine::get().mul_coercion_strategy();
                    let rule = crate::grammar::engine::get().decide_mul_result(lty, rty);
                    eprintln!("[GRAMMAR-DIFF][VM] mul.coercion_strategy={} left={} right={} rule={:?}", strat, lty, rty, rule);
                }
                BinaryOp::Div => {
                    let strat = crate::grammar::engine::get().div_coercion_strategy();
                    let rule = crate::grammar::engine::get().decide_div_result(lty, rty);
                    eprintln!("[GRAMMAR-DIFF][VM] div.coercion_strategy={} left={} right={} rule={:?}", strat, lty, rty, rule);
                }
                _ => {}
            }
        }
        if matches!(*op, BinaryOp::Add) && std::env::var("NYASH_GRAMMAR_ENFORCE_ADD").ok().as_deref() == Some("1") {
            let lty = match left { VMValue::String(_) => "String", VMValue::Integer(_) => "Integer", VMValue::Float(_) => "Float", VMValue::Bool(_) => "Bool", _ => "Other" };
            let rty = match right { VMValue::String(_) => "String", VMValue::Integer(_) => "Integer", VMValue::Float(_) => "Float", VMValue::Bool(_) => "Bool", _ => "Other" };
            if let Some((res, _)) = crate::grammar::engine::get().decide_add_result(lty, rty) {
                match res {
                    "String" => {
                        // Best-effort toString concat
                        fn vmv_to_string(v: &VMValue) -> String {
                            match v {
                                VMValue::String(s) => s.clone(),
                                VMValue::Integer(i) => i.to_string(),
                                VMValue::Float(f) => f.to_string(),
                                VMValue::Bool(b) => b.to_string(),
                                VMValue::Void => "void".to_string(),
                                VMValue::BoxRef(b) => b.to_string_box().value,
                                VMValue::Future(_) => "<future>".to_string(),
                            }
                        }
                        let ls = vmv_to_string(left);
                        let rs = vmv_to_string(right);
                        return Ok(VMValue::String(format!("{}{}", ls, rs)));
                    }
                    "Integer" => {
                        if let (VMValue::Integer(l), VMValue::Integer(r)) = (left, right) {
                            return Ok(VMValue::Integer(l + r));
                        }
                    }
                    _ => {}
                }
            }
        }
        // Fast path: logical AND/OR accept any truthy via as_bool
        if matches!(*op, BinaryOp::And | BinaryOp::Or) {
            let l = left.as_bool()?;
            let r = right.as_bool()?;
            return Ok(VMValue::Bool(match *op { BinaryOp::And => l && r, BinaryOp::Or => l || r, _ => unreachable!() }));
        }

        match (left, right) {
            (VMValue::Integer(l), VMValue::Integer(r)) => {
                let result = match op {
                    BinaryOp::Add => *l + *r,
                    BinaryOp::Sub => *l - *r,
                    BinaryOp::Mul => *l * *r,
                    BinaryOp::Div => {
                        if *r == 0 { return Err(VMError::DivisionByZero); }
                        *l / *r
                    },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(result))
            },

            (VMValue::String(l), VMValue::Integer(r)) => {
                match op {
                    BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r))),
                    _ => Err(VMError::TypeError("String-integer operations only support addition".to_string())),
                }
            },

            (VMValue::String(l), VMValue::Bool(r)) => {
                match op {
                    BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r))),
                    _ => Err(VMError::TypeError("String-bool operations only support addition".to_string())),
                }
            },

            (VMValue::String(l), VMValue::String(r)) => {
                match op { BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r))), _ => Err(VMError::TypeError("String operations only support addition".to_string())) }
            },

            // String + BoxRef concatenation
            (VMValue::String(l), VMValue::BoxRef(r)) => {
                let rs = self.try_boxref_to_string(r.as_ref()).unwrap_or_else(|| r.to_string_box().value);
                match op { BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, rs))), _ => Err(VMError::TypeError("String-BoxRef operations only support addition".to_string())) }
            },

            // BoxRef + String concatenation
            (VMValue::BoxRef(l), VMValue::String(r)) => {
                let ls = self.try_boxref_to_string(l.as_ref()).unwrap_or_else(|| l.to_string_box().value);
                match op { BinaryOp::Add => Ok(VMValue::String(format!("{}{}", ls, r))), _ => Err(VMError::TypeError("BoxRef-String operations only support addition".to_string())) }
            },

            // Arithmetic with BoxRef(IntegerBox) — support both legacy and new IntegerBox
            (VMValue::BoxRef(li), VMValue::BoxRef(ri)) if {
                li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some()
                    || li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().is_some()
            } && {
                ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some()
                    || ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().is_some()
            } => {
                let l = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                    .map(|x| x.value)
                    .or_else(|| li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .unwrap();
                let r = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                    .map(|x| x.value)
                    .or_else(|| ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .unwrap();
                let res = match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => { if r == 0 { return Err(VMError::DivisionByZero); } l / r },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer BoxRef operation: {:?}", op))),
                };
                Ok(VMValue::Integer(res))
            }
            // BoxRef + BoxRef string-like concatenation
            (VMValue::BoxRef(li), VMValue::BoxRef(ri)) => {
                if matches!(*op, BinaryOp::Add) {
                    let ls = self.try_boxref_to_string(li.as_ref()).unwrap_or_else(|| li.to_string_box().value);
                    let rs = self.try_boxref_to_string(ri.as_ref()).unwrap_or_else(|| ri.to_string_box().value);
                    return Ok(VMValue::String(format!("{}{}", ls, rs)));
                }
                Err(VMError::TypeError("Unsupported BoxRef+BoxRef operation".to_string()))
            }

            // Mixed Integer forms
            (VMValue::BoxRef(li), VMValue::Integer(r)) if li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some()
                || li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().is_some() => {
                let l = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                    .map(|x| x.value)
                    .or_else(|| li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .unwrap();
                let res = match op {
                    BinaryOp::Add => l + *r,
                    BinaryOp::Sub => l - *r,
                    BinaryOp::Mul => l * *r,
                    BinaryOp::Div => { if *r == 0 { return Err(VMError::DivisionByZero); } l / *r },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(res))
            }
            (VMValue::Integer(l), VMValue::BoxRef(ri)) if ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some()
                || ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().is_some() => {
                let r = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                    .map(|x| x.value)
                    .or_else(|| ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .unwrap();
                let res = match op {
                    BinaryOp::Add => *l + r,
                    BinaryOp::Sub => *l - r,
                    BinaryOp::Mul => *l * r,
                    BinaryOp::Div => { if r == 0 { return Err(VMError::DivisionByZero); } *l / r },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(res))
            }

            // 80/20 fallback: BoxRef(any) numeric via toString().parse::<i64>()
            (VMValue::BoxRef(lb), VMValue::BoxRef(rb)) => {
                let li = lb.to_string_box().value.trim().parse::<i64>().ok();
                let ri = rb.to_string_box().value.trim().parse::<i64>().ok();
                if let (Some(l), Some(r)) = (li, ri) {
                    let res = match op {
                        BinaryOp::Add => l + r,
                        BinaryOp::Sub => l - r,
                        BinaryOp::Mul => l * r,
                        BinaryOp::Div => { if r == 0 { return Err(VMError::DivisionByZero); } l / r },
                        _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                    };
                    if debug_bin { eprintln!("[VM] binop fallback BoxRef-BoxRef -> {}", res); }
                    Ok(VMValue::Integer(res))
                } else {
                    Err(VMError::TypeError(format!("Unsupported binary operation: {:?} on {:?} and {:?}", op, left, right)))
                }
            }
            (VMValue::BoxRef(lb), VMValue::Integer(r)) => {
                if let Ok(l) = lb.to_string_box().value.trim().parse::<i64>() {
                    let res = match op { BinaryOp::Add => l + *r, BinaryOp::Sub => l - *r, BinaryOp::Mul => l * *r, BinaryOp::Div => { if *r == 0 { return Err(VMError::DivisionByZero); } l / *r }, _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))), };
                    if debug_bin { eprintln!("[VM] binop fallback BoxRef-Int -> {}", res); }
                    Ok(VMValue::Integer(res))
                } else {
                    Err(VMError::TypeError(format!("Unsupported binary operation: {:?} on {:?} and {:?}", op, left, right)))
                }
            }
            (VMValue::Integer(l), VMValue::BoxRef(rb)) => {
                if let Ok(r) = rb.to_string_box().value.trim().parse::<i64>() {
                    let res = match op { BinaryOp::Add => *l + r, BinaryOp::Sub => *l - r, BinaryOp::Mul => *l * r, BinaryOp::Div => { if r == 0 { return Err(VMError::DivisionByZero); } *l / r }, _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))), };
                    if debug_bin { eprintln!("[VM] binop fallback Int-BoxRef -> {}", res); }
                    Ok(VMValue::Integer(res))
                } else {
                    Err(VMError::TypeError(format!("Unsupported binary operation: {:?} on {:?} and {:?}", op, left, right)))
                }
            }

            _ => Err(VMError::TypeError(format!("Unsupported binary operation: {:?} on {:?} and {:?}", op, left, right))),
        }
    }

    /// Execute unary operation
    pub(super) fn execute_unary_op(&self, op: &UnaryOp, operand: &VMValue) -> Result<VMValue, VMError> {
        match (op, operand) {
            (UnaryOp::Neg, VMValue::Integer(i)) => Ok(VMValue::Integer(-i)),
            (UnaryOp::Not, VMValue::Bool(b)) => Ok(VMValue::Bool(!b)),
            _ => Err(VMError::TypeError(format!("Unsupported unary operation: {:?} on {:?}", op, operand))),
        }
    }

    /// Execute comparison operation
    pub(super) fn execute_compare_op(&self, op: &CompareOp, left: &VMValue, right: &VMValue) -> Result<bool, VMError> {
        let debug_cmp = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1") ||
            std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1");
        if debug_cmp { eprintln!("[VM] execute_compare_op enter: op={:?}, left={:?}, right={:?}", op, left, right); }
        match (left, right) {
            // Mixed numeric
            (VMValue::Integer(l), VMValue::Float(r)) => {
                let l = *l as f64; let r = *r; Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r })
            }
            (VMValue::Float(l), VMValue::Integer(r)) => {
                let l = *l; let r = *r as f64; Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r })
            }
            // Bool
            (VMValue::Bool(l), VMValue::Bool(r)) => {
                Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, _ => return Err(VMError::TypeError(format!("Unsupported boolean comparison: {:?}", op))) })
            }
            // Void
            (VMValue::Void, VMValue::Void) => {
                Ok(match op { CompareOp::Eq => true, CompareOp::Ne => false, _ => return Err(VMError::TypeError("Cannot order Void".to_string())) })
            }
            (VMValue::Void, _) | (_, VMValue::Void) => {
                Ok(match op { CompareOp::Eq => false, CompareOp::Ne => true, _ => return Err(VMError::TypeError("Cannot order Void".to_string())) })
            }
            // Homogeneous
            (VMValue::Integer(l), VMValue::Integer(r)) => Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r }),
            (VMValue::Float(l), VMValue::Float(r)) => Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r }),
            (VMValue::String(l), VMValue::String(r)) => Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r }),

            // BoxRef(IntegerBox) comparisons (homogeneous)
            (VMValue::BoxRef(li), VMValue::BoxRef(ri)) => {
                if std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1") {
                    eprintln!("[VM] arm BoxRef-BoxRef: lt={}, rt={}", li.type_name(), ri.type_name());
                }
                if std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[VM] compare BoxRef vs BoxRef: left_type={}, right_type={}, left_str={}, right_str={}",
                        li.type_name(), ri.type_name(), li.to_string_box().value, ri.to_string_box().value
                    );
                }
                // String-like comparison: internal StringBox or Plugin StringBox
                fn boxref_to_string(b: &dyn crate::box_trait::NyashBox) -> Option<String> {
                    if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                        return Some(sb.value.clone());
                    }
                    if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if pb.box_type == "StringBox" {
                            let host = crate::runtime::get_global_plugin_host();
                            let s_opt: Option<String> = {
                                if let Ok(ro) = host.read() {
                                    if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", pb.inner.instance_id, &[]) {
                                        if let Some(vb) = val_opt {
                                            if let Some(sbb) = vb.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                                                Some(sbb.value.clone())
                                            } else { None }
                                        } else { None }
                                    } else { None }
                                } else { None }
                            };
                            if s_opt.is_some() { return s_opt; }
                        }
                    }
                    None
                }
                if let (Some(ls), Some(rs)) = (boxref_to_string(li.as_ref()), boxref_to_string(ri.as_ref())) {
                    return Ok(match op { CompareOp::Eq => ls == rs, CompareOp::Ne => ls != rs, CompareOp::Lt => ls < rs, CompareOp::Le => ls <= rs, CompareOp::Gt => ls > rs, CompareOp::Ge => ls >= rs });
                }
                // Try integer comparisons via downcast or parse fallback
                let l_opt = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                    .or_else(|| li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .or_else(|| li.to_string_box().value.parse::<i64>().ok());
                let r_opt = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                    .or_else(|| ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .or_else(|| ri.to_string_box().value.parse::<i64>().ok());
                if let (Some(l), Some(r)) = (l_opt, r_opt) {
                    return Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r });
                }
                Err(VMError::TypeError(format!("[BoxRef-BoxRef] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            }
            // Mixed String vs BoxRef (string-like)
            (VMValue::String(ls), VMValue::BoxRef(ri)) => {
                let rs_opt = if let Some(sb) = ri.as_any().downcast_ref::<crate::box_trait::StringBox>() { Some(sb.value.clone()) } else {
                    if let Some(pb) = ri.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if pb.box_type == "StringBox" {
                            let host = crate::runtime::get_global_plugin_host();
                            let tmp = if let Ok(ro) = host.read() {
                                if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", pb.inner.instance_id, &[]) {
                                    if let Some(vb) = val_opt {
                                        if let Some(sbb) = vb.as_any().downcast_ref::<crate::box_trait::StringBox>() { Some(sbb.value.clone()) } else { None }
                                    } else { None }
                                } else { None }
                            } else { None };
                            tmp
                        } else { None }
                    } else { None }
                };
                if let Some(rs) = rs_opt { return Ok(match op { CompareOp::Eq => *ls == rs, CompareOp::Ne => *ls != rs, CompareOp::Lt => *ls < rs, CompareOp::Le => *ls <= rs, CompareOp::Gt => *ls > rs, CompareOp::Ge => *ls >= rs }); }
                Err(VMError::TypeError(format!("[String-BoxRef] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            }
            (VMValue::BoxRef(li), VMValue::String(rs)) => {
                let ls_opt = if let Some(sb) = li.as_any().downcast_ref::<crate::box_trait::StringBox>() { Some(sb.value.clone()) } else {
                    if let Some(pb) = li.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if pb.box_type == "StringBox" {
                            let host = crate::runtime::get_global_plugin_host();
                            let tmp = if let Ok(ro) = host.read() {
                                if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", pb.inner.instance_id, &[]) {
                                    if let Some(vb) = val_opt { if let Some(sbb) = vb.as_any().downcast_ref::<crate::box_trait::StringBox>() { Some(sbb.value.clone()) } else { None } } else { None }
                                } else { None }
                            } else { None };
                            tmp
                        } else { None }
                    } else { None }
                };
                if let Some(ls) = ls_opt { return Ok(match op { CompareOp::Eq => ls == *rs, CompareOp::Ne => ls != *rs, CompareOp::Lt => ls < *rs, CompareOp::Le => ls <= *rs, CompareOp::Gt => ls > *rs, CompareOp::Ge => ls >= *rs }); }
                Err(VMError::TypeError(format!("[BoxRef-String] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            }
            // Mixed Integer (BoxRef vs Integer)
            (VMValue::BoxRef(li), VMValue::Integer(r)) => {
                let l_opt = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                    .or_else(|| li.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .or_else(|| li.to_string_box().value.parse::<i64>().ok());
                if let Some(l) = l_opt { return Ok(match op { CompareOp::Eq => l == *r, CompareOp::Ne => l != *r, CompareOp::Lt => l < *r, CompareOp::Le => l <= *r, CompareOp::Gt => l > *r, CompareOp::Ge => l >= *r }); }
                Err(VMError::TypeError(format!("[BoxRef-Integer] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            }
            (VMValue::Integer(l), VMValue::BoxRef(ri)) => {
                let r_opt = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                    .or_else(|| ri.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>().map(|x| x.value))
                    .or_else(|| ri.to_string_box().value.parse::<i64>().ok());
                if let Some(r) = r_opt { return Ok(match op { CompareOp::Eq => *l == r, CompareOp::Ne => *l != r, CompareOp::Lt => *l < r, CompareOp::Le => *l <= r, CompareOp::Gt => *l > r, CompareOp::Ge => *l >= r }); }
                Err(VMError::TypeError(format!("[Integer-BoxRef] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            }
            _ => {
                // 80/20 numeric fallback: coerce via to_string when possible
                fn to_i64(v: &VMValue) -> Option<i64> {
                    match v {
                        VMValue::Integer(i) => Some(*i),
                        VMValue::Bool(b) => Some(if *b { 1 } else { 0 }),
                        VMValue::String(s) => s.trim().parse::<i64>().ok(),
                        VMValue::Float(f) => Some(*f as i64),
                        VMValue::BoxRef(b) => b.to_string_box().value.trim().parse::<i64>().ok(),
                        _ => None,
                    }
                }
                if let (Some(l), Some(r)) = (to_i64(left), to_i64(right)) {
                    return Ok(match op { CompareOp::Eq => l == r, CompareOp::Ne => l != r, CompareOp::Lt => l < r, CompareOp::Le => l <= r, CompareOp::Gt => l > r, CompareOp::Ge => l >= r });
                }
                if std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1") {
                    let lty = match left { VMValue::BoxRef(b) => format!("BoxRef({})", b.type_name()), other => format!("{:?}", other) };
                    let rty = match right { VMValue::BoxRef(b) => format!("BoxRef({})", b.type_name()), other => format!("{:?}", other) };
                    eprintln!("[VM] compare default arm: op={:?}, left={}, right={}", op, lty, rty);
                }
                Err(VMError::TypeError(format!("[Default] Unsupported comparison: {:?} on {:?} and {:?}", op, left, right)))
            },
        }
    }
}
