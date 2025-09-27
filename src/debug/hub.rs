use std::io::Write;

/// Minimal debug hub: JSONL event emitter (dev-only; default OFF).
///
/// Env knobs:
/// - NYASH_DEBUG_ENABLE=1           master gate
/// - NYASH_DEBUG_KINDS=resolve,ssa  allowed cats (comma-separated)
/// - NYASH_DEBUG_SINK=path          file to append JSONL events
pub fn emit(cat: &str, kind: &str, fn_name: Option<&str>, region_id: Option<&str>, meta: serde_json::Value) {
    if std::env::var("NYASH_DEBUG_ENABLE").ok().as_deref() != Some("1") {
        return;
    }
    if let Ok(kinds) = std::env::var("NYASH_DEBUG_KINDS") {
        if !kinds.split(',').any(|k| k.trim().eq_ignore_ascii_case(cat)) {
            return;
        }
    }
    let sink = match std::env::var("NYASH_DEBUG_SINK") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let obj = serde_json::json!({
        "ts": ts,
        "phase": "builder",
        "fn": fn_name.unwrap_or("<unknown>"),
        "region_id": region_id.unwrap_or(""),
        "cat": cat,
        "kind": kind,
        "meta": meta,
    });
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&sink) {
        let _ = writeln!(f, "{}", obj.to_string());
    }
}

