//! Runtime features configuration

pub fn await_max_ms() -> u64 {
    std::env::var("NYASH_AWAIT_MAX_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000)
}

pub fn cleanup_allow_return() -> bool {
    match std::env::var("NYASH_CLEANUP_ALLOW_RETURN").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn cleanup_allow_throw() -> bool {
    match std::env::var("NYASH_CLEANUP_ALLOW_THROW").ok().as_deref() {
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn runtime_checkpoint_trace() -> bool {
    std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE").ok().as_deref() == Some("1")
}

pub fn trace_effects() -> bool {
    std::env::var("NYASH_TRACE_EFFECTS").ok().as_deref() == Some("1")
}

pub fn extern_trace() -> bool {
    std::env::var("NYASH_EXTERN_TRACE").ok().as_deref() == Some("1")
}

pub fn extern_strict() -> bool {
    std::env::var("NYASH_EXTERN_STRICT").ok().as_deref() == Some("1")
}

pub fn extern_route_slots() -> bool {
    std::env::var("NYASH_EXTERN_ROUTE_SLOTS").ok().as_deref() == Some("1")
}

pub fn scopebox_enable() -> bool {
    std::env::var("NYASH_SCOPEBOX_ENABLE").ok().as_deref() == Some("1")
}

pub fn scope_stats_log() -> bool {
    std::env::var("NYASH_SCOPE_STATS_LOG").ok().as_deref() == Some("1")
}

pub fn selfhost_read_tmp() -> bool {
    std::env::var("NYASH_SELFHOST_READ_TMP").ok().as_deref() == Some("1")
}

pub fn release_trace() -> bool {
    std::env::var("NYASH_RELEASE_TRACE").ok().as_deref() == Some("1")
}

pub fn unified_members() -> bool {
    std::env::var("NYASH_UNIFIED_MEMBERS").ok().as_deref() == Some("1")
}
