//! Small loop utilities for MirBuilder
use super::BasicBlockId;

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
pub(crate) fn current_header(builder: &super::MirBuilder) -> Option<BasicBlockId> {
    builder.loop_header_stack.last().copied()
}

/// Peek current loop exit block id
pub(crate) fn current_exit(builder: &super::MirBuilder) -> Option<BasicBlockId> {
    builder.loop_exit_stack.last().copied()
}

/// Returns true if the builder is currently inside at least one loop context.
pub(crate) fn in_loop(builder: &super::MirBuilder) -> bool {
    !builder.loop_header_stack.is_empty()
}

/// Current loop nesting depth (0 means not in a loop).
pub(crate) fn depth(builder: &super::MirBuilder) -> usize {
    builder.loop_header_stack.len()
}
