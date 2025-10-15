//! env_helpers — phase-by-phase migration facade for environment access
//!
//! Goals
//! - Normalize truthy parsing: 1|true|on|yes (case-insensitive)
//! - Provide alias resolution (HAKO_* → NYASH_* fallback) for reads
//! - Small, dependency-free helpers to be used in new code
//!
//! Policy
//! - New code should prefer these helpers
//! - Existing code can be migrated gradually (Phase 20+ for bulk moves)

fn get_raw(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Get the first available value among provided keys (in order)
pub fn get_first(keys: &[&str]) -> Option<String> {
    for k in keys { if let Some(v) = get_raw(k) { return Some(v); } }
    None
}

/// Truthy parser: 1|true|on|yes (case-insensitive)
pub fn is_truthy_val(v: &str) -> bool {
    matches!(v.to_ascii_lowercase().as_str(), "1"|"true"|"on"|"yes")
}

/// Read a boolean with alias fallback (e.g., HAKO_* then NYASH_*)
pub fn get_bool_with_alias(primary: &str, alias: &str) -> Option<bool> {
    if let Some(v) = get_first(&[primary, alias]) { Some(is_truthy_val(&v)) } else { None }
}

/// Read a boolean using any of the provided keys
pub fn get_bool_any(keys: &[&str]) -> Option<bool> {
    if let Some(v) = get_first(keys) { Some(is_truthy_val(&v)) } else { None }
}

/// Read string with alias fallback
pub fn get_string_with_alias(primary: &str, alias: &str) -> Option<String> {
    get_first(&[primary, alias])
}

/// Read u64 with default
pub fn get_u64_with_default(keys: &[&str], default: u64) -> u64 {
    if let Some(v) = get_first(keys) { v.parse::<u64>().unwrap_or(default) } else { default }
}

