/*!
 * Plugin Box Factory
 * 
 * Handles creation of plugin-based Box types through BID/FFI system
 * Integrates with v2 plugin system (BoxFactoryRegistry)
 */

use super::BoxFactory;
use crate::box_trait::NyashBox;
use crate::interpreter::RuntimeError;
use crate::runtime::get_global_registry;

/// Factory for plugin-based Box types
pub struct PluginBoxFactory {
    // Uses the global BoxFactoryRegistry from v2 plugin system
}

impl PluginBoxFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl BoxFactory for PluginBoxFactory {
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Use the existing v2 plugin system
        let registry = get_global_registry();
        
        if let Some(_provider) = registry.get_provider(name) {
            registry.create_box(name, args)
                .map_err(|e| RuntimeError::InvalidOperation {
                    message: format!("Plugin Box creation failed: {}", e),
                })
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("No plugin provider for Box type: {}", name),
            })
        }
    }
    
    fn box_types(&self) -> Vec<&str> {
        // TODO: Get list from BoxFactoryRegistry
        // For now, return empty as registry doesn't expose this yet
        vec![]
    }
    
    fn is_available(&self) -> bool {
        // Check if any plugins are loaded
        let _registry = get_global_registry();
        // TODO: Add method to check if registry has any providers
        true
    }
}