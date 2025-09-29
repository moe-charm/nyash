//! Runner tracing helpers (verbose-guarded)

/// Return whether CLI verbose logging is enabled
#[allow(dead_code)]
pub fn cli_verbose() -> bool {
    crate::config::env::cli_verbose()
}

#[macro_export]
macro_rules! cli_v {
    ($($arg:tt)*) => {{
        if crate::config::env::cli_verbose() { eprintln!($($arg)*); }
    }};
}

/// Unstructured trace output function used by pipeline helpers
pub fn log<S: AsRef<str>>(msg: S) {
    // Only emit when explicitly requested for resolver traces or when CLI verbose is on.
    let resolve_trace = std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1");
    if resolve_trace || crate::config::env::cli_verbose() {
        eprintln!("{}", msg.as_ref());
    }
}

/// Emit a single-line JSON resolver trace when NYASH_RESOLVE_TRACE_JSON=1
/// Shape: {"kind":"using","event":"resolve","target":"...","decision":"...","candidates":[...],"reason":"..."}
pub fn log_json_using(target: &str, decision: Option<&str>, candidates: &[String], reason: &str) {
    if std::env::var("NYASH_RESOLVE_TRACE_JSON").ok().as_deref() != Some("1") {
        return;
    }
    // Keep it robust: escape quotes minimally and avoid newlines
    fn esc(s: &str) -> String { s.replace('"', "\\\"").replace('\n', " ") }
    let tgt = esc(target);
    let dec = decision.map(esc);
    let cands = if candidates.is_empty() {
        String::from("[]")
    } else {
        let mut v = String::from("[");
        for (i, c) in candidates.iter().enumerate() {
            if i > 0 { v.push(','); }
            v.push('"'); v.push_str(&esc(c)); v.push('"');
        }
        v.push(']');
        v
    };
    let mut out = String::from("{\"kind\":\"using\",\"event\":\"resolve\",");
    out.push_str(&format!("\"target\":\"{}\",", tgt));
    if let Some(d) = dec { out.push_str(&format!("\"decision\":\"{}\",", d)); }
    out.push_str(&format!("\"candidates\":{},", cands));
    out.push_str(&format!("\"reason\":\"{}\"}}", esc(reason)));
    eprintln!("{}", out);
}
