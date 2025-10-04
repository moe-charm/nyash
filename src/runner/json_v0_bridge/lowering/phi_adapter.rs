use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use std::collections::HashMap;

/// BridgePhiOps implements phi_core::if_phi::PhiMergeOps on top of MirFunction
/// and a mutable variable map owned by the JSON v0 bridge lowering.
pub struct BridgePhiOps<'a> {
    pub f: &'a mut MirFunction,
    pub vars: &'a mut HashMap<String, ValueId>,
}

impl<'a> BridgePhiOps<'a> {
    pub fn new(f: &'a mut MirFunction, vars: &'a mut HashMap<String, ValueId>) -> Self {
        Self { f, vars }
    }
}

impl<'a> crate::mir::phi_core::if_phi::PhiMergeOps for BridgePhiOps<'a> {
    fn new_value(&mut self) -> ValueId {
        self.f.next_value_id()
    }
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        if let Some(bb) = self.f.get_block_mut(block) {
            bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs });
            Ok(())
        } else {
            Err(format!("merge block {:?} not found", block))
        }
    }
    fn update_var(&mut self, name: String, value: ValueId) {
        self.vars.insert(name, value);
    }
    fn debug_verify_phi_inputs(
        &mut self,
        merge_bb: BasicBlockId,
        inputs: &[(BasicBlockId, ValueId)],
    ) {
        #[cfg(debug_assertions)]
        {
            crate::mir::phi_core::common::debug_verify_phi_inputs(self.f, merge_bb, inputs);
        }
    }
}
