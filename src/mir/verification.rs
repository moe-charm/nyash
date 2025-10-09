/*!
 * MIR Verification - SSA form and semantic verification
 *
 * Implements dominance checking, SSA verification, and semantic analysis
 */

use super::{MirFunction, MirModule};
use crate::debug::log as dlog;
use crate::mir::verification_types::VerificationError;
mod cfg;
mod dom;
mod awaits;
mod barrier;
mod legacy;
mod compare;
mod static_self_fields;
mod phi_inputs;
mod utils;
mod ssa;


/// MIR verifier for SSA form and semantic correctness
pub struct MirVerifier {
    /// Current verification errors
    errors: Vec<VerificationError>,
}

impl MirVerifier {
    /// Create a new MIR verifier
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Verify an entire MIR module
    pub fn verify_module(&mut self, module: &MirModule) -> Result<(), Vec<VerificationError>> {
        self.errors.clear();

        for (_name, function) in &module.functions {
            if let Err(mut func_errors) = self.verify_function(function) {
                // Add function context to errors
                for _error in &mut func_errors {
                    // Could add function name to error context here
                }
                self.errors.extend(func_errors);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Verify a single MIR function
    pub fn verify_function(
        &mut self,
        function: &MirFunction,
    ) -> Result<(), Vec<VerificationError>> {
        let mut local_errors = Vec::new();

        // 1. Check SSA form
        if let Err(mut ssa_errors) = self.verify_ssa_form(function) {
            local_errors.append(&mut ssa_errors);
        }

        // 2. Check dominance relations
        if let Err(mut dom_errors) = self.verify_dominance(function) {
            local_errors.append(&mut dom_errors);
        }

        // 3. Check control flow integrity
        if let Err(mut cfg_errors) = self.verify_control_flow(function) {
            local_errors.append(&mut cfg_errors);
        }
        // 4. Check merge-block value usage (ensure Phi is used)
        if let Err(mut merge_errors) = self.verify_merge_uses(function) {
            local_errors.append(&mut merge_errors);
        }
        // 5. Minimal checks for WeakRef/Barrier
        if let Err(mut weak_barrier_errors) = self.verify_weakref_and_barrier(function) {
            local_errors.append(&mut weak_barrier_errors);
        }
        // 6. Light barrier-context diagnostic (strict mode only)
        if let Err(mut barrier_ctx) = self.verify_barrier_context(function) {
            local_errors.append(&mut barrier_ctx);
        }
        // 7. Forbid legacy instructions (must be rewritten to Core-15)
        if let Err(mut legacy_errors) = self.verify_no_legacy_ops(function) {
            local_errors.append(&mut legacy_errors);
        }
        // 8. Async semantics: ensure checkpoints around await
        if let Err(mut await_cp) = self.verify_await_checkpoints(function) {
            local_errors.append(&mut await_cp);
        }

        // 9. Forbid Box Compare(Eq/Ne): equality must route via op_eq() at MIR
        if let Err(mut cmp_errors) = compare::check_no_box_compare(function) {
            local_errors.append(&mut cmp_errors);
        }

        // 9.5. Forbid static self field emulation: getField/setField on constant class name
        if let Err(mut static_field_errors) = static_self_fields::check_no_static_self_field_calls(function) {
            local_errors.append(&mut static_field_errors);
        }

        // 9.6. PHI inputs coverage (dev-first): ensure PHI inputs cover reachable predecessors
        if std::env::var("NYASH_VERIFY_PHI_STRICT").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_VERIFY_MIR").ok().as_deref() == Some("1")
        {
            if let Err(mut phi_cov) = phi_inputs::check_phi_inputs_cover_predecessors(function) {
                local_errors.append(&mut phi_cov);
            }
        }

        // 10. PHI-off strict edge-copy policy (optional)
        if crate::config::env::mir_no_phi() && crate::config::env::verify_edge_copy_strict() {
            if let Err(mut ecs) = self.verify_edge_copy_strict(function) {
                local_errors.append(&mut ecs);
            }
        }

        if local_errors.is_empty() {
            Ok(())
        } else {
            if dlog::on("NYASH_DEBUG_VERIFIER") {
                eprintln!(
                    "[VERIFY] {} errors in function {}",
                    local_errors.len(),
                    function.signature.name
                );
                for e in &local_errors {
                    match e {
                        VerificationError::MergeUsesPredecessorValue {
                            value,
                            merge_block,
                            pred_block,
                        } => {
                            eprintln!(
                                "  • MergeUsesPredecessorValue: value=%{:?} merge_bb={:?} pred_bb={:?} -- hint: insert/use Phi in merge block for values from predecessors",
                                value, merge_block, pred_block
                            );
                        }
                        VerificationError::DominatorViolation {
                            value,
                            use_block,
                            def_block,
                        } => {
                            eprintln!(
                                "  • DominatorViolation: value=%{:?} use_bb={:?} def_bb={:?} -- hint: ensure definition dominates use, or route via Phi",
                                value, use_block, def_block
                            );
                        }
                        VerificationError::InvalidPhi {
                            phi_value,
                            block,
                            reason,
                        } => {
                            eprintln!(
                                "  • InvalidPhi: phi_dst=%{:?} in bb={:?} reason={} -- hint: check inputs cover all predecessors and placed at block start",
                                phi_value, block, reason
                            );
                        }
                        VerificationError::InvalidWeakRefSource {
                            weak_ref,
                            block,
                            instruction_index,
                            reason,
                        } => {
                            eprintln!(
                                "  • InvalidWeakRefSource: weak=%{:?} at {}:{} reason='{}' -- hint: source must be WeakRef(new)/WeakNew; ensure creation precedes load and value flows correctly",
                                weak_ref, block, instruction_index, reason
                            );
                        }
                        VerificationError::InvalidBarrierPointer {
                            ptr,
                            block,
                            instruction_index,
                            reason,
                        } => {
                            eprintln!(
                                "  • InvalidBarrierPointer: ptr=%{:?} at {}:{} reason='{}' -- hint: barrier pointer must be a valid ref (not void/null); ensure it is defined and non-void",
                                ptr, block, instruction_index, reason
                            );
                        }
                        VerificationError::SuspiciousBarrierContext {
                            block,
                            instruction_index,
                            note,
                        } => {
                            eprintln!(
                                "  • SuspiciousBarrierContext: at {}:{} note='{}' -- hint: place barrier within ±2 of load/store/ref ops in same block or disable strict check",
                                block, instruction_index, note
                            );
                        }
                        other => {
                            eprintln!("  • {:?}", other);
                        }
                    }
                }
            }
            Err(local_errors)
        }
    }

    /// When PHI-off strict mode is enabled, enforce that merge blocks do not
    /// introduce self-copies and that each predecessor provides a Copy into the
    /// merged destination for values used in the merge block that do not dominate it.
    fn verify_edge_copy_strict(
        &self,
        function: &MirFunction,
    ) -> Result<(), Vec<VerificationError>> {
        let mut errors = Vec::new();
        let preds = utils::compute_predecessors(function);
        let def_block = utils::compute_def_blocks(function);
        let dominators = utils::compute_dominators(function);

        for (merge_bid, merge_bb) in &function.blocks {
            let p = preds.get(merge_bid).cloned().unwrap_or_default();
            if p.len() < 2 {
                continue; // Only enforce on real merges (>=2 predecessors)
            }

            // Collect values used in merge block
            let mut used_values = std::collections::HashSet::new();
            for inst in merge_bb.all_instructions() {
                for v in inst.used_values() {
                    used_values.insert(v);
                }
            }

            // For each used value that doesn't dominate the merge block, enforce edge-copy policy
            for &u in &used_values {
                if let Some(&def_bb) = def_block.get(&u) {
                    // If the def dominates the merge block, edge copies are not required
                    let dominated = dominators
                        .get(merge_bid)
                        .map(|set| set.contains(&def_bb))
                        .unwrap_or(false);
                    if dominated && def_bb != *merge_bid {
                        continue;
                    }
                }

                // Merge block itself must not contain a Copy to the merged value
                let has_self_copy_in_merge = merge_bb
                    .instructions
                    .iter()
                    .any(|inst| matches!(inst, super::MirInstruction::Copy { dst, .. } if *dst == u));
                if has_self_copy_in_merge {
                    errors.push(VerificationError::EdgeCopyStrictViolation {
                        block: *merge_bid,
                        value: u,
                        pred_block: None,
                        reason: "merge block contains Copy to merged value; use predecessor copies only".to_string(),
                    });
                }

                // Each predecessor must provide a Copy into the merged destination
                for pred in &p {
                    let Some(pbb) = function.blocks.get(pred) else { continue; };
                    let has_copy = pbb.instructions.iter().any(|inst| matches!(
                        inst,
                        super::MirInstruction::Copy { dst, .. } if *dst == u
                    ));
                    if !has_copy {
                        errors.push(VerificationError::MergeUsesPredecessorValue {
                            value: u,
                            merge_block: *merge_bid,
                            pred_block: *pred,
                        });
                    }
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Reject legacy instructions that should be rewritten to Core-15 equivalents
    /// Skips check when NYASH_VERIFY_ALLOW_LEGACY=1
    fn verify_no_legacy_ops(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        legacy::check_no_legacy_ops(function)
    }

    /// Ensure that each Await instruction (or ExternCall(env.future.await)) is immediately
    /// preceded and followed by a checkpoint.
    /// A checkpoint is either MirInstruction::Safepoint or ExternCall("env.runtime", "checkpoint").
    fn verify_await_checkpoints(
        &self,
        function: &MirFunction,
    ) -> Result<(), Vec<VerificationError>> {
        awaits::check_await_checkpoints(function)
    }

    /// Verify WeakRef/Barrier minimal semantics
    fn verify_weakref_and_barrier(
        &self,
        function: &MirFunction,
    ) -> Result<(), Vec<VerificationError>> {
        barrier::check_weakref_and_barrier(function)
    }

    /// Light diagnostic: Barrier should be near memory ops in the same block (best-effort)
    /// Enabled only when NYASH_VERIFY_BARRIER_STRICT=1
    fn verify_barrier_context(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        barrier::check_barrier_context(function)
    }

    /// Verify SSA form properties
    fn verify_ssa_form(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        ssa::check_ssa_form(function)
    }

    /// Verify dominance relations (def must dominate use across blocks)
    fn verify_dominance(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        dom::check_dominance(function)
    }

    /// Verify control flow graph integrity
    fn verify_control_flow(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        cfg::check_control_flow(function)
    }

    /// Verify that blocks with multiple predecessors do not use predecessor-defined values directly.
    /// In merge blocks, values coming from predecessors must be routed through Phi.
    fn verify_merge_uses(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        cfg::check_merge_uses(function)
    }

    /// Get all verification errors from the last run
    pub fn get_errors(&self) -> &[VerificationError] {
        &self.errors
    }

    /// Clear verification errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

}

impl Default for MirVerifier {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {}
