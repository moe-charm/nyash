//! Plugin system configuration

pub fn plugin_only() -> bool {
    match std::env::var("NYASH_PLUGIN_ONLY").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        Some("1") | Some("true") | Some("on") => true,
        _ => false,
    }
}

pub fn plugin_abi_final() -> bool {
    match std::env::var("NYASH_PLUGIN_ABI").ok().as_deref() {
        Some("final") => true,
        _ => false,
    }
}

pub fn plugin_meta() -> bool {
    std::env::var("NYASH_PLUGIN_META").ok().as_deref() == Some("1")
}

pub fn plugin_caps_enforce() -> bool {
    match std::env::var("NYASH_PLUGIN_CAPS_ENFORCE").ok().as_deref() {
        Some("0") | Some("false") => false,
        _ => true,
    }
}

pub fn pipe_use_pyvm() -> bool {
    std::env::var("NYASH_PIPE_USE_PYVM").ok().as_deref() == Some("1")
}
