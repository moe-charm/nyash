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

// Call library-level shim with a temporary output buffer
pub fn invoke_alloc(
    invoke: InvokeFn,
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    tlv_args: &[u8],
) -> (i32, usize, Vec<u8>) {
    let mut out = vec![0u8; 1024];
    let mut out_len: usize = out.len();
    let code = unsafe {
        invoke(
            type_id,
            method_id,
            instance_id,
            tlv_args.as_ptr(),
            tlv_args.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    (code, out_len, out)
}
