//! plugin.rs — Plugin(TypeBox v2) routing adapter (Phase 15.75 split scaffold)
use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;
use super::map_callable::MapCallableBox;

/// Try routing a PluginBoxV2 receiver.
/// Phase 0-mini: scaffold only. Keep behavior in mod.rs for now.
/// Returns Ok(None) when not handled.
pub fn try_route_plugin_box(
    _interp: &mut MirInterpreter,
    bx: &std::sync::Arc<dyn NyashBox>,
    method: &str,
    args: &[VMValue],
) -> Result<Option<VMValue>, VMError> {
    // Plugin TypeBox v2
    if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
        // Central arity guard (vm_ops/boxcall)
        crate::vm_ops::boxcall::arity_guard_for(&p.box_type, method, args.len())?;
        if let Some(result) = MapCallableBox::try_route(_interp, &VMValue::BoxRef(bx.clone()), method, args) {
            return result.map(Some);
        }
        // Dev/test: optionally force HostHandleRouter path for Map.size/has/get/set
        // Fine-grained toggles default to NYASH_MAP_FORCE_HOST when present
        let force_map_all = std::env::var("NYASH_MAP_FORCE_HOST").ok().as_deref() == Some("1");
        let force_map_size = force_map_all || std::env::var("NYASH_MAP_SIZE_FORCE_HOST").ok().as_deref() == Some("1");
        let force_map_has = force_map_all || std::env::var("NYASH_MAP_HAS_FORCE_HOST").ok().as_deref() == Some("1");
        let force_map_get = force_map_all || std::env::var("NYASH_MAP_GET_FORCE_HOST").ok().as_deref() == Some("1");
        let force_map_set = force_map_all || std::env::var("NYASH_MAP_SET_FORCE_HOST").ok().as_deref() == Some("1");
        if p.box_type == "MapBox" {
            // Early host path for size()
            if method == "size" && args.is_empty() && force_map_size {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                let mut out_buf = vec![0u8; 64];
                let mut out_len: usize = out_buf.len();
                let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 200, std::ptr::null(), 0, out_buf.as_mut_ptr(), &mut out_len);
                if rc == -3 { out_len = 128; out_buf.resize(out_len, 0); let _ = crate::runtime::host_api::nyrt_host_call_slot(hh, 200, std::ptr::null(), 0, out_buf.as_mut_ptr(), &mut out_len); }
                if rc == 0 && out_len >= 6 {
                    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                        if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(Some(v)); }
                    }
                }
            }
            // Early host path for has(key)
            if method == "has" && args.len() == 1 && force_map_has {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                let args_boxes = vec![args[0].to_nyash_box()];
                let tlv_args = crate::runtime::plugin_ffi_common::encode_args(&args_boxes);
                let mut out_buf = vec![0u8; 64];
                let mut out_len: usize = out_buf.len();
                let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 202, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len);
                if rc == -3 { out_len = 128; out_buf.resize(out_len, 0); let _ = crate::runtime::host_api::nyrt_host_call_slot(hh, 202, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len); }
                if rc == 0 && out_len >= 6 {
                    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                        if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(Some(v)); }
                    }
                }
            }
            // Early host path for get(key)
            if method == "get" && args.len() == 1 && force_map_get {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                let args_boxes = vec![args[0].to_nyash_box()];
                let tlv_args = crate::runtime::plugin_ffi_common::encode_args(&args_boxes);
                let mut out_buf = vec![0u8; 64];
                let mut out_len: usize = out_buf.len();
                let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 203, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len);
                if rc == -3 { out_len = 128; out_buf.resize(out_len, 0); let _ = crate::runtime::host_api::nyrt_host_call_slot(hh, 203, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len); }
                if rc == 0 && out_len >= 6 {
                    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                        if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(Some(v)); }
                    }
                }
            }
            // Early host path for set(key,value)
            if method == "set" && args.len() == 2 && force_map_set {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                let args_boxes = vec![args[0].to_nyash_box(), args[1].to_nyash_box()];
                let tlv_args = crate::runtime::plugin_ffi_common::encode_args(&args_boxes);
                let mut out_buf = vec![0u8; 64];
                let mut out_len: usize = out_buf.len();
                let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 204, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len);
                if rc == -3 { out_len = 128; out_buf.resize(out_len, 0); let _ = crate::runtime::host_api::nyrt_host_call_slot(hh, 204, tlv_args.as_ptr(), tlv_args.len(), out_buf.as_mut_ptr(), &mut out_len); }
                if rc == 0 { return Ok(Some(VMValue::Void)); }
            }
        }

        // Stage-1 fallback STUB: convert VMValue args to NyashBox (core types→HostHandle)
        let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for v in args {
            if let VMValue::BoxRef(bx) = v {
                if crate::runtime::type_registry::is_core_box(bx.type_name()) {
                    let h = crate::runtime::host_handles::to_handle_arc(bx.clone());
                    argv.push(Box::new(crate::runtime::host_handle_box::HostHandleBox::new(h)));
                    continue;
                }
            }
            argv.push(v.to_nyash_box());
        }

        // Optional fast path: ArrayBox.size via slot 102
        if p.box_type == "ArrayBox" && (method == "size" || method == "len" || method == "length")
            && std::env::var("NYASH_ARRAY_SIZE_FORCE_HOST").ok().as_deref() == Some("1")
        {
            let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
            let mut out_buf = vec![0u8; 64];
            let mut out_len: usize = out_buf.len();
            let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 102, std::ptr::null(), 0, out_buf.as_mut_ptr(), &mut out_len);
            if rc == 0 && out_len >= 6 {
                if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                    if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(Some(v)); }
                }
            }
        }

        // Fallback host path for Map.* if forced and still not returned (size/has/get)
        if p.box_type == "MapBox" {
            let force_map_all = std::env::var("NYASH_MAP_FORCE_HOST").ok().as_deref() == Some("1");
            let force_map_size = force_map_all || std::env::var("NYASH_MAP_SIZE_FORCE_HOST").ok().as_deref() == Some("1");
            let force_map_has = force_map_all || std::env::var("NYASH_MAP_HAS_FORCE_HOST").ok().as_deref() == Some("1");
            let force_map_get = force_map_all || std::env::var("NYASH_MAP_GET_FORCE_HOST").ok().as_deref() == Some("1");
            if method == "size" && force_map_size {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                let mut out_buf = vec![0u8; 64];
                let mut out_len: usize = out_buf.len();
                let rc = crate::runtime::host_api::nyrt_host_call_slot(hh, 200, std::ptr::null(), 0, out_buf.as_mut_ptr(), &mut out_len);
                if rc == 0 && out_len >= 6 {
                    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                        if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(Some(v)); }
                    }
                }
            }
            // (duplicate has/get paths removed)
        }

        // Delegate to plugin host
        let out = crate::runtime::plugin_host_box::invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv);
        return match out {
            Ok(Some(ret)) => Ok(Some(VMValue::from_nyash_box(ret))),
            Ok(None) => Ok(Some(VMValue::Void)),
            Err(e) => Err(VMError::InvalidInstruction(format!("Plugin method {}.{} failed: {:?}", p.box_type, method, e))),
        };
    }
    Ok(None)
}
