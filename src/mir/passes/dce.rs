//! Dead Code Elimination (pure instruction DCE)
//!
//! Extracted from the monolithic optimizer to enable modular pass composition.

use crate::mir::{MirFunction, MirModule, ValueId};
use std::collections::HashSet;

/// Eliminate dead code (unused results of pure instructions) across the module.
/// Returns the number of eliminated instructions.
pub fn eliminate_dead_code(module: &mut MirModule) -> usize {
    let mut eliminated_total = 0usize;
    for (_func_name, func) in &mut module.functions {
        eliminated_total += eliminate_dead_code_in_function(func);
    }
    eliminated_total
}

fn eliminate_dead_code_in_function(function: &mut MirFunction) -> usize {
    // Collect values that must be kept (used results + effects)
    let mut used_values: HashSet<ValueId> = HashSet::new();

    // Mark values used by side-effecting instructions and terminators
    for (_bid, block) in &function.blocks {
        for instruction in &block.instructions {
            if !instruction.effects().is_pure() {
                if let Some(dst) = instruction.dst_value() {
                    used_values.insert(dst);
                }
                for u in instruction.used_values() {
                    used_values.insert(u);
                }
            }
        }
        if let Some(term) = &block.terminator {
            for u in term.used_values() {
                used_values.insert(u);
            }
        }
    }

    // Backward propagation: if a value is used, mark its operands as used
    let mut changed = true;
    while changed {
        changed = false;
        for (_bid, block) in &function.blocks {
            for instruction in &block.instructions {
                if let Some(dst) = instruction.dst_value() {
                    if used_values.contains(&dst) {
                        for u in instruction.used_values() {
                            if used_values.insert(u) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove unused pure instructions
    let mut eliminated = 0usize;
    for (_bbid, block) in &mut function.blocks {
        block.instructions.retain(|inst| {
            if inst.effects().is_pure() {
                if let Some(dst) = inst.dst_value() {
                    if !used_values.contains(&dst) {
                        // Keep indices stable is not required here; remove entirely
                        // Logging is suppressed to keep pass quiet by default
                        eliminated += 1;
                        return false;
                    }
                }
            }
            true
        });
    }
    if eliminated > 0 {
        function.update_cfg();
    }
    eliminated
}
