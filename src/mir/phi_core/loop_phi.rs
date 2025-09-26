/*!
 * phi_core::loop_phi – loop-specific PHI management (scaffold)
 *
 * Phase 1 defines minimal types only. The concrete logic remains in
 * `mir::loop_builder` and will be delegated/moved here in later phases.
 */

use crate::mir::{BasicBlockId, ValueId};
use crate::ast::ASTNode;

/// Loop-local placeholder of an incomplete PHI (header-time declaration).
/// Moved from loop_builder to centralize PHI-related types.
#[derive(Debug, Clone)]
pub struct IncompletePhi {
    pub phi_id: ValueId,
    pub var_name: String,
    pub known_inputs: Vec<(BasicBlockId, ValueId)>,
}

/// Common snapshot type used for continue/exit points
pub type VarSnapshot = std::collections::HashMap<String, ValueId>;
pub type SnapshotAt = (BasicBlockId, VarSnapshot);

#[derive(Default)]
pub struct LoopPhiManager;

impl LoopPhiManager {
    pub fn new() -> Self { Self::default() }
}

/// Operations required from a loop builder to finalize PHIs.
pub trait LoopPhiOps {
    fn new_value(&mut self) -> ValueId;
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String>;
    fn update_var(&mut self, name: String, value: ValueId);
    fn get_variable_at_block(&mut self, name: &str, block: BasicBlockId) -> Option<ValueId>;
    fn debug_verify_phi_inputs(&mut self, _merge_bb: BasicBlockId, _inputs: &[(BasicBlockId, ValueId)]) {}
}

/// Finalize PHIs at loop exit (merge of break points and header fall-through).
/// Behavior mirrors loop_builder's create_exit_phis using the provided ops.
pub fn build_exit_phis_with<O: LoopPhiOps>(
    ops: &mut O,
    header_id: BasicBlockId,
    exit_id: BasicBlockId,
    header_vars: &std::collections::HashMap<String, ValueId>,
    exit_snapshots: &[(BasicBlockId, VarSnapshot)],
) -> Result<(), String> {
    // 1) Collect all variable names possibly participating in exit PHIs
    let mut all_vars = std::collections::HashSet::new();
    for var_name in header_vars.keys() {
        all_vars.insert(var_name.clone());
    }
    for (_bid, snapshot) in exit_snapshots.iter() {
        for var_name in snapshot.keys() {
            all_vars.insert(var_name.clone());
        }
    }

    // 2) For each variable, gather incoming values
    for var_name in all_vars {
        let mut phi_inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();

        if let Some(&hv) = header_vars.get(&var_name) {
            phi_inputs.push((header_id, hv));
        }
        for (block_id, snapshot) in exit_snapshots.iter() {
            if let Some(&v) = snapshot.get(&var_name) {
                phi_inputs.push((*block_id, v));
            }
        }

        match phi_inputs.len() {
            0 => {} // nothing to do
            1 => {
                // single predecessor: direct binding
                ops.update_var(var_name, phi_inputs[0].1);
            }
            _ => {
                let dst = ops.new_value();
                ops.debug_verify_phi_inputs(exit_id, &phi_inputs);
                ops.emit_phi_at_block_start(exit_id, dst, phi_inputs)?;
                ops.update_var(var_name, dst);
            }
        }
    }
    Ok(())
}

/// Seal a header block by completing its incomplete PHIs with values from
/// continue snapshots and the latch block.
pub fn seal_incomplete_phis_with<O: LoopPhiOps>(
    ops: &mut O,
    block_id: BasicBlockId,
    latch_id: BasicBlockId,
    mut incomplete_phis: Vec<IncompletePhi>,
    continue_snapshots: &[(BasicBlockId, VarSnapshot)],
) -> Result<(), String> {
    for mut phi in incomplete_phis.drain(..) {
        // from continue points
        for (cid, snapshot) in continue_snapshots.iter() {
            if let Some(&v) = snapshot.get(&phi.var_name) {
                phi.known_inputs.push((*cid, v));
            }
        }
        // from latch
        let value_after = ops
            .get_variable_at_block(&phi.var_name, latch_id)
            .ok_or_else(|| format!("Variable {} not found at latch block", phi.var_name))?;
        phi.known_inputs.push((latch_id, value_after));

        ops.debug_verify_phi_inputs(block_id, &phi.known_inputs);
        ops.emit_phi_at_block_start(block_id, phi.phi_id, phi.known_inputs)?;
        ops.update_var(phi.var_name.clone(), phi.phi_id);
    }
    Ok(())
}

/// Prepare loop header PHIs by declaring one IncompletePhi per variable found
/// in `current_vars` (preheader snapshot), seeding each with (preheader_id, val)
/// and rebinding the variable to the newly allocated Phi result in the builder.
pub fn prepare_loop_variables_with<O: LoopPhiOps>(
    ops: &mut O,
    _header_id: BasicBlockId,
    preheader_id: BasicBlockId,
    current_vars: &std::collections::HashMap<String, ValueId>,
) -> Result<Vec<IncompletePhi>, String> {
    let mut incomplete_phis: Vec<IncompletePhi> = Vec::new();
    for (var_name, &value_before) in current_vars.iter() {
        let phi_id = ops.new_value();
        let inc = IncompletePhi {
            phi_id,
            var_name: var_name.clone(),
            known_inputs: vec![(preheader_id, value_before)],
        };
        incomplete_phis.push(inc);
        ops.update_var(var_name.clone(), phi_id);
    }
    Ok(incomplete_phis)
}

/// Collect variables assigned within a loop body and detect control-flow
/// statements (break/continue). Used for lightweight carrier hinting.
pub fn collect_carrier_assigns(node: &ASTNode, vars: &mut Vec<String>, has_ctrl: &mut bool) {
    match node {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() {
                if !vars.iter().any(|v| v == name) {
                    vars.push(name.clone());
                }
            }
        }
        ASTNode::Break { .. } | ASTNode::Continue { .. } => { *has_ctrl = true; }
        ASTNode::If { then_body, else_body, .. } => {
            let tp = ASTNode::Program { statements: then_body.clone(), span: crate::ast::Span::unknown() };
            collect_carrier_assigns(&tp, vars, has_ctrl);
            if let Some(eb) = else_body {
                let ep = ASTNode::Program { statements: eb.clone(), span: crate::ast::Span::unknown() };
                collect_carrier_assigns(&ep, vars, has_ctrl);
            }
        }
        ASTNode::Program { statements, .. } => {
            for s in statements { collect_carrier_assigns(s, vars, has_ctrl); }
        }
        _ => {}
    }
}

/// Save a block-local variable snapshot into the provided store.
pub fn save_block_snapshot(
    store: &mut std::collections::HashMap<BasicBlockId, VarSnapshot>,
    block: BasicBlockId,
    snapshot: &VarSnapshot,
) {
    store.insert(block, snapshot.clone());
}
