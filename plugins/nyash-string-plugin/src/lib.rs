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
use hako_abi_impl::{define_instance_storage, with_instance};

const OK: i32 = 0;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
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
    match method_id {
        M_BIRTH => {
            let init = read_arg_string(args, args_len, 0).unwrap_or_default();
            let id = allocate_instance_id();
            match store_instance(id, StrInstance { s: init }) {
                Ok(()) => write_tlv_handle(TYPE_ID_STRING, id, result, result_len),
                Err(code) => code,
            }
        }
        M_FINI => {
            if remove_instance(instance_id).is_some() {
                OK
            } else {
                E_HANDLE
            }
        }
        M_IS_EMPTY => match with_instance!(instance_id, |inst: &StrInstance| inst.s.is_empty()) {
            Ok(is_empty) => write_tlv_bool(is_empty, result, result_len),
            Err(code) => code,
        },
        M_LENGTH => match with_instance!(instance_id, |inst: &StrInstance| inst.s.len()) {
            Ok(len) => write_tlv_i64(len as i64, result, result_len),
            Err(code) => code,
        },
        M_SUBSTRING => {
            let start = read_arg_i64(args, args_len, 0).unwrap_or(0);
            let end = read_arg_i64(args, args_len, 1).unwrap_or(i64::MAX);
            match with_instance!(instance_id, |inst: &StrInstance| {
                hako_core_string::substring_bytes(&inst.s, start, end)
            }) {
                Ok(sub) => write_tlv_string(&sub, result, result_len),
                Err(code) => code,
            }
        }
        M_INDEX_OF => {
            let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
            let from = read_arg_i64(args, args_len, 1).unwrap_or(0);
            match with_instance!(instance_id, |inst: &StrInstance| {
                hako_core_string::index_of(&inst.s, &needle, from)
            }) {
                Ok(idx) => write_tlv_i64(idx, result, result_len),
                Err(code) => code,
            }
        }
        M_LAST_INDEX_OF => {
            let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
            let from = read_arg_i64(args, args_len, 1).unwrap_or(i64::MAX);
            match with_instance!(instance_id, |inst: &StrInstance| {
                hako_core_string::last_index_of(&inst.s, &needle, from)
            }) {
                Ok(idx) => write_tlv_i64(idx, result, result_len),
                Err(code) => code,
            }
        }
        M_CHAR_AT => {
            let idx = read_arg_i64(args, args_len, 0).unwrap_or(0);
            if idx < 0 {
                return E_ARGS;
            }
            match with_instance!(instance_id, |inst: &StrInstance| {
                inst.s.chars().nth(idx as usize).map(|c| c.to_string())
            }) {
                Ok(Some(s)) => write_tlv_string(&s, result, result_len),
                Ok(None) => write_tlv_string("", result, result_len),
                Err(code) => code,
            }
        }
        M_CHAR_CODE_AT => {
            let idx = read_arg_i64(args, args_len, 0).unwrap_or(0);
            if idx < 0 {
                return E_ARGS;
            }
            match with_instance!(instance_id, |inst: &StrInstance| {
                inst.s.chars().nth(idx as usize).map(|c| c as i64)
            }) {
                Ok(Some(codepoint)) => write_tlv_i64(codepoint, result, result_len),
                Ok(None) => E_ARGS,
                Err(code) => code,
            }
        }
        M_TO_UTF8 => match with_instance!(instance_id, |inst: &StrInstance| inst.s.clone()) {
            Ok(value) => write_tlv_string(&value, result, result_len),
            Err(code) => code,
        },
        M_CONCAT => {
            let rhs = if let Some((type_id, other_id)) = read_arg_handle(args, args_len, 0) {
                if type_id != TYPE_ID_STRING {
                    return E_TYPE;
                }
                match with_instance!(other_id, |inst: &StrInstance| inst.s.clone()) {
                    Ok(s) => s,
                    Err(code) => return code,
                }
            } else if let Some(s) = read_arg_string(args, args_len, 0) {
                s
            } else {
                return E_ARGS;
            };

            let base = match with_instance!(instance_id, |inst: &StrInstance| inst.s.clone()) {
                Ok(s) => s,
                Err(code) => return code,
            };
            let mut merged = base;
            merged.push_str(&rhs);

            let new_id = allocate_instance_id();
            match store_instance(new_id, StrInstance { s: merged }) {
                Ok(()) => write_tlv_handle(TYPE_ID_STRING, new_id, result, result_len),
                Err(code) => code,
            }
        }
        _ => E_METHOD,
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
