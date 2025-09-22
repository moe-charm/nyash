//! InvokePolicyPass (minimal scaffold)
//! Centralizes decision for plugin/hostcall to keep lowerer slim.
//! HostCall優先（Core-13方針）。ENV `NYASH_USE_PLUGIN_BUILTINS=1` の場合のみ
//! plugin_invoke を試し、解決できない場合はHostCallへフォールバックする。

#[derive(Debug, Clone)]
pub enum InvokeDecision {
    PluginInvoke {
        type_id: u32,
        method_id: u32,
        box_type: String,
        method: String,
        argc: usize,
        has_ret: bool,
    },
    HostCall {
        symbol: String,
        argc: usize,
        has_ret: bool,
        reason: &'static str,
    },
    Fallback {
        reason: &'static str,
    },
}

fn use_plugin_builtins() -> bool {
    #[cfg(feature = "jit-direct-only")]
    {
        return false;
    }
    #[cfg(not(feature = "jit-direct-only"))]
    {
        return std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1");
    }
}

/// Decide invocation policy for a known Box method.
pub fn decide_box_method(
    box_type: &str,
    method: &str,
    argc: usize,
    has_ret: bool,
) -> InvokeDecision {
    // HostCall mapping for common collections/strings/instance ops
    let symbol = match (box_type, method) {
        ("ArrayBox", "length") => crate::jit::r#extern::collections::SYM_ANY_LEN_H,
        ("StringBox", "length") | ("StringBox", "len") => "nyash.string.len_h",
        ("ArrayBox", "get") => crate::jit::r#extern::collections::SYM_ARRAY_GET_H,
        ("ArrayBox", "set") => crate::jit::r#extern::collections::SYM_ARRAY_SET_H,
        ("ArrayBox", "push") => crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H,
        ("MapBox", "size") => crate::jit::r#extern::collections::SYM_MAP_SIZE_H,
        ("MapBox", "get") => crate::jit::r#extern::collections::SYM_MAP_GET_HH,
        ("MapBox", "has") => crate::jit::r#extern::collections::SYM_MAP_HAS_H,
        ("MapBox", "set") => crate::jit::r#extern::collections::SYM_MAP_SET_H,
        ("StringBox", "is_empty") => crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H,
        ("StringBox", "charCodeAt") => crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H,
        _ => "", // unknown
    };
    // Prefer HostCall when available
    if !symbol.is_empty() {
        InvokeDecision::HostCall {
            symbol: symbol.to_string(),
            argc,
            has_ret,
            reason: "mapped_symbol",
        }
    } else if use_plugin_builtins() {
        // Try plugin_invoke as a secondary path when enabled
        if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
            if let Ok(h) = ph.resolve_method(box_type, method) {
                return InvokeDecision::PluginInvoke {
                    type_id: h.type_id,
                    method_id: h.method_id,
                    box_type: h.box_type,
                    method: method.to_string(),
                    argc,
                    has_ret,
                };
            }
        }
        InvokeDecision::Fallback {
            reason: "unknown_method",
        }
    } else {
        InvokeDecision::Fallback {
            reason: "unknown_method",
        }
    }
}
