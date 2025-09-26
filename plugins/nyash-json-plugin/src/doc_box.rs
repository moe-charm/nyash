//! JsonDocBox implementation

use crate::constants::*;
use crate::ffi;
use crate::provider::{
    provider_kind, provider_parse, DocInst, NodeRep, ProviderKind, DOCS, NEXT_ID, NODES,
};
use crate::tlv_helpers::*;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::{atomic::Ordering, Arc};

pub extern "C" fn jsondoc_resolve(name: *const c_char) -> u32 {
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

pub extern "C" fn jsondoc_invoke_id(
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
                    m.insert(id, DocInst::new());
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
                                let p = ffi::nyjson_parse_doc(
                                    c.as_ptr(),
                                    text.len(),
                                    &mut ec as *mut i32,
                                );
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
                                    let vp = ffi::nyjson_doc_root(dp as *mut std::os::raw::c_void);
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
                            ffi::nyjson_doc_free(dp as *mut std::os::raw::c_void);
                        }
                    }
                }
                return write_tlv_void(result, result_len);
            }
            _ => E_METHOD,
        }
    }
}
