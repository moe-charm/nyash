use crate::bid::{BidError, BidResult, LoadedPlugin};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use once_cell::sync::OnceCell;

/// Registry mapping Box names and type IDs to loaded plugins
pub struct PluginRegistry {
    by_name: HashMap<String, LoadedPlugin>,
    by_type_id: HashMap<u32, String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { by_name: HashMap::new(), by_type_id: HashMap::new() }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&LoadedPlugin> {
        self.by_name.get(name)
    }

    pub fn get_by_type_id(&self, type_id: u32) -> Option<&LoadedPlugin> {
        self.by_type_id.get(&type_id).and_then(|name| self.by_name.get(name))
    }

    /// Load plugins based on nyash.toml minimal parsing
    pub fn load_from_config(path: &str) -> BidResult<Self> {
        let content = fs::read_to_string(path).map_err(|_| BidError::PluginError)?;

        // Very small parser: look for lines like `FileBox = "nyash-filebox-plugin"`
        let mut mappings: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() { continue; }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim().trim_matches(' ').to_string();
                let val = v.trim().trim_matches('"').to_string();
                if key.chars().all(|c| c.is_alphanumeric() || c == '_' ) && !val.is_empty() {
                    mappings.insert(key, val);
                }
            }
        }

        // Candidate directories
        let mut candidates: Vec<PathBuf> = vec![
            PathBuf::from("./plugins/nyash-filebox-plugin/target/release"),
            PathBuf::from("./plugins/nyash-filebox-plugin/target/debug"),
        ];
        // Also parse plugin_paths.search_paths if present
        if let Some(sp_start) = content.find("search_paths") {
            if let Some(open) = content[sp_start..].find('[') {
                if let Some(close) = content[sp_start + open..].find(']') {
                    let list = &content[sp_start + open + 1.. sp_start + open + close];
                    for item in list.split(',') {
                        let p = item.trim().trim_matches('"');
                        if !p.is_empty() { candidates.push(PathBuf::from(p)); }
                    }
                }
            }
        }

        let mut reg = Self::new();

        for (box_name, plugin_name) in mappings.into_iter() {
            // Find dynamic library path
            if let Some(path) = super::loader::resolve_plugin_path(&plugin_name, &candidates) {
                let loaded = super::loader::LoadedPlugin::load_from_file(&path)?;
                reg.by_type_id.insert(loaded.type_id, box_name.clone());
                reg.by_name.insert(box_name, loaded);
            }
        }

        Ok(reg)
    }
}

// ===== Global registry (for interpreter access) =====
static PLUGIN_REGISTRY: OnceCell<PluginRegistry> = OnceCell::new();

/// Initialize global plugin registry from config
pub fn init_global_from_config(path: &str) -> BidResult<()> {
    let reg = PluginRegistry::load_from_config(path)?;
    let _ = PLUGIN_REGISTRY.set(reg);
    Ok(())
}

/// Get global plugin registry if initialized
pub fn global() -> Option<&'static PluginRegistry> {
    PLUGIN_REGISTRY.get()
}
