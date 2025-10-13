// host_handle_router/mod.rs — Staged extraction point for HostHandle routing
// Responsibility: Provide a thin facade for host handle operations.

#![allow(dead_code)]

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
    // Minimal, safe implementation for ArrayBox.len (slot 102), MapBox.size (slot 200), MapBox.has (slot 202), MapBox.get (slot 203)
    // All other slots return -999 (unimplemented)
    const ARRAY_LEN: u64 = 102;
    const MAP_SIZE: u64 = 200;
    const MAP_HAS: u64 = 202;
    const MAP_GET: u64 = 203;

    let handle = _handle;
    let selector = _selector_id;

    if selector == ARRAY_LEN {
        // Resolve HostHandle → Arc<dyn NyashBox>
        let Some(arc) = crate::runtime::host_handles::get(handle) else {
            return -1; // unknown handle
        };
        // Downcast to ArrayBox by type name to avoid importing concrete type in signature
        if arc.type_name() == "ArrayBox" {
            // Safe: len() is pure and does not mutate; encode as TLV Integer
            if let Some(arr) = arc.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let n = arr.len() as i64;
                let vmv = crate::backend::vm::VMValue::Integer(n);
                let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
            }
        }
        // Known selector but unsupported receiver type
        return -11;
    }

    if selector == MAP_SIZE {
        let Some(arc) = crate::runtime::host_handles::get(handle) else {
            return -1;
        };
        if arc.type_name() == "MapBox" {
            if let Some(m) = arc.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                // MapBox.size() returns IntegerBox inside Box<dyn NyashBox>
                let n_box = m.size();
                if let Some(int) = n_box.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    let vmv = crate::backend::vm::VMValue::Integer(int.value);
                    let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                    return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                }
                // Fallback: compute via keys().size() if downcast fails (defensive)
                let ks = m.keys();
                if let Some(arr) = ks.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    let vmv = crate::backend::vm::VMValue::Integer(arr.len() as i64);
                    let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                    return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                }
                return -12;
            }
        }
        return -11;
    }

    if selector == MAP_HAS {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "MapBox" {
            if let Some(m) = arc.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                // Decode first TLV arg as VMValue, convert to Box for MapBox.has
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let Some((_tag, _sz, _payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
                    if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(_tag, _payload) {
                        let key_box = vmv.to_nyash_box();
                        let res_box = m.has(key_box);
                        if let Some(bb) = res_box.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                            let vmv = crate::backend::vm::VMValue::Bool(bb.value);
                            let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                            return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                        }
                        return -14; // unexpected return type
                    }
                }
                return -13; // arg decode failed
            }
        }
        return -11;
    }

    if selector == MAP_GET {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "MapBox" {
            if let Some(m) = arc.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                // Decode first TLV arg as VMValue, convert to Box for MapBox.get
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
                    if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        let key_box = vmv.to_nyash_box();
                        let res_box = m.get(key_box);
                        let vmv_out = crate::backend::vm::VMValue::from_nyash_box(res_box);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv_out);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                }
                return -13; // arg decode failed
            }
        }
        return -11;
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
