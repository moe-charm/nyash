// host_handle_router/mod.rs — Staged extraction point for HostHandle routing
// Responsibility: Provide a thin facade for host handle operations.

#![allow(dead_code)]

// TEMPORARILY: Provide stub for route_slot to fix build
// Re-enable router module after resolving HostHandle/VMValue type mismatches

/// Stub implementation of route_slot
/// Returns -999 (unimplemented) for all calls
/// TODO: Implement actual routing logic or migrate back to host_api
#[allow(clippy::too_many_arguments)]
pub fn route_slot(
    _handle: u64,
    _selector_id: u64,
    _args_ptr: *const u8,
    _args_len: usize,
    _out_ptr: *mut u8,
    _out_len: *mut usize,
) -> i32 {
    // Stub: return error code for unimplemented
    -999
}

/*
pub mod router {
    use crate::runtime::host_api; // temporary: route back to existing APIs

    pub fn call_method(receiver: &host_api::HostHandle, method: &str, args: &[host_api::VMValue]) -> Result<host_api::VMValue, String> {
        // TODO: move logic from host_api into this module gradually.
        host_api::call_method(receiver, method, args)
    }

    pub fn new_box(name: &str, args: &[host_api::VMValue]) -> Result<host_api::HostHandle, String> {
        host_api::new_box(name, args)
    }
}
*/
