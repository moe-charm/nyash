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

/// Ensure plugin host and providers are loaded (best‑effort)
pub fn ensure_loaded(_config_path: Option<&str>) {
    if crate::runtime::env_gate_box::plugins_disabled() { return; }
    if !crate::runtime::env_gate_box::plugin_policy_on() { return; }
    let _ = crate::runtime::plugin_boot_box::boot();
    let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(["ArrayBox", "MapBox", "StringBox", "FileBox"].as_ref());
}

/// Create a box using Plugin → Registry → Embedded order (best‑effort)
pub fn new_box(
    box_type: &str,
    args: &[Box<dyn NyashBox>],
) -> Result<Box<dyn NyashBox>, RuntimeError> {
    // 1) PluginHost direct
    if crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled() {
        let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(&[box_type]);
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
        let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(&[box_type]);
        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
            if let Ok(b) = h.create_box(box_type, args) { return Ok(b); }
        }
        }
    }

    // 2) v2 BoxFactoryRegistry provider
    // plugin-on かつ core box の場合は、Registry 経由の builtin フォールバックを抑止（plugin 一貫化）
    let plugin_on = crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled();
    let is_core = crate::runtime::type_registry::is_core_box(box_type);
    if !(plugin_on && is_core) {
        let reg = crate::runtime::get_global_registry();
        if let Ok(b) = reg.create_box(box_type, args) {
            return Ok(b);
        }
    }

    // 3) Final fallback: unified registry (legacy factory set)
    let res = {
        let uni = crate::runtime::unified_registry::get_global_unified_registry();
        let mut guard = uni.lock().unwrap();
        guard.create_box(box_type, args)
    };
    match res {
        Ok(b) => Ok(b),
        Err(e) => {
            // Core builtins last‑chance fallback（最終安全網）
            // plugin‑on（HAKO_PLUGIN_POLICY=auto かつ plugins 有効）では抑止して、
            // 挙動を plugin→registry に限定する（NewBox→birth 一貫化）。
            // In plugin-on mode, if plugin creation failed after re-probes,
            // allow a last-resort builtin fallback to keep VM flows stable
            // during bring-up. Diagnostics can observe this via upstream logs.
            // plugin-off のみ、既存の安全網を維持（forward compatibility）。
            // Strict plugin-on: when HAKO/NYASH_PLUGIN_ON_STRICT=1, forbid builtin fallback and fail-fast
            let strict = crate::runtime::env_gate_box::bool_any(&["NYASH_PLUGIN_ON_STRICT","HAKO_PLUGIN_ON_STRICT"]);
            if strict { return Err(e); }
            if let Some(res) = crate::runtime::type_registry::create_core_box(box_type, args) {
                return res;
            } else {
                Err(e)
            }
        },
    }
}
