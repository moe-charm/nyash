//! Global environment configuration aggregator (管理棟)
//!
//! Consolidates NYASH_* environment variables across subsystems and
//! optionally applies overrides from `nyash.toml`.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct NyashEnv {
    // ARCHIVED: JIT-related configuration moved to archive/jit-cranelift/ during Phase 15
    // pub jit: crate::jit::config::JitConfig,
    /// Arbitrary key-value overrides loaded from nyash.toml [env]
    pub overrides: BTreeMap<String, String>,
}

impl NyashEnv {
    pub fn from_env() -> Self {
        Self {
            // ARCHIVED: JIT config during Phase 15
            // jit: crate::jit::config::JitConfig::from_env(),
            overrides: BTreeMap::new(),
        }
    }
    /// Apply current struct values into process environment
    pub fn apply_env(&self) {
        // ARCHIVED: JIT config during Phase 15
        // self.jit.apply_env();
        for (k, v) in &self.overrides {
            std::env::set_var(k, v);
        }
    }
}

// Global current env config (thread-safe)
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use std::collections::HashSet;
use once_cell::sync::OnceCell as StdOnceLock;

static GLOBAL_ENV: OnceCell<RwLock<NyashEnv>> = OnceCell::new();
// フェーズM.2: PHI_ON_GATED_WARNED削除（phi-legacy簡略化により不要）

pub fn current() -> NyashEnv {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(cfg) = lock.read() {
            return cfg.clone();
        }
    }
    NyashEnv::from_env()
}

pub fn set_current(cfg: NyashEnv) {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(mut w) = lock.write() {
            *w = cfg;
            return;
        }
    }
    let _ = GLOBAL_ENV.set(RwLock::new(cfg));
}

// ---- Deprecation helper (print once per key when verbose) ----
#[allow(dead_code)]
static DEPREC_EMITTED: StdOnceLock<RwLock<HashSet<String>>> = StdOnceLock::new();
#[allow(dead_code)]
fn deprec_once(key: &str, message: &str) {
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() != Some("1") { return; }
    let lock = DEPREC_EMITTED.get_or_init(|| RwLock::new(HashSet::new()));
    if let Ok(g) = lock.read() { if g.contains(key) { return; } }
    if let Ok(mut w) = lock.write() { if w.insert(key.to_string()) { eprintln!("{}", message); } }
}


/// Load overrides from nyash.toml `[env]` table and apply them to process env.
///
/// Example:
/// [env]
/// NYASH_JIT_THRESHOLD = "1"
/// NYASH_CLI_VERBOSE = "1"
pub fn bootstrap_from_toml_env() {
    // Allow disabling nyash.toml env bootstrapping for isolated smokes/CI
    if std::env::var("NYASH_SKIP_TOML_ENV").ok().as_deref() == Some("1") {
        return;
    }
    // Accept brand alias environment variables before reading files
    alias_prefixes_bootstrap();
    // Resolve hako.toml/nyash.toml/hakorune.toml from CWD, fallback to *_ROOT if necessary.
    #[allow(unused_assignments)]
    let mut content = String::new();
    // Prefer hako.toml first
    let path_hako = std::path::Path::new("hako.toml");
    if path_hako.exists() {
        match std::fs::read_to_string(path_hako) {
            Ok(s) => { content = s; }
            Err(_) => return,
        }
    } else {
        // Try nyash.toml in CWD next
        let path_ny = std::path::Path::new("nyash.toml");
        if path_ny.exists() {
            match std::fs::read_to_string(path_ny) {
                Ok(s) => { content = s; }
                Err(_) => return,
            }
        } else {
            // Try hakorune.toml in CWD next
        let path_alt = std::path::Path::new("hakorune.toml");
        if path_alt.exists() {
            match std::fs::read_to_string(path_alt) {
                Ok(s) => { content = s; }
                Err(_) => return,
            }
        } else if let Some(root) = env_root_any() {
            // Try $ROOT/nyash.toml then $ROOT/hakorune.toml
            // Also prefer hako.toml under root
            let ph = std::path::Path::new(&root).join("hako.toml");
            if let Ok(s) = std::fs::read_to_string(&ph) { content = s; }
            else {
                let p = std::path::Path::new(&root).join("nyash.toml");
                if let Ok(s) = std::fs::read_to_string(&p) { content = s; }
                else {
                    let p2 = std::path::Path::new(&root).join("hakorune.toml");
                    match std::fs::read_to_string(p2) {
                    Ok(s) => { content = s; }
                    Err(_) => return,
                    }
                }
            }
        } else {
            return;
        }
        }
    }
    let Ok(value) = toml::from_str::<toml::Value>(&content) else {
        return;
    };
    let Some(env_tbl) = value.get("env").and_then(|v| v.as_table()) else {
        return;
    };
    let mut overrides: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in env_tbl {
        if let Some(s) = v.as_str() {
            std::env::set_var(k, s);
            overrides.insert(k.clone(), s.to_string());
        } else if let Some(b) = v.as_bool() {
            let sv = if b { "1" } else { "0" };
            std::env::set_var(k, sv);
            overrides.insert(k.clone(), sv.to_string());
        } else if let Some(n) = v.as_integer() {
            let sv = n.to_string();
            std::env::set_var(k, &sv);
            overrides.insert(k.clone(), sv);
        }
    }
    // Merge into global
    let mut cur = current();
    cur.overrides.extend(overrides);
    set_current(cur);
}

/// Copy HAKU_*/HRN_* to NYASH_* if unset (non-destructive), incl. *_ROOT.
pub fn alias_prefixes_bootstrap() {
    // Root alias first
    if std::env::var("NYASH_ROOT").ok().is_none() {
        if let Ok(v) = std::env::var("HAKO_ROOT") { std::env::set_var("NYASH_ROOT", v); }
        else if let Ok(v) = std::env::var("HAKU_ROOT") { std::env::set_var("NYASH_ROOT", v); }
        else if let Ok(v) = std::env::var("HRN_ROOT") { std::env::set_var("NYASH_ROOT", v); }
    }
    // General mapping
    let snapshot: Vec<(String, String)> = std::env::vars().collect();
    for (k, v) in snapshot.iter() {
        if let Some(tail) = k.strip_prefix("HAKO_")
            .or_else(|| k.strip_prefix("HAKU_"))
            .or_else(|| k.strip_prefix("HRN_")) {
            let ny = format!("NYASH_{}", tail);
            if std::env::var(&ny).ok().is_none() {
                std::env::set_var(ny, v);
            }
        }
    }
}

/// Return NYASH_ROOT or its aliases (HAKU_ROOT/HRN_ROOT)
fn env_root_any() -> Option<String> {
    std::env::var("NYASH_ROOT").ok()
        .or_else(|| std::env::var("HAKO_ROOT").ok())
        .or_else(|| std::env::var("HAKU_ROOT").ok())
        .or_else(|| std::env::var("HRN_ROOT").ok())
}

