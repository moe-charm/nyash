//! InvokePolicyPass (minimal scaffold)
//! Centralizes decision for plugin/hostcall to keep lowerer slim.
//! HostCall優先（Core-13方針）。ENV `NYASH_USE_PLUGIN_BUILTINS=1` の場合のみ
//! plugin_invoke を試し、解決できない場合はHostCallへフォールバックする。

#[derive(Debug, Clone)]
pub enum InvokeDecision {
    PluginInvoke { type_id: u32, method_id: u32, box_type: String, method: String, argc: usize, has_ret: bool },
    HostCall { symbol: String, argc: usize, has_ret: bool, reason: &'static str },
    Fallback { reason: &'static str },
}

fn use_plugin_builtins() -> bool {
    #[cfg(feature = "jit-direct-only")]
    { return false; }
    #[cfg(not(feature = "jit-direct-only"))]
    { return std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1"); }
}

/// Decide invocation policy for a known Box method.
pub fn decide_box_method(box_type: &str, method: &str, argc: usize, has_ret: bool) -> InvokeDecision {
    // Config-based resolution first（AOT 下位化でも安定して使える純粋関数）
    if use_plugin_builtins() {
        if let Some(mi) = crate::jit::policy::config_resolver::resolve_method_from_config(box_type, method) {
            return InvokeDecision::PluginInvoke { type_id: mi.type_id, method_id: mi.method_id, box_type: box_type.to_string(), method: method.to_string(), argc, has_ret };
        }
    }
    // HostCall mapping for common collections/strings/instance ops
    let symbol = match (box_type, method) {
        ("ArrayBox", "length") => crate::jit::r#extern::collections::SYM_ANY_LEN_H,
        ("StringBox", "length") | ("StringBox", "len") => "nyash.string.len_h",
        ("ArrayBox", "get") => crate::jit::r#extern::collections::SYM_ARRAY_GET_H,
        ("ArrayBox", "set") => crate::jit::r#extern::collections::SYM_ARRAY_SET_H,
        ("ArrayBox", "push") => crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H,
        ("MapBox",   "size") => crate::jit::r#extern::collections::SYM_MAP_SIZE_H,
        ("MapBox",   "get") => crate::jit::r#extern::collections::SYM_MAP_GET_HH,
        ("MapBox",   "has") => crate::jit::r#extern::collections::SYM_MAP_HAS_H,
        ("MapBox",   "set") => crate::jit::r#extern::collections::SYM_MAP_SET_H,
        ("StringBox","is_empty") => crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H,
        ("StringBox","charCodeAt") => crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H,
        _ => "" // unknown
    };
    // Prefer HostCall when available
    if !symbol.is_empty() {
        InvokeDecision::HostCall { symbol: symbol.to_string(), argc, has_ret, reason: "mapped_symbol" }
    } else {
        InvokeDecision::Fallback { reason: "unknown_method" }
    }
}
