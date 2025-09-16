/*!
 * Terminators (scaffolding)
 *
 * Thin re-exports of flow-level terminators. Call sites can gradually
 * migrate to `terminators::*` without changing behavior.
 */

pub use super::flow::{emit_branch, emit_jump, emit_return};

