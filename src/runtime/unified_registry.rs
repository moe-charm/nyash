/*!
 * Global Unified Box Registry
 * 
 * Manages the global instance of UnifiedBoxRegistry
 * Integrates all Box creation sources (builtin, user-defined, plugin)
 */

use super::super::box_factory::{UnifiedBoxRegistry, builtin::BuiltinBoxFactory, plugin::PluginBoxFactory};
use std::sync::{Arc, Mutex, OnceLock};

/// Global registry instance
static GLOBAL_REGISTRY: OnceLock<Arc<Mutex<UnifiedBoxRegistry>>> = OnceLock::new();

/// Initialize the global unified registry
pub fn init_global_unified_registry() {
    GLOBAL_REGISTRY.get_or_init(|| {
        let mut registry = UnifiedBoxRegistry::new();
        
        // Register built-in Box factory (highest priority)
        registry.register(Arc::new(BuiltinBoxFactory::new()));
        
        // Register plugin Box factory (lowest priority)
        registry.register(Arc::new(PluginBoxFactory::new()));
        
        // TODO: User-defined Box factory will be registered by interpreter
        
        Arc::new(Mutex::new(registry))
    });
}

/// Get the global unified registry
pub fn get_global_unified_registry() -> Arc<Mutex<UnifiedBoxRegistry>> {
    init_global_unified_registry();
    GLOBAL_REGISTRY.get().unwrap().clone()
}

/// Register a user-defined Box factory (called by interpreter)
pub fn register_user_defined_factory(factory: Arc<dyn super::super::box_factory::BoxFactory>) {
    let registry = get_global_unified_registry();
    let mut registry_lock = registry.lock().unwrap();
    
    // Insert at position 1 (after builtin, before plugin)
    // This maintains priority: builtin > user > plugin
    if registry_lock.factories.len() >= 2 {
        registry_lock.factories.insert(1, factory);
    } else {
        registry_lock.register(factory);
    }
}