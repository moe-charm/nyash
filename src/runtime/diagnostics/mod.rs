//! DiagnosticsBox — unified debug helpers (dbg_on/once/trace)
//!
//! Minimal facade to centralize debug flags and one-time logging.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub fn dbg_on() -> bool {
    crate::runtime::env_gate_box::debug_plugin()
}

static DBG_ONCE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub fn dbg_once(key: &str, msg: &str) {
    if !dbg_on() { return; }
    let set = DBG_ONCE.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut g) = set.lock() {
        if g.insert(key.to_string()) { eprintln!("{}", msg); }
    }
}

/// Emit a structured JSON event when enabled by an env flag (future extension)
pub fn trace_event(kind: &str, json_fields: &str) {
    if !crate::runtime::env_gate_box::diag_trace() { return; }
    // Expect json_fields to be a JSON object body without surrounding braces.
    // Example: "\"box\":\"ArrayBox\",\"status\":\"ok\""
    eprintln!("{{\"kind\":\"{}\",{}}}", kind, json_fields);
}
