/*!
 * Runner plugin/registry initialization (extracted)
 */

use super::*;

impl NyashRunner {
    /// Initialize global runtime registry and load configured plugins.
    ///
    /// Behavior (no-op changes to defaults):
    /// - Always initializes the unified type/box registry.
    /// - Loads native BID plugins unless `NYASH_DISABLE_PLUGINS=1`.
    /// - Ensures built‑ins override list includes ArrayBox/MapBox/FileBox/TOMLBox
    ///   by merging `NYASH_PLUGIN_OVERRIDE_TYPES` with required entries.
    /// - When `--load-ny-plugins` or `NYASH_LOAD_NY_PLUGINS=1` is set, best‑effort
    ///   loads Nyash scripts listed under `nyash.toml`'s `ny_plugins`.
    ///
    /// Side effects:
    /// - Sets `NYASH_USE_PLUGIN_BUILTINS=1` if unset, to allow interpreter side creation.
    /// - Updates `NYASH_PLUGIN_OVERRIDE_TYPES` env var (merged values).
    pub(crate) fn init_runtime_and_plugins(&self, groups: &crate::cli::CliGroups) {
        // Unified registry
        runtime::init_global_unified_registry();
        // Plugins (guarded)
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            runner_plugin_init::init_bid_plugins();
            crate::runner::box_index::refresh_box_index();
        }
        // Allow interpreter to create plugin-backed boxes via unified registry
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().is_none() {
            std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
        }
        // Merge override types env (ensure FileBox/TOMLBox present)
        let mut override_types: Vec<String> = if let Ok(list) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
            list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        } else { vec!["ArrayBox".into(), "MapBox".into()] };
        for t in ["FileBox", "TOMLBox"] { if !override_types.iter().any(|x| x == t) { override_types.push(t.into()); } }
        std::env::set_var("NYASH_PLUGIN_OVERRIDE_TYPES", override_types.join(","));

        // Rebuild unified box registry cache to reflect env-driven overrides
        // (ArrayBox/MapBox/FileBox/TOMLBox move to plugin providers when available)
        {
            let reg_arc = runtime::unified_registry::get_global_unified_registry();
            let mut guard = reg_arc.lock().expect("unified registry lock");
            guard.rebuild_cache();
        }

        // Optional Ny script plugins loader (best-effort)
        if groups.load_ny_plugins || std::env::var("NYASH_LOAD_NY_PLUGINS").ok().as_deref() == Some("1") {
            if let Ok(text) = std::fs::read_to_string("nyash.toml") {
                if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                    if let Some(np) = doc.get("ny_plugins") {
                        let mut list: Vec<String> = Vec::new();
                        if let Some(arr) = np.as_array() { for v in arr { if let Some(s) = v.as_str() { list.push(s.to_string()); } } }
                        else if let Some(tbl) = np.as_table() { for (_k, v) in tbl { if let Some(s) = v.as_str() { list.push(s.to_string()); } else if let Some(arr) = v.as_array() { for e in arr { if let Some(s) = e.as_str() { list.push(s.to_string()); } } } } }
                        if !list.is_empty() {
                            let list_only = std::env::var("NYASH_NY_PLUGINS_LIST_ONLY").ok().as_deref() == Some("1");
                            println!("🧩 Ny script plugins ({}):", list.len());
                            for p in list {
                                if list_only { println!("  • {}", p); continue; }
                                match std::fs::read_to_string(&p) {
                                    Ok(code) => {
                                        // Legacy interpreter removed - ny_plugins execution disabled
                                        println!("[ny_plugins] {}: SKIP (legacy interpreter removed)", p);
                                    }
                                    Err(e) => println!("[ny_plugins] {}: FAIL (read: {})", p, e),
                                }
                            }
                        }
                    }
                }
            }
        }

        // Provider verify (受け口): env で warn/strict のみ動作（未設定時は無処理）
        match crate::runtime::provider_verify::verify_from_env() {
            Ok(()) => {}
            Err(e) => { eprintln!("❌ {}", e); std::process::exit(1); }
        }

        // Provider Lock — lock after registry and plugins are initialized (受け口)
        // Default: no-op behavior change. Exposed for future verify→lock sequencing.
        crate::runtime::provider_lock::lock_providers();
    }
}
