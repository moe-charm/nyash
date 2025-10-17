// host_handle_router/mod.rs — Staged extraction point for HostHandle routing
// Responsibility: Provide a thin facade for host handle operations.

#![allow(dead_code)]

// Public constants (slots / error codes)
pub mod consts;

use std::sync::Arc;

use consts::{
    ARRAY_GET, ARRAY_SET, ARRAY_SIZE, ERR_BAD_ARGS, ERR_BAD_RETURN, ERR_UNSUPPORTED,
    ERR_UNKNOWN_HANDLE, MAP_GET, MAP_HAS, MAP_KEYS, MAP_SET, MAP_SIZE, MAP_VALUES,
    STRING_LEN,
};

fn plugin_box_matches<'a>(
    arc: &'a Arc<dyn crate::box_trait::NyashBox>,
    expected: &str,
) -> Option<&'a crate::runtime::plugin_loader_v2::PluginBoxV2> {
    arc.as_any()
        .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
        .filter(|pb| pb.box_type == expected)
}

// Unified helper: 0-argument method (size/len/keys/values)
fn route_method_0args(
    handle: u64,
    box_type: &str,
    method: &str,
    _out_ptr: *mut u8,
    _out_len: *mut usize,
) -> i32 {
    let Some(arc) = crate::runtime::host_handles::get(handle) else { return ERR_UNKNOWN_HANDLE; };
    if let Some(pb) = plugin_box_matches(&arc, box_type) {
        match crate::runtime::plugin_host_box::invoke_instance_method(box_type, method, pb.inner.instance_id, &[]) {
            Ok(Some(ret)) => {
                let vmv = crate::backend::vm::VMValue::from_nyash_box(ret);
                let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
            }
            _ => return ERR_BAD_RETURN,
        }
    }
    ERR_UNSUPPORTED
}

// Unified helper: 1-argument method (get/has)
fn route_method_1arg(
    handle: u64,
    box_type: &str,
    method: &str,
    _args_ptr: *const u8,
    _args_len: usize,
    _out_ptr: *mut u8,
    _out_len: *mut usize,
) -> i32 {
    let Some(arc) = crate::runtime::host_handles::get(handle) else { return ERR_UNKNOWN_HANDLE; };
    if let Some(pb) = plugin_box_matches(&arc, box_type) {
        let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
        if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
            if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                let arg_box = vmv.to_nyash_box();
                match crate::runtime::plugin_host_box::invoke_instance_method(box_type, method, pb.inner.instance_id, &[arg_box]) {
                    Ok(Some(ret)) => {
                        let vmv_out = crate::backend::vm::VMValue::from_nyash_box(ret);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv_out);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                    Ok(None) => {
                        let vmv_out = crate::backend::vm::VMValue::Void;
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv_out);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                    Err(_) => return ERR_BAD_RETURN,
                }
            }
        }
        return ERR_BAD_ARGS;
    }
    ERR_UNSUPPORTED
}

// Unified helper: 2-argument method (set)
fn route_method_2args(
    handle: u64,
    box_type: &str,
    method: &str,
    _args_ptr: *const u8,
    _args_len: usize,
    _out_ptr: *mut u8,
    _out_len: *mut usize,
) -> i32 {
    let Some(arc) = crate::runtime::host_handles::get(handle) else { return ERR_UNKNOWN_HANDLE; };
    if let Some(pb) = plugin_box_matches(&arc, box_type) {
        let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
        if let (Some((tag0, _sz0, payload0)), Some((tag1, _sz1, payload1))) = (
            crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 0),
            crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 1),
        ) {
            if let (Some(v0), Some(v1)) = (
                crate::runtime::host_api::vmvalue_from_tlv(tag0, payload0),
                crate::runtime::host_api::vmvalue_from_tlv(tag1, payload1),
            ) {
                let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![
                    v0.to_nyash_box(),
                    v1.to_nyash_box(),
                ];
                match crate::runtime::plugin_host_box::invoke_instance_method(box_type, method, pb.inner.instance_id, &argv) {
                    Ok(_) => {
                        let vmv = crate::backend::vm::VMValue::Void;
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                    _ => return ERR_BAD_RETURN,
                }
            }
        }
        return ERR_BAD_ARGS;
    }
    ERR_UNSUPPORTED
}

// Phase‑in plan:
// - Start by handling a single, harmless slot (Array.len = 102) to validate wiring.
// - Keep the rest returning -999 until type unification and broader routing are ready.
// Caller: src/runtime/host_api.rs:270 (nyrt_host_call_slot delegates here)
// See: host_handle_router/README.md for architectural intent

/// Stub implementation of route_slot
/// Returns -999 (unimplemented) for all calls until HostHandle/VMValue unification is complete
#[allow(clippy::too_many_arguments)]
pub fn route_slot(
    _handle: u64,
    _selector_id: u64,
    _args_ptr: *const u8,
    _args_len: usize,
    _out_ptr: *mut u8,
    _out_len: *mut usize,
) -> i32 {
    // Minimal, safe implementation for ArrayBox.get/set/len (slots 100/101/102), MapBox.size/has/get/set (200/202/203/204), StringBox.len (300)
    // All other slots return -999 (unimplemented)
    let handle = _handle;
    let selector = _selector_id;

    if selector == ARRAY_SIZE {
        return route_method_0args(handle, "ArrayBox", "size", _out_ptr, _out_len);
    }

    if selector == ARRAY_GET {
        return route_method_1arg(handle, "ArrayBox", "get", _args_ptr, _args_len, _out_ptr, _out_len);
    }

    if selector == ARRAY_SET {
        return route_method_2args(handle, "ArrayBox", "set", _args_ptr, _args_len, _out_ptr, _out_len);
    }

    if selector == MAP_SIZE {
        return route_method_0args(handle, "MapBox", "size", _out_ptr, _out_len);
    }

    if selector == MAP_HAS {
        return route_method_1arg(handle, "MapBox", "has", _args_ptr, _args_len, _out_ptr, _out_len);
    }

    if selector == MAP_GET {
        return route_method_1arg(handle, "MapBox", "get", _args_ptr, _args_len, _out_ptr, _out_len);
    }

    if selector == MAP_SET {
        return route_method_2args(handle, "MapBox", "set", _args_ptr, _args_len, _out_ptr, _out_len);
    }

    if selector == MAP_KEYS {
        return route_method_0args(handle, "MapBox", "keys", _out_ptr, _out_len);
    }

    if selector == MAP_VALUES {
        return route_method_0args(handle, "MapBox", "values", _out_ptr, _out_len);
    }

    if selector == STRING_LEN {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return ERR_UNKNOWN_HANDLE; };
        if arc.type_name() == "StringBox" {
            // Test hook: simulate return-type mismatch (-14) when explicitly requested.
            // This helps boundary testing without a dedicated plugin.
            if crate::runtime::env_gate_box::bool_any(&[
                "HAKO_HOSTHANDLE_TEST_RET_MISMATCH",
                "NYASH_HOSTHANDLE_TEST_RET_MISMATCH",
            ]) {
                return ERR_BAD_RETURN;
            }
            if let Some(sb) = arc.as_any().downcast_ref::<crate::StringBox>() {
                let n = sb.value.len() as i64;
                let vmv = crate::backend::vm::VMValue::Integer(n);
                let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
            }
        }
        return ERR_UNSUPPORTED;
    }

    // Default: not implemented yet
    -999
}

/*
pub mod router {
    use crate::runtime::host_api; // temporary: route back to existing APIs

    pub fn call_method(receiver: &host_api::HostHandle, method: &str, args: &[host_api::VMValue]) -> Result<host_api::VMValue, String> {
        // TODO: move logic from host_api into this module gradually.
        host_api::call_method(receiver, method, args)
    }

    pub fn new_box(name: &str, args: &[host_api::VMValue]) -> Result<host_api::HostHandle, String> {
        host_api::new_box(name, args)
    }
}
*/
