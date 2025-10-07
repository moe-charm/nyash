/*!
 * lifecycle_contracts – unified unborn/birth policy helpers
 *
 * Goals
 * - Single source for unborn diagnostics and birth idempotence semantics
 * - Backend-agnostic (no VM types). Callers pass flags/keys as needed.
 */

/// Unified diagnostic message for operations on unborn instances.
pub fn unborn_error_message() -> &'static str {
    "operation on unborn instance (call birth() first)"
}

/// Decide whether an operation on an instance should fail due to unborn state.
///
/// Parameters
/// - seen_new: whether the instance was observed via NewBox()
/// - seen_birth: whether birth() has been observed (or currently in birth())
#[inline]
pub fn is_unborn_violation(seen_new: bool, seen_birth: bool) -> bool {
    seen_new && !seen_birth
}

/// Record a birth event into a born-set. Returns true if this is a duplicate birth.
/// (Idempotent semantics at higher layer may treat duplicates as no-op.)
pub fn record_birth<T: std::hash::Hash + Eq>(born_set: &mut std::collections::HashSet<T>, key: T) -> bool
where
    T: Clone,
{
    // insert returns false if already present → duplicate birth
    !born_set.insert(key)
}
