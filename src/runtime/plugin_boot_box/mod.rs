//! PluginBootBox — single entry to initialize plugin host and register providers
//!
//! Responsibility
//! - Idempotent boot of the v2 plugin host (load config once).
//! - Register providers (box → library) into the v2 BoxFactoryRegistry.
//! - Honor policy/env gates (HAKO_/NYASH_ aliases) via EnvGateBox.
//!
//! Notes
//! - Keep this box dependency‑light. It is called from runner/unified_registry/provider.

use std::sync::OnceLock;

static BOOTED: OnceLock<bool> = OnceLock::new();

fn policy_on() -> bool {
    crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled()
}

/// Boot plugin host and register providers exactly once. Returns true if booted (or already booted).
pub fn boot() -> bool {
    if let Some(v) = BOOTED.get() { return *v; }

    let ok = (|| {
        if !policy_on() { return true; } // treat as success when policy off

        // Choose config candidates (prefer explicit override)
        let mut tried: Vec<String> = Vec::new();
        if let Some(cfg) = crate::common::env_helpers::get_first(&["NYASH_PLUGIN_CONFIG", "HAKO_PLUGIN_CONFIG"]) {
            tried.push(cfg.clone());
            if crate::runtime::init_global_plugin_host(&cfg).is_ok() {
                return register_providers_from_current();
            }
        }
        for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
            tried.push((*cfg).into());
            if crate::runtime::init_global_plugin_host(cfg).is_ok() {
                return register_providers_from_current();
            }
        }
        // Optionally log (quiet by default)
        if crate::runtime::env_gate_box::debug_plugin() {
            eprintln!("[plugin-boot] failed to init from candidates: {:?}", tried);
        }
        true // do not hard fail; caller may choose to restrict builtin fallback
    })();

    let _ = BOOTED.set(ok);
    ok
}

fn register_providers_from_current() -> bool {
    if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
        if let Some(conf) = h.config_ref() {
            let reg = crate::runtime::get_global_registry();
            for (lib, def) in &conf.libraries {
                for b in &def.boxes {
                    reg.apply_plugin_config(&crate::runtime::PluginConfig { plugins: [(b.clone(), lib.clone())].into(), });
                }
            }
            return true;
        }
    }
    false
}


/// Best-effort re-probe for missing providers. Useful when boot happened before
/// config was fully located, or when a subset of libraries was loaded.
/// Returns true if any provider/library was (re)applied.
pub fn reprobe_providers_for(boxes: &[&str]) -> bool {
    if !policy_on() { return false; }
    let mut changed = false;
    // 1) If host has config, attempt to load libraries for requested boxes
    if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
        if let Some(conf) = h.config_ref() {
            for b in boxes {
                if let Some((lib, def)) = conf.find_library_for_box(b) {
                    let _ = h.load_library_direct(lib, &def.path, &def.boxes);
                    changed = true;
                    if crate::runtime::env_gate_box::diag_trace() {
                        crate::runtime::diagnostics::trace_event(
                            "plugin_reprobe_provider",
                            &format!("\"box\":\"{}\",\"lib\":\"{}\"", b, lib),
                        );
                    }
                }
            }
            if changed { return true; }
        }
    }
    // 2) Try known config candidates once more, then register providers
    for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
        if crate::runtime::init_global_plugin_host(cfg).is_ok() {
            let ok = super::plugin_boot_box::register_providers_from_current();
            changed |= ok;
            if ok { break; }
        }
    }
    changed
}
