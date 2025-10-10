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

    // Determine if config declares this box (plugin-only when policy is on)
    let mut plugin_only = false;
    if crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled() {
        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
            if let Some(conf) = h.config_ref() {
                if conf.find_library_for_box(box_type).is_some() {
                    plugin_only = true;
                    if crate::runtime::env_gate_box::debug_plugin() {
                        eprintln!("[provider] plugin-only declared: {}", box_type);
                    }
                }
            }
        }
    }

    // 1) PluginHost direct
    if crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled() {
        // Deterministic guard: deny IO/NET‑cap boxes
        let det = crate::runtime::env_gate_box::deterministic();
        if det {
            if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
                if let Some(caps) = host.get_box_capabilities(box_type) {
                    const CAP_IO: u64 = 1u64 << 0;
                    const CAP_NET: u64 = 1u64 << 1;
                    if (caps & (CAP_IO | CAP_NET)) != 0 {
                        return Err(RuntimeError::InvalidOperation { message: format!(
                            "deterministic mode: IO plugin box denied: {}", box_type
                        )});
                    }
                } else if matches!(
                    box_type,
                    "FileBox" | "ServerBox" | "ClientBox" | "RequestBox" | "ResponseBox" | "SockServerBox" | "SockClientBox" | "SockConnBox"
                ) {
                    return Err(RuntimeError::InvalidOperation { message: format!(
                        "deterministic mode: NET plugin box denied: {}", box_type
                    )});
                }
            }
        }
        // Proactively try targeted load for this box before first create (best‑effort)
        if !det {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[provider] plugin.create_box initial miss: {} (reprobe)", box_type);
            }
            for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
                if let Ok(doc) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                    if let Some((lib, def)) = doc.find_library_for_box(box_type) {
                        if let Ok(h) = crate::runtime::get_global_plugin_host().read() { let _ = h.load_library_direct(lib, &def.path, &def.boxes); }
                        break;
                    }
                }
            }
            let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(&[box_type]);
            if crate::runtime::env_gate_box::debug_plugin() { eprintln!("[provider] reprobe done; retry plugin.create_box({})", box_type); }
        }
        if crate::runtime::env_gate_box::debug_plugin() {
            eprintln!("[provider] try plugin.create_box({})", box_type);
        }
        if let Some(b) = {
            let host = crate::runtime::get_global_plugin_host();
            host.read().ok().and_then(|h| h.create_box(box_type, args).ok())
        } {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[provider] plugin.create_box success: {}", box_type);
            }
            return Ok(b);
        }
        // Partial config: attempt to load a single library that provides this box
        if !det {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[provider] plugin.create_box initial miss: {} (reprobe)", box_type);
            }
            for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
                if let Ok(doc) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                    if let Some((lib, def)) = doc.find_library_for_box(box_type) {
                        if crate::runtime::env_gate_box::debug_plugin() {
                            eprintln!("[provider] load_library_direct {} for {}", lib, box_type);
                        }
                        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                            let _ = h.load_library_direct(lib, &def.path, &def.boxes);
                        }
                        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                            if let Ok(b) = h.create_box(box_type, args) { return Ok(b); }
                        }
                    }
                }
            }
            let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(&[box_type]);
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[provider] reprobe done; retry plugin.create_box({})", box_type);
            }
            if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                if let Ok(b) = h.create_box(box_type, args) { return Ok(b); }
            }
        }
    }

    // 2) v2 BoxFactoryRegistry provider
    // plugin-on かつ core box の場合は、Registry 経由の builtin フォールバックを抑止（plugin 一貫化）
    let plugin_on = crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled();
    let is_core = crate::runtime::type_registry::is_core_box(box_type);
    if !(plugin_on && (is_core || plugin_only)) {
        let reg = crate::runtime::get_global_registry();
        if crate::runtime::env_gate_box::debug_plugin() {
            eprintln!("[provider] try v2-registry.create_box({})", box_type);
        }
        if let Ok(b) = reg.create_box(box_type, args) {
            if crate::runtime::env_gate_box::debug_plugin() {
                eprintln!("[provider] v2-registry success: {}", box_type);
            }
            return Ok(b);
        }
    }

    // If config declares this box and plugin path failed, do not fallback (plugin-only)
    if plugin_only {
        return Err(RuntimeError::InvalidOperation { message: format!("plugin-only box could not be created: {}", box_type) });
    }

    // 3) Final fallback: unified registry (legacy factory set)
    if crate::runtime::env_gate_box::debug_plugin() {
        eprintln!("[provider] try unified-registry.create_box({})", box_type);
    }
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
                if crate::runtime::env_gate_box::debug_plugin() {
                    eprintln!("[provider] builtin fallback success: {}", box_type);
                }
                return res;
            } else {
                if crate::runtime::env_gate_box::debug_plugin() {
                    eprintln!("[provider] builtin fallback failed: {}", box_type);
                }
                Err(e)
            }
        },
    }
}
