//! Host API for Plugin Support
//!
//! ## Problem Solved
//! Originally, host API functions lived in a dedicated kernel crate (`nyash_kernel`, now `hako_kernel`), but:
//! 1. hako_kernel → nyash-rust created circular dependency
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
use crate::runtime::host_handles;

use crate::runtime::host_api::nyrt_host_call_slot;


// ========== Array API ==========

/// Create a new ArrayBox and return its host handle
///
/// This is the critical function that was being stripped by LTO.
/// Plugins use this via dlsym() to create arrays.
#[no_mangle]
pub extern "C" fn nyash_array_new_h() -> i64 {
    crate::runtime::provider_box::ensure_loaded(None);
    match crate::runtime::provider_box::new_box("ArrayBox", &[]) {
        Ok(b) => {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[host-api] nyash_array_new_h constructed {}", b.type_name());
            }
            host_handles::to_handle_box(b) as i64
        }
        Err(e) => {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[host-api] nyash_array_new_h failed: {:?}", e);
            }
            -1
        }
    }
}

// NOTE: Other host API functions (get_h, set_h, etc.) remain in hako_kernel
// for now. They will be migrated in a future refactoring.
// The critical fix is nyash_array_new_h, which plugins actually call.

// --- Anchor to prevent LTO from stripping the symbol ---
// Some linkers may drop even #[no_mangle] extern "C" fns if they appear unused
// in the main binary. Keep a #[used] static that references the function pointer
// so it is always retained and exported (together with -rdynamic).
#[used]
#[no_mangle]
pub static NYASH_HOST_API_KEEPERS: [extern "C" fn() -> i64; 1] = [nyash_array_new_h];



// Keep the slot-dispatch entry point as well (used by plugins to call host boxes)
#[used]
#[no_mangle]
pub static NYASH_HOST_API_KEEP_SLOT: extern "C" fn(
    u64,
    u64,
    *const u8,
    usize,
    *mut u8,
    *mut usize,
) -> i32 = nyrt_host_call_slot;
