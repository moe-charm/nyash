//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), size(1), get(2), has(3), set(4), fini(u32::MAX)
//! Extension: support both i64 and UTF-8 string keys; values remain i64.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// Error codes
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// Methods
const METHOD_BIRTH: u32 = 0;
const METHOD_SIZE: u32 = 1;
const METHOD_GET: u32 = 2; // args: i64 key -> TLV i64
const METHOD_HAS: u32 = 3; // args: i64 key -> TLV bool
const METHOD_SET: u32 = 4; // args: key(int|string), value(i64) -> TLV i64 (size)
const METHOD_REMOVE: u32 = 6; // args: key(int|string) -> TLV bool (removed)
const METHOD_CLEAR: u32 = 7; // args: () -> TLV i64 (size after clear=0)
const METHOD_KEYS_S: u32 = 8; // args: () -> TLV string (newline-joined keys)
const METHOD_GET_OR: u32 = 9; // args: key(int|string), default(i64) -> TLV i64
const METHOD_FINI: u32 = u32::MAX;
// Extended string-key methods
const METHOD_SET_STR: u32 = 10; // setS(name: string, val: i64) -> i64(size)
const METHOD_GET_STR: u32 = 11; // getS(name: string) -> i64
const METHOD_HAS_STR: u32 = 12; // hasS(name: string) -> bool
const METHOD_VALUES_S: u32 = 13; // valuesStr() -> string (newline-joined)
const METHOD_TO_JSON: u32 = 14; // toJson() -> string

// Type id (nyash.toml に合わせる)
const TYPE_ID_MAP: u32 = 11;

struct MapInstance {
    data_i64: HashMap<i64, i64>,
    data_str: HashMap<String, i64>,
}
static INSTANCES: Lazy<Mutex<HashMap<u32, MapInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

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
    if type_id != TYPE_ID_MAP {
        return NYB_E_INVALID_TYPE;
    }
    unsafe {
        match method_id {
            METHOD_BIRTH => {
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return NYB_E_SHORT_BUFFER;
                }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        id,
                        MapInstance {
                            data_i64: HashMap::new(),
                            data_str: HashMap::new(),
                        },
                    );
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
                std::ptr::copy_nonoverlapping(id.to_le_bytes().as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                if let Ok(mut map) = INSTANCES.lock() {
                    map.remove(&instance_id);
                    NYB_SUCCESS
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_SIZE => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        write_tlv_i64(sz as i64, result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET => {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            match inst.data_i64.get(&ik).copied() {
                                Some(v) => write_tlv_i64(v, result, result_len),
                                None => NYB_E_INVALID_ARGS,
                            }
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            match inst.data_str.get(&sk).copied() {
                                Some(v) => write_tlv_i64(v, result, result_len),
                                None => NYB_E_INVALID_ARGS,
                            }
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_HAS => {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            write_tlv_bool(inst.data_i64.contains_key(&ik), result, result_len)
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            write_tlv_bool(inst.data_str.contains_key(&sk), result, result_len)
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_SET => {
                // value は i64 限定
                let val = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_i64.insert(ik, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_str.insert(sk, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_REMOVE => {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        // try int key
                        if let Some(ik) = read_arg_i64(args, args_len, 0) {
                            return write_tlv_bool(
                                inst.data_i64.remove(&ik).is_some(),
                                result,
                                result_len,
                            );
                        }
                        // try string key
                        if let Some(sk) = read_arg_string(args, args_len, 0) {
                            return write_tlv_bool(
                                inst.data_str.remove(&sk).is_some(),
                                result,
                                result_len,
                            );
                        }
                        NYB_E_INVALID_ARGS
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_CLEAR => {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_i64.clear();
                        inst.data_str.clear();
                        return write_tlv_i64(0, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_KEYS_S => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let mut keys: Vec<String> =
                            Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                        for k in inst.data_i64.keys() {
                            keys.push(k.to_string());
                        }
                        for k in inst.data_str.keys() {
                            keys.push(k.clone());
                        }
                        keys.sort();
                        let out = keys.join("\n");
                        return write_tlv_string(&out, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET_OR => {
                let defv = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        // prefer exact match, else default
                        if let Some(ik) = read_arg_i64(args, args_len, 0) {
                            let v = inst.data_i64.get(&ik).copied().unwrap_or(defv);
                            return write_tlv_i64(v, result, result_len);
                        }
                        if let Some(sk) = read_arg_string(args, args_len, 0) {
                            let v = inst.data_str.get(&sk).copied().unwrap_or(defv);
                            return write_tlv_i64(v, result, result_len);
                        }
                        NYB_E_INVALID_ARGS
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_SET_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                let val = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_str.insert(key, val);
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        return write_tlv_i64(sz as i64, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_str.get(&key).copied() {
                            Some(v) => write_tlv_i64(v, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_HAS_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(inst.data_str.contains_key(&key), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_VALUES_S => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let mut vals: Vec<String> =
                            Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                        for v in inst.data_i64.values() {
                            vals.push(v.to_string());
                        }
                        for v in inst.data_str.values() {
                            vals.push(v.to_string());
                        }
                        let out = vals.join("\n");
                        return write_tlv_string(&out, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_TO_JSON => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let mut s = String::from("{");
                        let mut first = true;
                        for (k, v) in inst.data_str.iter() {
                            if !first {
                                s.push(',');
                            }
                            first = false;
                            // JSON string key
                            s.push('"');
                            s.push_str(&escape_json(k));
                            s.push_str("\": ");
                            s.push_str(&v.to_string());
                        }
                        for (k, v) in inst.data_i64.iter() {
                            if !first {
                                s.push(',');
                            }
                            first = false;
                            // numeric key as string per JSON
                            s.push('"');
                            s.push_str(&k.to_string());
                            s.push_str("\": ");
                            s.push_str(&v.to_string());
                        }
                        s.push('}');
                        return write_tlv_string(&s, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}
*/

// ---- Nyash TypeBox (FFI minimal PoC) ----
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

extern "C" fn mapbox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "size" | "len" => METHOD_SIZE,
        "get" => METHOD_GET,
        "has" => METHOD_HAS,
        "set" => METHOD_SET,
        "getS" => METHOD_GET_STR,
        "hasS" => METHOD_HAS_STR,
        _ => 0,
    }
}

extern "C" fn mapbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_SIZE => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    let sz = inst.data_i64.len() + inst.data_str.len();
                    return write_tlv_i64(sz as i64, result, result_len);
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_GET => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_i64.get(&ik).copied() {
                            Some(v) => write_tlv_i64(v, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_str.get(&sk).copied() {
                            Some(v) => write_tlv_i64(v, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_HAS => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(inst.data_i64.contains_key(&ik), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(inst.data_str.contains_key(&sk), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_SET => {
            // key: i64 or string, value: i64
            if let Some(val) = read_arg_i64(args, args_len, 1) {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_i64.insert(ik, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_str.insert(sk, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_GET_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    match inst.data_str.get(&key).copied() {
                        Some(v) => write_tlv_i64(v, result, result_len),
                        None => NYB_E_INVALID_ARGS,
                    }
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_HAS_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    write_tlv_bool(inst.data_str.contains_key(&key), result, result_len)
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_KEYS_S => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    let mut keys: Vec<String> =
                        Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                    for k in inst.data_i64.keys() {
                        keys.push(k.to_string());
                    }
                    for k in inst.data_str.keys() {
                        keys.push(k.clone());
                    }
                    keys.sort();
                    let out = keys.join("\n");
                    return write_tlv_string(&out, result, result_len);
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_VALUES_S => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    let mut vals: Vec<String> =
                        Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                    for v in inst.data_i64.values() {
                        vals.push(v.to_string());
                    }
                    for v in inst.data_str.values() {
                        vals.push(v.to_string());
                    }
                    let out = vals.join("\n");
                    return write_tlv_string(&out, result, result_len);
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        _ => NYB_E_INVALID_METHOD,
    }
}

#[no_mangle]
pub static nyash_typebox_MapBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"MapBox\0".as_ptr() as *const c_char,
    resolve: Some(mapbox_resolve),
    invoke_id: Some(mapbox_invoke_id),
    capabilities: 0,
};

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// TLV helpers
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
fn write_tlv_bool(bv: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    let b = [if bv { 1u8 } else { 0u8 }];
    write_tlv_result(&[(1u8, &b)], result, result_len)
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
            if tag == 3 && size == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(&buf[off + 4..off + 12]);
                return Some(i64::from_le_bytes(b));
            } else if tag == 2 && size == 4 {
                let mut b = [0u8; 4];
                b.copy_from_slice(&buf[off + 4..off + 8]);
                return Some(i32::from_le_bytes(b) as i64);
            } else {
                return None;
            }
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
