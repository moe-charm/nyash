use inkwell::{
    basic_block::BasicBlock,
    values::{BasicValueEnum, FunctionValue, IntValue},
};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{
    function::MirFunction,
    instruction::MirInstruction,
    ValueId,
};

use super::builder_cursor::BuilderCursor;

/// LoopForm scaffolding — fixed block layout for while/loop normalization
pub struct LoopFormContext<'ctx> {
    pub header: BasicBlock<'ctx>,
    pub body: BasicBlock<'ctx>,
    pub dispatch: BasicBlock<'ctx>,
    pub latch: BasicBlock<'ctx>,
    pub exit: BasicBlock<'ctx>,
    pub loop_id: u32,
}

impl<'ctx> LoopFormContext<'ctx> {
    /// Create a new LoopForm block set under `function` with a readable name prefix.
    pub fn new(
        codegen: &CodegenContext<'ctx>,
        function: FunctionValue<'ctx>,
        loop_id: u32,
        prefix: &str,
    ) -> Self {
        let header = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_header", prefix, loop_id));
        let body = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_body", prefix, loop_id));
        let dispatch = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_dispatch", prefix, loop_id));
        let latch = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_latch", prefix, loop_id));
        let exit = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_exit", prefix, loop_id));
        Self { header, body, dispatch, latch, exit, loop_id }
    }
}

/// Lower a while-like loop using LoopForm shape (Phase 1: scaffold only).
/// - condition: MIR value producing i1/i64 truthy
/// - body_mir: MIR instructions of loop body
/// Note: In Phase 1, this function is not invoked by default lowering; it is a gated scaffold.
pub fn lower_while_loopform<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    _func: &MirFunction,
    _llvm_func: FunctionValue<'ctx>,
    _condition: &ValueId,
    _body_mir: &[MirInstruction],
    _loop_id: u32,
    _prefix: &str,
) -> Result<(), String> {
    // Gate via env; currently a no-op scaffold so the call sites can be added safely later.
    let enabled = std::env::var("NYASH_ENABLE_LOOPFORM").ok().as_deref() == Some("1");
    if !enabled {
        return Ok(());
    }
    // Intentionally minimal implementation placeholder to keep compilation stable.
    // The full lowering will:
    // 1) Create LoopFormContext blocks
    // 2) Emit header with conditional branch to body/dispatch
    // 3) Lower body and build Signal(tag,payload)
    // 4) In dispatch, create PHIs (payload/tag) and switch(tag) to latch/exit
    // 5) Latch branches back to header
    // For now, do nothing to avoid interfering with current lowering flow.
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[LoopForm] scaffold active but not wired (Phase 1)");
    }
    Ok(())
}

