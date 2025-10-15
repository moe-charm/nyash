//! Common Subexpression Elimination (CSE) for pure MIR instructions.
//!
//! Note: Current implementation mirrors the prior monolithic behavior and
//! counts eliminations without rewriting uses (SSA update is TODO). This keeps
//! behavior identical while modularizing the pass for future enhancement.

use crate::mir::{Callee, MirFunction, MirInstruction, MirModule, ValueId};
use std::collections::HashMap;

/// Run CSE across the module. Returns the number of eliminated expressions.
pub fn eliminate_common_subexpressions(module: &mut MirModule) -> usize {
    let mut eliminated = 0usize;
    for (_name, func) in module.functions.iter_mut() {
        eliminated += cse_in_function(func);
    }
    eliminated
}

fn cse_in_function(function: &mut MirFunction) -> usize {
    let mut expression_map: HashMap<String, ValueId> = HashMap::new();
    let mut eliminated = 0usize;

    for (_bid, block) in &mut function.blocks {
        for inst in &mut block.instructions {
            // Safety: never CSE external boundary calls (Call→Callee::Extern)
            match inst {
                MirInstruction::Call { callee: Some(Callee::Extern(_)), .. } => { continue; }
                _ => {}
            }
            if inst.effects().is_pure() {
                let key = instruction_key(inst);
                if let Some(&existing) = expression_map.get(&key) {
                    if let Some(dst) = inst.dst_value() {
                        // Count as eliminated; rewriting uses is a future improvement.
                        let _ = (existing, dst); // keep variables referenced
                        eliminated += 1;
                    }
                } else if let Some(dst) = inst.dst_value() {
                    expression_map.insert(key, dst);
                }
            }
        }
    }
    eliminated
}

fn instruction_key(i: &MirInstruction) -> String {
    match i {
        MirInstruction::Const { value, .. } => format!("const_{:?}", value),
        MirInstruction::BinOp { op, lhs, rhs, .. } => {
            format!("binop_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32())
        }
        MirInstruction::Compare { op, lhs, rhs, .. } => {
            format!("cmp_{:?}_{}_{}", op, lhs.as_u32(), rhs.as_u32())
        }
        MirInstruction::Call { func, args, .. } => {
            let args_str = args
                .iter()
                .map(|v| v.as_u32().to_string())
                .collect::<Vec<_>>()
                .join(",");
            format!("call_{}_{}", func.as_u32(), args_str)
        }
        other => format!("other_{:?}", other),
    }
}
