/*!
 * VM Control-Flow helpers (scaffolding)
 *
 * Purpose: Encapsulate block transitions, branching decisions, and phi entry bookkeeping.
 * Status: Initial skeleton for future extraction from vm.rs
 */

use crate::mir::BasicBlockId;
use super::vm::{VMError};

/// Result of a block step when evaluating a terminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Continue in the same block (advance pc)
    Continue,
    /// Jump to target block
    Jump(BasicBlockId),
    /// Function returned (handled by VM)
    Return,
}

/// Record a block transition inside VM bookkeeping
pub fn record_transition(
    previous_block: &mut Option<BasicBlockId>,
    loop_recorder: &mut crate::backend::vm_phi::LoopExecutor,
    from: BasicBlockId,
    to: BasicBlockId,
) -> Result<(), VMError> {
    *previous_block = Some(from);
    loop_recorder.record_transition(from, to);
    Ok(())
}

