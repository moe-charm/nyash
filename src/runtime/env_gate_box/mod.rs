//! EnvGateBox — Normalize environment reads (aliases/truthy)
//!
//! Responsibility
//! - Centralize environment flag/string reads for runtime.
//! - Apply alias fallback (HAKO_* → NYASH_* and vice versa when needed).
//! - Normalize truthy values: 1|true|on|yes (case-insensitive).
//!
//! Notes
//! - Thin facade over `crate::common::env_helpers` to keep callers concise.

/// Return true if any of the given keys is set to a truthy value
/// (1|true|on|yes, case-insensitive).
#[inline]
pub fn bool_any(keys: &[&str]) -> bool {
    crate::common::env_helpers::get_bool_any(keys).unwrap_or(false)
}

/// Return true if the primary or alias key is set to a truthy value.
#[inline]
pub fn bool_alias(primary: &str, alias: &str) -> bool {
    crate::common::env_helpers::get_bool_with_alias(primary, alias).unwrap_or(false)
}

/// Read a string from primary or alias key. Returns `default` if neither is set.
#[inline]
pub fn string_alias_or(primary: &str, alias: &str, default: &str) -> String {
    crate::common::env_helpers::get_string_with_alias(primary, alias)
        .unwrap_or_else(|| default.to_string())
}

/// Determine if plugin policy is effectively ON (auto|force).
#[inline]
pub fn plugin_policy_on() -> bool {
    if let Some(v) = crate::common::env_helpers::get_string_with_alias(
        "NYASH_PLUGIN_POLICY",
        "HAKO_PLUGIN_POLICY",
    ) {
        let s = v.to_ascii_lowercase();
        s == "auto" || s == "force"
    } else {
        // Default ON (auto) when unset
        true
    }
}

/// Return true if plugins are explicitly disabled.
#[inline]
pub fn plugins_disabled() -> bool {
    bool_any(&["NYASH_DISABLE_PLUGINS"]) ||
    // historical alias
    bool_any(&["HAKO_DISABLE_PLUGINS"])
}

/// Debug gate for verbose plugin logs.
#[inline]
pub fn debug_plugin() -> bool { bool_any(&["NYASH_DEBUG_PLUGIN", "HAKO_DEBUG_PLUGIN"]) }

/// Diagnostics structured trace gate.
#[inline]
pub fn diag_trace() -> bool { bool_any(&["NYASH_DIAG_TRACE", "HAKO_DIAG_TRACE"]) }

/// Deterministic execution gate (deny on-demand reprobe; deny IO plugins)
#[inline]
pub fn deterministic() -> bool { bool_any(&["HAKO_DETERMINISTIC", "NYASH_DETERMINISTIC"]) }


/// HostHandle routing/slot trace (short‑lived, default OFF). Primary=HAKO_*, alias=NYASH_*.
#[inline]
pub fn host_handle_trace() -> bool { bool_any(&["HAKO_HOST_HANDLE_TRACE", "NYASH_HOST_HANDLE_TRACE"]) }

/// Normalize a boolean-like string to Some(true/false) or None if unrecognized.
/// Accepts: 1|true|on|yes => true, 0|false|off|no => false (case-insensitive).
#[inline]
pub fn parse_bool_str(v: &str) -> Option<bool> {
    let s = v.trim().to_ascii_lowercase();
    match s.as_str() {
        "1" | "true" | "on" | "yes" => Some(true),
        "0" | "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

/// Read boolean with alias and default using parse_bool_str normalization.
#[inline]
pub fn bool_alias_or(primary: &str, alias: &str, default: bool) -> bool {
    if let Some(v) = crate::common::env_helpers::get_first(&[primary, alias]) {
        parse_bool_str(&v).unwrap_or(default)
    } else {
        default
    }
}
