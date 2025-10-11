//! Method resolution system for plugin loader v2
//!
//! This module handles all method ID resolution, method handle resolution,
//! and metadata queries for plugin methods.

use crate::bid::{BidError, BidResult};
use crate::runtime::plugin_loader_v2::enabled::PluginLoaderV2;

impl PluginLoaderV2 {
    /// Resolve a method ID for a given box type and method name
    pub(crate) fn resolve_method_id(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
        // Prefer specs (ingested from nyash_box.toml or TypeBox FFI)
        if let Ok(map) = self.box_specs.read() {
            // Prefer TypeBox resolve() when available (plugin-provided mapping)
            for ((_lib, bt), spec) in map.iter() {
                if bt == box_type {
                    if let Some(res_fn) = spec.resolve_fn {
                        if let Ok(cstr) = std::ffi::CString::new(method_name) {
                            let mid = res_fn(cstr.as_ptr());
                            if mid != 0 { return Ok(mid); }
                        }
                    }
                }
            }
            // Fallback to static spec mapping (e.g., from hako_box.toml)
            for ((_lib, bt), spec) in map.iter() {
                if bt == box_type {
                    if let Some(ms) = spec.methods.get(method_name) {
                        return Ok(ms.method_id);
                    }
                }
            }
        }

        // Then try config mapping (nested under libraries.<lib>.<Box>)
        if let Some(cfg) = self.config.as_ref() {
            let toml_value = self.get_toml_value()?;

            // Find library for box
            if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                    if let Some(method_spec) = box_conf.methods.get(method_name) { return Ok(method_spec.method_id); }
                }
            }
        }

        // Last resort: file-based legacy mapping (dev-only)
        let res = self.resolve_method_id_from_file(box_type, method_name);
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
            let toml_value_opt = self.get_toml_value().ok();
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
        let toml_value = self.get_toml_value()?;
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
