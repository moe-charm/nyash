//! VM runtime configuration

pub fn vm_birth_after_new_fallback() -> bool {
    match std::env::var("NYASH_VM_BIRTH_AFTER_NEW").ok() {
        Some(v) => {
            let on = matches!(v.as_str(), "1" | "true" | "on" | "TRUE" | "ON");
            if on && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[deprecated] NYASH_VM_BIRTH_AFTER_NEW is deprecated; Builder/Bridge emits birth(). Disable this flag to match production path.");
            }
            on
        }
        None => false,
    }
}

pub fn vm_pic_stats() -> bool {
    std::env::var("NYASH_VM_PIC_STATS").ok().as_deref() == Some("1")
}

pub fn vm_vt_trace() -> bool {
    std::env::var("NYASH_VM_VT_TRACE").ok().as_deref() == Some("1")
}

pub fn vm_pic_trace() -> bool {
    std::env::var("NYASH_VM_PIC_TRACE").ok().as_deref() == Some("1")
}

pub fn vm_pic_threshold() -> u32 {
    std::env::var("NYASH_VM_PIC_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
}

pub fn vm_allow_user_instance_boxcall() -> bool {
    match std::env::var("NYASH_VM_USER_INSTANCE_BOXCALL").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => !super::using::using_is_prod(),
    }
}

pub fn vm_use_py() -> bool {
    std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1")
}

pub fn vm_use_dispatch() -> bool {
    std::env::var("NYASH_VM_USE_DISPATCH").ok().as_deref() == Some("1")
}

pub fn vm_resolve_trace() -> bool {
    std::env::var("NYASH_VM_RESOLVE_TRACE").ok().as_deref() == Some("1")
}

pub fn vm_boxcall_plugin_first() -> bool {
    std::env::var("NYASH_VM_BOXCALL_PLUGIN_FIRST").ok().as_deref() == Some("1")
}

pub fn vm_global_tail_fallback() -> bool {
    std::env::var("NYASH_VM_GLOBAL_TAIL_FALLBACK").ok().as_deref() == Some("1")
}

pub fn vm_plugin_prefer_string() -> bool {
    std::env::var("NYASH_PLUGIN_PREFER_STRING").ok().as_deref() == Some("1")
}

pub fn vm_plugin_prefer_array() -> bool {
    std::env::var("NYASH_PLUGIN_PREFER_ARRAY").ok().as_deref() == Some("1")
}

pub fn vm_plugin_prefer_map() -> bool {
    std::env::var("NYASH_PLUGIN_PREFER_MAP").ok().as_deref() == Some("1")
}

pub fn vm_reenter_limit() -> Option<usize> {
    std::env::var("NYASH_VM_REENTER_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
}

pub fn vm_reenter_trace() -> bool {
    std::env::var("NYASH_VM_REENTER_TRACE").ok().as_deref() == Some("1")
}
