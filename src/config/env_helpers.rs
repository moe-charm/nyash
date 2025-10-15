//! Unified environment variable helpers
//!
//! Provides consistent, type-safe access to environment variables.
//!
//! ## Usage
//! ```rust
//! use crate::config::env_helpers::*;
//!
//! // Boolean flag check (== "1")
//! if env_flag("NYASH_CLI_VERBOSE") { ... }
//!
//! // Boolean with multiple truthy values ("1", "true", "on")
//! if env_bool("NYASH_ENABLE_FEATURE") { ... }
//!
//! // String with default
//! let root = env_or("NYASH_ROOT", ".");
//!
//! // Parse to integer
//! let timeout = env_u64("NYASH_TIMEOUT_MS", 5000);
//! ```

/// Check if env var is set to "1"
///
/// Returns `true` if the environment variable is set to exactly "1".
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_flag;
/// if env_flag("NYASH_CLI_VERBOSE") {
///     println!("Verbose mode enabled");
/// }
/// ```
#[inline]
pub fn env_flag(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}

/// Check if env var is set to any truthy value
///
/// Returns `true` if the environment variable is set to "1", "true", or "on".
/// Case-sensitive.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_bool;
/// if env_bool("NYASH_ENABLE_FEATURE") {
///     println!("Feature enabled");
/// }
/// ```
#[inline]
pub fn env_bool(name: &str) -> bool {
    match std::env::var(name).ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

/// Check if env var is set to any truthy value (case-insensitive)
///
/// Returns `true` if the environment variable is set to "1", "true", or "on"
/// (case-insensitive).
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_bool_ci;
/// if env_bool_ci("NYASH_ENABLE_FEATURE") {
///     // Accepts "TRUE", "True", "ON", etc.
/// }
/// ```
#[inline]
pub fn env_bool_ci(name: &str) -> bool {
    match std::env::var(name)
        .ok()
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

/// Check if env var exists (any value)
///
/// Returns `true` if the environment variable is set, regardless of value.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_exists;
/// if env_exists("NYASH_ROOT") {
///     println!("NYASH_ROOT is set");
/// }
/// ```
#[inline]
pub fn env_exists(name: &str) -> bool {
    std::env::var(name).is_ok()
}

/// Get env var with default value
///
/// Returns the environment variable value, or the default if not set.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_or;
/// let root = env_or("NYASH_ROOT", ".");
/// ```
#[inline]
pub fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Get env var or empty string
///
/// Returns the environment variable value, or an empty string if not set.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_string;
/// let config_path = env_string("NYASH_CONFIG_PATH");
/// ```
#[inline]
pub fn env_string(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

/// Parse env var to u64 with default
///
/// Parses the environment variable as a u64, returning the default if not set
/// or if parsing fails.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_u64;
/// let timeout_ms = env_u64("NYASH_TIMEOUT_MS", 5000);
/// ```
#[inline]
pub fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Parse env var to usize with default
///
/// Parses the environment variable as a usize, returning the default if not set
/// or if parsing fails.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_usize;
/// let budget = env_usize("NYASH_SCHED_POLL_BUDGET", 100);
/// ```
#[inline]
pub fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Parse env var to u32 with default
///
/// Parses the environment variable as a u32, returning the default if not set
/// or if parsing fails.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_u32;
/// let port = env_u32("NYASH_PORT", 8080);
/// ```
/// Parse the first available env var among given names to u64 with default
#[inline]
pub fn env_u64_any(names: &[&str], default: u64) -> u64 {
    for &n in names {
        if let Ok(v) = std::env::var(n) {
            if let Ok(parsed) = v.parse() { return parsed; }
        }
    }
    default
}
#[inline]
pub fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Get optional string
///
/// Returns `Some(value)` if the environment variable is set, otherwise `None`.
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_opt;
/// if let Some(root) = env_opt("NYASH_ROOT") {
///     println!("Root: {}", root);
/// }
/// ```
#[inline]
pub fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Check if env var is NOT set to a falsy value (default: true)
///
/// Returns `false` if the environment variable is set to "0", "false", or "off".
/// Returns `true` otherwise (including when not set).
///
/// # Examples
/// ```
/// # use crate::config::env_helpers::env_bool_default_true;
/// // NYASH_FEATURE not set → true
/// // NYASH_FEATURE=1 → true
/// // NYASH_FEATURE=0 → false
/// if env_bool_default_true("NYASH_FEATURE") {
///     println!("Feature enabled by default");
/// }
/// ```
#[inline]
pub fn env_bool_default_true(name: &str) -> bool {
    !matches!(
        std::env::var(name).ok().as_deref(),
        Some("0") | Some("false") | Some("off")
    )
}
