//! Host API Symbol Anchors - Force-link host C ABI symbols
//!
//! ## Problem
//! LTO (Link Time Optimization) removes "unused" host API functions even with -rdynamic,
//! causing dlsym() failures when plugins try to call them at runtime.
//!
//! ## Root Cause
//! 1. Host API functions are in static library (nyash_kernel crate)
//! 2. No direct references from main binary (plugins reference at runtime via dlsym)
//! 3. Functions have #[no_mangle] but NOT #[used]
//! 4. LTO sees no references → removes functions
//! 5. -rdynamic configured but can't export symbols that don't exist
//!
//! ## Solution
//! Create static references with #[used] to prevent dead code elimination.
//! The #[used] attribute forces linker to retain symbols, then -rdynamic makes them visible.
//!
//! ## Two-Stage Process
//! 1. Retention: #[used] static references (this file)
//! 2. Export: -rdynamic linker flag (.cargo/config.toml)

use super::array::*;
use super::future::*;
use super::instance::*;
use super::map::*;
use super::string::*;

/// Wrapper to make function pointer array Sync-safe
pub struct FnPtrArray(pub &'static [*const ()]);
unsafe impl Sync for FnPtrArray {}

/// Host API function pointers - prevents dead code elimination
///
/// These static references force the linker to retain all host API functions
/// that plugins might need at runtime via dlsym().
#[used]
#[no_mangle]
pub static NYASH_HOST_API_ANCHORS: FnPtrArray = FnPtrArray(&[
    // ========== Array API ==========
    // Core array operations for ArrayBox plugin callbacks
    nyash_array_get_h as *const (),    // get(idx) -> i64
    nyash_array_set_h as *const (),    // set(idx, val) -> i64
    nyash_array_push_h as *const (),   // push(val) -> i64
    nyash_array_length_h as *const (), // length() -> i64
    nyash_array_new_h as *const (),    // new() -> i64 (⚠️ THIS WAS BEING STRIPPED!)
    // ========== Map API ==========
    // Core map operations for MapBox plugin callbacks
    nyash_map_size_h as *const (), // size() -> i64
    nyash_map_get_h as *const (),  // get(key: i64) -> i64
    nyash_map_set_h as *const (),  // set(key: i64, val: i64) -> i64
    nyash_map_has_h as *const (),  // has(key: i64) -> i64(bool)
    nyash_map_get_hh as *const (), // get(key: any) -> any (handle version)
    nyash_map_set_hh as *const (), // set(key: any, val: any) -> i64
    nyash_map_has_hh as *const (), // has(key: any) -> i64(bool)
    // ========== String API ==========
    // String handle conversion for plugin interop
    nyash_string_to_i8p_h as *const (), // StringBox handle -> *mut i8
    // ========== Instance API ==========
    // Generic instance field access for user-defined Boxes
    nyash_instance_get_field_h as *const (), // get_field(handle, name) -> i64
    nyash_instance_set_field_h as *const (), // set_field(handle, name, val) -> i64
    // ========== Future API ==========
    // Async operation support
    nyash_future_spawn_method_h as *const (), // spawn_method(...) -> i64(future_id)
]);

/// Total count of anchored host API functions
///
/// Used for validation and debugging. If this doesn't match the array length,
/// it indicates a maintenance error.
#[used]
#[no_mangle]
pub static NYASH_HOST_API_COUNT: usize = 16;

/// Validation: Ensure count matches array length
///
/// This const assertion will fail at compile time if the count is wrong.
const _: () = assert!(
    NYASH_HOST_API_ANCHORS.0.len() == NYASH_HOST_API_COUNT,
    "NYASH_HOST_API_COUNT doesn't match actual anchor array length"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_count() {
        assert_eq!(
            NYASH_HOST_API_ANCHORS.0.len(),
            NYASH_HOST_API_COUNT,
            "Anchor count mismatch"
        );
    }

    #[test]
    fn test_no_null_pointers() {
        for (i, ptr) in NYASH_HOST_API_ANCHORS.0.iter().enumerate() {
            assert!(!ptr.is_null(), "Anchor at index {} is null", i);
        }
    }
}
