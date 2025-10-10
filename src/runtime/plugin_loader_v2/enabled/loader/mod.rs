mod config;
mod library;
mod metadata;
mod singletons;
mod specs;
pub(crate) mod util;

use super::host_bridge::BoxInvokeFn;
use super::types::{LoadedPluginV2, PluginBoxMetadata, PluginHandleInner};
use crate::bid::{BidError, BidResult};
use crate::box_trait::NyashBox;
use crate::config::nyash_toml_v2::{LibraryDefinition, NyashConfigV2};
use specs::LoadedBoxSpec;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct PluginLoaderV2 {
    pub(super) plugins: RwLock<HashMap<String, Arc<LoadedPluginV2>>>,
    pub config: Option<NyashConfigV2>,
    pub(super) config_path: Option<crate::runtime::types::verified_path::VerifiedPath>,
    // Cached parsed nyash.toml (hot‑path use; reduces repeated I/O and parse)
    pub(super) cached_toml: Option<toml::Value>,
    pub(super) singletons: RwLock<HashMap<(String, String), Arc<PluginHandleInner>>>,
    pub(super) box_specs: RwLock<HashMap<(String, String), LoadedBoxSpec>>,
}

impl PluginLoaderV2 {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            config: None,
            config_path: None,
            cached_toml: None,
            singletons: RwLock::new(HashMap::new()),
            box_specs: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_config(&mut self, config_path: &str) -> BidResult<()> {
        config::load_config(self, config_path)
    }

    pub fn load_all_plugins(&self) -> BidResult<()> {
        library::load_all_plugins(self)
    }

    pub fn load_plugin_direct(&self, lib_name: &str, lib_def: &LibraryDefinition) -> BidResult<()> {
        library::load_plugin(self, lib_name, lib_def)
    }

    pub fn box_invoke_fn_for_type_id(&self, type_id: u32) -> Option<BoxInvokeFn> {
        metadata::box_invoke_fn_for_type_id(self, type_id)
    }

    pub fn metadata_for_type_id(&self, type_id: u32) -> Option<PluginBoxMetadata> {
        metadata::metadata_for_type_id(self, type_id)
    }

    pub fn construct_existing_instance(
        &self,
        type_id: u32,
        instance_id: u32,
    ) -> Option<Box<dyn NyashBox>> {
        metadata::construct_existing_instance(self, type_id, instance_id)
    }

    pub fn ingest_box_specs_from_nyash_box(
        &self,
        lib_name: &str,
        box_names: &[String],
        nyash_box_toml_path: &std::path::Path,
    ) {
        specs::ingest_box_specs_from_nyash_box(self, lib_name, box_names, nyash_box_toml_path);
    }

    pub fn extern_call(
        &self,
        iface_name: &str,
        method_name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> BidResult<Option<Box<dyn NyashBox>>> {
        super::extern_functions::extern_call(iface_name, method_name, args)
    }

    pub fn config_path_str(&self) -> &str {
        self.config_path
            .as_ref()
            .map(|vp| vp.as_str())
            .unwrap_or("nyash.toml")
    }

    /// Get capabilities bitset for (lib, box) if known.
    pub fn get_caps_for(&self, lib_name: &str, box_type: &str) -> Option<u64> {
        let map = self.box_specs.read().ok()?;
        let spec = map.get(&(lib_name.to_string(), box_type.to_string()))?;
        spec.capabilities
    }

    /// Get TOML value with reload support (unified helper)
    ///
    /// Returns cached TOML value unless NYASH_PLUGIN_CONFIG_RELOAD=1 is set,
    /// in which case it reloads from the config file.
    pub(crate) fn get_toml_value(&self) -> BidResult<toml::Value> {
        let reload = crate::runtime::env_gate_box::bool_any(&["NYASH_PLUGIN_CONFIG_RELOAD", "HAKO_PLUGIN_CONFIG_RELOAD"]);
        if !reload {
            if let Some(v) = self.cached_toml.clone() {
                return Ok(v);
            }
        }
        let cfg_path = self.config_path_str();
        let content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
        toml::from_str(&content).map_err(|_| BidError::PluginError)
    }
}
