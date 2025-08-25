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

            // Arithmetic with BoxRef(IntegerBox)
            (VMValue::BoxRef(li), VMValue::BoxRef(ri)) if li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some() && ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some() => {
                let l = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().unwrap().value;
                let r = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().unwrap().value;
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
            (VMValue::BoxRef(li), VMValue::Integer(r)) if li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some() => {
                let l = li.as_any().downcast_ref::<crate::box_trait::IntegerBox>().unwrap().value;
                let res = match op {
                    BinaryOp::Add => l + *r,
                    BinaryOp::Sub => l - *r,
                    BinaryOp::Mul => l * *r,
                    BinaryOp::Div => { if *r == 0 { return Err(VMError::DivisionByZero); } l / *r },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(res))
            }
            (VMValue::Integer(l), VMValue::BoxRef(ri)) if ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().is_some() => {
                let r = ri.as_any().downcast_ref::<crate::box_trait::IntegerBox>().unwrap().value;
                let res = match op {
                    BinaryOp::Add => *l + r,
                    BinaryOp::Sub => *l - r,
                    BinaryOp::Mul => *l * r,
                    BinaryOp::Div => { if r == 0 { return Err(VMError::DivisionByZero); } *l / r },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(res))
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
            _ => Err(VMError::TypeError(format!("Unsupported comparison: {:?} on {:?} and {:?}", op, left, right))),
        }
    }
}
