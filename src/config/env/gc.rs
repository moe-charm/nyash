//! Garbage collector configuration

pub fn gc_trace() -> bool {
    std::env::var("NYASH_GC_TRACE").ok().as_deref() == Some("1")
}

pub fn gc_barrier_trace() -> bool {
    std::env::var("NYASH_GC_BARRIER_TRACE").ok().as_deref() == Some("1")
}

pub fn gc_barrier_strict() -> bool {
    std::env::var("NYASH_GC_BARRIER_STRICT").ok().as_deref() == Some("1")
}

pub fn gc_trace_level() -> u8 {
    std::env::var("NYASH_GC_TRACE_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn gc_mode() -> String {
    std::env::var("NYASH_GC_MODE").unwrap_or_else(|_| "counting".to_string())
}

pub fn gc_metrics() -> bool {
    std::env::var("NYASH_GC_METRICS").ok().as_deref() == Some("1")
}

pub fn gc_metrics_json() -> bool {
    std::env::var("NYASH_GC_METRICS_JSON").ok().as_deref() == Some("1")
}

pub fn gc_leak_diag() -> bool {
    std::env::var("NYASH_GC_LEAK_DIAG").ok().as_deref() == Some("1")
}

pub fn gc_alloc_threshold() -> Option<u64> {
    std::env::var("NYASH_GC_ALLOC_THRESHOLD").ok().and_then(|s| s.parse().ok())
}

pub fn gc_collect_sp_interval() -> Option<u64> {
    std::env::var("NYASH_GC_COLLECT_SP_INTERVAL").ok().and_then(|s| s.parse().ok())
}

pub fn gc_collect_alloc_bytes() -> Option<u64> {
    std::env::var("NYASH_GC_COLLECT_ALLOC_BYTES").ok().and_then(|s| s.parse().ok())
}
