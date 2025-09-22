//! JIT Events (v0): minimal JSONL appender for compile/execute/fallback/trap
//!
//! Emission is opt-in via env:
//! - NYASH_JIT_EVENTS=1 prints to stdout (one JSON per line)
//! - NYASH_JIT_EVENTS_PATH=/path/to/file.jsonl appends to file

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

// Compile-phase counters (process-local)
static LOWER_FALLBACK_COUNT: AtomicU64 = AtomicU64::new(0);

/// Reset compile-phase counters (call at the beginning of each lower/compile)
pub fn lower_counters_reset() {
    LOWER_FALLBACK_COUNT.store(0, Ordering::Relaxed);
}

/// Get number of fallback decisions observed during lowering
pub fn lower_fallbacks_get() -> u64 {
    LOWER_FALLBACK_COUNT.load(Ordering::Relaxed)
}

fn record_lower_decision(extra: &serde_json::Value) {
    // We record even when emission is disabled, to allow strict-mode checks.
    if let serde_json::Value::Object(map) = extra {
        if let Some(serde_json::Value::String(dec)) = map.get("decision") {
            if dec == "fallback" {
                LOWER_FALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn base_emit_enabled() -> bool {
    std::env::var("NYASH_JIT_EVENTS").ok().as_deref() == Some("1")
        || std::env::var("NYASH_JIT_EVENTS_PATH").is_ok()
}

fn should_emit_lower() -> bool {
    // Unify observability: if base events are on (stdout/file) or explicit compile flag, emit.
    base_emit_enabled() || std::env::var("NYASH_JIT_EVENTS_COMPILE").ok().as_deref() == Some("1")
}

fn should_emit_runtime() -> bool {
    base_emit_enabled() || std::env::var("NYASH_JIT_EVENTS_RUNTIME").ok().as_deref() == Some("1")
}

fn write_line(s: &str) {
    if let Ok(path) = std::env::var("NYASH_JIT_EVENTS_PATH") {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}", s)
            });
    } else {
        println!("{}", s);
    }
}

#[derive(Serialize)]
struct Event<'a, T: Serialize> {
    kind: &'a str,
    function: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    handle: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ms: Option<u128>,
    #[serde(flatten)]
    extra: T,
}

pub fn emit<T: Serialize>(
    kind: &str,
    function: &str,
    handle: Option<u64>,
    ms: Option<u128>,
    extra: T,
) {
    if !base_emit_enabled() {
        return;
    }
    let ev = Event {
        kind,
        function,
        handle,
        ms,
        extra,
    };
    if let Ok(s) = serde_json::to_string(&ev) {
        write_line(&s);
    }
}

fn emit_any(
    kind: &str,
    function: &str,
    handle: Option<u64>,
    ms: Option<u128>,
    extra: serde_json::Value,
) {
    let ev = Event {
        kind,
        function,
        handle,
        ms,
        extra,
    };
    if let Ok(s) = serde_json::to_string(&ev) {
        write_line(&s);
    }
}

/// Emit an event during lowering (compile-time planning). Adds phase="lower".
pub fn emit_lower(mut extra: serde_json::Value, kind: &str, function: &str) {
    // Always record decisions for strict-mode enforcement
    record_lower_decision(&extra);
    if !should_emit_lower() {
        return;
    }
    if let serde_json::Value::Object(ref mut map) = extra {
        map.insert("phase".into(), serde_json::Value::String("lower".into()));
    }
    emit_any(kind, function, None, None, extra);
}

/// Emit an event during runtime execution. Adds phase="execute".
pub fn emit_runtime(mut extra: serde_json::Value, kind: &str, function: &str) {
    if !should_emit_runtime() {
        return;
    }
    if let serde_json::Value::Object(ref mut map) = extra {
        map.insert("phase".into(), serde_json::Value::String("execute".into()));
    }
    emit_any(kind, function, None, None, extra);
}
