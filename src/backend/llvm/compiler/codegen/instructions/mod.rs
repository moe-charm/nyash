mod arith;
mod arith_ops;
mod arrays;
mod blocks;
mod boxcall;
pub mod builder_cursor;
mod call;
mod consts;
pub mod ctx;
mod externcall;
pub mod flow;
mod loopform;
mod maps;
mod mem;
mod newbox;
mod resolver;
pub mod string_ops;
mod strings;
mod terminators; // scaffolding: re-exports flow terminators
mod select;      // scaffolding: prepare for cond/short-circuit helpers

pub(super) use arith::lower_compare;
pub(super) use arith_ops::{lower_binop, lower_unary};
pub(super) use blocks::{create_basic_blocks, precreate_phis};
pub(super) use boxcall::{lower_boxcall, lower_boxcall_boxed, lower_boxcall_via_ctx};
pub(super) use call::lower_call;
pub(super) use consts::lower_const;
pub(super) use externcall::lower_externcall;
pub(super) use flow::{emit_branch, emit_jump, emit_return};
// Future: swap callers to use `terminators::*` instead of `flow::*` directly
pub(super) use terminators::{emit_branch as term_emit_branch, emit_jump as term_emit_jump, emit_return as term_emit_return};
pub(super) use select::normalize_branch_condition;
pub(super) use loopform::dev_check_dispatch_only_phi;
pub(super) use loopform::normalize_header_phis_for_latch;
pub(super) use loopform::{lower_while_loopform, LoopFormContext};
pub(super) use mem::lower_copy;
pub(super) use mem::{lower_load, lower_store};
pub(super) use newbox::lower_newbox;
pub(super) use resolver::Resolver;
