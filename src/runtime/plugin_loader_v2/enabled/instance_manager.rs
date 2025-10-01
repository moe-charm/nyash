//! Instance management for plugin boxes

use crate::bid::{BidError, BidResult};
use crate::box_trait::NyashBox;
use crate::runtime::plugin_loader_v2::enabled::{PluginLoaderV2, types::{PluginBoxV2, PluginHandleInner}};
use std::sync::Arc;

fn dbg_on() -> bool {
    std::env::var("PLUGIN_DEBUG").is_ok()
}

impl PluginLoaderV2 {
    /// Create a new plugin box instance
    pub fn create_box(
        &self,
        box_type: &str,
        _args: &[Box<dyn NyashBox>],
    ) -> BidResult<Box<dyn NyashBox>> {
        // Non-recursive: directly call plugin 'birth' and construct PluginBoxV2

        // Resolve type_id, birth_id, and fini_id
        let (type_id, birth_id, fini_id) = resolve_box_ids(self, box_type)?;

        // Get loaded plugin invoke
        let _plugins = self.plugins.read().map_err(|_| BidError::PluginError)?;

        // Call birth (no args TLV) and read returned instance id (little-endian u32 in bytes 0..4)
        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] invoking birth: box_type={} type_id={} birth_id={}",
                box_type, type_id, birth_id
            );
        }

        let tlv = crate::runtime::plugin_ffi_common::encode_empty_args();
        let (code, out_len, out_buf) = super::host_bridge::invoke_alloc(
            super::super::nyash_plugin_invoke_v2_shim,
            type_id,
            birth_id,
            0,
            &tlv,
        );

        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] create_box: box_type={} type_id={} birth_id={} code={} out_len={}",
                box_type, type_id, birth_id, code, out_len
            );
            if out_len > 0 {
                eprintln!(
                    "[PluginLoaderV2] create_box: out[0..min(8)]={:02x?}",
                    &out_buf[..out_len.min(8)]
                );
            }
        }

        if code != 0 || out_len < 4 {
            return Err(BidError::PluginError);
        }

        let instance_id = u32::from_le_bytes([out_buf[0], out_buf[1], out_buf[2], out_buf[3]]);

        let bx = PluginBoxV2 {
            box_type: box_type.to_string(),
            inner: Arc::new(PluginHandleInner {
                type_id,
                invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
                instance_id,
                fini_method_id: fini_id,
                finalized: std::sync::atomic::AtomicBool::new(false),
            }),
        };

        // Diagnostics: register for leak tracking (optional)
        crate::runtime::leak_tracker::register_plugin(box_type, instance_id);
        Ok(Box::new(bx))
    }

    /// Shutdown singletons: finalize and clear all singleton handles
    pub fn shutdown_singletons(&self) {
        let mut map = self.singletons.write().unwrap();
        for (_, handle) in map.drain() {
            if let Ok(inner) = Arc::try_unwrap(handle) {
                inner.finalize_now();
            }
        }
    }
}

/// Resolve box IDs (type_id, birth_id, fini_id) from configuration or specs
fn resolve_box_ids(
    loader: &PluginLoaderV2,
    box_type: &str,
) -> BidResult<(u32, u32, Option<u32>)> {
    let (mut type_id_opt, mut birth_id_opt, mut fini_id) = (None, None, None);

    // Try config mapping first (when available)
    if let Some(cfg) = loader.config.as_ref() {
        let cfg_path = loader.config_path.as_deref().unwrap_or("nyash.toml");
        let reload = std::env::var("NYASH_PLUGIN_CONFIG_RELOAD").ok().as_deref() == Some("1");
        let toml_value: toml::Value = if !reload {
            loader.cached_toml.clone().ok_or(BidError::PluginError)?
        } else {
            let s = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            toml::from_str(&s).map_err(|_| BidError::PluginError)?
        };

        if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
            if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                type_id_opt = Some(box_conf.type_id);
                birth_id_opt = box_conf.methods.get("birth").map(|m| m.method_id);
                fini_id = box_conf.methods.get("fini").map(|m| m.method_id);
            }
        }
    }

    // Fallback: use TypeBox FFI spec if config is missing for this box
    if type_id_opt.is_none() || birth_id_opt.is_none() {
        if let Ok(map) = loader.box_specs.read() {
            // Find any spec that matches this box_type
            if let Some((_, spec)) = map.iter().find(|((_lib, bt), _)| bt == &box_type) {
                if type_id_opt.is_none() {
                    type_id_opt = spec.type_id;
                }
                if birth_id_opt.is_none() {
                    if let Some(ms) = spec.methods.get("birth") {
                        birth_id_opt = Some(ms.method_id);
                    } else if let Some(res_fn) = spec.resolve_fn {
                        if let Ok(cstr) = std::ffi::CString::new("birth") {
                            let mid = res_fn(cstr.as_ptr());
                            if mid != 0 {
                                birth_id_opt = Some(mid);
                            }
                        }
                    }
                }
            }
        }
    }

    let type_id = type_id_opt.ok_or(BidError::InvalidType)?;
    let birth_id = birth_id_opt.ok_or(BidError::InvalidMethod)?;

    Ok((type_id, birth_id, fini_id))
}
