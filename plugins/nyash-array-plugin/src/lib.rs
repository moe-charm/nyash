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
const METHOD_SLICE: u32 = 5; // args: i64 start, i64 end -> returns PluginHandle(ArrayBox)
const METHOD_FINI: u32 = u32::MAX; // destructor

// Assign a unique type_id for ArrayBox (as declared in nyash.toml)
const TYPE_ID_ARRAY: u32 = 12;

// ===== Instance state (PoC: store i64 values only) =====
#[derive(Clone)]
enum ArrayValue {
    I64(i64),
    Str(String),
    Handle(u32, u32),
    Host(u64),
}

struct ArrayInstance {
    data: Vec<ArrayValue>,
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
        "len" | "length" | "size" => METHOD_LENGTH,
        "get" => METHOD_GET,
        "set" => METHOD_SET,
        "push" => METHOD_PUSH,
        "slice" => METHOD_SLICE,
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
            METHOD_SLICE => {
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[array-plugin] SLICE enter instance_id={} args_len={}",
                        instance_id, args_len
                    );
                }
                // Read arguments
                let start = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                let end = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };

                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let len = inst.data.len() as i64;
                        let mut i0 = if start < 0 { 0 } else { start.min(len) } as usize;
                        let mut i1 = if end < 0 {
                            len as usize
                        } else {
                            end.max(0).min(len) as usize
                        };
                        if i0 > i1 {
                            i0 = i1;
                        }

                        if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[array-plugin] SLICE i0={} i1={} len={} (start={},end={})",
                                i0, i1, len, start, end
                            );
                        }

                        // Stage selector removed (host-handle path disabled); always return plugin handle
                        let new_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut mapw) = INSTANCES.lock() {
                            mapw.insert(
                                new_id,
                                ArrayInstance {
                                    data: inst.data[i0..i1].to_vec(),
                                },
                            );
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                        if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[array-plugin] SLICE created plugin instance={} items={}",
                                new_id,
                                i1 - i0
                            );
                        }
                        return write_tlv_handle(TYPE_ID_ARRAY, new_id, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_BIRTH => {
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[array-plugin] BIRTH enter");
                }
                // Create new ArrayBox instance and return TLV(handle: tag=8, payload=type_id(4)+instance_id(4))
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(id, ArrayInstance { data: Vec::new() });
                } else {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[array-plugin] BIRTH writing handle len ptr={:?} len_before={}",
                        result_len,
                        unsafe { *result_len }
                    );
                }
                {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!(
                            "[array-plugin] BIRTH new id={} type_id={}",
                            id, TYPE_ID_ARRAY
                        );
                    }
                    write_tlv_handle(TYPE_ID_ARRAY, id, result, result_len)
                }
            }
            METHOD_LENGTH => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
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
                        return write_tlv_value(&inst.data[i], result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_SET => {
                let idx = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                let val = match read_arg_value(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        match hako_core_array::classify_set_index(inst.data.len(), idx) {
                            hako_core_array::SetIndex::Replace(i) => inst.data[i] = val,
                            hako_core_array::SetIndex::Append => inst.data.push(val),
                            hako_core_array::SetIndex::Oob => return NYB_E_INVALID_ARGS,
                        }
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }
                } else {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_PUSH => {
                let val = match read_arg_value(args, args_len, 0) {
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
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[array-plugin] BIRTH lock failed");
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

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
    if type_id != TYPE_ID_ARRAY {
        return NYB_E_INVALID_TYPE;
    }
    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
        eprintln!(
            "[array-plugin] nyash_plugin_invoke dispatch type={} method={} instance={}",
            type_id, method_id, instance_id
        );
    }
    array_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}

#[no_mangle]
#[used]
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

fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len)
}

fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

fn write_tlv_value(val: &ArrayValue, result: *mut u8, result_len: *mut usize) -> i32 {
    match val {
        ArrayValue::I64(n) => write_tlv_i64(*n, result, result_len),
        ArrayValue::Str(s) => write_tlv_string(s, result, result_len),
        ArrayValue::Handle(t, id) => write_tlv_handle(*t, *id, result, result_len),
        ArrayValue::Host(h) => write_tlv_host_handle(*h, result, result_len),
    }
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
                let s = &buf[off + 4..off + 4 + size];
                return Some(String::from_utf8_lossy(s).to_string());
            } else {
                return None;
            }
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
            if tag == 8 && size == 8 {
                let mut t = [0u8; 4];
                t.copy_from_slice(&buf[off + 4..off + 8]);
                let mut id = [0u8; 4];
                id.copy_from_slice(&buf[off + 8..off + 12]);
                return Some((u32::from_le_bytes(t), u32::from_le_bytes(id)));
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

fn read_arg_host_handle(args: *const u8, args_len: usize, n: usize) -> Option<u64> {
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
            if tag == 9 && size == 8 {
                let mut h = [0u8; 8];
                h.copy_from_slice(&buf[off + 4..off + 12]);
                return Some(u64::from_le_bytes(h));
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

fn read_arg_value(args: *const u8, args_len: usize, n: usize) -> Option<ArrayValue> {
    if let Some(v) = read_arg_i64(args, args_len, n) {
        return Some(ArrayValue::I64(v));
    }
    if let Some((t, id)) = read_arg_handle(args, args_len, n) {
        return Some(ArrayValue::Handle(t, id));
    }
    if let Some(h) = read_arg_host_handle(args, args_len, n) {
        return Some(ArrayValue::Host(h));
    }
    if let Some(s) = read_arg_string(args, args_len, n) {
        return Some(ArrayValue::Str(s));
    }
    None
}

// ===== Helper functions for HostHandle implementation =====

/// Build TLV with two i64 arguments (for Array.set calls)
fn build_tlv_i64_i64(idx: i64, value: i64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + 8);
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: i64 value (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&value.to_le_bytes());
    buf
}

/// Write HostHandle (tag=9) return value
fn write_tlv_host_handle(handle_id: u64, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8);
    // header: version=1, argc=1
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    // tag=9 (HostHandle), size=8
    buf.push(9u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&handle_id.to_le_bytes());
    unsafe {
        let need = buf.len();
        if result.is_null() || *result_len < need {
            *result_len = need;
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, need);
        *result_len = need;
    }
    NYB_SUCCESS
}
