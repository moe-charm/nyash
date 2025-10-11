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

// Use shared ABI implementation for TLV codec
use hako_abi_impl::tlv::{
    read_arg_i64, read_arg_string, read_arg_handle, read_arg_host_handle,
    write_tlv_i64, write_tlv_string, write_tlv_handle, write_tlv_host_handle
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

                // Extract slice data with limited lock scope to avoid deadlock
                let slice_data = {
                    let map = match INSTANCES.lock() {
                        Ok(m) => m,
                        Err(_) => {
                            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                                eprintln!("[array-plugin] SLICE lock failed");
                            }
                            return NYB_E_PLUGIN_ERROR;
                        }
                    };
                    let inst = match map.get(&instance_id) {
                        Some(i) => i,
                        None => return NYB_E_INVALID_HANDLE,
                    };

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

                    inst.data[i0..i1].to_vec()
                }; // Lock released here

                // Create new instance with separate lock (no deadlock)
                let new_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                {
                    let mut mapw = match INSTANCES.lock() {
                        Ok(m) => m,
                        Err(_) => return NYB_E_PLUGIN_ERROR,
                    };
                    mapw.insert(new_id, ArrayInstance { data: slice_data });
                }

                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[array-plugin] SLICE created plugin instance={}",
                        new_id
                    );
                }
                return write_tlv_handle(TYPE_ID_ARRAY, new_id, result, result_len);
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

// write_tlv_handle, write_tlv_i64, write_tlv_string are now provided by hako_abi_impl::tlv (imported at top)

fn write_tlv_value(val: &ArrayValue, result: *mut u8, result_len: *mut usize) -> i32 {
    match val {
        ArrayValue::I64(n) => write_tlv_i64(*n, result, result_len),
        ArrayValue::Str(s) => write_tlv_string(s, result, result_len),
        ArrayValue::Handle(t, id) => write_tlv_handle(*t, *id, result, result_len),
        ArrayValue::Host(h) => write_tlv_host_handle(*h, result, result_len),
    }
}

// read_arg_i64, read_arg_string, read_arg_handle, read_arg_host_handle are now provided by hako_abi_impl::tlv (imported at top)

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

// write_tlv_host_handle is now provided by hako_abi_impl::tlv (imported at top)

// Unused helper functions removed: preflight, build_tlv_i64_i64
