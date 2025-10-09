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
use crate::runtime::{get_global_plugin_host, get_global_registry, init_global_plugin_host, PluginConfig};

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

        // Early plugin load (best-effort):
        // If policy/environment indicates plugin usage, attempt to load nyash.toml/hako.toml
        // and register providers into the v2 BoxFactoryRegistry so `new ArrayBox/MapBox` works
        // even when creation happens before runner plugin init.
        let policy = std::env::var("NYASH_PLUGIN_POLICY")
            .ok()
            .or_else(|| std::env::var("HAKO_PLUGIN_POLICY").ok());
        let disable_by_policy = match policy.as_deref() {
            None => true, // default OFF unless explicitly set to auto/force
            Some(s) if s.eq_ignore_ascii_case("off") => true,
            Some(s) if s.eq_ignore_ascii_case("auto") || s.eq_ignore_ascii_case("force") => false,
            _ => false,
        };
        if !disable_by_policy && std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            // Try a small set of common config paths quietly
            // Prefer nyash.toml first (project canonical), then fall back to hako/hakorune
            let candidates = ["nyash.toml", "hako.toml", "hakorune.toml"]; 
            for cfg in candidates.iter() {
                if init_global_plugin_host(cfg).is_ok() {
                    // Mirror runner behavior: register providers from loaded config
                    if let Ok(host_guard) = get_global_plugin_host().read() {
                        if let Some(conf) = host_guard.config_ref() {
                            let reg = get_global_registry();
                            for (lib_name, lib_def) in &conf.libraries {
                                for box_name in &lib_def.boxes {
                                    reg.apply_plugin_config(&PluginConfig { plugins: [(box_name.clone(), lib_name.clone())].into(), });
                                }
                            }
                        }
                    }
                    break;
                } else {
                    // Partial load fallback: load only core override boxes to avoid hard failure
                    if let Ok(cfg_v2) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                        let host = get_global_plugin_host();
                        let core_boxes = ["ArrayBox", "MapBox", "StringBox"]; 
                        for (lib_name, lib_def) in &cfg_v2.libraries {
                            let include = lib_def.boxes.iter().any(|b| core_boxes.iter().any(|c| c==b));
                            if include {
                                let _ = host.read().ok().and_then(|h| h.load_library_direct(lib_name, &lib_def.path, &lib_def.boxes).ok());
                                // Register providers for boxes listed under this lib (subset is fine)
                                let reg = get_global_registry();
                                for box_name in &lib_def.boxes {
                                    if core_boxes.iter().any(|c| c == box_name) {
                                        reg.apply_plugin_config(&PluginConfig { plugins: [(box_name.clone(), lib_name.clone())].into(), });
                                    }
                                }
                            }
                        }
                        // Even if partial, stop scanning further candidates to keep deterministic
                        break;
                    }
                }
            }
        }

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
