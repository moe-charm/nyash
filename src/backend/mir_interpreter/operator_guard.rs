//! operator_guard.rs — Centralized guard for Operator Boxes adoption/interception
//!
//! Purpose
//! - Avoid scattered CompareOperator/AddOperator re-entry across VM paths
//! - Provide a single point to decide adopt vs native evaluation
//! - Keep hot paths (compare/add) native by default; adopt only when explicitly enabled

use super::*;

/// Kinds of operator boxes we may guard (extensible)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperatorKind {
    Compare, /* Add */
}

#[inline]
pub fn should_adopt(kind: OperatorKind) -> bool {
    match kind {
        OperatorKind::Compare => crate::config::env::operator_box_compare_adopt(),
        // OperatorKind::Add => crate::config::env::operator_box_add_adopt(),
    }
}

impl super::MirInterpreter {
    /// Intercept by function name before any frame/regs work (exec_function_inner entry point).
    /// Returns Some(Result) when interception handled the call; None to continue normal flow.
    pub(super) fn operator_guard_intercept_entry(
        &mut self,
        func_name: &str,
        arg_vals: Option<&[VMValue]>,
    ) -> Option<Result<VMValue, VMError>> {
        // CompareOperator.apply/3 — intercept when adopt is OFF
        if func_name.starts_with("CompareOperator.apply/") {
            if let Some(argv) = arg_vals {
                if argv.len() == 3 {
                    use crate::mir::CompareOp;
                    let op = match &argv[0] {
                        VMValue::String(s) => match s.as_str() {
                            "Eq" => Some(CompareOp::Eq),
                            "Ne" => Some(CompareOp::Ne),
                            "Lt" => Some(CompareOp::Lt),
                            "Le" => Some(CompareOp::Le),
                            "Gt" => Some(CompareOp::Gt),
                            "Ge" => Some(CompareOp::Ge),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(kind) = op {
                        let res = match (argv.get(1), argv.get(2)) {
                            (Some(a), Some(b)) => self.eval_cmp(kind, a.clone(), b.clone()).map(VMValue::Bool),
                            _ => Ok(VMValue::Bool(false)),
                        };
                        return Some(res);
                    }
                }
            }
            return Some(Ok(VMValue::Bool(false)));
        }
        
        // Unary Operator.apply/1 — Intercept; evaluate via native rules
        if func_name.ends_with("Operator.apply/1") {
            if let Some(argv) = arg_vals {
                if argv.len() == 1 {
                    let x = argv[0].clone();
                    // Neg
                    if func_name.starts_with("NegOperator.apply/") {
                        let v = match x {
                            VMValue::Integer(i) => Ok(VMValue::Integer(-i)),
                            VMValue::Float(f) => Ok(VMValue::Float(-f)),
                            other => Err(VMError::TypeError(format!("neg expects number, got {:?}", other))),
                        };
                        return Some(v);
                    }
                    // Not
                    if func_name.starts_with("NotOperator.apply/") {
                        match crate::backend::abi_util::to_bool_vm(&x) {
                            Ok(b) => return Some(Ok(VMValue::Bool(!b))),
                            Err(e) => return Some(Err(VMError::TypeError(e))),
                        }
                    }
                    // BitNot
                    if func_name.starts_with("BitNotOperator.apply/") {
                        let v = match x {
                            VMValue::Integer(i) => Ok(VMValue::Integer(!i)),
                            other => Err(VMError::TypeError(format!("bitnot expects integer, got {:?}", other))),
                        };
                        return Some(v);
                    }
                }
            }
            return Some(Ok(VMValue::Void));
        }
// Arithmetic/Bitwise Operator.apply/2 — Intercept; evaluate via eval_binop
        if func_name.ends_with("Operator.apply/2") {
            use crate::mir::BinaryOp;
            let kind = if func_name.starts_with("AddOperator.apply/") {
                Some(BinaryOp::Add)
            } else if func_name.starts_with("SubOperator.apply/") {
                Some(BinaryOp::Sub)
            } else if func_name.starts_with("MulOperator.apply/") {
                Some(BinaryOp::Mul)
            } else if func_name.starts_with("DivOperator.apply/") {
                Some(BinaryOp::Div)
            } else if func_name.starts_with("ModOperator.apply/") {
                Some(BinaryOp::Mod)
            } else if func_name.starts_with("ShlOperator.apply/") {
                Some(BinaryOp::Shl)
            } else if func_name.starts_with("ShrOperator.apply/") {
                Some(BinaryOp::Shr)
            } else if func_name.starts_with("BitAndOperator.apply/") {
                Some(BinaryOp::BitAnd)
            } else if func_name.starts_with("BitOrOperator.apply/") {
                Some(BinaryOp::BitOr)
            } else if func_name.starts_with("BitXorOperator.apply/") {
                Some(BinaryOp::BitXor)
            } else { None };
            if let Some(opk) = kind {
                if let Some(argv) = arg_vals {
                    if argv.len() == 2 {
                        let a = argv[0].clone();
                        let b = argv[1].clone();
                        match self.eval_binop(opk, a, b) {
                            Ok(v) => return Some(Ok(v)),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                }
                return Some(Ok(VMValue::Void));
            }
        }
        None
    }
}
