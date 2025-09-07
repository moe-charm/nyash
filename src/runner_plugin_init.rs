/*!
 * Runner plugin initialization (extracted from runner.rs)
 *
 * Purpose: Initialize v2 plugin system from nyash.toml and apply config
 * Behavior: Quiet by default; use NYASH_CLI_VERBOSE=1 or NYASH_DEBUG_PLUGIN=1 for logs
 */

use crate::runtime::{init_global_plugin_host, get_global_registry, get_global_plugin_host, PluginConfig};

pub fn init_bid_plugins() {
    let cli_verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
    let plugin_debug = std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1");
    if plugin_debug { eprintln!("🔍 DEBUG: Initializing v2 plugin system"); }

    if let Ok(()) = init_global_plugin_host("nyash.toml") {
        if plugin_debug || cli_verbose {
            eprintln!("🔌 plugin host initialized from nyash.toml");
            // Show which plugin loader backend compiled in (enabled/stub)
            println!("[plugin-loader] backend={}", crate::runtime::plugin_loader_v2::backend_kind());
        }
        let host = get_global_plugin_host();
        let host = host.read().unwrap();
        if let Some(config) = host.config_ref() {
            let registry = get_global_registry();
            for (lib_name, lib_def) in &config.libraries {
                for box_name in &lib_def.boxes {
                    if plugin_debug { eprintln!("  📦 Registering plugin provider for {}", box_name); }
                    registry.apply_plugin_config(&PluginConfig { plugins: [(box_name.clone(), lib_name.clone())].into(), });
                }
            }
            if plugin_debug || cli_verbose {
                eprintln!("✅ plugin host fully configured");
            }
        }
    } else if plugin_debug || cli_verbose {
        eprintln!("⚠️ Failed to load nyash.toml - plugins disabled");
    }
}
