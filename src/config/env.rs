//! Global environment configuration aggregator (管理棟)
//!
//! Consolidates NYASH_* environment variables across subsystems and
//! optionally applies overrides from `nyash.toml`.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct NyashEnv {
    /// JIT-related configuration (delegates to jit::config)
    pub jit: crate::jit::config::JitConfig,
    /// Arbitrary key-value overrides loaded from nyash.toml [env]
    pub overrides: BTreeMap<String, String>,
}

impl NyashEnv {
    pub fn from_env() -> Self {
        Self { jit: crate::jit::config::JitConfig::from_env(), overrides: BTreeMap::new() }
    }
    /// Apply current struct values into process environment
    pub fn apply_env(&self) {
        self.jit.apply_env();
        for (k, v) in &self.overrides { std::env::set_var(k, v); }
    }
}

// Global current env config (thread-safe)
use once_cell::sync::OnceCell;
use std::sync::RwLock;

static GLOBAL_ENV: OnceCell<RwLock<NyashEnv>> = OnceCell::new();

pub fn current() -> NyashEnv {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(cfg) = lock.read() { return cfg.clone(); }
    }
    NyashEnv::from_env()
}

pub fn set_current(cfg: NyashEnv) {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(mut w) = lock.write() { *w = cfg; return; }
    }
    let _ = GLOBAL_ENV.set(RwLock::new(cfg));
}

/// Load overrides from nyash.toml `[env]` table and apply them to process env.
///
/// Example:
/// [env]
/// NYASH_JIT_THRESHOLD = "1"
/// NYASH_CLI_VERBOSE = "1"
pub fn bootstrap_from_toml_env() {
    let path = "nyash.toml";
    let content = match std::fs::read_to_string(path) { Ok(s) => s, Err(_) => return };
    let Ok(value) = toml::from_str::<toml::Value>(&content) else { return };
    let Some(env_tbl) = value.get("env").and_then(|v| v.as_table()) else { return };
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

/// Get await maximum milliseconds, centralized here for consistency.
pub fn await_max_ms() -> u64 {
    std::env::var("NYASH_AWAIT_MAX_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(5000)
}

// ---- Phase 11.8 MIR cleanup toggles ----
pub fn mir_core13() -> bool { std::env::var("NYASH_MIR_CORE13").ok().as_deref() == Some("1") }
pub fn mir_ref_boxcall() -> bool { std::env::var("NYASH_MIR_REF_BOXCALL").ok().as_deref() == Some("1") || mir_core13() }
pub fn mir_array_boxcall() -> bool { std::env::var("NYASH_MIR_ARRAY_BOXCALL").ok().as_deref() == Some("1") || mir_core13() }
pub fn mir_plugin_invoke() -> bool { std::env::var("NYASH_MIR_PLUGIN_INVOKE").ok().as_deref() == Some("1") }
pub fn plugin_only() -> bool { std::env::var("NYASH_PLUGIN_ONLY").ok().as_deref() == Some("1") }

// ---- Optimizer diagnostics ----
pub fn opt_debug() -> bool { std::env::var("NYASH_OPT_DEBUG").is_ok() }
pub fn opt_diag() -> bool { std::env::var("NYASH_OPT_DIAG").is_ok() }
pub fn opt_diag_forbid_legacy() -> bool { std::env::var("NYASH_OPT_DIAG_FORBID_LEGACY").is_ok() }
pub fn opt_diag_fail() -> bool { std::env::var("NYASH_OPT_DIAG_FAIL").is_ok() }

// ---- GC/Runtime tracing (execution-affecting visibility) ----
pub fn gc_trace() -> bool { std::env::var("NYASH_GC_TRACE").ok().as_deref() == Some("1") }
pub fn gc_barrier_trace() -> bool { std::env::var("NYASH_GC_BARRIER_TRACE").ok().as_deref() == Some("1") }
pub fn runtime_checkpoint_trace() -> bool { std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE").ok().as_deref() == Some("1") }
pub fn vm_pic_stats() -> bool { std::env::var("NYASH_VM_PIC_STATS").ok().as_deref() == Some("1") }
pub fn vm_vt_trace() -> bool { std::env::var("NYASH_VM_VT_TRACE").ok().as_deref() == Some("1") }
pub fn vm_pic_trace() -> bool { std::env::var("NYASH_VM_PIC_TRACE").ok().as_deref() == Some("1") }
pub fn gc_barrier_strict() -> bool { std::env::var("NYASH_GC_BARRIER_STRICT").ok().as_deref() == Some("1") }

/// Return 0 (off) to 3 (max) for `NYASH_GC_TRACE`.
pub fn gc_trace_level() -> u8 {
    match std::env::var("NYASH_GC_TRACE").ok().as_deref() {
        Some("1") => 1,
        Some("2") => 2,
        Some("3") => 3,
        Some(_) => 1,
        None => 0,
    }
}

// ---- Rewriter flags (optimizer transforms)
pub fn rewrite_debug() -> bool { std::env::var("NYASH_REWRITE_DEBUG").ok().as_deref() == Some("1") }
pub fn rewrite_safepoint() -> bool { std::env::var("NYASH_REWRITE_SAFEPOINT").ok().as_deref() == Some("1") }
pub fn rewrite_future() -> bool { std::env::var("NYASH_REWRITE_FUTURE").ok().as_deref() == Some("1") }

// ---- Phase 12: Nyash ABI (vtable) toggles ----
pub fn abi_vtable() -> bool { std::env::var("NYASH_ABI_VTABLE").ok().as_deref() == Some("1") }
pub fn abi_strict() -> bool { std::env::var("NYASH_ABI_STRICT").ok().as_deref() == Some("1") }

// ---- ExternCall strict diagnostics ----
pub fn extern_strict() -> bool { std::env::var("NYASH_EXTERN_STRICT").ok().as_deref() == Some("1") }
