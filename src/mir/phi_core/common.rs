/*!
 * phi_core::common – shared types and invariants (scaffold)
 *
 * Phase 1 keeps this minimal; future phases may move debug asserts and
 * predicate set checks here for both if/loop PHI normalization.
 */

/// Placeholder for future shared PHI input type alias.
/// Using the same tuple form as MIR Phi instruction inputs.
pub type PhiInput = (crate::mir::BasicBlockId, crate::mir::ValueId);

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
    if let Some(block) = function.blocks.get(&merge_bb) {
        for (pred, _v) in inputs.iter() {
            debug_assert!(
                block.predecessors.contains(pred),
                "PHI incoming pred {:?} is not a predecessor of merge bb {:?}",
                pred,
                merge_bb
            );
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
