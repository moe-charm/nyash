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
        for type_name in types {
            // First registered factory wins (priority order)
            cache.entry(type_name.to_string())
                .or_insert(factory_index);
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
        
        // Fallback: linear search through all factories
        for factory in &self.factories {
            if factory.box_types().contains(&name) && factory.is_available() {
                return factory.create_box(name, args);
            }
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Unknown Box type: {}", name),
        })
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