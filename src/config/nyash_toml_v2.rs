//! nyash.toml v2 configuration parser
//! 
//! Ultimate simple design: nyash.toml-centric architecture + minimal FFI
//! No Host VTable, single entry point (nyash_plugin_invoke)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct NyashConfigV2 {
    /// Library definitions (multi-box capable)
    #[serde(default)]
    pub libraries: HashMap<String, LibraryDefinition>,
    
    /// Plugin search paths
    #[serde(default)]
    pub plugin_paths: PluginPaths,
}

/// Library definition (simplified)
#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryDefinition {
    /// Box types provided by this library
    pub boxes: Vec<String>,
    
    /// Path to the shared library
    pub path: String,
}

/// Plugin search paths
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PluginPaths {
    #[serde(default)]
    pub search_paths: Vec<String>,
}

/// Box type configuration (nested under library)
#[derive(Debug, Deserialize, Serialize)]
pub struct BoxTypeConfig {
    /// Box type ID
    pub type_id: u32,
    
    /// ABI version (default: 1)
    #[serde(default = "default_abi_version")]
    pub abi_version: u32,
    
    /// Method definitions
    pub methods: HashMap<String, MethodDefinition>,
}

/// Method definition (simplified - no argument info needed)
#[derive(Debug, Deserialize, Serialize)]
pub struct MethodDefinition {
    /// Method ID for FFI
    pub method_id: u32,
}

fn default_abi_version() -> u32 {
    1
}

impl NyashConfigV2 {
    /// Parse nyash.toml file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        
        // Parse as raw TOML first to handle nested box configs
        let mut config: toml::Value = toml::from_str(&content)?;
        
        // Extract library definitions
        let libraries = Self::parse_libraries(&mut config)?;
        
        // Extract plugin paths
        let plugin_paths = if let Some(paths) = config.get("plugin_paths") {
            paths.clone().try_into::<PluginPaths>()?
        } else {
            PluginPaths::default()
        };
        
        Ok(NyashConfigV2 {
            libraries,
            plugin_paths,
        })
    }
    
    /// Parse library definitions with nested box configs
    fn parse_libraries(config: &mut toml::Value) -> Result<HashMap<String, LibraryDefinition>, Box<dyn std::error::Error>> {
        let mut libraries = HashMap::new();
        
        if let Some(libs_section) = config.get("libraries").and_then(|v| v.as_table()) {
            for (lib_name, lib_value) in libs_section {
                if let Some(lib_table) = lib_value.as_table() {
                    let boxes = lib_table.get("boxes")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();
                    
                    let path = lib_table.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(lib_name)
                        .to_string();
                    
                    libraries.insert(lib_name.clone(), LibraryDefinition {
                        boxes,
                        path,
                    });
                }
            }
        }
        
        Ok(libraries)
    }
    
    /// Get box configuration from nested structure
    /// e.g., [libraries."libnyash_filebox_plugin.so".FileBox]
    pub fn get_box_config(&self, lib_name: &str, box_name: &str, config_value: &toml::Value) -> Option<BoxTypeConfig> {
        config_value
            .get("libraries")
            .and_then(|v| v.get(lib_name))
            .and_then(|v| v.get(box_name))
            .and_then(|v| v.clone().try_into::<BoxTypeConfig>().ok())
    }
    
    /// Find library that provides a specific box type
    pub fn find_library_for_box(&self, box_type: &str) -> Option<(&str, &LibraryDefinition)> {
        self.libraries.iter()
            .find(|(_, lib)| lib.boxes.contains(&box_type.to_string()))
            .map(|(name, lib)| (name.as_str(), lib))
    }
    
    /// Resolve plugin path from search paths
    pub fn resolve_plugin_path(&self, plugin_name: &str) -> Option<String> {
        // Try exact path first
        if std::path::Path::new(plugin_name).exists() {
            return Some(plugin_name.to_string());
        }
        
        // Search in configured paths
        for search_path in &self.plugin_paths.search_paths {
            let path = std::path::Path::new(search_path).join(plugin_name);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_v2_config() {
        let toml_str = r#"
[libraries]
"libnyash_filebox_plugin.so" = {
    boxes = ["FileBox"],
    path = "./target/release/libnyash_filebox_plugin.so"
}

[libraries."libnyash_filebox_plugin.so".FileBox]
type_id = 6
abi_version = 1

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
birth = { method_id = 0 }
open = { method_id = 1 }
close = { method_id = 4 }
"#;
        
        let config: toml::Value = toml::from_str(toml_str).unwrap();
        let nyash_config = NyashConfigV2::from_file("test.toml");
        // Test would need actual file...
    }
}