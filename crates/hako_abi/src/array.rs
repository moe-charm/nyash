//! Array ABI contract

use super::handles::HakoHandle;

/// Array operations ABI
pub trait ArrayAbi {
    /// Create new array
    fn array_new() -> HakoHandle;

    /// Get element at index (returns i64 or 0 if not found)
    fn array_get(handle: HakoHandle, idx: i64) -> i64;

    /// Set element at index (returns 0 on success)
    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64;

    /// Push element (returns new length)
    fn array_push(handle: HakoHandle, val: i64) -> i64;

    /// Get array length
    fn array_len(handle: HakoHandle) -> i64;
}
