//! Common observe helpers (dev-only; default OFF)
//!
//! Small utilities to standardize env-gated tracing and line formatting.

use std::sync::OnceLock;

#[inline]
pub fn trace_enabled(env: &str) -> bool {
    // Cache per env name (coarse: we only use a few well-known names)
    // Accept: 1/true/on/yes (case-insensitive)
    static TRUE_SET: OnceLock<Vec<&'static str>> = OnceLock::new();
    let trues = TRUE_SET.get_or_init(|| vec!["1", "true", "on", "yes"]);
    if let Ok(val) = std::env::var(env) {
        let v = val.to_ascii_lowercase();
        return trues.iter().any(|t| *t == v);
    }
    false
}

#[inline]
pub fn eprintln_tag(tag: &str, msg: &str) {
    eprintln!("[{}] {}", tag, msg);
}

