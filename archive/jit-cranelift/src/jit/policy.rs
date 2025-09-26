//! JIT Policy (Box-First): centralizes runtime decisions
//!
//! Minimal v0:
//! - read_only: if true, deny write-effects in jit-direct and other independent paths
//! - hostcall_whitelist: symbolic names allowed (future use)

use once_cell::sync::OnceCell;
use std::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct JitPolicy {
    pub read_only: bool,
    pub hostcall_whitelist: Vec<String>,
}

impl JitPolicy {
    pub fn from_env() -> Self {
        let ro = std::env::var("NYASH_JIT_READ_ONLY").ok().as_deref() == Some("1");
        // Comma-separated hostcall names
        let hc = std::env::var("NYASH_JIT_HOSTCALL_WHITELIST")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Self {
            read_only: ro,
            hostcall_whitelist: hc,
        }
    }
}

static GLOBAL: OnceCell<RwLock<JitPolicy>> = OnceCell::new();

pub fn current() -> JitPolicy {
    if let Some(l) = GLOBAL.get() {
        if let Ok(g) = l.read() {
            return g.clone();
        }
    }
    JitPolicy::from_env()
}

pub fn set_current(p: JitPolicy) {
    if let Some(l) = GLOBAL.get() {
        if let Ok(mut w) = l.write() {
            *w = p;
            return;
        }
    }
    let _ = GLOBAL.set(RwLock::new(p));
}

// Submodule: invoke decision policy
pub mod invoke;
