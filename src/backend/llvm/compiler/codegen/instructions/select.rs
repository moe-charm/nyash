/*!
 * Select & Condition helpers (scaffolding)
 *
 * Placeholder for condition normalization / short-circuit pre-processing
 * to keep `function.rs` focused on structure. Implementations will be
 * added incrementally; for now, this module is documentation-only.
 */

use crate::mir::{function::MirFunction, ValueId};

/// Normalize a branch condition if needed (scaffolding).
/// Currently returns the input unchanged; provides a single place
/// to adjust semantics later (e.g., truthy rules, short-circuit pre-pass).
pub(crate) fn normalize_branch_condition(_func: &MirFunction, cond: &ValueId) -> ValueId {
    *cond
}
