//! Method resolution system for plugin loader v2
//!
//! This module handles all method ID resolution, method handle resolution,
//! and metadata queries for plugin methods.

use crate::bid::{BidError, BidResult};
use crate::runtime::plugin_loader_v2::enabled::PluginLoaderV2;

impl PluginLoaderV2 {
    /// Resolve a method ID for a given box type and method name
    pub(crate) fn resolve_method_id(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
        // Trace helper
        fn trace(box_type: &str, method_name: &str, provider: &str, mid: Option<u32>) {
            if std::env::var("NYASH_METHOD_REG_TRACE").ok().as_deref() == Some("1") {
                match mid {
                    Some(id) => eprintln!(
                        r#"{{"kind":"method_resolve","class":"{}","method":"{}","provider":"{}","method_id":{}}}"#,
                        box_type, method_name, provider, id
                    ),
                    None => eprintln!(
                        r#"{{"kind":"method_resolve","class":"{}","method":"{}","provider":"{}","status":"miss"}}"#,
                        box_type, method_name, provider
                    ),
                }
            }
        }
        // Prefer specs (ingested from nyash_box.toml or TypeBox FFI)
        if let Ok(map) = self.box_specs.read() {
            // Direct spec lookup for this box
            for ((_lib, bt), spec) in map.iter() {
                if bt == box_type {
                    if let Some(ms) = spec.methods.get(method_name) {
                        trace(box_type, method_name, "spec", Some(ms.method_id));
                        return Ok(ms.method_id);
                    }
                    if let Some(res_fn) = spec.resolve_fn {
                        if let Ok(cstr) = std::ffi::CString::new(method_name) {
                            // Calling a plain extern "C" function is safe; no unsafe needed here.
                            let mid = res_fn(cstr.as_ptr());
                            if mid != 0 { trace(box_type, method_name, "spec_fn", Some(mid)); return Ok(mid); }
                        }
                    }
                }
            }
        }

        // Then try config mapping (nested under libraries.<lib>.<Box>)
        if let Some(cfg) = self.config.as_ref() {
            let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
            // Use cached TOML when available (dev reload via env bypasses cache)
            let reload = std::env::var("NYASH_PLUGIN_CONFIG_RELOAD").ok().as_deref() == Some("1");
            let toml_value: toml::Value = if !reload {
                if let Some(v) = self.cached_toml.clone() { v } else {
                    let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
                    toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?
                }
            } else {
                let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
                toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?
            };

            // Find library for box
            if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                    if let Some(method_spec) = box_conf.methods.get(method_name) {
                        trace(box_type, method_name, "config", Some(method_spec.method_id));
                        return Ok(method_spec.method_id);
                    }
                }
            }
        }

        // Last resort: file-based legacy mapping (dev-only)
        let res = self.resolve_method_id_from_file(box_type, method_name);
        if let Ok(id) = res { trace(box_type, method_name, "legacy", Some(id)); } else { trace(box_type, method_name, "miss", None); }
        res
    }

    /// Resolve method ID from file (legacy fallback)
    fn resolve_method_id_from_file(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
        // Legacy file-based resolution (to be deprecated)
        match (box_type, method_name) {
            ("StringBox", "concat") => Ok(102),
            ("StringBox", "upper") => Ok(103),
            ("CounterBox", "inc") => Ok(102),
            ("CounterBox", "get") => Ok(103),
            _ => Err(BidError::InvalidMethod),
        }
    }

    /// Check if a method returns a Result type
    pub fn method_returns_result(&self, box_type: &str, method_name: &str) -> bool {
        if let Some(cfg) = self.config.as_ref() {
            let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
            let reload = std::env::var("NYASH_PLUGIN_CONFIG_RELOAD").ok().as_deref() == Some("1");
            let toml_value_opt = if !reload {
                self.cached_toml.clone()
            } else {
                std::fs::read_to_string(cfg_path).ok().and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            };
            if let Some(toml_value) = toml_value_opt {
                if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                    if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                        if let Some(method_spec) = box_conf.methods.get(method_name) {
                            return method_spec.returns_result;
                        }
                    }
                }
            }
        }
        // Default to false for unknown methods
        false
    }

    /// Resolve (type_id, method_id, returns_result) for a box_type.method
    pub fn resolve_method_handle(
        &self,
        box_type: &str,
        method_name: &str,
    ) -> BidResult<(u32, u32, bool)> {
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let reload = std::env::var("NYASH_PLUGIN_CONFIG_RELOAD").ok().as_deref() == Some("1");
        let toml_value: toml::Value = if !reload {
            self.cached_toml.clone().ok_or(BidError::PluginError)?
        } else {
            let s = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            toml::from_str(&s).map_err(|_| BidError::PluginError)?
        };
        let (lib_name, _) = cfg
            .find_library_for_box(box_type)
            .ok_or(BidError::InvalidType)?;
        let bc = cfg
            .get_box_config(lib_name, box_type, &toml_value)
            .ok_or(BidError::InvalidType)?;
        let m = bc.methods.get(method_name).ok_or(BidError::InvalidMethod)?;
        Ok((bc.type_id, m.method_id, m.returns_result))
    }
}

/// Helper functions for method resolution
pub(super) fn is_special_method(method_name: &str) -> bool {
    matches!(method_name, "birth" | "fini" | "toString")
}

/// Get default method IDs for special methods
pub(super) fn get_special_method_id(method_name: &str) -> Option<u32> {
    match method_name {
        "birth" => Some(1),
        "toString" => Some(100),
        "fini" => Some(999),
        _ => None,
    }
}
