//! Nyash NoBirthBox Plugin — minimal TypeBox that does not implement birth()

#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,     // 'TYBX'
    pub version: u16,     // 1
    pub struct_size: u16, // sizeof(NyashTypeBoxFfi)
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

// method ids: choose 4 for ping; fini = u32::MAX
const M_PING: u32 = 4;
const M_FINI: u32 = u32::MAX;

use std::ffi::CStr;
extern "C" fn nobirth_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() { return 0; }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        // intentionally no "birth"
        "ping" => M_PING,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn nobirth_invoke_id(
    _instance_id: u32,
    method_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            M_PING => write_tlv_string("pong", result, result_len),
            M_FINI => OK,
            _ => E_METHOD,
        }
    }
}

// minimal TLV helpers and error/status codes
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_ARGS: i32 = -4;
const E_METHOD: i32 = -3;

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    let needed = 4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>();
    let mut buf: Vec<u8> = Vec::with_capacity(needed);
    buf.extend_from_slice(&1u16.to_le_bytes()); // ver
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes()); // argc
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return E_SHORT;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    OK
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

#[no_mangle]
pub static nyash_typebox_NoBirthBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"NoBirthBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(nobirth_resolve),
    invoke_id: Some(nobirth_invoke_id),
    capabilities: 0,
};
