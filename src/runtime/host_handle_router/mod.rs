// host_handle_router/mod.rs — Staged extraction point for HostHandle routing
// Responsibility: Provide a thin facade for host handle operations.

#![allow(dead_code)]

// TODO(HostHandle Architecture): Implement after resolving HostHandle/VMValue type mismatches
//       Blocker: Type unification between plugin and builtin boxes
//       Caller: src/runtime/host_api.rs:270 (nyrt_host_call_slot delegates here)
//       Impact: All HostHandle slot calls return -999 (unimplemented)
//       See: host_handle_router/README.md for architectural intent

/// Stub implementation of route_slot
/// Returns -999 (unimplemented) for all calls until HostHandle/VMValue unification is complete
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
