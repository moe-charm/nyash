/*!
 * Runner plugin initialization (extracted from runner.rs)
 *
 * Purpose: Initialize v2 plugin system from nyash.toml and apply config
 * Behavior: Quiet by default; use NYASH_CLI_VERBOSE=1 or NYASH_DEBUG_PLUGIN=1 for logs
 */

use crate::runtime::{
    get_global_plugin_host, get_global_registry, init_global_plugin_host, PluginConfig,
};
use crate::config::env;

pub fn init_bid_plugins() {
    let cli_verbose = env::cli_verbose();
    let plugin_debug = std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1");
    if plugin_debug {
        eprintln!("🔍 DEBUG: Initializing v2 plugin system");
    }

    // Direct library load (dev): NYASH_PLUGIN_DIRECT_LIB, _PATH, _BOXES (comma-separated)
    if let (Ok(lib), Ok(path), Ok(boxes)) = (
        std::env::var("NYASH_PLUGIN_DIRECT_LIB"),
        std::env::var("NYASH_PLUGIN_DIRECT_PATH"),
        std::env::var("NYASH_PLUGIN_DIRECT_BOXES"),
    ) {
        let boxes_vec: Vec<String> = boxes.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if !boxes_vec.is_empty() {
            let host = get_global_plugin_host();
            if host.read().unwrap().load_library_direct(&lib, &path, &boxes_vec).is_ok() {
                if (plugin_debug || cli_verbose) && !env::cli_quiet() {
                    eprintln!("🔌 plugin host loaded direct lib: {} ({}) -> boxes={:?}", lib, path, boxes_vec);
                }
                // Also register providers in registry
                let registry = get_global_registry();
                for bx in &boxes_vec {
                    registry.apply_plugin_config(&PluginConfig { plugins: [(bx.clone(), lib.clone())].into(), });
                }
                // Do not return; allow default config to load as well
            }
        }
    }

    // Allow explicit config override for tests/tools
    if let Ok(custom_cfg) = std::env::var("NYASH_PLUGIN_CONFIG") {
        if !custom_cfg.trim().is_empty() {
            if init_global_plugin_host(&custom_cfg).is_ok() {
                // Register providers into BoxFactoryRegistry
                if let Some(config) = get_global_plugin_host().read().unwrap().config_ref() {
                    let registry = get_global_registry();
                    for (lib_name, lib_def) in &config.libraries {
                        for box_name in &lib_def.boxes {
                            if plugin_debug { eprintln!("  📦 Registering plugin provider for {}", box_name); }
                            registry.apply_plugin_config(&PluginConfig { plugins: [(box_name.clone(), lib_name.clone())].into(), });
                        }
                    }
                }
                if (plugin_debug || cli_verbose) && !env::cli_quiet() {
                    eprintln!("🔌 plugin host initialized from {}", custom_cfg);
                    eprintln!(
                        "[plugin-loader] backend={}",
                        crate::runtime::plugin_loader_v2::backend_kind()
                    );
                    eprintln!("✅ plugin host fully configured");
                }
                return;
            }
        }
    }

    // Try hako.toml/nyash.toml from CWD, then fallback to hakorune.toml and $NYASH_ROOT/*
    let mut tried: Vec<String> = Vec::new();
    let mut ok = false;
    let path_hako = "hako.toml".to_string();
    tried.push(path_hako.clone());
    if init_global_plugin_host(&path_hako).is_ok() {
        ok = true;
    } else if {
        let ny = "nyash.toml".to_string();
        tried.push(ny.clone());
        init_global_plugin_host(&ny).is_ok()
    } {
        ok = true;
    } else if {
        let alt = "hakorune.toml".to_string();
        tried.push(alt.clone());
        init_global_plugin_host(&alt).is_ok()
    } {
        ok = true;
    } else if let Ok(root) = std::env::var("NYASH_ROOT") {
        let ph = std::path::Path::new(&root).join("hako.toml");
        let phs = ph.to_string_lossy().to_string();
        tried.push(phs.clone());
        if init_global_plugin_host(&phs).is_ok() {
            ok = true;
        } else {
            let pn = std::path::Path::new(&root).join("nyash.toml");
            let pns = pn.to_string_lossy().to_string();
            tried.push(pns.clone());
            if init_global_plugin_host(&pns).is_ok() { ok = true; }
            else {
                let p2 = std::path::Path::new(&root).join("hakorune.toml");
                let p2s = p2.to_string_lossy().to_string();
                tried.push(p2s.clone());
                if init_global_plugin_host(&p2s).is_ok() { ok = true; }
            }
        }
    }
    if ok {
        if (plugin_debug || cli_verbose) && !env::cli_quiet() {
            eprintln!("🔌 plugin host initialized from nyash.toml");
            // Show which plugin loader backend compiled in (enabled/stub)
            eprintln!(
                "[plugin-loader] backend={}",
                crate::runtime::plugin_loader_v2::backend_kind()
            );
        }
        let host = get_global_plugin_host();
        let host = host.read().unwrap();
        if let Some(config) = host.config_ref() {
            let registry = get_global_registry();
            for (lib_name, lib_def) in &config.libraries {
                for box_name in &lib_def.boxes {
                    if plugin_debug {
                        eprintln!("  📦 Registering plugin provider for {}", box_name);
                    }
                    registry.apply_plugin_config(&PluginConfig {
                        plugins: [(box_name.clone(), lib_name.clone())].into(),
                    });
                }
            }
            if (plugin_debug || cli_verbose) && !env::cli_quiet() {
                eprintln!("✅ plugin host fully configured");
            }
        }
    } else if (plugin_debug || cli_verbose) && !env::cli_quiet() {
        // Keep first line stable for smoke filter; print details only in verbose logs.
        eprintln!("Failed to load nyash.toml - plugins disabled");
        if plugin_debug {
            eprintln!("[plugin-loader] tried paths: {:?}", tried);
        }
    }
}
