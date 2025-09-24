//! Nyash JSON Plugin — TypeBox v2 (serde_json backend for bring-up)
//! Provides JsonDocBox / JsonNodeBox to parse and traverse JSON safely.

use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

// ---- Result codes ----
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

// ---- Method IDs ----
// JsonDocBox
const JD_BIRTH: u32 = 0;
const JD_PARSE: u32 = 1;
const JD_ROOT: u32 = 2;
const JD_ERROR: u32 = 3;
const JD_FINI: u32 = u32::MAX;

// JsonNodeBox
const JN_BIRTH: u32 = 0;
const JN_KIND: u32 = 1;
const JN_GET: u32 = 2;
const JN_SIZE: u32 = 3;
const JN_AT: u32 = 4;
const JN_STR: u32 = 5;
const JN_INT: u32 = 6;
const JN_BOOL: u32 = 7;
const JN_FINI: u32 = u32::MAX;

// ---- Type IDs (for Handle TLV) ----
const T_JSON_DOC: u32 = 70;
const T_JSON_NODE: u32 = 71;

// ---- Instances ----
#[derive(Clone)]
enum NodeRep {
    Serde(Arc<Value>),
    Yy { doc_id: u32, ptr: usize },
}

struct DocInst {
    root: Option<Arc<Value>>, // Serde provider
    doc_ptr: Option<usize>,   // Yyjson provider (opaque pointer value)
    last_err: Option<String>,
}
static DOCS: Lazy<Mutex<HashMap<u32, DocInst>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NODES: Lazy<Mutex<HashMap<u32, NodeRep>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// ---- TypeBox v2 FFI ----
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

// ---- JsonDocBox ----
extern "C" fn jsondoc_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => JD_BIRTH,
        "parse" => JD_PARSE,
        "root" => JD_ROOT,
        "error" => JD_ERROR,
        _ => 0,
    }
}

// Provider selection (serde_json vs yyjson skeleton)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ProviderKind {
    Serde,
    Yyjson,
}

fn provider_kind() -> ProviderKind {
    match std::env::var("NYASH_JSON_PROVIDER").ok().as_deref() {
        Some("yyjson") | Some("YYJSON") => ProviderKind::Yyjson,
        _ => ProviderKind::Serde,
    }
}

fn provider_parse(text: &str) -> Result<Value, String> {
    match provider_kind() {
        ProviderKind::Serde => serde_json::from_str::<Value>(text).map_err(|e| e.to_string()),
        ProviderKind::Yyjson => {
            // Skeleton phase: call into C shim to validate linkage, then fallback to serde_json
            unsafe {
                if let Ok(c) = CString::new(text.as_bytes()) {
                    let _ = nyash_json_shim_parse(c.as_ptr(), text.len());
                }
            }
            serde_json::from_str::<Value>(text).map_err(|e| e.to_string())
        }
    }
}

extern "C" {
    fn nyash_json_shim_parse(text: *const std::os::raw::c_char, len: usize) -> i32;
    fn nyjson_parse_doc(text: *const c_char, len: usize, out_err_code: *mut i32) -> *mut c_void;
    fn nyjson_doc_free(doc: *mut c_void);
    fn nyjson_doc_root(doc: *mut c_void) -> *mut c_void;
    fn nyjson_is_null(v: *mut c_void) -> i32;
    fn nyjson_is_bool(v: *mut c_void) -> i32;
    fn nyjson_is_int(v: *mut c_void) -> i32;
    fn nyjson_is_real(v: *mut c_void) -> i32;
    fn nyjson_is_str(v: *mut c_void) -> i32;
    fn nyjson_is_arr(v: *mut c_void) -> i32;
    fn nyjson_is_obj(v: *mut c_void) -> i32;
    fn nyjson_get_bool_val(v: *mut c_void) -> i32;
    fn nyjson_get_sint_val(v: *mut c_void) -> i64;
    fn nyjson_get_str_val(v: *mut c_void) -> *const c_char;
    fn nyjson_arr_size_val(v: *mut c_void) -> usize;
    fn nyjson_arr_get_val(v: *mut c_void, idx: usize) -> *mut c_void;
    fn nyjson_obj_size_val(v: *mut c_void) -> usize;
    fn nyjson_obj_get_key(v: *mut c_void, key: *const c_char) -> *mut c_void;
}

extern "C" fn jsondoc_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            JD_BIRTH => {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = DOCS.lock() {
                    m.insert(
                        id,
                        DocInst {
                            root: None,
                            doc_ptr: None,
                            last_err: None,
                        },
                    );
                } else {
                    return E_PLUGIN;
                }
                return write_u32(id, result, result_len);
            }
            JD_PARSE => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = DOCS.lock() {
                    if let Some(doc) = m.get_mut(&instance_id) {
                        match provider_kind() {
                            ProviderKind::Serde => {
                                match provider_parse(&text) {
                                    Ok(v) => {
                                        doc.root = Some(Arc::new(v));
                                        doc.doc_ptr = None;
                                        doc.last_err = None;
                                    }
                                    Err(e) => {
                                        doc.root = None;
                                        doc.doc_ptr = None;
                                        doc.last_err = Some(e.to_string());
                                    }
                                }
                                return write_tlv_void(result, result_len);
                            }
                            ProviderKind::Yyjson => {
                                let c = CString::new(text.as_bytes()).unwrap_or_default();
                                let mut ec: i32 = -1;
                                let p =
                                    nyjson_parse_doc(c.as_ptr(), text.len(), &mut ec as *mut i32);
                                if p.is_null() {
                                    doc.root = None;
                                    doc.doc_ptr = None;
                                    doc.last_err = Some(format!("E{}", ec));
                                } else {
                                    doc.root = None;
                                    doc.doc_ptr = Some(p as usize);
                                    doc.last_err = None;
                                }
                                return write_tlv_void(result, result_len);
                            }
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            JD_ROOT => {
                if let Ok(m) = DOCS.lock() {
                    if let Some(doc) = m.get(&instance_id) {
                        match provider_kind() {
                            ProviderKind::Serde => {
                                if let Some(root_arc) = doc.root.as_ref().map(|r| Arc::clone(r)) {
                                    let node_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                                    if let Ok(mut nn) = NODES.lock() {
                                        nn.insert(node_id, NodeRep::Serde(root_arc));
                                    }
                                    return write_tlv_handle(
                                        T_JSON_NODE,
                                        node_id,
                                        result,
                                        result_len,
                                    );
                                }
                                return E_PLUGIN;
                            }
                            ProviderKind::Yyjson => {
                                if let Some(dp) = doc.doc_ptr {
                                    let vp = nyjson_doc_root(dp as *mut c_void);
                                    let node_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                                    if let Ok(mut nn) = NODES.lock() {
                                        nn.insert(
                                            node_id,
                                            NodeRep::Yy {
                                                doc_id: instance_id,
                                                ptr: vp as usize,
                                            },
                                        );
                                    }
                                    return write_tlv_handle(
                                        T_JSON_NODE,
                                        node_id,
                                        result,
                                        result_len,
                                    );
                                }
                                return E_PLUGIN;
                            }
                        }
                    }
                }
                return E_PLUGIN;
            }
            JD_ERROR => {
                if let Ok(m) = DOCS.lock() {
                    if let Some(doc) = m.get(&instance_id) {
                        let s = doc.last_err.clone().unwrap_or_default();
                        return write_tlv_string(&s, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            JD_FINI => {
                if let Ok(mut m) = DOCS.lock() {
                    if let Some(mut di) = m.remove(&instance_id) {
                        if let Some(dp) = di.doc_ptr.take() {
                            nyjson_doc_free(dp as *mut c_void);
                        }
                    }
                }
                return write_tlv_void(result, result_len);
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_JsonDocBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"JsonDocBox\0".as_ptr() as *const c_char,
    resolve: Some(jsondoc_resolve),
    invoke_id: Some(jsondoc_invoke_id),
    capabilities: 0,
};

// ---- JsonNodeBox ----
extern "C" fn jsonnode_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => JN_BIRTH,
        "kind" => JN_KIND,
        "get" => JN_GET,
        "size" => JN_SIZE,
        "at" => JN_AT,
        "str" => JN_STR,
        "int" => JN_INT,
        "bool" => JN_BOOL,
        _ => 0,
    }
}

extern "C" fn jsonnode_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        let node_rep = match NODES.lock() {
            Ok(m) => match m.get(&instance_id) {
                Some(v) => v.clone(),
                None => return E_HANDLE,
            },
            Err(_) => return E_PLUGIN,
        };
        match method_id {
            JN_BIRTH => {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = NODES.lock() {
                    m.insert(id, NodeRep::Serde(Arc::new(Value::Null)));
                } else {
                    return E_PLUGIN;
                }
                return write_u32(id, result, result_len);
            }
            JN_KIND => match provider_kind() {
                ProviderKind::Serde => {
                    let k = match node_rep {
                        NodeRep::Serde(ref a) => match &**a {
                            Value::Null => "null",
                            Value::Bool(_) => "bool",
                            Value::Number(n) => {
                                if n.is_i64() {
                                    "int"
                                } else {
                                    "real"
                                }
                            }
                            Value::String(_) => "string",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                        },
                        _ => "null",
                    };
                    write_tlv_string(k, result, result_len)
                }
                ProviderKind::Yyjson => {
                    let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                        ptr as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };
                    let k = if v.is_null() {
                        "null"
                    } else if nyjson_is_obj(v) != 0 {
                        "object"
                    } else if nyjson_is_arr(v) != 0 {
                        "array"
                    } else if nyjson_is_str(v) != 0 {
                        "string"
                    } else if nyjson_is_int(v) != 0 {
                        "int"
                    } else if nyjson_is_real(v) != 0 {
                        "real"
                    } else if nyjson_is_bool(v) != 0 {
                        "bool"
                    } else {
                        "null"
                    };
                    write_tlv_string(k, result, result_len)
                }
            },
            JN_GET => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                match provider_kind() {
                    ProviderKind::Serde => {
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        if let NodeRep::Serde(ref a) = node_rep {
                            if let Value::Object(map) = &**a {
                                if let Some(child) = map.get(&key) {
                                    if let Ok(mut mm) = NODES.lock() {
                                        mm.insert(id, NodeRep::Serde(Arc::new(child.clone())));
                                    }
                                    return write_tlv_handle(T_JSON_NODE, id, result, result_len);
                                }
                            }
                        }
                        if let Ok(mut mm) = NODES.lock() {
                            mm.insert(id, NodeRep::Serde(Arc::new(Value::Null)));
                        }
                        write_tlv_handle(T_JSON_NODE, id, result, result_len)
                    }
                    ProviderKind::Yyjson => {
                        let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                            ptr as *mut c_void
                        } else {
                            std::ptr::null_mut()
                        };
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        let mut out_ptr: *mut c_void = std::ptr::null_mut();
                        if !v.is_null() && nyjson_is_obj(v) != 0 {
                            let c = CString::new(key).unwrap_or_default();
                            out_ptr = nyjson_obj_get_key(v, c.as_ptr());
                        }
                        let doc_id = if let NodeRep::Yy { doc_id, .. } = node_rep {
                            doc_id
                        } else {
                            0
                        };
                        let rep = if out_ptr.is_null() {
                            NodeRep::Yy { doc_id, ptr: 0 }
                        } else {
                            NodeRep::Yy {
                                doc_id,
                                ptr: out_ptr as usize,
                            }
                        };
                        if let Ok(mut mm) = NODES.lock() {
                            mm.insert(id, rep);
                        }
                        write_tlv_handle(T_JSON_NODE, id, result, result_len)
                    }
                }
            }
            JN_SIZE => match provider_kind() {
                ProviderKind::Serde => {
                    let n = match node_rep {
                        NodeRep::Serde(ref a) => match &**a {
                            Value::Array(a) => a.len() as i64,
                            Value::Object(o) => o.len() as i64,
                            _ => 0,
                        },
                        _ => 0,
                    };
                    write_tlv_i64(n, result, result_len)
                }
                ProviderKind::Yyjson => {
                    let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                        ptr as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };
                    let n = if !v.is_null() {
                        if nyjson_is_arr(v) != 0 {
                            nyjson_arr_size_val(v) as i64
                        } else if nyjson_is_obj(v) != 0 {
                            nyjson_obj_size_val(v) as i64
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    write_tlv_i64(n, result, result_len)
                }
            },
            JN_AT => {
                let idx = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                if idx < 0 {
                    return E_ARGS;
                }
                match provider_kind() {
                    ProviderKind::Serde => {
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        if let NodeRep::Serde(ref a) = node_rep {
                            if let Value::Array(arr) = &**a {
                                let i = idx as usize;
                                if i < arr.len() {
                                    if let Ok(mut mm) = NODES.lock() {
                                        mm.insert(id, NodeRep::Serde(Arc::new(arr[i].clone())));
                                    }
                                    return write_tlv_handle(T_JSON_NODE, id, result, result_len);
                                }
                            }
                        }
                        if let Ok(mut mm) = NODES.lock() {
                            mm.insert(id, NodeRep::Serde(Arc::new(Value::Null)));
                        }
                        write_tlv_handle(T_JSON_NODE, id, result, result_len)
                    }
                    ProviderKind::Yyjson => {
                        let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                            ptr as *mut c_void
                        } else {
                            std::ptr::null_mut()
                        };
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        let mut child: *mut c_void = std::ptr::null_mut();
                        if !v.is_null() && nyjson_is_arr(v) != 0 {
                            child = nyjson_arr_get_val(v, idx as usize);
                        }
                        let doc_id = if let NodeRep::Yy { doc_id, .. } = node_rep {
                            doc_id
                        } else {
                            0
                        };
                        let rep = if child.is_null() {
                            NodeRep::Yy { doc_id, ptr: 0 }
                        } else {
                            NodeRep::Yy {
                                doc_id,
                                ptr: child as usize,
                            }
                        };
                        if let Ok(mut mm) = NODES.lock() {
                            mm.insert(id, rep);
                        }
                        write_tlv_handle(T_JSON_NODE, id, result, result_len)
                    }
                }
            }
            JN_STR => match provider_kind() {
                ProviderKind::Serde => {
                    if let NodeRep::Serde(ref a) = node_rep {
                        match &**a {
                            Value::String(s) => write_tlv_string(s, result, result_len),
                            Value::Object(o) => {
                                if let Some(Value::String(s)) = o.get("value") {
                                    write_tlv_string(s, result, result_len)
                                } else {
                                    write_tlv_string("", result, result_len)
                                }
                            }
                            _ => write_tlv_string("", result, result_len),
                        }
                    } else {
                        write_tlv_string("", result, result_len)
                    }
                }
                ProviderKind::Yyjson => {
                    let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                        ptr as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };
                    if !v.is_null() && nyjson_is_str(v) != 0 {
                        let s = nyjson_get_str_val(v);
                        if s.is_null() {
                            write_tlv_string("", result, result_len)
                        } else {
                            let rs = CStr::from_ptr(s).to_string_lossy().to_string();
                            write_tlv_string(&rs, result, result_len)
                        }
                    } else if !v.is_null() && nyjson_is_obj(v) != 0 {
                        let key = CString::new("value").unwrap();
                        let child = nyjson_obj_get_key(v, key.as_ptr());
                        if !child.is_null() && nyjson_is_str(child) != 0 {
                            let s = nyjson_get_str_val(child);
                            if s.is_null() {
                                write_tlv_string("", result, result_len)
                            } else {
                                let rs = CStr::from_ptr(s).to_string_lossy().to_string();
                                write_tlv_string(&rs, result, result_len)
                            }
                        } else {
                            write_tlv_string("", result, result_len)
                        }
                    } else {
                        write_tlv_string("", result, result_len)
                    }
                }
            },
            JN_INT => match provider_kind() {
                ProviderKind::Serde => {
                    if let NodeRep::Serde(ref a) = node_rep {
                        match &**a {
                            Value::Number(n) => {
                                write_tlv_i64(n.as_i64().unwrap_or(0), result, result_len)
                            }
                            Value::Object(o) => {
                                if let Some(Value::Number(n)) = o.get("value") {
                                    write_tlv_i64(n.as_i64().unwrap_or(0), result, result_len)
                                } else {
                                    write_tlv_i64(0, result, result_len)
                                }
                            }
                            _ => write_tlv_i64(0, result, result_len),
                        }
                    } else {
                        write_tlv_i64(0, result, result_len)
                    }
                }
                ProviderKind::Yyjson => {
                    let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                        ptr as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };
                    if !v.is_null() && nyjson_is_int(v) != 0 {
                        write_tlv_i64(nyjson_get_sint_val(v) as i64, result, result_len)
                    } else if !v.is_null() && nyjson_is_obj(v) != 0 {
                        let key = CString::new("value").unwrap();
                        let child = nyjson_obj_get_key(v, key.as_ptr());
                        if !child.is_null() && nyjson_is_int(child) != 0 {
                            write_tlv_i64(nyjson_get_sint_val(child) as i64, result, result_len)
                        } else {
                            write_tlv_i64(0, result, result_len)
                        }
                    } else {
                        write_tlv_i64(0, result, result_len)
                    }
                }
            },
            JN_BOOL => match provider_kind() {
                ProviderKind::Serde => {
                    if let NodeRep::Serde(ref a) = node_rep {
                        if let Value::Bool(b) = **a {
                            write_tlv_bool(b, result, result_len)
                        } else {
                            write_tlv_bool(false, result, result_len)
                        }
                    } else {
                        write_tlv_bool(false, result, result_len)
                    }
                }
                ProviderKind::Yyjson => {
                    let v = if let NodeRep::Yy { ptr, .. } = node_rep {
                        ptr as *mut c_void
                    } else {
                        std::ptr::null_mut()
                    };
                    if !v.is_null() && nyjson_is_bool(v) != 0 {
                        write_tlv_bool(nyjson_get_bool_val(v) != 0, result, result_len)
                    } else {
                        write_tlv_bool(false, result, result_len)
                    }
                }
            },
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_JsonNodeBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"JsonNodeBox\0".as_ptr() as *const c_char,
    resolve: Some(jsonnode_resolve),
    invoke_id: Some(jsonnode_invoke_id),
    capabilities: 0,
};

// ---- TLV helpers (copied/minimized) ----
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
fn write_u32(v: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    unsafe {
        if result.is_null() || *result_len < 4 {
            *result_len = 4;
            return E_SHORT;
        }
        let b = v.to_le_bytes();
        std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
        *result_len = 4;
    }
    OK
}
fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    // Align with common helpers: use tag=9 for void/host-handle-like empty
    write_tlv_result(&[(9u8, &[])], result, result_len)
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
            if tag != 6 {
                return None;
            }
            let s = String::from_utf8_lossy(&buf[off + 4..off + 4 + size]).to_string();
            return Some(s);
        }
        off += 4 + size;
    }
    None
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
