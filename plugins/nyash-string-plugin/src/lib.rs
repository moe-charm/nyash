//! Nyash StringBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), length(1), is_empty(2), charCodeAt(3), fini(u32::MAX)
//!
//! ## Phase 2-1: Instance Manager Macros Applied

use std::ffi::CStr;
use std::os::raw::c_char;

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{
    read_arg_handle, read_arg_i64, read_arg_string, write_tlv_bool, write_tlv_handle,
    write_tlv_i64, write_tlv_string,
};

// Import instance manager macros
use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};

const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

const M_BIRTH: u32 = 0;
const M_LENGTH: u32 = 1; // also resolves for size
const M_IS_EMPTY: u32 = 2;
const M_CHAR_CODE_AT: u32 = 3;
const M_CONCAT: u32 = 4; // concat(other: String|Handle) -> Handle(new)
const M_FROM_UTF8: u32 = 5; // fromUtf8(data: String|Bytes) -> Handle(new)
const M_TO_UTF8: u32 = 6; // toUtf8() -> String
const M_SUBSTRING: u32 = 7; // substring(start,end) -> String
const M_INDEX_OF: u32 = 8; // indexOf(sub[, from]) -> i64
const M_LAST_INDEX_OF: u32 = 9; // lastIndexOf(sub[, from]) -> i64
const M_CHAR_AT: u32 = 10; // charAt(idx) -> String (1-char)
const M_FINI: u32 = u32::MAX;

const TYPE_ID_STRING: u32 = 13; // Match hako.toml/nyash.toml canonical type_id

struct StrInstance {
    s: String,
}

define_instance_storage!(StrInstance);

// legacy v1 abi/init removed

// ===== TypeBox FFI v2 only - no v1 compatibility =====
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

extern "C" fn string_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => M_BIRTH,
        "len" | "length" | "size" => M_LENGTH,
        "isEmpty" => M_IS_EMPTY,
        "charCodeAt" => M_CHAR_CODE_AT,
        "charAt" => M_CHAR_AT,
        "substring" => M_SUBSTRING,
        "indexOf" => M_INDEX_OF,
        "lastIndexOf" => M_LAST_INDEX_OF,
        "concat" => M_CONCAT,
        "fromUtf8" => M_FROM_UTF8,
        "toUtf8" | "toString" => M_TO_UTF8, // Map toString to toUtf8
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn string_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            M_BIRTH => {
                // Create new StringBox instance
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let init = read_arg_string(args, args_len, 0).unwrap_or_else(|| String::new());
                eprintln!("[StringBox] M_BIRTH called: id={}, init={:?}", id, init);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, StrInstance { s: init.clone() });
                    eprintln!("[StringBox] Inserted into INST map");
                    return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
                } else {
                    return E_PLUGIN;
                }
            }
            M_FINI => {
                // Destroy StringBox instance
                if let Ok(mut m) = INST.lock() {
                    m.remove(&instance_id);
                    return OK;
                } else {
                    return E_PLUGIN;
                }
            }
            M_IS_EMPTY => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_bool(inst.s.is_empty(), result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_LENGTH => {
                eprintln!("[StringBox] M_LENGTH called: instance_id={}", instance_id);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let len = inst.s.len();
                        eprintln!(
                            "[StringBox] Found instance, string={:?}, len={}",
                            inst.s, len
                        );
                        return write_tlv_i64(len as i64, result, result_len);
                    } else {
                        eprintln!(
                            "[StringBox] Instance {} not found in INST map!",
                            instance_id
                        );
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_SUBSTRING => {
                // args: start(i64), end(i64)
                let start = read_arg_i64(args, args_len, 0).unwrap_or(0);
                let end = read_arg_i64(args, args_len, 1).unwrap_or(i64::MAX);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let sub = hako_core_string::substring_bytes(&inst.s, start, end);
                        return write_tlv_string(&sub, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_INDEX_OF => {
                // args: sub(String), from(i64 optional)
                let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
                let from = read_arg_i64(args, args_len, 1).unwrap_or(0);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let idx = hako_core_string::index_of(&inst.s, &needle, from);
                        return write_tlv_i64(idx, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_LAST_INDEX_OF => {
                // args: sub(String), from(i64 optional)
                let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
                let from = read_arg_i64(args, args_len, 1).unwrap_or(i64::MAX);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let idx = hako_core_string::last_index_of(&inst.s, &needle, from);
                        return write_tlv_i64(idx, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_CHAR_AT => {
                // args: idx(i64) -> returns String (one unicode scalar value)
                let idx = read_arg_i64(args, args_len, 0).unwrap_or(0);
                if idx < 0 {
                    return E_ARGS;
                }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let ch_opt = inst.s.chars().nth(idx as usize);
                        let s = ch_opt.map(|c| c.to_string()).unwrap_or_default();
                        return write_tlv_string(&s, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_TO_UTF8 => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_string(&inst.s, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_CONCAT => {
                // support String/Bytes or StringBox handle
                let (ok, rhs) = if let Some((t, inst)) = read_arg_handle(args, args_len, 0) {
                    if t != TYPE_ID_STRING {
                        return E_TYPE;
                    }
                    if let Ok(m) = INST.lock() {
                        if let Some(s2) = m.get(&inst) {
                            (true, s2.s.clone())
                        } else {
                            (false, String::new())
                        }
                    } else {
                        return E_PLUGIN;
                    }
                } else if let Some(s) = read_arg_string(args, args_len, 0) {
                    (true, s)
                } else {
                    (false, String::new())
                };
                if !ok {
                    return E_ARGS;
                }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let mut new_s = inst.s.clone();
                        new_s.push_str(&rhs);
                        drop(m);
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut mm) = INST.lock() {
                            mm.insert(id, StrInstance { s: new_s });
                        }
                        return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
#[used]
pub static nyash_typebox_StringBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"StringBox\0".as_ptr() as *const c_char,
    resolve: Some(string_resolve),
    invoke_id: Some(string_invoke_id),
    capabilities: 0,
};

// All TLV functions (read_arg_*, write_tlv_*) are now imported from hako_abi_impl::tlv
// Removed duplicate implementations: preflight, write_tlv_result, write_tlv_i64, write_tlv_bool,
// write_tlv_handle, write_tlv_string, read_arg_i64, read_arg_handle, read_arg_string

// Static-link per-Box invoke symbol for host static registration
#[no_mangle]
pub extern "C" fn nyash_string_plugin_invoke_static(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    string_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}
