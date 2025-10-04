/*!
 * External Call Handling
 *
 * Manages env.* methods and external interface calls
 * Provides bridge to host environment functionality
 */

use crate::mir::{Effect, EffectMask};
use crate::mir::externs::registry as extreg;
use crate::mir::builder::effects::EffectResolverBox;

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

        // Rune (selfhost placeholder)
        ("rune", "eval") => Some((
            // Route env.rune.eval → nyrt.rune.eval (adapter handles provider/env)
            "nyrt.rune".to_string(),
            "eval".to_string(),
            EffectMask::READ,
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

/// Determine effects for an external call
pub fn compute_extern_effects(iface: &str, method: &str) -> EffectMask {
    // Prefer unified resolver when enabled, then registry table, then legacy heuristics
    let use_resolver = matches!(
        std::env::var("NYASH_USE_EFFECT_RESOLVER").ok().as_deref(),
        Some("1" | "true" | "on")
    );
    if use_resolver {
        if let Some(eff) = EffectResolverBox::new(false).resolve_extern(iface, method) {
            return eff;
        }
    }
    if let Some(eff) = extreg::effects_for(iface, method) {
        return eff;
    }
    match (iface, method) {
        // Runtime time source: monotonic millisecond timestamp (read-only)
        ("nyrt.time", "now_ms") => EffectMask::READ,
        // Collections: size queries are read-only
        ("nyrt.array", "size") | ("nyrt.map", "size") => EffectMask::READ,
        // Control flow changes (explicit)
        (_, "exit") | (_, "panic") | (_, "throw") => EffectMask::IO.add(Effect::Control),
        // Default: conservative I/O
        _ => EffectMask::IO,
    }
}
