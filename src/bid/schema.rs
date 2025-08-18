/*!
 * BID Schema Parsing - YAML/JSON schema for Box Interface Definitions
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::bid::BidError;

/// BID Definition - Root structure for BID files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidDefinition {
    pub version: u32,
    pub interfaces: Vec<BidInterface>,
    pub metadata: Option<BidMetadata>,
}

/// Metadata for a BID definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
}

/// Interface definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidInterface {
    pub name: String,
    #[serde(rename = "box")]
    pub box_type: Option<String>,  // Box type name
    pub methods: Vec<BidMethod>,
}

/// Method definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidMethod {
    pub name: String,
    pub params: Vec<BidParameter>,
    pub returns: BidTypeRef,
    pub effect: Option<String>,
}

/// Parameter definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidParameter {
    #[serde(flatten)]
    pub param_type: BidTypeRef,
}

/// Type reference in BID files
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BidTypeRef {
    /// Simple type: { string: "name" }
    Named(HashMap<String, String>),
    /// Just a type name: "void"
    Simple(String),
}

impl BidDefinition {
    /// Load BID definition from YAML file
    pub fn load_from_file(path: &Path) -> Result<Self, BidError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BidError::IoError(e.to_string()))?;
        
        let bid: BidDefinition = serde_yaml::from_str(&content)
            .map_err(|e| BidError::ParseError(format!("YAML parse error: {}", e)))?;
        
        // Validate the definition
        bid.validate()?;
        
        Ok(bid)
    }
    
    /// Validate the BID definition
    pub fn validate(&self) -> Result<(), BidError> {
        // Check version
        if self.version > 1 {
            return Err(BidError::UnsupportedVersion(self.version));
        }
        
        // Check for duplicate interface names
        let mut interface_names = std::collections::HashSet::new();
        for interface in &self.interfaces {
            if interface_names.contains(&interface.name) {
                return Err(BidError::DuplicateInterface(interface.name.clone()));
            }
            interface_names.insert(interface.name.clone());
        }
        
        // Validate each interface
        for interface in &self.interfaces {
            interface.validate()?;
        }
        
        Ok(())
    }
    
    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<&BidInterface> {
        self.interfaces.iter().find(|i| i.name == name)
    }
    
    /// Get the definition name (from metadata or derived from first interface)
    pub fn name(&self) -> String {
        if let Some(ref metadata) = self.metadata {
            if let Some(ref name) = metadata.name {
                return name.clone();
            }
        }
        
        // Derive from first interface name
        if let Some(interface) = self.interfaces.first() {
            // Extract the last part of the interface name
            // e.g., "env.console" -> "console"
            interface.name.split('.').last().unwrap_or(&interface.name).to_string()
        } else {
            "unknown".to_string()
        }
    }
}

impl BidInterface {
    /// Validate the interface
    pub fn validate(&self) -> Result<(), BidError> {
        // Check for duplicate method names
        let mut method_names = std::collections::HashSet::new();
        for method in &self.methods {
            if method_names.contains(&method.name) {
                return Err(BidError::DuplicateMethod(method.name.clone()));
            }
            method_names.insert(method.name.clone());
        }
        
        // Validate each method
        for method in &self.methods {
            method.validate()?;
        }
        
        Ok(())
    }
}

impl BidMethod {
    /// Validate the method
    pub fn validate(&self) -> Result<(), BidError> {
        // Check for duplicate parameter names
        let mut param_names = std::collections::HashSet::new();
        for (i, param) in self.params.iter().enumerate() {
            let param_name = param.get_name().unwrap_or_else(|| format!("param_{}", i));
            if param_names.contains(&param_name) {
                return Err(BidError::DuplicateParameter(param_name));
            }
            param_names.insert(param_name);
        }
        
        Ok(())
    }
}

impl BidParameter {
    /// Get the parameter name (from the type definition)
    pub fn get_name(&self) -> Option<String> {
        match &self.param_type {
            BidTypeRef::Named(map) => {
                // Return the first value (parameter name)
                map.values().next().cloned()
            },
            BidTypeRef::Simple(_) => None,
        }
    }
    
    /// Get the parameter type name
    pub fn get_type(&self) -> String {
        match &self.param_type {
            BidTypeRef::Named(map) => {
                // Return the first key (type name)
                map.keys().next().cloned().unwrap_or_else(|| "unknown".to_string())
            },
            BidTypeRef::Simple(type_name) => type_name.clone(),
        }
    }
}

impl BidTypeRef {
    /// Get the type name
    pub fn type_name(&self) -> String {
        match self {
            BidTypeRef::Named(map) => {
                map.keys().next().cloned().unwrap_or_else(|| "unknown".to_string())
            },
            BidTypeRef::Simple(type_name) => type_name.clone(),
        }
    }
    
    /// Check if this is a void type
    pub fn is_void(&self) -> bool {
        self.type_name() == "void"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_console_bid() {
        let yaml_content = r#"
version: 0
interfaces:
  - name: env.console
    box: Console
    methods:
      - name: log
        params:
          - { string: msg }
        returns: void
        effect: io
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", yaml_content).unwrap();
        
        let bid = BidDefinition::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(bid.version, 0);
        assert_eq!(bid.interfaces.len(), 1);
        
        let interface = &bid.interfaces[0];
        assert_eq!(interface.name, "env.console");
        assert_eq!(interface.box_type, Some("Console".to_string()));
        assert_eq!(interface.methods.len(), 1);
        
        let method = &interface.methods[0];
        assert_eq!(method.name, "log");
        assert_eq!(method.params.len(), 1);
        assert!(method.returns.is_void());
        
        let param = &method.params[0];
        assert_eq!(param.get_type(), "string");
        assert_eq!(param.get_name(), Some("msg".to_string()));
    }

    #[test]
    fn test_bid_definition_name() {
        let yaml_content = r#"
version: 0
interfaces:
  - name: env.console
    methods: []
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", yaml_content).unwrap();
        
        let bid = BidDefinition::load_from_file(temp_file.path()).unwrap();
        assert_eq!(bid.name(), "console");
    }

    #[test]
    fn test_duplicate_interface_validation() {
        let yaml_content = r#"
version: 0
interfaces:
  - name: env.console
    methods: []
  - name: env.console
    methods: []
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", yaml_content).unwrap();
        
        let result = BidDefinition::load_from_file(temp_file.path());
        assert!(result.is_err());
        
        if let Err(BidError::DuplicateInterface(name)) = result {
            assert_eq!(name, "env.console");
        } else {
            panic!("Expected DuplicateInterface error");
        }
    }
}