use crate::mir::definitions::call_unified::TypeCertainty;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Unified,
    BoxCall,
}

/// Decide routing policy for a method call (Unified vs BoxCall) without changing behavior.
/// Rules (behavior-preserving):
/// - UnknownBox → BoxCall (unified is unstable for unknown receivers)
/// - Core boxes: StringBox/ArrayBox/MapBox → BoxCall (legacy path preferred)
/// - User boxes: names not ending with "Box" → BoxCall
/// - Otherwise Unified
pub fn choose_route(box_name: &str, method: &str, certainty: TypeCertainty, arity: usize) -> Route {
    // Global override for experiments: force Unified route (env only; default OFF)
    if std::env::var("NYASH_ROUTER_FORCE_UNIFIED").ok().as_deref() == Some("1") {
        if router_trace_enabled() {
            eprintln!(
                "[router] route={:?} reason={} recv={} method={} arity={} certainty={:?}",
                Route::Unified, "force_env", box_name, method, arity, certainty
            );
        }
        return Route::Unified;
    }
    let mut reason = "unified";
    let route = if box_name == "UnknownBox" {
        reason = "unknown_recv";
        Route::BoxCall
    } else if box_name == "InstanceBox" && matches!(method,
        "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
    ) {
        // User instance should not receive string-APIs via BoxCall; prefer unified so
        // downstream inference can normalize to StringBox safely.
        reason = "instance-string-like-to-unified";
        Route::Unified
    } else if box_name == "ParserBox" && matches!(method,
        "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
    ) {
        // Defensive: string-like methods on ParserBox should route to BoxCall;
        // infer on BoxCall side will normalize to StringBox and hit string bridge.
        reason = "parserbox-string-like";
        Route::BoxCall
    } else if matches!(box_name, "DebugBox" | "FileBox") && matches!(method,
        "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
    ) {
        // Extra defensive: debug/file boxes shouldn't own string APIs; prefer BoxCall bridge.
        reason = "nonstring-box-string-like";
        Route::BoxCall
    } else if crate::runtime::type_registry::is_core_box(box_name) {
        // Core boxes default to BoxCall to preserve legacy behavior.
        // Exception: length-like (size/len/length, arity=0) on String/Array route to Unified
        // so builder can normalize to stable externs (avoids recv SSA fragility).
        if ((box_name == "StringBox") || (box_name == "ArrayBox"))
            && arity == 0
            && matches!(method, "size" | "len" | "length")
        {
            reason = "core_box_len_unified";
            Route::Unified
        } else {
            reason = "core_box";
            Route::BoxCall
        }
    } else if !box_name.ends_with("Box") {
        reason = "user_instance";
        Route::BoxCall
    } else {
        Route::Unified
    };

    if router_trace_enabled() {
        eprintln!(
            "[router] route={:?} reason={} recv={} method={} arity={} certainty={:?}",
            route, reason, box_name, method, arity, certainty
        );
    }

    route
}

#[inline]
fn router_trace_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var("NYASH_ROUTER_TRACE") {
        Ok(val) => matches!(val.as_str(), "1" | "true" | "on" | "yes"),
        Err(_) => false,
    })
}
