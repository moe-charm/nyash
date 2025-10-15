use crate::mir::{MirFunction, MirInstruction, BasicBlockId, ValueId};
use crate::mir::verification_types::VerificationError;
use std::collections::{HashMap, HashSet};

/// Ensure that each Phi's inputs cover all reachable predecessors of its block.
pub fn check_phi_inputs_cover_predecessors(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors = Vec::new();

    // Build predecessor map from successors
    let mut preds: HashMap<BasicBlockId, HashSet<BasicBlockId>> = HashMap::new();
    for (bid, bb) in function.blocks.iter() {
        for succ in &bb.successors {
            preds.entry(*succ).or_default().insert(*bid);
        }
    }

    for (bb_id, bb) in function.blocks.iter() {
        // Consider only blocks with at least two predecessors
        let Some(bb_preds) = preds.get(bb_id) else { continue; };
        if bb_preds.len() < 2 { continue; }

        // Collect PHIs in the block (dst, inputs)
        let phis: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = bb.instructions.iter().filter_map(|inst| {
            if let MirInstruction::Phi { dst, inputs } = inst { Some((*dst, inputs.clone())) } else { None }
        }).collect();

        if phis.is_empty() { continue; }

        // For each Phi: check that every reachable predecessor has an entry
        for (dst, inputs) in phis {
            let mut seen: HashSet<BasicBlockId> = HashSet::new();
            for (pred, _v) in &inputs { seen.insert(*pred); }

            for pred in bb_preds.iter() {
                // Skip unreachable predecessors: ones that end with Return/Throw
                if let Some(pbb) = function.get_block(*pred) {
                    let unreachable = pbb.ends_with_return() || matches!(pbb.terminator, Some(MirInstruction::Throw{..}));
                    if unreachable { continue; }
                }
                if !seen.contains(pred) {
                    errors.push(VerificationError::InvalidPhi {
                        phi_value: dst,
                        block: *bb_id,
                        reason: format!("missing input from predecessor bb{:?}", pred),
                    });
                }
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

