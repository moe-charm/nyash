//! FFI bridge for plugin method invocation and TLV encoding/decoding

use crate::bid::{BidError, BidResult};
use crate::box_trait::NyashBox;
use crate::runtime::plugin_loader_v2::enabled::PluginLoaderV2;
use std::sync::Arc;

fn dbg_on() -> bool {
    std::env::var("PLUGIN_DEBUG").is_ok()
}

impl PluginLoaderV2 {
    /// Invoke a method on a plugin instance with TLV encoding/decoding
    pub fn invoke_instance_method(
        &self,
        box_type: &str,
        method_name: &str,
        instance_id: u32,
        args: &[Box<dyn NyashBox>],
    ) -> BidResult<Option<Box<dyn NyashBox>>> {
        // Resolve (lib_name, type_id) either from config or cached specs
        let (lib_name, type_id) = resolve_type_info(self, box_type)?;

        // Resolve method id via config or TypeBox resolve()
        let method_id = match self.resolve_method_id(box_type, method_name) {
            Ok(mid) => mid,
            Err(e) => {
                if dbg_on() {
                    eprintln!(
                        "[PluginLoaderV2] ERR: method resolve failed for {}.{}: {:?}",
                        box_type, method_name, e
                    );
                }
                return Err(BidError::InvalidMethod);
            }
        };

        // Get plugin handle
        let plugins = self.plugins.read().map_err(|_| BidError::PluginError)?;
        let _plugin = plugins.get(&lib_name).ok_or(BidError::PluginError)?;

        // Encode TLV args via shared helper (numeric→string→toString)
        let tlv = crate::runtime::plugin_ffi_common::encode_args(args);

        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] call {}.{}: type_id={} method_id={} instance_id={}",
                box_type, method_name, type_id, method_id, instance_id
            );
        }

        let (_code, out_len, out) = super::host_bridge::invoke_alloc(
            super::super::nyash_plugin_invoke_v2_shim,
            type_id,
            method_id,
            instance_id,
            &tlv,
        );

        // Decode TLV (first entry) generically
        decode_tlv_result(box_type, &out[..out_len])
    }
}

/// Resolve type information for a box
fn resolve_type_info(loader: &PluginLoaderV2, box_type: &str) -> BidResult<(String, u32)> {
    if let Some(cfg) = loader.config.as_ref() {
        let cfg_path = loader.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value =
            toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?)
                .map_err(|_| BidError::PluginError)?;

        if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
            if let Some(bc) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                return Ok((lib_name.to_string(), bc.type_id));
            } else {
                let key = (lib_name.to_string(), box_type.to_string());
                let map = loader.box_specs.read().map_err(|_| BidError::PluginError)?;
                let tid = map
                    .get(&key)
                    .and_then(|s| s.type_id)
                    .ok_or(BidError::InvalidType)?;
                return Ok((lib_name.to_string(), tid));
            }
        }
    } else {
        let map = loader.box_specs.read().map_err(|_| BidError::PluginError)?;
        if let Some(((lib, _), spec)) = map.iter().find(|((_, bt), _)| bt == box_type) {
            return Ok((lib.clone(), spec.type_id.ok_or(BidError::InvalidType)?));
        }
    }
    Err(BidError::InvalidType)
}

/// Decode TLV result into a NyashBox
fn decode_tlv_result(box_type: &str, data: &[u8]) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some((tag, _sz, payload)) =
        crate::runtime::plugin_ffi_common::decode::tlv_first(data)
    {
        let bx: Box<dyn NyashBox> = match tag {
            1 => Box::new(crate::box_trait::BoolBox::new(
                crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false),
            )),
            2 => Box::new(crate::box_trait::IntegerBox::new(
                crate::runtime::plugin_ffi_common::decode::i32(payload).unwrap_or(0) as i64,
            )),
            3 => {
                // i64 payload
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    Box::new(crate::box_trait::IntegerBox::new(i64::from_le_bytes(b)))
                } else {
                    Box::new(crate::box_trait::IntegerBox::new(0))
                }
            }
            5 => {
                let x = crate::runtime::plugin_ffi_common::decode::f64(payload).unwrap_or(0.0);
                Box::new(crate::boxes::FloatBox::new(x))
            }
            6 | 7 => {
                let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                Box::new(crate::box_trait::StringBox::new(s))
            }
            8 => {
                // Plugin handle (type_id, instance_id) → wrap into PluginBoxV2
                if let Some((ret_type, inst)) =
                    crate::runtime::plugin_ffi_common::decode::plugin_handle(payload)
                {
                    let handle = Arc::new(super::types::PluginHandleInner {
                        type_id: ret_type,
                        invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
                        instance_id: inst,
                        fini_method_id: None,
                        finalized: std::sync::atomic::AtomicBool::new(false),
                    });
                    Box::new(super::types::PluginBoxV2 {
                        box_type: box_type.to_string(),
                        inner: handle,
                    })
                } else {
                    Box::new(crate::box_trait::VoidBox::new())
                }
            }
            9 => {
                // Host handle (u64) → try to map back to BoxRef, else void
                if let Some(u) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                    if let Some(arc) = crate::runtime::host_handles::get(u) {
                        return Ok(Some(arc.share_box()));
                    }
                }
                Box::new(crate::box_trait::VoidBox::new())
            }
            _ => Box::new(crate::box_trait::VoidBox::new()),
        };
        return Ok(Some(bx));
    }
    Ok(Some(Box::new(crate::box_trait::VoidBox::new())))
}