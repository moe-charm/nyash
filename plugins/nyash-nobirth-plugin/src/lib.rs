//! Nyash NoBirthBox Plugin — minimal TypeBox that does not implement birth()

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::write_tlv_string;

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
    if name.is_null() {
        return 0;
    }
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

// Error/status codes
const OK: i32 = 0;
const E_METHOD: i32 = -3;

// TLV functions (write_tlv_result, write_tlv_string) now imported from hako_abi_impl

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
