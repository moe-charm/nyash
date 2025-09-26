//! Nyash ArrayBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), length(1), get(2), push(3), fini(u32::MAX)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// ===== Error Codes (aligned with existing plugins) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Method IDs =====
const METHOD_BIRTH: u32 = 0; // constructor -> returns instance_id (u32 LE, no TLV)
const METHOD_LENGTH: u32 = 1; // returns TLV i64
const METHOD_GET: u32 = 2; // args: i64 index -> returns TLV i64
const METHOD_PUSH: u32 = 3; // args: i64 value -> returns TLV i64 (new length)
const METHOD_SET: u32 = 4; // args: i64 index, i64 value -> returns TLV i64 (new length)
const METHOD_FINI: u32 = u32::MAX; // destructor

// Assign a unique type_id for ArrayBox (as declared in nyash.toml)
const TYPE_ID_ARRAY: u32 = 10;

// ===== Instance state (PoC: store i64 values only) =====
struct ArrayInstance {
    data: Vec<i64>,
}

static INSTANCES: Lazy<Mutex<HashMap<u32, ArrayInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

// legacy v1 entry points removed

// ===== TypeBox FFI (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,        // 'TYBX'
    pub version: u16,        // 1
    pub struct_size: u16,    // sizeof(NyashTypeBoxFfi)
    pub name: *const c_char, // C string
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn array_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "len" | "length" => METHOD_LENGTH,
        "get" => METHOD_GET,
        "set" => METHOD_SET,
        "push" => METHOD_PUSH,
        _ => 0,
    }
}

extern "C" fn array_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            METHOD_LENGTH => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_GET => {
                let idx = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if idx < 0 {
                    return NYB_E_INVALID_ARGS;
                }
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let i = idx as usize;
                        if i >= inst.data.len() {
                            return NYB_E_INVALID_ARGS;
                        }
                        return write_tlv_i64(inst.data[i], result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_SET => {
                let idx = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                let val = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if idx < 0 {
                    return NYB_E_INVALID_ARGS;
                }
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        let i = idx as usize;
                        let len = inst.data.len();
                        if i < len {
                            inst.data[i] = val;
                        } else if i == len {
                            inst.data.push(val);
                        } else {
                            return NYB_E_INVALID_ARGS;
                        }
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_PUSH => {
                let val = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data.push(val);
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_ArrayBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ArrayBox\0".as_ptr() as *const c_char,
    resolve: Some(array_resolve),
    invoke_id: Some(array_invoke_id),
    capabilities: 0,
};

// ===== Minimal TLV helpers (compatible with host expectations) =====
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

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes()); // argc
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len)
}

/// Read nth TLV argument as i64 (tag 3)
fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // skip header
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let _rsv = buf[off + 1];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag != 3 || size != 8 {
                return None;
            }
            let mut b = [0u8; 8];
            b.copy_from_slice(&buf[off + 4..off + 4 + 8]);
            return Some(i64::from_le_bytes(b));
        }
        off += 4 + size;
    }
    None
}
