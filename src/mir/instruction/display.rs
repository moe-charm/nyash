//! Display implementation for MIR Instructions
//!
//! Provides human-readable string representation of MIR instructions for debugging and analysis.

use crate::mir::instruction::MirInstruction;
use std::fmt;

impl fmt::Display for MirInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirInstruction::Const { dst, value } => {
                write!(f, "{} = const {}", dst, value)
            }
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                write!(f, "{} = {} {:?} {}", dst, lhs, op, rhs)
            }
            MirInstruction::UnaryOp { dst, op, operand } => {
                write!(f, "{} = {:?} {}", dst, op, operand)
            }
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                write!(f, "{} = {} {:?} {}", dst, lhs, op, rhs)
            }
            MirInstruction::Load { dst, ptr } => {
                write!(f, "{} = load {}", dst, ptr)
            }
            MirInstruction::Store { value, ptr } => {
                write!(f, "store {} -> {}", value, ptr)
            }
            MirInstruction::Call {
                dst,
                func,
                callee: _, // TODO: Use callee for type-safe resolution display
                args,
                effects,
            } => {
                if let Some(dst) = dst {
                    write!(
                        f,
                        "{} = call {}({}); effects: {}",
                        dst,
                        func,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", "),
                        effects
                    )
                } else {
                    write!(
                        f,
                        "call {}({}); effects: {}",
                        func,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", "),
                        effects
                    )
                }
            }
            MirInstruction::PluginInvoke {
                dst,
                box_val,
                method,
                args,
                effects: _,
            } => {
                if let Some(dst) = dst {
                    write!(
                        f,
                        "{} = plugin_invoke {}.{}({})",
                        dst,
                        box_val,
                        method,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    write!(
                        f,
                        "plugin_invoke {}.{}({})",
                        box_val,
                        method,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            MirInstruction::Return { value } => {
                if let Some(value) = value {
                    write!(f, "ret {}", value)
                } else {
                    write!(f, "ret void")
                }
            }
            MirInstruction::ExternCall {
                dst,
                iface_name,
                method_name,
                args,
                effects,
            } => {
                if let Some(dst) = dst {
                    write!(
                        f,
                        "{} = extern_call {}.{}({}); effects: {}",
                        dst,
                        iface_name,
                        method_name,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", "),
                        effects
                    )
                } else {
                    write!(
                        f,
                        "extern_call {}.{}({}); effects: {}",
                        iface_name,
                        method_name,
                        args.iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<_>>()
                            .join(", "),
                        effects
                    )
                }
            }
            _ => write!(f, "{:?}", self), // Fallback for other instructions
        }
    }
}