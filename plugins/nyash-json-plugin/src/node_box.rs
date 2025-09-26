//! JsonNodeBox implementation

use crate::constants::*;
use crate::ffi::*;
use crate::provider::{provider_kind, NodeRep, ProviderKind, NEXT_ID, NODES};
use crate::tlv_helpers::*;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::{atomic::Ordering, Arc};

pub extern "C" fn jsonnode_resolve(name: *const c_char) -> u32 {
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

pub extern "C" fn jsonnode_invoke_id(
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
            JN_FINI => {
                if let Ok(mut m) = NODES.lock() {
                    m.remove(&instance_id);
                }
                return write_tlv_void(result, result_len);
            }
            _ => E_METHOD,
        }
    }
}
