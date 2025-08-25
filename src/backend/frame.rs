/*!
 * VM Frame management (scaffolding)
 *
 * Purpose: Provide a dedicated struct for per-function execution state (pc, block, locals).
 * Status: Initial skeleton; VM still stores fields directly.
 */

use crate::mir::{BasicBlockId, ValueId};

#[derive(Debug, Default, Clone)]
pub struct ExecutionFrame {
    pub current_block: Option<BasicBlockId>,
    pub pc: usize,
    pub last_result: Option<ValueId>,
}

impl ExecutionFrame {
    pub fn new() -> Self { Self { current_block: None, pc: 0, last_result: None } }
    pub fn reset(&mut self) { self.current_block = None; self.pc = 0; self.last_result = None; }
}

