//! Host API for Plugin Support
//!
//! ## Problem Solved
//! Originally, host API functions were in nyash_kernel crate, but:
//! 1. nyash_kernel → nyash-rust created circular dependency
//! 2. LTO removed "unused" functions even with -rdynamic
//! 3. Plugins failed with dlsym() undefined symbol errors
//!
//! ## Solution
//! Implement host API functions directly in nyash-rust to:
//! 1. Avoid circular dependencies
//! 2. Ensure functions are always linked
//! 3. Simplify the build process
//!
//! ## Host API Functions
//! Functions exposed for plugin callbacks via dlsym():
//! - Array API: nyash_array_new_h, nyash_array_get_h, etc.
//! - Map API: nyash_map_size_h, nyash_map_get_h, etc.
//! - String API: nyash_string_to_i8p_h
//! - Instance API: nyash_instance_get_field_h, etc.
//! - Future API: nyash_future_spawn_method_h

use crate::box_trait::NyashBox;
use crate::boxes::array::ArrayBox;
use crate::runtime::host_handles;

// ========== Array API ==========

/// Create a new ArrayBox and return its host handle
///
/// This is the critical function that was being stripped by LTO.
/// Plugins use this via dlsym() to create arrays.
#[no_mangle]
pub extern "C" fn nyash_array_new_h() -> i64 {
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(ArrayBox::new());
    host_handles::to_handle_arc(arc) as i64
}

// NOTE: Other host API functions (get_h, set_h, etc.) remain in nyash_kernel
// for now. They will be migrated in a future refactoring.
// The critical fix is nyash_array_new_h, which plugins actually call.
