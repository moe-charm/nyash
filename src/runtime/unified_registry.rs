/*!
 * Global Unified Box Registry
 *
 * Manages the global instance of UnifiedBoxRegistry
 * Integrates all Box creation sources (builtin, user-defined, plugin)
 */

use crate::box_factory::builtin::BuiltinBoxFactory;
#[cfg(feature = "plugins")]
use crate::box_factory::plugin::PluginBoxFactory;
use crate::box_factory::UnifiedBoxRegistry;
use std::sync::{Arc, Mutex, OnceLock};

// For early plugin load and provider registration (plugin-first preferred)
// keep imports minimal to avoid warnings; plugin boot handled by plugin_boot_box

/// Global registry instance
static GLOBAL_REGISTRY: OnceLock<Arc<Mutex<UnifiedBoxRegistry>>> = OnceLock::new();

/// Initialize the global unified registry
pub fn init_global_unified_registry() {
    GLOBAL_REGISTRY.get_or_init(|| {
        // Phase 15.5: Use environment variable policy (StrictPluginFirst for "Everything is Plugin")
        let mut registry = UnifiedBoxRegistry::with_env_policy();
        // Default: enable builtins unless building with feature "plugins-only"
        #[cfg(not(feature = "plugins-only"))]
        {
            registry.register(std::sync::Arc::new(BuiltinBoxFactory::new()));
        }

        // Register plugin Box factory (primary)
        #[cfg(feature = "plugins")]
        {
            registry.register(Arc::new(PluginBoxFactory::new()));
        }

        // TODO: User-defined Box factory will be registered by interpreter

        // Phase 15.5: FactoryPolicy determines actual priority order
        // StrictPluginFirst: plugins > user > builtin (SOLVES StringBox/IntegerBox issue)
        // BuiltinFirst: builtin > user > plugin (legacy default)

        // Register minimal static metadata for core collections (no invoke yet)
        crate::runtime::static_plugins::register_static_plugins();

        // Early plugin load (best-effort):
        // If policy/environment indicates plugin usage, attempt to load nyash.toml/hako.toml
        // and register providers into the v2 BoxFactoryRegistry so `new ArrayBox/MapBox` works
        // even when creation happens before runner plugin init.
        // Defer plugin boot to PluginBootBox (idempotent)
        let _ = crate::runtime::plugin_boot_box::boot();

        Arc::new(Mutex::new(registry))
    });
}

/// Get the global unified registry
pub fn get_global_unified_registry() -> Arc<Mutex<UnifiedBoxRegistry>> {
    init_global_unified_registry();
    GLOBAL_REGISTRY.get().unwrap().clone()
}

/// Register a user-defined Box factory (called by interpreter)
pub fn register_user_defined_factory(factory: Arc<dyn crate::box_factory::BoxFactory>) {
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
