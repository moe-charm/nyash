use crate::bid::{BidError, BidResult, LoadedPlugin, MethodTypeInfo, ArgTypeMapping};
use crate::config::nyash_toml_v2::{NyashConfigV2, BoxTypeConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use once_cell::sync::OnceCell;

/// Registry mapping Box names and type IDs to loaded plugins
pub struct PluginRegistry {
    by_name: HashMap<String, LoadedPlugin>,
    by_type_id: HashMap<u32, String>,
    /// 型情報: Box名 -> メソッド名 -> MethodTypeInfo
    type_info: HashMap<String, HashMap<String, MethodTypeInfo>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { 
            by_name: HashMap::new(), 
            by_type_id: HashMap::new(),
            type_info: HashMap::new(),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&LoadedPlugin> {
        self.by_name.get(name)
    }

    pub fn get_by_type_id(&self, type_id: u32) -> Option<&LoadedPlugin> {
        self.by_type_id.get(&type_id).and_then(|name| self.by_name.get(name))
    }
    
    /// 指定されたBox・メソッドの型情報を取得
    pub fn get_method_type_info(&self, box_name: &str, method_name: &str) -> Option<&MethodTypeInfo> {
        self.type_info.get(box_name)?.get(method_name)
    }

    /// Load plugins based on nyash.toml v2
    pub fn load_from_config(path: &str) -> BidResult<Self> {
        eprintln!("🔍 DEBUG: load_from_config called with path: {}", path);
        
        // Parse nyash.toml v2
        let config = NyashConfigV2::from_file(path).map_err(|e| {
            eprintln!("🔍 DEBUG: Failed to parse config: {}", e);
            BidError::PluginError
        })?;
        
        // Also need raw toml for nested box configs
        let raw_config: toml::Value = toml::from_str(&fs::read_to_string(path).unwrap_or_default())
            .unwrap_or(toml::Value::Table(Default::default()));
        
        let mut reg = Self::new();
        
        // Process each library
        for (lib_name, lib_def) in &config.libraries {
            eprintln!("🔍 Processing library: {} -> {}", lib_name, lib_def.path);
            
            // Resolve plugin path
            let plugin_path = if std::path::Path::new(&lib_def.path).exists() {
                lib_def.path.clone()
            } else {
                config.resolve_plugin_path(&lib_def.path)
                    .unwrap_or(lib_def.path.clone())
            };
            
            eprintln!("🔍 Loading plugin from: {}", plugin_path);
            
            // Load the plugin (simplified - no more init/abi)
            // For now, we'll use the old loader but ignore type_id from plugin
            // TODO: Update LoadedPlugin to work with invoke-only plugins
            
            // Process each box type provided by this library
            for box_name in &lib_def.boxes {
                eprintln!("  📦 Registering box type: {}", box_name);
                
                // Get box config from nested structure
                if let Some(box_config) = config.get_box_config(lib_name, box_name, &raw_config) {
                    eprintln!("    - Type ID: {}", box_config.type_id);
                    eprintln!("    - ABI version: {}", box_config.abi_version);
                    eprintln!("    - Methods: {}", box_config.methods.len());
                    
                    // Store method info
                    let mut method_info = HashMap::new();
                    for (method_name, method_def) in &box_config.methods {
                        eprintln!("      • {}: method_id={}", method_name, method_def.method_id);
                        
                        // For now, create empty MethodTypeInfo
                        // Arguments are checked at runtime via TLV
                        method_info.insert(method_name.clone(), MethodTypeInfo {
                            args: vec![],
                            returns: None,
                        });
                    }
                    
                    reg.type_info.insert(box_name.clone(), method_info);
                    
                    // TODO: Create simplified LoadedPlugin without init/abi
                    // For now, skip actual plugin loading
                    eprintln!("    ⚠️  Plugin loading temporarily disabled (migrating to invoke-only)");
                }
            }
        }
        
        eprintln!("🔍 Registry loaded with {} box types", reg.type_info.len());
        
        Ok(reg)
    }
}

// ===== Global registry (for interpreter access) =====
static PLUGIN_REGISTRY: OnceCell<PluginRegistry> = OnceCell::new();

/// Initialize global plugin registry from config
pub fn init_global_from_config(path: &str) -> BidResult<()> {
    eprintln!("🔍 DEBUG: init_global_from_config called with path: {}", path);
    let reg = PluginRegistry::load_from_config(path)?;
    let _ = PLUGIN_REGISTRY.set(reg);
    Ok(())
}

/// Get global plugin registry if initialized
pub fn global() -> Option<&'static PluginRegistry> {
    PLUGIN_REGISTRY.get()
}