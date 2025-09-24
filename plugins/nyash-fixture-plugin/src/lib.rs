//! Nyash FixtureBox Plugin — Minimal stable fixture for tests

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

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

static INSTANCES: Lazy<Mutex<HashMap<u32, FixtureInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

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
        METHOD_GET => write_tlv_str("ok", result, result_len),
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
    let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut map) = INSTANCES.lock() {
        map.insert(id, FixtureInstance { alive: true });
    } else {
        return NYB_E_PLUGIN_ERROR;
    }
    let bytes = id.to_le_bytes();
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
    *result_len = 4;
    NYB_SUCCESS
}

unsafe fn fini(instance_id: u32) -> i32 {
    if let Ok(mut map) = INSTANCES.lock() {
        map.remove(&instance_id);
        NYB_SUCCESS
    } else {
        NYB_E_PLUGIN_ERROR
    }
}

unsafe fn echo(args: *const u8, args_len: usize, result: *mut u8, result_len: *mut usize) -> i32 {
    // Expect TLV with 1 argument: tag=6 (String)
    if args.is_null() || args_len < 4 {
        return NYB_E_INVALID_ARGS;
    }
    let slice = std::slice::from_raw_parts(args, args_len);
    // Minimal TLV parse: skip header (ver/argc) and verify first entry is String
    if slice.len() < 8 {
        return NYB_E_INVALID_ARGS;
    }
    if slice[0] != 1 || slice[1] != 0 { /* ver=1 little endian */ }
    // position 4.. is first entry; [tag, rsv, sz_lo, sz_hi, payload...]
    let tag = slice[4];
    if tag != 6 {
        return NYB_E_INVALID_ARGS;
    }
    let sz = u16::from_le_bytes([slice[6], slice[7]]) as usize;
    if 8 + sz > slice.len() {
        return NYB_E_INVALID_ARGS;
    }
    let payload = &slice[8..8 + sz];
    let s = match std::str::from_utf8(payload) {
        Ok(t) => t,
        Err(_) => return NYB_E_INVALID_ARGS,
    };
    write_tlv_str(s, result, result_len)
}

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
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
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

fn write_tlv_str(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

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
