/*!
 * PHI Management - PHI node creation and block sealing
 */

use super::LoopBuilder;
use crate::mir::BasicBlockId;

impl<'a> LoopBuilder<'a> {
    // =============================================================
    // PHI Helpers — prepare/finalize PHIs and block sealing
    // =============================================================

    /// ループ変数の準備（事前検出または遅延生成）
    pub(super) fn prepare_loop_variables(
        &mut self,
        header_id: BasicBlockId,
        preheader_id: BasicBlockId,
        loop_carried_vars: &std::collections::HashSet<String>,
    ) -> Result<(), String> {
        let current_vars = self.get_current_variable_map();

        // Filter out function parameters (they don't need PHI nodes unless reassigned)
        let mut loop_vars = current_vars.clone();
        if let Some(ref function) = self.parent_builder.current_function {
            for param_id in &function.params {
                // Remove any variable that still has its original parameter ValueId
                loop_vars.retain(|_name, &mut vid| vid != *param_id);
            }
        }

        // 🔥 ROOT CAUSE FIX: Only create PHIs for true loop-carried variables
        // (variables defined in preheader AND assigned inside loop body)
        loop_vars.retain(|name, _| loop_carried_vars.contains(name));

        // Use filtered vars for PHI generation
        crate::mir::phi_core::loop_phi::save_block_snapshot(
            &mut self.block_var_maps,
            preheader_id,
            &loop_vars,
        );
        let incs = crate::mir::phi_core::loop_phi::prepare_loop_variables_with(
            self,
            header_id,
            preheader_id,
            &loop_vars,
        )?;
        self.incomplete_phis.insert(header_id, incs);
        Ok(())
    }

    /// ブロックをシールし、不完全なPhi nodeを完成させる
    pub(super) fn seal_block(&mut self, block_id: BasicBlockId, latch_id: BasicBlockId) -> Result<(), String> {
        if let Some(incomplete_phis) = self.incomplete_phis.remove(&block_id) {
            let cont_snaps = self.continue_snapshots.clone();
            crate::mir::phi_core::loop_phi::seal_incomplete_phis_with(
                self,
                block_id,
                latch_id,
                incomplete_phis,
                &cont_snaps,
            )?;
        }
        self.mark_block_sealed(block_id)?;
        Ok(())
    }

    /// Exitブロックで変数のPHIを生成（breakポイントでの値を統一）
    pub(super) fn create_exit_phis(&mut self, header_id: BasicBlockId, exit_id: BasicBlockId) -> Result<(), String> {
        let header_vars = self.get_current_variable_map();
        let exit_snaps = self.exit_snapshots.clone();
        crate::mir::phi_core::loop_phi::build_exit_phis_with(
            self,
            header_id,
            exit_id,
            &header_vars,
            &exit_snaps,
        )
    }

    pub(super) fn mark_block_unsealed(&mut self, _block_id: BasicBlockId) -> Result<(), String> {
        // ブロックはデフォルトでunsealedなので、特に何もしない
        // （既にBasicBlock::newでsealed: falseに初期化されている）
        Ok(())
    }

    pub(super) fn mark_block_sealed(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                block.seal();
                Ok(())
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }
}