/*!
 * External Call Handling
 *
 * Manages env.* methods and external interface calls
 * Provides bridge to host environment functionality
 */

use crate::mir::{Effect, EffectMask};

/// Table-like spec for env.* methods
/// Returns (iface_name, method_name, effects, returns_value)
pub fn get_env_method_spec(
    iface: &str,
    method: &str,
) -> Option<(String, String, EffectMask, bool)> {
    match (iface, method) {
        // Future/async operations
        ("future", "delay") => Some((
            "env.future".to_string(),
            "delay".to_string(),
            EffectMask::READ.add(Effect::Io),
            true,
        )),
        ("future", "spawn") => Some((
            "env.future".to_string(),
            "spawn".to_string(),
            EffectMask::IO,
            true,
        )),

        // Task management
        ("task", "currentToken") => Some((
            "env.task".to_string(),
            "currentToken".to_string(),
            EffectMask::READ,
            true,
        )),
        ("task", "cancelCurrent") => Some((
            "env.task".to_string(),
            "cancelCurrent".to_string(),
            EffectMask::IO,
            false,
        )),

        // Console I/O
        ("console", "log") => Some((
            "env.console".to_string(),
            "log".to_string(),
            EffectMask::IO,
            false,
        )),
        ("console", "readLine") => Some((
            "env.console".to_string(),
            "readLine".to_string(),
            EffectMask::IO,
            true,
        )),
        ("console", "error") => Some((
            "env.console".to_string(),
            "error".to_string(),
            EffectMask::IO,
            false,
        )),

        // Canvas operations
        ("canvas", m) if matches!(m, "fillRect" | "fillText" | "clear") => Some((
            "env.canvas".to_string(),
            method.to_string(),
            EffectMask::IO,
            false,
        )),

        // File system
        ("fs", "readFile") => Some((
            "env.fs".to_string(),
            "readFile".to_string(),
            EffectMask::IO,
            true,
        )),
        ("fs", "writeFile") => Some((
            "env.fs".to_string(),
            "writeFile".to_string(),
            EffectMask::IO,
            false,
        )),
        ("fs", "exists") => Some((
            "env.fs".to_string(),
            "exists".to_string(),
            EffectMask::READ,
            true,
        )),

        // Network
        ("net", "fetch") => Some((
            "env.net".to_string(),
            "fetch".to_string(),
            EffectMask::IO,
            true,
        )),
        ("net", "listen") => Some((
            "env.net".to_string(),
            "listen".to_string(),
            EffectMask::IO,
            true,
        )),

        // Process/system
        ("process", "exit") => Some((
            "env.process".to_string(),
            "exit".to_string(),
            EffectMask::IO.add(Effect::Control),
            false,
        )),
        ("process", "argv") => Some((
            "env.process".to_string(),
            "argv".to_string(),
            EffectMask::READ,
            true,
        )),
        ("process", "env") => Some((
            "env.process".to_string(),
            "env".to_string(),
            EffectMask::READ,
            true,
        )),

        // Unknown
        _ => None,
    }
}

/// Parse external call name into interface and method
/// E.g., "nyash.builtin.print" -> ("nyash.builtin", "print")
pub fn parse_extern_name(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        (parts[1].to_string(), parts[0].to_string())
    } else {
        ("nyash".to_string(), name.to_string())
    }
}

/// Check if a name refers to an environment interface
pub fn is_env_interface(name: &str) -> bool {
    matches!(name,
        "env" | "env.console" | "env.fs" | "env.net" |
        "env.canvas" | "env.task" | "env.future" | "env.process"
    )
}

/// Determine effects for an external call
pub fn compute_extern_effects(iface: &str, method: &str) -> EffectMask {
    match (iface, method) {
        // Pure reads
        (_, m) if m.starts_with("get") || m == "argv" || m == "env" => {
            EffectMask::READ
        }
        // Control flow changes
        (_, "exit") | (_, "panic") | (_, "throw") => {
            EffectMask::IO.add(Effect::Control)
        }
        // Memory allocation
        (_, m) if m.starts_with("new") || m.starts_with("create") => {
            EffectMask::IO.add(Effect::Alloc)
        }
        // Default to I/O
        _ => EffectMask::IO,
    }
}