//! JIT configuration aggregator
//!
//! Centralizes JIT-related flags so callers and tests can use a single
//! source of truth instead of scattering env access across modules.

#[derive(Debug, Clone, Default)]
pub struct JitConfig {
    pub exec: bool,             // NYASH_JIT_EXEC
    pub stats: bool,            // NYASH_JIT_STATS
    pub stats_json: bool,       // NYASH_JIT_STATS_JSON
    pub dump: bool,             // NYASH_JIT_DUMP
    pub threshold: Option<u32>, // NYASH_JIT_THRESHOLD
    pub phi_min: bool,          // NYASH_JIT_PHI_MIN
    pub hostcall: bool,         // NYASH_JIT_HOSTCALL
    pub handle_debug: bool,     // NYASH_JIT_HANDLE_DEBUG
    pub native_f64: bool,       // NYASH_JIT_NATIVE_F64
    pub native_bool: bool,      // NYASH_JIT_NATIVE_BOOL
    pub native_bool_abi: bool,  // NYASH_JIT_ABI_B1 (experimental)
    pub ret_bool_b1: bool,      // NYASH_JIT_RET_B1 (footing; currently returns i64 0/1)
    pub relax_numeric: bool,    // NYASH_JIT_HOSTCALL_RELAX_NUMERIC (i64->f64 coercion)
}

impl JitConfig {
    pub fn from_env() -> Self {
        let getb = |k: &str| std::env::var(k).ok().as_deref() == Some("1");
        let threshold = std::env::var("NYASH_JIT_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        // Respect explicit dump flag, but also treat a non-empty NYASH_JIT_DOT path
        // as an implicit request to enable dump (so Box/CLI/env stay consistent).
        let dump_flag = getb("NYASH_JIT_DUMP")
            || std::env::var("NYASH_JIT_DOT")
                .ok()
                .map(|s| !s.is_empty())
                .unwrap_or(false);
        Self {
            exec: getb("NYASH_JIT_EXEC"),
            stats: getb("NYASH_JIT_STATS"),
            stats_json: getb("NYASH_JIT_STATS_JSON"),
            dump: dump_flag,
            threshold,
            phi_min: getb("NYASH_JIT_PHI_MIN"),
            hostcall: getb("NYASH_JIT_HOSTCALL"),
            handle_debug: getb("NYASH_JIT_HANDLE_DEBUG"),
            native_f64: getb("NYASH_JIT_NATIVE_F64"),
            native_bool: getb("NYASH_JIT_NATIVE_BOOL"),
            native_bool_abi: getb("NYASH_JIT_ABI_B1"),
            ret_bool_b1: getb("NYASH_JIT_RET_B1"),
            relax_numeric: getb("NYASH_JIT_HOSTCALL_RELAX_NUMERIC"),
        }
    }

    /// Apply current struct values into environment variables.
    /// This keeps existing env untouched unless the value is explicitly set here.
    pub fn apply_env(&self) {
        let setb = |k: &str, v: bool| {
            if v {
                std::env::set_var(k, "1");
            }
        };
        setb("NYASH_JIT_EXEC", self.exec);
        setb("NYASH_JIT_STATS", self.stats);
        setb("NYASH_JIT_STATS_JSON", self.stats_json);
        setb("NYASH_JIT_DUMP", self.dump);
        if let Some(t) = self.threshold {
            std::env::set_var("NYASH_JIT_THRESHOLD", t.to_string());
        }
        setb("NYASH_JIT_PHI_MIN", self.phi_min);
        setb("NYASH_JIT_HOSTCALL", self.hostcall);
        setb("NYASH_JIT_HANDLE_DEBUG", self.handle_debug);
        setb("NYASH_JIT_NATIVE_F64", self.native_f64);
        setb("NYASH_JIT_NATIVE_BOOL", self.native_bool);
        setb("NYASH_JIT_ABI_B1", self.native_bool_abi);
        setb("NYASH_JIT_RET_B1", self.ret_bool_b1);
        setb("NYASH_JIT_HOSTCALL_RELAX_NUMERIC", self.relax_numeric);
    }
}

// Global current JIT config (thread-safe), defaults to env when unset
use once_cell::sync::OnceCell;
use std::sync::RwLock;

static GLOBAL_JIT_CONFIG: OnceCell<RwLock<JitConfig>> = OnceCell::new();

/// Get current JIT config (falls back to env-derived default if unset)
pub fn current() -> JitConfig {
    if let Some(lock) = GLOBAL_JIT_CONFIG.get() {
        if let Ok(cfg) = lock.read() {
            return cfg.clone();
        }
    }
    JitConfig::from_env()
}

/// Set current JIT config (overrides env lookups in hot paths)
pub fn set_current(cfg: JitConfig) {
    if let Some(lock) = GLOBAL_JIT_CONFIG.get() {
        if let Ok(mut w) = lock.write() {
            *w = cfg;
            return;
        }
    }
    let _ = GLOBAL_JIT_CONFIG.set(RwLock::new(cfg));
}

// --- Runtime capability probing (minimal, safe defaults) ---

#[derive(Debug, Clone, Copy, Default)]
pub struct JitCapabilities {
    pub supports_b1_sig: bool,
}

/// Probe JIT backend capabilities once. Safe default: b1 signatures are unsupported.
pub fn probe_capabilities() -> JitCapabilities {
    // Current toolchain: allow forcing via env for experiments; otherwise false.
    // When upgrading Cranelift to a version with B1 signature support, set NYASH_JIT_ABI_B1_SUPPORT=1
    let forced = std::env::var("NYASH_JIT_ABI_B1_SUPPORT").ok().as_deref() == Some("1");
    JitCapabilities {
        supports_b1_sig: forced,
    }
}

/// Apply runtime capabilities onto a JitConfig (e.g., disable b1 ABI when unsupported)
pub fn apply_runtime_caps(mut cfg: JitConfig, caps: JitCapabilities) -> JitConfig {
    if cfg.native_bool_abi && !caps.supports_b1_sig {
        cfg.native_bool_abi = false;
    }
    cfg
}
