// Host bridge helpers for TypeBox invoke (v2)

// Library-level shim signature used across the runtime (compat convenience)
pub type InvokeFn = unsafe extern "C" fn(
    u32, /* type_id (for dispatch) */
    u32, /* method_id */
    u32, /* instance_id */
    *const u8,
    usize,
    *mut u8,
    *mut usize,
) -> i32;

// Native v2 per-Box signature
pub type BoxInvokeFn = extern "C" fn(
    u32, /* instance_id */
    u32, /* method_id */
    *const u8,
    usize,
    *mut u8,
    *mut usize,
) -> i32;

// Optional Final ABI per-Box signature (Phase A skeleton)
// Note: actual plugins may use richer NyValue/NyResult structures; this alias
// is a minimal placeholder and only used for probing/log messages in Phase A.
pub type FinalInvokeFn = extern "C" fn(
    u32, /* type_id */
    u32, /* method_id */
    u32, /* instance_id */
    *const super::types::NyValueFfi,
    usize,
    *mut super::types::NyResultFfi,
) -> i32;

// Call library-level shim with a temporary output buffer
pub fn invoke_alloc(
    invoke: InvokeFn,
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    tlv_args: &[u8],
) -> (i32, usize, Vec<u8>) {
    let debug = crate::runtime::env_gate_box::debug_plugin();
    let mut cap: usize = 1024;
    let cap_max: usize = 262144;
    let mut out: Vec<u8> = vec![0u8; cap];
    let mut out_len: usize = out.len();
    loop {
        if debug { eprintln!("[host_bridge] invoke_alloc ENTER: type_id={} method_id={} instance_id={} out_len_before={} (cap={})", type_id, method_id, instance_id, out_len, cap); }
        let code = unsafe {
            invoke(
                type_id, method_id, instance_id,
                tlv_args.as_ptr(), tlv_args.len(),
                out.as_mut_ptr(), &mut out_len,
            )
        };
        let short = out_len > cap;
        if debug { eprintln!("[host_bridge] invoke_alloc EXIT: code={} out_len_after={} (cap={}) short={}", code, out_len, cap, short); }
        if short {
            let new_cap = cap_max.min(out_len.max(cap.saturating_mul(2)));
            if new_cap <= cap { return (code, out_len, out); }
            cap = new_cap;
            out = vec![0u8; cap];
            out_len = cap;
            continue;
        }
        return (code, out_len, out);
    }
}
