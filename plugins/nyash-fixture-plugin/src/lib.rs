//! Nyash FixtureBox Plugin — Minimal stable fixture for tests
//!
//! ## Phase 2-1: Instance Manager Macros Applied
//! - ✅ 3 lines (INSTANCES + INSTANCE_COUNTER) → 1 line (define_instance_storage!)
//! - ✅ 2 lock() blocks in birth()/fini() → macro helpers

// Import shared TLV codec + instance manager macros from hako_abi_impl
use hako_abi_impl::tlv::{read_arg_string, write_tlv_string};
use hako_abi_impl::define_instance_storage;

// ===== Error Codes (BID-1 alignment) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Method IDs =====
const METHOD_BIRTH: u32 = 0; // constructor
const METHOD_ECHO: u32 = 1; // echo string arg
const METHOD_GET: u32 = 2; // returns a constant string
const METHOD_FINI: u32 = u32::MAX; // destructor

// Assign a unique type_id for FixtureBox (avoid collisions with known IDs)
const TYPE_ID_FIXTURE: u32 = 101;

// ===== Instance state (optional) =====
struct FixtureInstance {
    alive: bool,
}

// Instance storage (replaces 3 lines of boilerplate)
define_instance_storage!(FixtureInstance);

// ===== v1 legacy entry (kept for loader shim compatibility) =====
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if type_id != TYPE_ID_FIXTURE {
        return NYB_E_INVALID_TYPE;
    }
    unsafe { dispatch(method_id, instance_id, args, args_len, result, result_len) }
}

// ===== v2 TypeBox FFI =====
#[allow(non_camel_case_types)]
type InvokeFn = extern "C" fn(
    u32, /* instance_id */
    u32, /* method_id */
    *const u8,
    usize,
    *mut u8,
    *mut usize,
) -> i32;

#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<InvokeFn>,
    pub capabilities: u64,
}

unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn fixture_resolve(name: *const std::os::raw::c_char) -> u32 {
    unsafe {
        if name.is_null() {
            return 0;
        }
        let s = std::ffi::CStr::from_ptr(name).to_string_lossy();
        match s.as_ref() {
            "birth" => METHOD_BIRTH,
            "echo" => METHOD_ECHO,
            "get" => METHOD_GET,
            "fini" => METHOD_FINI,
            _ => 0,
        }
    }
}

extern "C" fn fixture_invoke(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { dispatch(method_id, instance_id, args, args_len, result, result_len) }
}

#[no_mangle]
pub static nyash_typebox_FixtureBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x5459_4258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"FixtureBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(fixture_resolve),
    invoke_id: Some(fixture_invoke),
    capabilities: 0,
};

// ===== Shared dispatch and helpers =====
unsafe fn dispatch(
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => birth(result, result_len),
        METHOD_FINI => fini(instance_id),
        METHOD_ECHO => echo(args, args_len, result, result_len),
        METHOD_GET => write_tlv_string("ok", result, result_len),
        _ => NYB_E_INVALID_METHOD,
    }
}

unsafe fn birth(result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    if preflight(result, result_len, 4) {
        return NYB_E_SHORT_BUFFER;
    }

    let id = allocate_instance_id();
    if let Err(e) = store_instance(id, FixtureInstance { alive: true }) {
        return e;
    }

    let bytes = id.to_le_bytes();
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
    *result_len = 4;
    NYB_SUCCESS
}

unsafe fn fini(instance_id: u32) -> i32 {
    remove_instance(instance_id);
    NYB_SUCCESS
}

unsafe fn echo(args: *const u8, args_len: usize, result: *mut u8, result_len: *mut usize) -> i32 {
    // Use shared TLV codec to read string argument
    let s = match read_arg_string(args, args_len, 0) {
        Some(s) => s,
        None => return NYB_E_INVALID_ARGS,
    };
    write_tlv_string(&s, result, result_len)
}

// Removed duplicate TLV functions - now using shared codec from hako_abi_impl:
// - write_tlv_result() -> use hako_abi_impl::tlv::write_tlv_string
// - write_tlv_str() -> use hako_abi_impl::tlv::write_tlv_string
// - Manual TLV parsing in echo() -> use hako_abi_impl::tlv::read_arg_string

fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() {
            return false;
        }
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return true;
        }
    }
    false
}
