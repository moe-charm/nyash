/*!
 * Control Flow Operations - break/continue/jump handling
 */

use super::LoopBuilder;
use crate::mir::{BasicBlockId, ConstValue, ValueId};

impl<'a> LoopBuilder<'a> {
    // =============================================================
    // Control Helpers — break/continue/jumps/unreachable handling
    // =============================================================

    /// Emit a jump to `target` from the current block and record predecessor metadata.
    pub(super) fn jump_with_pred(&mut self, target: BasicBlockId) -> Result<(), String> {
        let cur_block = self.current_block()?;
        self.emit_jump(target)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, target, cur_block);
        Ok(())
    }

    /// Switch insertion to a fresh (unreachable) block and place a Void const to keep callers satisfied.
    pub(super) fn switch_to_unreachable_block_with_void(&mut self) -> Result<ValueId, String> {
        let next_block = self.new_block();
        self.set_current_block(next_block)?;
        let void_id = self.new_value();
        self.emit_const(void_id, ConstValue::Void)?;
        Ok(void_id)
    }

    /// Handle a `break` statement: jump to loop exit and continue in a fresh unreachable block.
    pub(super) fn do_break(&mut self) -> Result<ValueId, String> {
        // Snapshot variables at break point for exit PHI generation
        let snapshot = self.get_current_variable_map();
        let cur_block = self.current_block()?;
        self.exit_snapshots.push((cur_block, snapshot));

        if let Some(exit_bb) = crate::mir::builder::loops::current_exit(self.parent_builder) {
            self.jump_with_pred(exit_bb)?;
        }
        self.switch_to_unreachable_block_with_void()
    }

    /// Handle a `continue` statement: snapshot vars, jump to latch (LoopFormBox) or header (Legacy), then continue in a fresh unreachable block.
    pub(super) fn do_continue(&mut self) -> Result<ValueId, String> {
        // Snapshot variables at current block to be considered as a predecessor input
        let snapshot = self.get_current_variable_map();
        let cur_block = self.current_block()?;
        self.block_var_maps.insert(cur_block, snapshot.clone());
        self.continue_snapshots.push((cur_block, snapshot));

        // 🔥 FIX: LoopFormBox path jumps to latch, Legacy path jumps to header
        if let Some(latch) = self.loop_latch {
            // LoopFormBox path: continue → latch
            eprintln!("[continue] LoopFormBox: jumping to latch={:?}", latch);
            self.jump_with_pred(latch)?;
        } else if let Some(header) = self.loop_header {
            // Legacy path: continue → header (backward compatibility)
            eprintln!("[continue] Legacy: jumping to header={:?}", header);
            self.jump_with_pred(header)?;
        } else {
            eprintln!("[continue] ❌ ERROR: No latch or header set!");
        }

        self.switch_to_unreachable_block_with_void()
    }
}