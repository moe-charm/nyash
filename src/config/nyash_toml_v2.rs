//! nyash.toml v2 configuration parser
//! 
//! Supports both legacy single-box plugins and new multi-box plugins

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct NyashConfigV2 {
    /// Plugins section (contains both legacy and new format)
    #[serde(default)]
    pub plugins: PluginsSection,
}

/// Plugins section (both legacy and v2)
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PluginsSection {
    /// Legacy single-box plugins (box_name -> plugin_name)
    #[serde(flatten)]
    pub legacy_plugins: HashMap<String, String>,
    
    /// New multi-box plugin libraries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<HashMap<String, LibraryDefinition>>,
    
    /// Box type definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<HashMap<String, BoxTypeDefinition>>,
}

/// Plugin libraries section (not used in new structure)
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginLibraries {
    #[serde(flatten)]
    pub libraries: HashMap<String, LibraryDefinition>,
}

/// Library definition
#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryDefinition {
    pub plugin_path: String,
    pub provides: Vec<String>,
}

/// Box type definition
#[derive(Debug, Deserialize, Serialize)]
pub struct BoxTypeDefinition {
    pub library: String,
    pub type_id: u32,
    pub methods: HashMap<String, MethodDefinition>,
}

/// Method definition
#[derive(Debug, Deserialize, Serialize)]
pub struct MethodDefinition {
    #[serde(default)]
    pub args: Vec<ArgumentDefinition>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
}

/// Argument definition
#[derive(Debug, Deserialize, Serialize)]
pub struct ArgumentDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    pub from: String,
    pub to: String,
}

impl NyashConfigV2 {
    /// Parse nyash.toml file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: NyashConfigV2 = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Check if using v2 format
    pub fn is_v2_format(&self) -> bool {
        self.plugins.libraries.is_some() || self.plugins.types.is_some()
    }
    
    /// Get all box types provided by a library
    pub fn get_box_types_for_library(&self, library_name: &str) -> Vec<String> {
        if let Some(libs) = &self.plugins.libraries {
            if let Some(lib_def) = libs.get(library_name) {
                return lib_def.provides.clone();
            }
        }
        vec![]
    }
    
    /// Get library name for a box type
    pub fn get_library_for_box_type(&self, box_type: &str) -> Option<String> {
        // Check v2 format first
        if let Some(types) = &self.plugins.types {
            if let Some(type_def) = types.get(box_type) {
                return Some(type_def.library.clone());
            }
        }
        
        // Fall back to legacy format
        self.plugins.legacy_plugins.get(box_type).cloned()
    }
    
    /// Access legacy plugins directly (for backward compatibility)
    pub fn get_legacy_plugins(&self) -> &HashMap<String, String> {
        &self.plugins.legacy_plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_legacy_format() {
        let toml_str = r#"
[plugins]
FileBox = "nyash-filebox-plugin"

[plugins.FileBox.methods]
read = { args = [] }
"#;
        
        let config: NyashConfigV2 = toml::from_str(toml_str).unwrap();
        assert_eq!(config.plugins.get("FileBox"), Some(&"nyash-filebox-plugin".to_string()));
        assert!(!config.is_v2_format());
    }
    
    #[test]
    fn test_parse_v2_format() {
        let toml_str = r#"
[plugins.libraries]
"nyash-network" = {
    plugin_path = "libnyash_network.so",
    provides = ["SocketBox", "HTTPServerBox"]
}

[plugins.types.SocketBox]
library = "nyash-network"
type_id = 100
methods = { bind = { args = [] } }
"#;
        
        let config: NyashConfigV2 = toml::from_str(toml_str).unwrap();
        assert!(config.is_v2_format());
        assert_eq!(config.get_box_types_for_library("nyash-network"), vec!["SocketBox", "HTTPServerBox"]);
    }
}