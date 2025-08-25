/*!
 * Loop Builder Facade (Minimal API)
 *
 * Goal: Decouple loop construction from concrete MirBuilder types by exposing
 * a small trait that both legacy (mir::builder::MirBuilder) and modularized
 * builders can implement. This enables shared helpers and gradual migration.
 *
 * Note: Only legacy MirBuilder is wired for now to keep WIP modularized code
 * out of the main build. The modularized builder can implement this trait
 * later without changing callers.
 */

use super::{BasicBlockId, MirInstruction, ValueId};

/// Minimal API for constructing loops and emitting instructions
pub trait LoopBuilderApi {
    /// Allocate a new basic block id
    fn new_block(&mut self) -> BasicBlockId;
    /// Get current block id
    fn current_block(&self) -> Result<BasicBlockId, String>;
    /// Switch current block, creating it if needed
    fn start_new_block(&mut self, block: BasicBlockId) -> Result<(), String>;
    /// Emit an instruction to the current block
    fn emit(&mut self, inst: MirInstruction) -> Result<(), String>;
    /// Allocate a new SSA value id
    fn new_value(&mut self) -> ValueId;
}

/// Helper: simplified loop lowering usable by any LoopBuilderApi implementor
pub fn build_simple_loop<L: LoopBuilderApi>(
    lb: &mut L,
    condition: ValueId,
    build_body: &mut dyn FnMut(&mut L) -> Result<(), String>,
) -> Result<ValueId, String> {
    let header = lb.new_block();
    let body = lb.new_block();
    let after = lb.new_block();

    // Jump to header
    lb.emit(MirInstruction::Jump { target: header })?;

    // Header: branch on provided condition
    lb.start_new_block(header)?;
    lb.emit(MirInstruction::Branch { condition, then_bb: body, else_bb: after })?;

    // Body
    lb.start_new_block(body)?;
    build_body(lb)?;
    lb.emit(MirInstruction::Jump { target: header })?;

    // After: return void value
    lb.start_new_block(after)?;
    let void_id = lb.new_value();
    lb.emit(MirInstruction::Const { dst: void_id, value: super::instruction::ConstValue::Void })?;
    Ok(void_id)
}

// === Legacy wiring: implement LoopBuilderApi for mir::builder::MirBuilder ===
impl LoopBuilderApi for super::builder::MirBuilder {
    fn new_block(&mut self) -> BasicBlockId { self.block_gen.next() }
    fn current_block(&self) -> Result<BasicBlockId, String> {
        self.current_block.ok_or_else(|| "No current block".to_string())
    }
    fn start_new_block(&mut self, block: BasicBlockId) -> Result<(), String> {
        super::builder::MirBuilder::start_new_block(self, block)
    }
    fn emit(&mut self, inst: MirInstruction) -> Result<(), String> {
        super::builder::MirBuilder::emit_instruction(self, inst)
    }
    fn new_value(&mut self) -> ValueId { self.value_gen.next() }
}

