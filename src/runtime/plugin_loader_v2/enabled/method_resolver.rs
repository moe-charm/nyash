//! Method resolution system for plugin loader v2
//!
//! This module handles all method ID resolution, method handle resolution,
//! and metadata queries for plugin methods.

use crate::bid::{BidError, BidResult};
use crate::runtime::plugin_loader_v2::enabled::PluginLoaderV2;
use std::collections::HashMap;

impl PluginLoaderV2 {
    /// Resolve a method ID for a given box type and method name
    pub(crate) fn resolve_method_id(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
        // First try config mapping
        if let Some(cfg) = self.config.as_ref() {
            let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");

            // Load and parse TOML
            let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;

            // Find library for box
            if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                    if let Some(method_spec) = box_conf.methods.get(method_name) {
                        return Ok(method_spec.method_id);
                    }
                }
            }
        }

        // Fallback to TypeBox FFI spec
        if let Ok(map) = self.box_specs.read() {
            // Try direct lookup first
            for ((lib, bt), spec) in map.iter() {
                if bt == box_type {
                    // Check methods map
                    if let Some(ms) = spec.methods.get(method_name) {
                        return Ok(ms.method_id);
                    }

                    // Try resolve function
                    if let Some(res_fn) = spec.resolve_fn {
                        if let Ok(cstr) = std::ffi::CString::new(method_name) {
                            let mid = unsafe { res_fn(cstr.as_ptr()) };
                            if mid != 0 {
                                return Ok(mid);
                            }
                        }
                    }
                }
            }
        }

        // Try file-based resolution as last resort
        self.resolve_method_id_from_file(box_type, method_name)
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

            if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                        if let Some(box_conf) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                            if let Some(method_spec) = box_conf.methods.get(method_name) {
                                return method_spec.returns_result;
                            }
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
        let toml_value: toml::Value =
            toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?)
                .map_err(|_| BidError::PluginError)?;
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