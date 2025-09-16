/*!
 * Select & Condition helpers (scaffolding)
 *
 * Placeholder for condition normalization / short-circuit pre-processing
 * to keep `function.rs` focused on structure. Implementations will be
 * added incrementally; for now, this module is documentation-only.
 */

use crate::mir::{function::MirFunction, ValueId};
use crate::mir::MirType;
use super::super::types; // access mapping helpers if needed later

/// Normalize a branch condition if needed (scaffolding).
/// Currently returns the input unchanged; provides a single place
/// to adjust semantics later (e.g., truthy rules, short-circuit pre-pass).
pub(crate) fn normalize_branch_condition(func: &MirFunction, cond: &ValueId) -> ValueId {
    // Minimal truthy normalization hook.
    // Strategy (no new instructions here):
    // - If we have a recorded type for `cond` and it is a boolean-like i1/i64 (0/1), return as-is.
    // - Otherwise, return the original cond and let flow/emit handle `!= 0` lowering as today.
    if let Some(ty) = func.metadata.value_types.get(cond) {
        match ty {
            MirType::I1 | MirType::I64 | MirType::Bool => { return *cond; }
            _ => {}
        }
    }
    *cond
}
