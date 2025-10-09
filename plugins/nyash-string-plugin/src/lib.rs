//! Nyash StringBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), length(1), is_empty(2), charCodeAt(3), fini(u32::MAX)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

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

static INST: Lazy<Mutex<HashMap<u32, StrInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// legacy v1 abi/init removed

/* legacy v1 entry removed
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
    if type_id != TYPE_ID_STRING {
        return E_TYPE;
    }
    unsafe {
        match method_id {
            M_BIRTH => {
                if result_len.is_null() {
                    return E_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                // Optional init from first arg (String/Bytes)
                let init = read_arg_string(args, args_len, 0).unwrap_or_else(|| String::new());
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, StrInstance { s: init });
                } else {
                    return E_PLUGIN;
                }
                let b = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
                *result_len = 4;
                OK
            }
            M_FINI => {
                if let Ok(mut m) = INST.lock() {
                    m.remove(&instance_id);
                    OK
                } else {
                    E_PLUGIN
                }
            }
            M_LENGTH => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_i64(inst.s.len() as i64, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
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
            M_CHAR_CODE_AT => {
                let idx = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                if idx < 0 {
                    return E_ARGS;
                }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        // Interpret index as char-index into Unicode scalar values
                        let i = idx as usize;
                        let ch_opt = inst.s.chars().nth(i);
                        let code = ch_opt.map(|c| c as u32 as i64).unwrap_or(0);
                        return write_tlv_i64(code, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_CONCAT => {
                // Accept either Handle(tag=8) to another StringBox, or String/Bytes payload
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
            M_FROM_UTF8 => {
                // Create new instance from UTF-8 (accept String/Bytes)
                let s = if let Some(s) = read_arg_string(args, args_len, 0) {
                    s
                } else {
                    return E_ARGS;
                };
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, StrInstance { s });
                } else {
                    return E_PLUGIN;
                }
                return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
            }
            M_FROM_UTF8 => {
                // Create new instance from UTF-8 (accept String/Bytes)
                let s = if let Some(s) = read_arg_string(args, args_len, 0) {
                    s
                } else {
                    return E_ARGS;
                };
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, StrInstance { s });
                } else {
                    return E_PLUGIN;
                }
                return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
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
            _ => E_METHOD,
        }
    }
}
*/

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
                        let sref = &inst.s;
                        let len = sref.len() as i64;
                        let i0 = start.max(0).min(len) as usize;
                        let i1 = end.max(0).min(len) as usize;
                        if i0 > i1 { return write_tlv_string("", result, result_len); }
                        let bytes = sref.as_bytes();
                        let sub = String::from_utf8_lossy(&bytes[i0..i1]).to_string();
                        return write_tlv_string(&sub, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            M_INDEX_OF => {
                // args: sub(String), from(i64 optional)
                let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
                let from = read_arg_i64(args, args_len, 1).unwrap_or(0);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let sref = &inst.s;
                        if needle.is_empty() { return write_tlv_i64(0, result, result_len); }
                        let start = from.max(0) as usize;
                        if start >= sref.len() { return write_tlv_i64(-1, result, result_len); }
                        let idx = sref[start..].find(&needle).map(|i| (start + i) as i64).unwrap_or(-1);
                        return write_tlv_i64(idx, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            M_LAST_INDEX_OF => {
                // args: sub(String), from(i64 optional)
                let needle = read_arg_string(args, args_len, 0).unwrap_or_default();
                let from = read_arg_i64(args, args_len, 1).unwrap_or(i64::MAX);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let sref = &inst.s;
                        if needle.is_empty() {
                            let pos = (sref.len() as i64).min(from.max(0));
                            return write_tlv_i64(pos, result, result_len);
                        }
                        let bound = from.max(0) as usize;
                        let bound = bound.min(sref.len());
                        let slice = &sref[..bound];
                        let idx = slice.rfind(&needle).map(|i| i as i64).unwrap_or(-1);
                        return write_tlv_i64(idx, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            M_CHAR_AT => {
                // args: idx(i64) -> returns String (one unicode scalar value)
                let idx = read_arg_i64(args, args_len, 0).unwrap_or(0);
                if idx < 0 { return E_ARGS; }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let ch_opt = inst.s.chars().nth(idx as usize);
                        let s = ch_opt.map(|c| c.to_string()).unwrap_or_default();
                        return write_tlv_string(&s, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
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
pub static nyash_typebox_StringBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"StringBox\0".as_ptr() as *const c_char,
    resolve: Some(string_resolve),
    invoke_id: Some(string_invoke_id),
    capabilities: 0,
};

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
        return E_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
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
            return E_SHORT;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    OK
}
fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len)
}
fn write_tlv_bool(v: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(1u8, &[if v { 1u8 } else { 0u8 }])], result, result_len)
}
fn write_tlv_handle(
    type_id: u32,
    instance_id: u32,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&type_id.to_le_bytes());
    payload.extend_from_slice(&instance_id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], result, result_len)
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}
fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag != 3 || size != 8 {
                return None;
            }
            let mut b = [0u8; 8];
            b.copy_from_slice(&buf[off + 4..off + 12]);
            return Some(i64::from_le_bytes(b));
        }
        off += 4 + size;
    }
    None
}
fn read_arg_handle(args: *const u8, args_len: usize, n: usize) -> Option<(u32, u32)> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag != 8 || size != 8 {
                return None;
            }
            let mut t = [0u8; 4];
            t.copy_from_slice(&buf[off + 4..off + 8]);
            let mut id = [0u8; 4];
            id.copy_from_slice(&buf[off + 8..off + 12]);
            return Some((u32::from_le_bytes(t), u32::from_le_bytes(id)));
        }
        off += 4 + size;
    }
    None
}
fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == 6 || tag == 7 {
                let s = String::from_utf8_lossy(&buf[off + 4..off + 4 + size]).to_string();
                return Some(s);
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}
