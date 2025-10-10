//! Runtime features configuration

use crate::config::env_helpers::{env_bool, env_flag, env_u64_any};

pub fn await_max_ms() -> u64 {
    env_u64_any(&["HAKO_AWAIT_MAX_MS", "NYASH_AWAIT_MAX_MS"], 5000)
}

pub fn cleanup_allow_return() -> bool {
    env_bool("NYASH_CLEANUP_ALLOW_RETURN")
}

pub fn cleanup_allow_throw() -> bool {
    env_bool("NYASH_CLEANUP_ALLOW_THROW")
}

pub fn runtime_checkpoint_trace() -> bool {
    env_flag("NYASH_RUNTIME_CHECKPOINT_TRACE")
}

pub fn trace_effects() -> bool {
    env_flag("NYASH_TRACE_EFFECTS")
}

pub fn extern_trace() -> bool {
    env_flag("NYASH_EXTERN_TRACE")
}

pub fn extern_strict() -> bool {
    env_flag("NYASH_EXTERN_STRICT")
}

pub fn extern_route_slots() -> bool {
    env_flag("NYASH_EXTERN_ROUTE_SLOTS")
}

pub fn scopebox_enable() -> bool {
    env_flag("NYASH_SCOPEBOX_ENABLE")
}

pub fn scope_stats_log() -> bool {
    env_flag("NYASH_SCOPE_STATS_LOG")
}

pub fn selfhost_read_tmp() -> bool {
    env_flag("NYASH_SELFHOST_READ_TMP")
}

pub fn release_trace() -> bool {
    env_flag("NYASH_RELEASE_TRACE")
}

pub fn unified_members() -> bool {
    env_flag("NYASH_UNIFIED_MEMBERS")
}
