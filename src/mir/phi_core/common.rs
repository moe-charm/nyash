/*!
 * phi_core::common – shared types and invariants (scaffold)
 *
 * Phase 1 keeps this minimal; future phases may move debug asserts and
 * predicate set checks here for both if/loop PHI normalization.
 */

/// Placeholder for future shared PHI input type alias.
/// Using the same tuple form as MIR Phi instruction inputs.
pub type PhiInput = (crate::mir::BasicBlockId, crate::mir::ValueId);


/// Determine if a predecessor block is unreachable for PHI purposes.
/// Unreachable when the block ends with Return or Throw (but not Jump/Branch).
pub fn is_unreachable_pred(function: &crate::mir::MirFunction, pred: crate::mir::BasicBlockId) -> bool {
    if let Some(block) = function.get_block(pred) {
        block.ends_with_return() || matches!(block.terminator, Some(crate::mir::MirInstruction::Throw{..}))
    } else { false }
}

#[cfg(debug_assertions)]
pub fn debug_verify_phi_inputs(
    function: &crate::mir::MirFunction,
    merge_bb: crate::mir::BasicBlockId,
    inputs: &[(crate::mir::BasicBlockId, crate::mir::ValueId)],
) {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for (pred, _v) in inputs.iter() {
        debug_assert_ne!(
            *pred, merge_bb,
            "PHI incoming predecessor must not be the merge block itself"
        );
        debug_assert!(
            seen.insert(*pred),
            "Duplicate PHI incoming predecessor detected: {:?}",
            pred
        );
    }
    // Use a computed predecessor map instead of relying on in-place block.predecessors
    // because builder may emit PHIs before sealing CFG edges on blocks.
    use std::collections::{HashMap};
    use std::collections;
    let mut preds_map: HashMap<crate::mir::BasicBlockId, collections::HashSet<crate::mir::BasicBlockId>> = HashMap::new();
    for (bid, block) in function.blocks.iter() {
        for succ in &block.successors {
            preds_map.entry(*succ).or_default().insert(*bid);
        }
    }
    if let Some(preds) = preds_map.get(&merge_bb) {
        if preds.len() >= 2 {
            for (pred, _v) in inputs.iter() {
                debug_assert!(
                    preds.contains(pred),
                    "PHI incoming pred {:?} is not a predecessor of merge bb {:?}",
                    pred,
                    merge_bb
                );
            }
        }
    }
}

#[cfg(not(debug_assertions))]
pub fn debug_verify_phi_inputs(
    _function: &crate::mir::MirFunction,
    _merge_bb: crate::mir::BasicBlockId,
    _inputs: &[(crate::mir::BasicBlockId, crate::mir::ValueId)],
) {
}

/// Determine assigned variable name from branch-local results.
/// If both sides assign the same name, return it; if only one side assigns,
/// return that one; otherwise None.
pub fn determine_assigned_name(
    then_var: &Option<String>,
    else_var: &Option<String>,
) -> Option<String> {
    match (then_var, else_var) {
        (Some(tv), Some(ev)) if tv == ev => Some(tv.clone()),
        (Some(tv), None) => Some(tv.clone()),
        (None, Some(ev)) => Some(ev.clone()),
        _ => None,
    }
}
