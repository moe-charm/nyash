use crate::mir::function::MirFunction;
use crate::mir::verification_types::VerificationError;
use crate::mir::verification::utils;

/// Verify dominance: def must dominate use across blocks (Phi inputs excluded)
pub fn check_dominance(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    if crate::config::env::verify_allow_no_phi() { return Ok(()); }
    let mut errors = Vec::new();
    let def_block = utils::compute_def_blocks(function);
    let dominators = utils::compute_dominators(function);
    for (use_block_id, block) in &function.blocks {
        for instruction in block.all_instructions() {
            if let crate::mir::MirInstruction::Phi { .. } = instruction { continue; }
            for used_value in instruction.used_values() {
                if let Some(&def_bb) = def_block.get(&used_value) {
                    if def_bb != *use_block_id {
                        let doms = dominators.get(use_block_id).unwrap();
                        if !doms.contains(&def_bb) {
                            errors.push(VerificationError::DominatorViolation {
                                value: used_value,
                                use_block: *use_block_id,
                                def_block: def_bb,
                            });
                        }
                    }
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

