//! Small loop utilities for MirBuilder
use super::{BasicBlockId, MirBuilder};

/// Create header/body/after blocks for a loop and push loop context.
pub(crate) fn create_loop_blocks(builder: &mut MirBuilder) -> (BasicBlockId, BasicBlockId, BasicBlockId) {
    let header = builder.block_gen.next();
    let body = builder.block_gen.next();
    let after = builder.block_gen.next();
    push_loop_context(builder, header, after);
    (header, body, after)
}

/// Push loop context (header/exit) onto the MirBuilder stacks.
pub(crate) fn push_loop_context(
    builder: &mut super::MirBuilder,
    header: BasicBlockId,
    exit: BasicBlockId,
) {
    builder.loop_header_stack.push(header);
    builder.loop_exit_stack.push(exit);
}

/// Pop loop context (header/exit) from the MirBuilder stacks.
pub(crate) fn pop_loop_context(builder: &mut super::MirBuilder) {
    let _ = builder.loop_header_stack.pop();
    let _ = builder.loop_exit_stack.pop();
}

/// Peek current loop header block id
#[allow(dead_code)]
pub(crate) fn current_header(builder: &super::MirBuilder) -> Option<BasicBlockId> {
    builder.loop_header_stack.last().copied()
}

/// Peek current loop exit block id
pub(crate) fn current_exit(builder: &super::MirBuilder) -> Option<BasicBlockId> {
    builder.loop_exit_stack.last().copied()
}

/// Returns true if the builder is currently inside at least one loop context.
#[allow(dead_code)]
pub(crate) fn in_loop(builder: &super::MirBuilder) -> bool {
    !builder.loop_header_stack.is_empty()
}

/// Current loop nesting depth (0 means not in a loop).
#[allow(dead_code)]
pub(crate) fn depth(builder: &super::MirBuilder) -> usize {
    builder.loop_header_stack.len()
}

/// Add predecessor edge metadata to a basic block.
pub(crate) fn add_predecessor(
    builder: &mut MirBuilder,
    block: BasicBlockId,
    pred: BasicBlockId,
) -> Result<(), String> {
    if let Some(ref mut function) = builder.current_function {
        if let Some(bb) = function.get_block_mut(block) {
            bb.add_predecessor(pred);
            return Ok(());
        }
        return Err(format!("Block {} not found", block));
    }
    Err("No current function".to_string())
}
