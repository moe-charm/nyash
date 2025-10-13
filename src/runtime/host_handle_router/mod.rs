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
    // Minimal, safe implementation for ArrayBox.get/set/len (slots 100/101/102), MapBox.size/has/get/set (200/202/203/204), StringBox.len (300)
    // All other slots return -999 (unimplemented)
    const ARRAY_GET: u64 = 100;
    const ARRAY_SET: u64 = 101;
    const ARRAY_LEN: u64 = 102;
    const MAP_SIZE: u64 = 200;
    const MAP_HAS: u64 = 202;
    const MAP_GET: u64 = 203;
    const MAP_SET: u64 = 204;
    const STRING_LEN: u64 = 300;

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
            #[cfg(feature = "legacy-boxes")]
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

    if selector == ARRAY_GET {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "ArrayBox" {
            #[cfg(feature = "legacy-boxes")]
            if let Some(arr) = arc.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
                    if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        let idx = vmv.as_integer().unwrap_or(0);
                        let res = arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
                        let vmv = crate::backend::vm::VMValue::from_nyash_box(res);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                }
                return -13;
            }
        }
        return -11;
    }

    if selector == ARRAY_SET {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "ArrayBox" {
            #[cfg(feature = "legacy-boxes")]
            if let Some(arr) = arc.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let (Some((tag0, _sz0, payload0)), Some((tag1, _sz1, payload1))) = (
                    crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 0),
                    crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 1),
                ) {
                    if let (Some(v0), Some(v1)) = (
                        crate::runtime::host_api::vmvalue_from_tlv(tag0, payload0),
                        crate::runtime::host_api::vmvalue_from_tlv(tag1, payload1),
                    ) {
                        let idx = v0.as_integer().unwrap_or(0);
                        let _ = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), v1.to_nyash_box());
                        let vmv = crate::backend::vm::VMValue::Void;
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                }
                return -13;
            }
        }
        return -11;
    }

    if selector == MAP_SIZE {
        let Some(arc) = crate::runtime::host_handles::get(handle) else {
            return -1;
        };
        if arc.type_name() == "MapBox" {
            // Plugin path (avoid legacy dependency when plugins are enabled)
            if let Some(pb) = arc.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                if let Ok(Some(ret)) = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "size", pb.inner.instance_id, &[]) {
                    let vmv = crate::backend::vm::VMValue::from_nyash_box(ret);
                    let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                    return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                }
                return -14;
            }
            #[cfg(feature = "legacy-boxes")]
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
                #[cfg(feature = "legacy-boxes")]
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
            // Plugin path
            if let Some(pb) = arc.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
                    if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        let key_box = vmv.to_nyash_box();
                        if let Ok(Some(ret)) = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "has", pb.inner.instance_id, &[key_box]) {
                            if let Some(bb) = ret.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                                let vmv = crate::backend::vm::VMValue::Bool(bb.value);
                                let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                            }
                            return -14;
                        }
                        return -14;
                    }
                }
                return -13;
            }
            #[cfg(feature = "legacy-boxes")]
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
            // Plugin path
            if let Some(pb) = arc.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(args) {
                    if let Some(vmv) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        let key_box = vmv.to_nyash_box();
                        match crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "get", pb.inner.instance_id, &[key_box]) {
                            Ok(Some(ret)) => {
                                let vmv_out = crate::backend::vm::VMValue::from_nyash_box(ret);
                                let buf = crate::runtime::host_api::tlv_encode_one(&vmv_out);
                                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                            }
                            Ok(None) => {
                                // missing -> null (encode Void)
                                let vmv_out = crate::backend::vm::VMValue::Void;
                                let buf = crate::runtime::host_api::tlv_encode_one(&vmv_out);
                                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                            }
                            Err(_) => return -14,
                        }
                    }
                }
                return -13;
            }
            #[cfg(feature = "legacy-boxes")]
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

    if selector == MAP_SET {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "MapBox" {
            // Plugin path
            if let Some(pb) = arc.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                if let (Some((tag0, _sz0, payload0)), Some((tag1, _sz1, payload1))) = (
                    crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 0),
                    crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 1),
                ) {
                    if let (Some(v0), Some(v1)) = (
                        crate::runtime::host_api::vmvalue_from_tlv(tag0, payload0),
                        crate::runtime::host_api::vmvalue_from_tlv(tag1, payload1),
                    ) {
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method(
                            "MapBox", "set", pb.inner.instance_id, &[v0.to_nyash_box(), v1.to_nyash_box()]);
                        let vmv = crate::backend::vm::VMValue::Void;
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                    }
                }
                return -13;
            }
            #[cfg(feature = "legacy-boxes")]
            if let Some(m) = arc.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                let args = unsafe { crate::runtime::host_api::slice_from_raw(_args_ptr, _args_len) };
                // Expect 2 args (key, value)
                if let Some((tag0, _sz0, payload0)) = crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 0) {
                    if let Some((tag1, _sz1, payload1)) = crate::runtime::plugin_ffi_common::decode::tlv_nth(args, 1) {
                        if let (Some(v0), Some(v1)) = (
                            crate::runtime::host_api::vmvalue_from_tlv(tag0, payload0),
                            crate::runtime::host_api::vmvalue_from_tlv(tag1, payload1),
                        ) {
                            let res = m.set(v0.to_nyash_box(), v1.to_nyash_box());
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(res);
                            let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                            return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
                        }
                    }
                }
                return -13;
            }
        }
        return -11;
    }

    if selector == STRING_LEN {
        let Some(arc) = crate::runtime::host_handles::get(handle) else { return -1; };
        if arc.type_name() == "StringBox" {
            // Test hook: simulate return-type mismatch (-14) when explicitly requested.
            // This helps boundary testing without a dedicated plugin.
            if std::env::var("HAKO_HOSTHANDLE_TEST_RET_MISMATCH").ok().as_deref() == Some("1") {
                return -14;
            }
            if let Some(sb) = arc.as_any().downcast_ref::<crate::StringBox>() {
                let n = sb.value.len() as i64;
                let vmv = crate::backend::vm::VMValue::Integer(n);
                let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                return crate::runtime::host_api::encode_out(_out_ptr, _out_len, &buf);
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
