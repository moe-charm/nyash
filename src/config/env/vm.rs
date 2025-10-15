//! VM runtime configuration

use crate::config::env_helpers::{env_bool_ci, env_flag, env_u32};

pub fn vm_birth_after_new_fallback() -> bool {
    let on = env_bool_ci("NYASH_VM_BIRTH_AFTER_NEW");
    if on && env_flag("NYASH_CLI_VERBOSE") {
        eprintln!("[deprecated] NYASH_VM_BIRTH_AFTER_NEW is deprecated; Builder/Bridge emits birth(). Disable this flag to match production path.");
    }
    on
}

pub fn vm_pic_stats() -> bool {
    env_flag("NYASH_VM_PIC_STATS")
}

pub fn vm_vt_trace() -> bool {
    env_flag("NYASH_VM_VT_TRACE")
}

pub fn vm_pic_trace() -> bool {
    env_flag("NYASH_VM_PIC_TRACE")
}

pub fn vm_pic_threshold() -> u32 {
    env_u32("NYASH_VM_PIC_THRESHOLD", 8)
}

pub fn vm_allow_user_instance_boxcall() -> bool {
    match std::env::var("NYASH_VM_USER_INSTANCE_BOXCALL").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => !super::using::using_is_prod(),
    }
}

pub fn vm_use_py() -> bool {
    env_flag("NYASH_VM_USE_PY")
}

pub fn vm_use_dispatch() -> bool {
    env_flag("NYASH_VM_USE_DISPATCH")
}

pub fn vm_resolve_trace() -> bool {
    env_flag("NYASH_VM_RESOLVE_TRACE")
}

pub fn vm_global_tail_fallback() -> bool {
    env_flag("NYASH_VM_GLOBAL_TAIL_FALLBACK")
}

pub fn vm_plugin_prefer_string() -> bool {
    env_flag("NYASH_PLUGIN_PREFER_STRING")
}

pub fn vm_plugin_prefer_array() -> bool {
    env_flag("NYASH_PLUGIN_PREFER_ARRAY")
}

pub fn vm_plugin_prefer_map() -> bool {
    env_flag("NYASH_PLUGIN_PREFER_MAP")
}

pub fn vm_reenter_limit() -> Option<usize> {
    std::env::var("NYASH_VM_REENTER_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
}

pub fn vm_reenter_trace() -> bool {
    env_flag("NYASH_VM_REENTER_TRACE")
}
