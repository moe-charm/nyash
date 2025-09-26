use crate::mir::function::MirFunction;
use crate::mir::{BasicBlockId, ValueId};
use crate::mir::verification_types::VerificationError;
use crate::mir::verification::utils;
use std::collections::{HashMap, HashSet};

/// Verify CFG references and reachability
pub fn check_control_flow(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors = Vec::new();
    for (block_id, block) in &function.blocks {
        for successor in &block.successors {
            if !function.blocks.contains_key(successor) {
                errors.push(VerificationError::ControlFlowError {
                    block: *block_id,
                    reason: format!("References non-existent block {}", successor),
                });
            }
        }
    }
    let reachable = utils::compute_reachable_blocks(function);
    for block_id in function.blocks.keys() {
        if !reachable.contains(block_id) && *block_id != function.entry_block {
            errors.push(VerificationError::UnreachableBlock { block: *block_id });
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Verify that merge blocks do not use predecessor-defined values directly (must go through Phi)
pub fn check_merge_uses(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    if crate::config::env::verify_allow_no_phi() { return Ok(()); }
    let mut errors = Vec::new();
    let preds = utils::compute_predecessors(function);
    let def_block = utils::compute_def_blocks(function);
    let dominators = utils::compute_dominators(function);
    let mut phi_dsts_in_block: HashMap<BasicBlockId, HashSet<ValueId>> = HashMap::new();
    for (bid, block) in &function.blocks {
        let set = phi_dsts_in_block.entry(*bid).or_default();
        for inst in block.all_instructions() {
            if let crate::mir::MirInstruction::Phi { dst, .. } = inst { set.insert(*dst); }
        }
    }
    for (bid, block) in &function.blocks {
        let Some(pred_list) = preds.get(bid) else { continue };
        if pred_list.len() < 2 { continue; }
        let phi_dsts = phi_dsts_in_block.get(bid);
        let doms_of_block = dominators.get(bid).unwrap();
        for inst in block.all_instructions() {
            if let crate::mir::MirInstruction::Phi { .. } = inst { continue; }
            for used in inst.used_values() {
                if let Some(&db) = def_block.get(&used) {
                    if !doms_of_block.contains(&db) {
                        let is_phi_dst = phi_dsts.map(|s| s.contains(&used)).unwrap_or(false);
                        if !is_phi_dst {
                            errors.push(VerificationError::MergeUsesPredecessorValue { value: used, merge_block: *bid, pred_block: db });
                        }
                    }
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

