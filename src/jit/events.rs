//! JIT Events (v0): minimal JSONL appender for compile/execute/fallback/trap
//!
//! Emission is opt-in via env:
//! - NYASH_JIT_EVENTS=1 prints to stdout (one JSON per line)
//! - NYASH_JIT_EVENTS_PATH=/path/to/file.jsonl appends to file

use serde::Serialize;

fn should_emit() -> bool {
    std::env::var("NYASH_JIT_EVENTS").ok().as_deref() == Some("1")
        || std::env::var("NYASH_JIT_EVENTS_PATH").is_ok()
}

fn write_line(s: &str) {
    if let Ok(path) = std::env::var("NYASH_JIT_EVENTS_PATH") {
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| {
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

pub fn emit<T: Serialize>(kind: &str, function: &str, handle: Option<u64>, ms: Option<u128>, extra: T) {
    if !should_emit() { return; }
    let ev = Event { kind, function, handle, ms, extra };
    if let Ok(s) = serde_json::to_string(&ev) { write_line(&s); }
}

