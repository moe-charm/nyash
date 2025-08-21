/*!
 * Unified Box Factory Architecture
 * 
 * Phase 9.78: 統合BoxFactoryアーキテクチャ
 * すべてのBox生成（ビルトイン、ユーザー定義、プラグイン）を統一的に扱う
 * 
 * Design principles:
 * - "Everything is Box" 哲学の実装レベルでの体現
 * - birth/finiライフサイクルの明確な責務分離
 * - 保守性と拡張性の劇的向上
 */

use crate::box_trait::NyashBox;
use crate::interpreter::RuntimeError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Unified interface for all Box creation
pub trait BoxFactory: Send + Sync {
    /// Create a new Box instance with given arguments
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError>;
    
    /// Check if this factory is currently available
    fn is_available(&self) -> bool {
        true
    }
    
    /// Get list of Box types this factory can create
    fn box_types(&self) -> Vec<&str>;
    
    /// Check if this factory supports birth/fini lifecycle
    fn supports_birth(&self) -> bool {
        true
    }

    /// Identify builtin factory to enforce reserved-name protections
    fn is_builtin_factory(&self) -> bool {
        false
    }
}

/// Registry that manages all BoxFactory implementations
pub struct UnifiedBoxRegistry {
    /// Ordered list of factories (priority: builtin > user > plugin)
    pub factories: Vec<Arc<dyn BoxFactory>>,
    
    /// Quick lookup cache for performance
    type_cache: RwLock<HashMap<String, usize>>, // maps type name to factory index
}

impl UnifiedBoxRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
            type_cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new factory
    pub fn register(&mut self, factory: Arc<dyn BoxFactory>) {
        // Get all types this factory can create
        let types = factory.box_types();
        let factory_index = self.factories.len();
        
        // Update cache
        let mut cache = self.type_cache.write().unwrap();
        // Reserved core types that must remain builtin-owned
        fn is_reserved_type(name: &str) -> bool {
            matches!(
                name,
                // Core value types
                "StringBox" | "IntegerBox" | "BoolBox" | "FloatBox" | "NullBox"
                    // Core containers and result
                    | "ArrayBox" | "MapBox" | "ResultBox"
                    // Core method indirection
                    | "MethodBox"
            )
        }
        for type_name in types {
            // Enforce reserved names: only builtin factory may claim them
            if is_reserved_type(type_name) && !factory.is_builtin_factory() {
                eprintln!(
                    "[UnifiedBoxRegistry] ❌ Rejecting registration of reserved type '{}' by non-builtin factory #{}",
                    type_name, factory_index
                );
                continue;
            }

            // First registered factory wins (priority order)
            let entry = cache.entry(type_name.to_string());
            use std::collections::hash_map::Entry;
            match entry {
                Entry::Occupied(existing) => {
                    // Collision: type already claimed by earlier factory
                    eprintln!("[UnifiedBoxRegistry] ⚠️ Duplicate registration for '{}': keeping factory #{}, ignoring later factory #{}",
                              existing.key(), existing.get(), factory_index);
                }
                Entry::Vacant(v) => {
                    v.insert(factory_index);
                }
            }
        }
        
        self.factories.push(factory);
    }
    
    /// Create a Box using the unified interface
    pub fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Check cache first
        let cache = self.type_cache.read().unwrap();
        if let Some(&factory_index) = cache.get(name) {
            if let Some(factory) = self.factories.get(factory_index) {
                if factory.is_available() {
                    return factory.create_box(name, args);
                }
            }
        }
        drop(cache);
        
        // Linear search through all factories
        for factory in &self.factories {
            if !factory.is_available() {
                continue;
            }
            
            // For factories that advertise types, check if they support this type
            let box_types = factory.box_types();
            if !box_types.is_empty() && !box_types.contains(&name) {
                continue;
            }
            
            // Try to create the box (factories with empty box_types() will always be tried)
            match factory.create_box(name, args) {
                Ok(boxed) => return Ok(boxed),
                Err(_) => continue, // Try next factory
            }
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Unknown Box type: {}", name),
        })
    }

    /// Check whether a type name is known to the registry
    pub fn has_type(&self, name: &str) -> bool {
        // Check cache first
        {
            let cache = self.type_cache.read().unwrap();
            if let Some(&idx) = cache.get(name) {
                if let Some(factory) = self.factories.get(idx) {
                    if factory.is_available() { return true; }
                }
            }
        }
        // Fallback: scan factories that can enumerate types
        for factory in &self.factories {
            if !factory.is_available() { continue; }
            let types = factory.box_types();
            if !types.is_empty() && types.contains(&name) { return true; }
        }
        false
    }
    
    /// Get all available Box types
    pub fn available_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        for factory in &self.factories {
            if factory.is_available() {
                for type_name in factory.box_types() {
                    types.push(type_name.to_string());
                }
            }
        }
        types.sort();
        types.dedup();
        types
    }
}

/// Re-export submodules
pub mod builtin;
pub mod user_defined;
pub mod plugin;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = UnifiedBoxRegistry::new();
        assert_eq!(registry.available_types().len(), 0);
    }
}
