//! Nyash TOMLBox Plugin - minimal parse + query + toJson

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{read_arg_string, write_tlv_bool, write_tlv_string};

const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

const M_BIRTH: u32 = 0; // constructor -> instance
const M_PARSE: u32 = 1; // parse(text) -> bool
const M_GET: u32 = 2; // get(path.dot.segments) -> string (toml-display) or empty
const M_TO_JSON: u32 = 3; // toJson() -> string (JSON)
const M_FINI: u32 = u32::MAX; // fini()

const TYPE_ID_TOML: u32 = 54;

struct TomlInstance {
    value: Option<toml::Value>,
}

static INST: Lazy<Mutex<HashMap<u32, TomlInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
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
    if type_id != TYPE_ID_TOML {
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
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, TomlInstance { value: None });
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
            M_PARSE => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.value = toml::from_str::<toml::Value>(&text).ok();
                        return write_tlv_bool(inst.value.is_some(), result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_GET => {
                let path = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let mut cur = match &inst.value {
                            Some(v) => v,
                            None => {
                                return write_tlv_string("", result, result_len);
                            }
                        };
                        if !path.is_empty() {
                            for seg in path.split('.') {
                                match cur.get(seg) {
                                    Some(v) => cur = v,
                                    None => {
                                        return write_tlv_string("", result, result_len);
                                    }
                                }
                            }
                        }
                        let out = cur.to_string();
                        return write_tlv_string(&out, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_TO_JSON => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(v) = &inst.value {
                            // Convert via serde_json::Value
                            let sv = toml_to_json(v);
                            return match serde_json::to_string(&sv) {
                                Ok(s) => write_tlv_string(&s, result, result_len),
                                Err(_) => write_tlv_string("{}", result, result_len),
                            };
                        } else {
                            return write_tlv_string("{}", result, result_len);
                        }
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

// ===== TypeBox ABI v2 (resolve/invoke_id) =====
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

use std::ffi::CStr;
extern "C" fn toml_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "parse" => M_PARSE,
        "get" => M_GET,
        "toJson" | "toJSON" => M_TO_JSON,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn toml_invoke_id(
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
                if result_len.is_null() {
                    return E_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, TomlInstance { value: None });
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
            M_PARSE => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.value = toml::from_str::<toml::Value>(&text).ok();
                        return write_tlv_bool(inst.value.is_some(), result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_GET => {
                let path = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let mut cur = match &inst.value {
                            Some(v) => v,
                            None => {
                                return write_tlv_string("", result, result_len);
                            }
                        };
                        if !path.is_empty() {
                            for seg in path.split('.') {
                                match cur.get(seg) {
                                    Some(v) => cur = v,
                                    None => {
                                        return write_tlv_string("", result, result_len);
                                    }
                                }
                            }
                        }
                        return write_tlv_string(&cur.to_string(), result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_TO_JSON => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(v) = &inst.value {
                            if let Ok(s) = serde_json::to_string(v) {
                                return write_tlv_string(&s, result, result_len);
                            }
                        }
                        return write_tlv_string("{}", result, result_len);
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
pub static nyash_typebox_TOMLBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"TOMLBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(toml_resolve),
    invoke_id: Some(toml_invoke_id),
    capabilities: 0,
};

fn toml_to_json(v: &toml::Value) -> serde_json::Value {
    match v {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::from(*i),
        toml::Value::Float(f) => serde_json::Value::from(*f),
        toml::Value::Boolean(b) => serde_json::Value::from(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(toml_to_json).collect()),
        toml::Value::Table(map) => {
            let mut m = serde_json::Map::new();
            for (k, vv) in map.iter() {
                m.insert(k.clone(), toml_to_json(vv));
            }
            serde_json::Value::Object(m)
        }
    }
}

// TLV codec functions (preflight, write_tlv_*, read_arg_string) are now imported from hako_abi_impl::tlv
// This removes ~70 lines of duplicate TLV implementation code.

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
