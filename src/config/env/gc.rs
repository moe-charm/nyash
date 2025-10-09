//! Garbage collector configuration

use crate::config::env_helpers::{env_flag, env_or};

pub fn gc_trace() -> bool {
    env_flag("NYASH_GC_TRACE")
}

pub fn gc_barrier_trace() -> bool {
    env_flag("NYASH_GC_BARRIER_TRACE")
}

pub fn gc_barrier_strict() -> bool {
    env_flag("NYASH_GC_BARRIER_STRICT")
}

pub fn gc_trace_level() -> u8 {
    std::env::var("NYASH_GC_TRACE_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn gc_mode() -> String {
    env_or("NYASH_GC_MODE", "counting")
}

pub fn gc_metrics() -> bool {
    env_flag("NYASH_GC_METRICS")
}

pub fn gc_metrics_json() -> bool {
    env_flag("NYASH_GC_METRICS_JSON")
}

pub fn gc_leak_diag() -> bool {
    env_flag("NYASH_GC_LEAK_DIAG")
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
