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
    /// Execute binary operation
    pub(super) fn execute_binary_op(&self, op: &BinaryOp, left: &VMValue, right: &VMValue) -> Result<VMValue, VMError> {
        let debug_bin = std::env::var("NYASH_VM_DEBUG_BIN").ok().as_deref() == Some("1");
        if debug_bin { eprintln!("[VM] binop {:?} {:?} {:?}", op, left, right); }
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
                match op { BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r.to_string_box().value))), _ => Err(VMError::TypeError("String-BoxRef operations only support addition".to_string())) }
            },

            // BoxRef + String concatenation
            (VMValue::BoxRef(l), VMValue::String(r)) => {
                match op { BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l.to_string_box().value, r))), _ => Err(VMError::TypeError("BoxRef-String operations only support addition".to_string())) }
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
