//! InvokePolicyPass (minimal scaffold)
//! Centralizes decision for plugin/hostcall/any to keep lowerer slim.
//! Current implementation covers a small subset (ArrayBox length/get/set/push,
//! MapBox size/get/has/set) when NYASH_USE_PLUGIN_BUILTINS=1, falling back
//! to existing hostcall symbols otherwise. Extend incrementally.

#[derive(Debug, Clone)]
pub enum InvokeDecision {
    PluginInvoke { type_id: u32, method_id: u32, box_type: String, method: String, argc: usize, has_ret: bool },
    HostCall { symbol: String, argc: usize, has_ret: bool, reason: &'static str },
    Fallback { reason: &'static str },
}

fn use_plugin_builtins() -> bool {
    std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1")
}

/// Decide invocation policy for a known Box method.
pub fn decide_box_method(box_type: &str, method: &str, argc: usize, has_ret: bool) -> InvokeDecision {
    // Prefer plugin path when enabled and method is resolvable
    if use_plugin_builtins() {
        if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
            if let Ok(h) = ph.resolve_method(box_type, method) {
                return InvokeDecision::PluginInvoke { type_id: h.type_id, method_id: h.method_id, box_type: h.box_type, method: method.to_string(), argc, has_ret };
            }
        }
    }
    // Minimal hostcall mapping for common collections/math symbols
    let symbol = match (box_type, method) {
        ("ArrayBox", "length") | ("StringBox", "length") | ("StringBox", "len") => crate::jit::r#extern::collections::SYM_ANY_LEN_H,
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
    if symbol.is_empty() {
        InvokeDecision::Fallback { reason: "unknown_method" }
    } else {
        InvokeDecision::HostCall { symbol: symbol.to_string(), argc, has_ret, reason: "mapped_symbol" }
    }
}
