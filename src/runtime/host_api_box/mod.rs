//! HostAPIBox — Thin facade over host reverse-call C ABI
//!
//! Responsibility
//! - Provide small, safe wrappers around the raw C ABI functions exported by the host
//!   (nyrt_host_call_slot/name).
//! - Centralize selector ids (slots) used by plugins to operate on HostHandle.
//! - Offer a grow-on-demand buffer strategy caller can reuse in host-side bridges/tests.
//!
//! Notes
//! - This box does not change semantics; it wraps existing functions in `crate::runtime::host_api`.
//! - Keep it dependency-light to avoid pulling extra modules into C ABI layer.

/// Slot selector ids (by convention). Keep in sync with host_api.rs comments.
#[allow(dead_code)]
pub mod slots {
    pub const ARRAY_GET: u64 = 100;
    pub const ARRAY_SET: u64 = 101;
    pub const ARRAY_LEN: u64 = 102;

    pub const MAP_SIZE: u64 = 200;
    pub const MAP_LEN: u64 = 201;
    pub const MAP_HAS: u64 = 202;
    pub const MAP_GET: u64 = 203;
    pub const MAP_SET: u64 = 204;

    pub const STRING_LEN: u64 = 300;
}

/// Call the host slot API with a growable output buffer.
/// Returns Ok<Vec<u8>> with the full TLV payload on success, or Err(rc) on error.
pub fn call_slot_grow(handle: u64, selector_id: u64, args: &[u8]) -> Result<Vec<u8>, i32> {
    let mut cap: usize = 128; // start moderately sized
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = crate::runtime::host_api::nyrt_host_call_slot(
            handle,
            selector_id,
            if args.is_empty() { std::ptr::null() } else { args.as_ptr() },
            args.len(),
            out.as_mut_ptr(),
            &mut out_len,
        );
        if rc == 0 {
            out.truncate(out_len.min(out.len()));
            return Ok(out);
        }
        // -3 is SHORT_BUFFER in host_api::encode_out
        if rc == -3 {
            cap = cap.saturating_mul(2);
            if cap > 128 * 1024 { return Err(rc); }
            continue;
        }
        return Err(rc);
    }
}

/// Call the host name API with a growable output buffer.
/// TODO: Implement or migrate back to host_api
pub fn call_name_grow(_handle: u64, _method: &str, _args: &[u8]) -> Result<Vec<u8>, i32> {
    Err(-999) // Stub: unimplemented
}


