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
use std::sync::OnceLock;
use crate::box_factory::RuntimeError;

/// Ensure plugin host and providers are loaded (best‑effort)
pub fn ensure_loaded(_config_path: Option<&str>) {
    if crate::runtime::env_gate_box::plugins_disabled() { return; }
    if !crate::runtime::env_gate_box::plugin_policy_on() { return; }
    let _ = crate::runtime::plugin_boot_box::boot();
    let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(["ArrayBox", "MapBox", "StringBox", "FileBox"].as_ref());

    // One‑line digest (policy/config/loaded) — print once per process
    static PRINTED: OnceLock<bool> = OnceLock::new();
    if PRINTED.get().copied().unwrap_or(false) == false {
        let policy = crate::common::env_helpers::get_string_with_alias("HAKO_PLUGIN_POLICY","NYASH_PLUGIN_POLICY")
            .unwrap_or_else(|| "auto".to_string());
        let cfg_path = {
            let l = crate::runtime::plugin_loader_v2::get_global_loader_v2();
            let tmp = if let Ok(ro) = l.read() { ro.config_path_str().to_string() } else { "-".to_string() };
            tmp
        };
        // Compute loaded core boxes via registry providers
        let reg = crate::runtime::get_global_registry();
        let mut loaded_core: Vec<&str> = Vec::new();
        for bt in ["ArrayBox","MapBox","StringBox"].iter() {
            if let Some(crate::runtime::BoxProvider::Plugin(_)) = reg.get_provider(bt) {
                loaded_core.push(bt);
            }
        }
        // Stage-2 flag (Map.values()/keys() → HostHandle Array)
        let stage2 = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE").ok().as_deref() == Some("1");
        // Runtime anchors presence (dlsym) check
        let (anchors_ok, missing) = anchors_dlsym_check();
        let _ = ANCHORS_OK.set(anchors_ok);
        let miss_str = if missing.is_empty() { String::new() } else { missing.join(",") };
        let _ = ANCHORS_MISSING.set(miss_str.clone());
        eprintln!(
            "[provider] policy={} config={} loaded={{{}}} anchors={} stage2={}",
            policy,
            cfg_path,
            loaded_core.join(","),
            if anchors_ok { "ok" } else { "miss" },
            if stage2 { "on" } else { "off" }
        );
        if !anchors_ok { eprintln!("[provider/check] anchors-dlsym=miss missing={}", miss_str); } else { eprintln!("[provider/check] anchors-dlsym=ok"); }
        let _ = PRINTED.set(true);

    }
}

/// Create a box using Plugin → Registry → Embedded order (best‑effort)
pub fn new_box(
    box_type: &str,
    args: &[Box<dyn NyashBox>],
) -> Result<Box<dyn NyashBox>, RuntimeError> {
    let debug = crate::runtime::env_gate_box::debug_plugin();

    // Determine if config marks this box as plugin-only
    let mut plugin_only = false;

    if crate::runtime::env_gate_box::plugin_policy_on()
        && !crate::runtime::env_gate_box::plugins_disabled()
    {
        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
            if let Some(conf) = h.config_ref() {
                if let Some((lib, _)) = conf.find_library_for_box(box_type) {
                    if h.has_loaded_lib(lib) {
                        plugin_only = true;
                    }
                }
            }
        }
    }

    let plugin_on = crate::runtime::env_gate_box::plugin_policy_on()
        && !crate::runtime::env_gate_box::plugins_disabled();
    let strict = match crate::common::env_helpers::get_string_with_alias("HAKO_PLUGIN_POLICY","NYASH_PLUGIN_POLICY").unwrap_or_else(||"auto".to_string()).to_ascii_lowercase().as_str() { "force" => true, _ => false };

    // Fail‑Fast if strict and host anchors are missing
    if strict {
        if let Some(false) = ANCHORS_OK.get().copied() {
            return Err(RuntimeError::InvalidOperation { message: format!(
                "plugin policy=force: missing host anchors: {}",
                ANCHORS_MISSING.get().cloned().unwrap_or_else(|| "unknown".to_string())
            )});
        }
    }

    if debug {
        eprintln!(
            "[provider] create_box<'{}'> start plugin_on={} strict={} plugin_only={}",
            box_type, plugin_on, strict, plugin_only
        );
    }

    // 1) PluginHost path (preferred)
    let mut plugin_reprobe = false;
    if plugin_on {
        let det = crate::runtime::env_gate_box::deterministic();
        if det {
            if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
                if let Some(caps) = host.get_box_capabilities(box_type) {
                    const CAP_IO: u64 = 1u64 << 0;
                    const CAP_NET: u64 = 1u64 << 1;
                    if (caps & (CAP_IO | CAP_NET)) != 0 {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!(
                                "deterministic mode: IO plugin box denied: {}",
                                box_type
                            ),
                        });
                    }
                } else if matches!(
                    box_type,
                    "FileBox"
                        | "ServerBox"
                        | "ClientBox"
                        | "RequestBox"
                        | "ResponseBox"
                        | "SockServerBox"
                        | "SockClientBox"
                        | "SockConnBox"
                ) {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!(
                            "deterministic mode: NET plugin box denied: {}",
                            box_type
                        ),
                    });
                }
            }
        }

        let mut plugin_err: Option<String> = None;
        let mut try_plugin_create = || -> Option<Box<dyn NyashBox>> {
            match crate::runtime::get_global_plugin_host().read() {
                Ok(host) => match host.create_box(box_type, args) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        if debug {
                            eprintln!("[provider] plugin.create_box({}) -> Err({:?})", box_type, e);
                        }
                        plugin_err = Some(format!("{:?}", e));
                        None
                    }
                },
                Err(_) => None,
            }
        };

        if let Some(b) = try_plugin_create() {
            if debug {
                eprintln!("[provider] plugin success: {}", box_type);
            }
            return Ok(b);
        }

        if !det {
            plugin_reprobe = true;
            for cfg in ["nyash.toml", "hako.toml", "hakorune.toml"].iter() {
                if let Ok(doc) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                    if let Some((lib, def)) = doc.find_library_for_box(box_type) {
                        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
                            let _ = h.load_library_direct(lib, &def.path, &def.boxes);
                        }
                        break;
                    }
                }
            }
            let _ = crate::runtime::plugin_boot_box::reprobe_providers_for(&[box_type]);
            if let Some(b) = try_plugin_create() {
                if debug {
                    eprintln!("[provider] plugin success (reprobe): {}", box_type);
                }
                return Ok(b);
            }
        }

        if debug {
            eprintln!(
                "[provider] plugin miss: {} (reprobe_performed={} last_err={:?})",
                box_type, plugin_reprobe, plugin_err
            );
        } else if plugin_err.is_some() {
            eprintln!("[provider] plugin miss (quiet): {} last_err={:?}", box_type, plugin_err);
        }
    }

    // 2) v2 BoxFactoryRegistry provider
    // Allow v2-registry when it maps to a Plugin provider — this is still a plugin path,
    // not a builtin fallback. This keeps policy=auto/force semantics while enabling
    // creation through the registry indirection.
    let reg = crate::runtime::get_global_registry();
    if let Some(crate::runtime::BoxProvider::Plugin(_lib)) = reg.get_provider(box_type) {
        if debug {
            eprintln!("[provider] try v2-registry(Plugin).create_box({})", box_type);
        }
        if let Ok(b) = reg.create_box(box_type, args) {
            if debug {
                eprintln!("[provider] v2-registry(Plugin) success: {}", box_type);
            }
            return Ok(b);
        }
    } else if !plugin_on {
        // When plugins are OFF, allow v2-registry (builtin/user providers) as before
        if debug {
            eprintln!("[provider] try v2-registry(builtin/user).create_box({})", box_type);
        }
        if let Ok(b) = reg.create_box(box_type, args) {
            if debug { eprintln!("[provider] v2-registry(builtin/user) success: {}", box_type); }
            return Ok(b);
        }
    }

    // plugin-only boxes must not fall back once plugin path failed
    if plugin_only {
        if debug {
            eprintln!("[provider] plugin-only fallback blocked: {}", box_type);
        }
        return Err(RuntimeError::InvalidOperation {
            message: format!("plugin-only box could not be created: {}", box_type),
        });
    }

    // 3) Final fallback: unified registry (legacy factory set)
    if debug {
        eprintln!("[provider] try unified-registry.create_box({})", box_type);
    }
    let res = {
        let uni = crate::runtime::unified_registry::get_global_unified_registry();
        let guard = uni.lock().unwrap();
        guard.create_box(box_type, args)
    };
    match res {
        Ok(b) => Ok(b),
        Err(e) => {
            if strict {
                if debug {
                    eprintln!(
                        "[provider] builtin fallback blocked by strict mode: {}",
                        box_type
                    );
                }
                return Err(e);
            }
            if let Some(res) = crate::runtime::type_registry::create_core_box(box_type, args) {
                if debug {
                    eprintln!("[provider] builtin fallback success: {}", box_type);
                }
                res
            } else {
                if debug {
                    eprintln!("[provider] builtin fallback failed: {}", box_type);
                }
                Err(e)
            }
        }
    }
}


// ---- Anchors dlsym self-check (unix only) ---------------------------------
static ANCHORS_OK: OnceLock<bool> = OnceLock::new();
static ANCHORS_MISSING: OnceLock<String> = OnceLock::new();

fn anchors_dlsym_check() -> (bool, Vec<&'static str>) {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    unsafe {
        let names: [&'static str; 3] = [
            "nyash_array_new_h",
            "nyrt_host_call_slot",
            "nyash_host_from_plugin_handle",
        ];
        // Use libloading to access the current process symbols
        let lib = libloading::os::unix::Library::this();
        let mut missing: Vec<&'static str> = Vec::new();
        for &n in names.iter() {
            let cstr = std::ffi::CString::new(n).unwrap();
            // libloading requires a NUL-terminated name
            if lib.get::<*const ()>(cstr.as_bytes_with_nul()).is_err() {
                missing.push(n);
            }
        }
        if missing.is_empty() { (true, Vec::new()) } else { (false, missing) }
    }
    #[cfg(any(not(unix), target_arch = "wasm32"))]
    { (true, Vec::new()) }
}
