//! ProviderBox — Thin boundary for plugin/config/embedded resolution
//!
//! Responsibility:
//! - Ensure plugin host is initialized (from config or partial load)
//! - Provide a single entry point to create boxes with Hako ABI face
//!   (prefer PluginHost → v2 BoxFactoryRegistry → embedded fallback)
//!
//! Notes:
//! - This module is intentionally small and self‑contained to keep the
//!   initialization/selection logic out of VM handlers.

use crate::box_trait::NyashBox;
use crate::box_factory::RuntimeError;

fn plugin_policy_on() -> bool {
    matches!(
        std::env::var("NYASH_PLUGIN_POLICY").ok().or_else(|| std::env::var("HAKO_PLUGIN_POLICY").ok()).as_deref(),
        Some(s) if s.eq_ignore_ascii_case("auto") || s.eq_ignore_ascii_case("force")
    )
}

/// Ensure plugin host and providers are loaded (best‑effort)
pub fn ensure_loaded(config_path: Option<&str>) {
    if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() == Some("1") {
        return;
    }
    if !plugin_policy_on() {
        return;
    }
    // Initialize from explicit or common paths
    let paths: [&str; 3] = [
        config_path.unwrap_or("nyash.toml"),
        "hako.toml",
        "hakorune.toml",
    ];
    for p in paths.iter() {
        if crate::runtime::plugin_loader_unified::init_global_plugin_host(p).is_ok() {
            // Register providers in v2 BoxFactoryRegistry
            if let Ok(host_guard) = crate::runtime::get_global_plugin_host().read() {
                if let Some(cfg) = host_guard.config_ref() {
                    let reg = crate::runtime::get_global_registry();
                    for (lib, def) in &cfg.libraries {
                        for b in &def.boxes {
                            reg.apply_plugin_config(&crate::runtime::PluginConfig { plugins: [(b.clone(), lib.clone())].into(), });
                        }
                    }
                    // Eager-load core boxes (Array/Map/String) to ensure per-Box invoke mapping is available
                    // and avoid E_PLUGIN during first use under plugin-on.
                    if std::env::var("NYASH_DEBUG_PLUGIN").unwrap_or_default() == "1" {
                        eprintln!("[ProviderBox] eager-load core plugins if present (Array/Map/String)");
                    }
                    for core in ["ArrayBox", "MapBox", "StringBox"].iter() {
                        if let Some((lib, def)) = cfg.find_library_for_box(core) {
                            let _ = host_guard.load_library_direct(lib, &def.path, &def.boxes);
                        }
                    }
                }
            }
            break;
        }
    }
}

/// Create a box using Plugin → Registry → Embedded order (best‑effort)
pub fn new_box(
    box_type: &str,
    args: &[Box<dyn NyashBox>],
) -> Result<Box<dyn NyashBox>, RuntimeError> {
    // 1) PluginHost direct
    if plugin_policy_on() && std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
        if let Some(b) = {
            let host = crate::runtime::get_global_plugin_host();
            host.read().ok().and_then(|h| h.create_box(box_type, args).ok())
        } {
            return Ok(b);
        }
        // Partial config: attempt to load a single library that provides this box
        for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
            if let Ok(doc) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                if let Some((lib, def)) = doc.find_library_for_box(box_type) {
                    {
                        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                            let _ = h.load_library_direct(lib, &def.path, &def.boxes);
                        }
                    }
                    if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                        if let Ok(b) = h.create_box(box_type, args) {
                            return Ok(b);
                        }
                    }
                }
            }
        }
    }

    // 2) v2 BoxFactoryRegistry provider
    let reg = crate::runtime::get_global_registry();
    if let Ok(b) = reg.create_box(box_type, args) {
        return Ok(b);
    }

    // 3) Final fallback: unified registry (legacy factory set)
    let res = {
        let uni = crate::runtime::unified_registry::get_global_unified_registry();
        let mut guard = uni.lock().unwrap();
        guard.create_box(box_type, args)
    };
    match res {
        Ok(b) => Ok(b),
        Err(e) => Err(e),
    }
}
