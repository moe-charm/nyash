//! MethodRegistryBox — facade for method resolution and tracing
//!
//! Responsibility
//! - Single entry for resolving method ids/handles, consolidating trace.
//! - Delegates to plugin_loader_unified and v2 resolver under the hood.

use crate::bid::BidResult;

pub fn resolve_method_id(box_type: &str, method_name: &str) -> BidResult<u32> {
    if let Some((_tid, mid)) = crate::runtime::type_registry::resolve_builtin_method_handle(box_type, method_name) {
        return Ok(mid);
    }
    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
    let guard = host.read().map_err(|_| crate::bid::BidError::PluginError)?;
    // Prefer v2 resolver via unified host
    let mh = guard.resolve_method(box_type, method_name)?;
    if crate::runtime::env_gate_box::bool_any(&["NYASH_METHOD_REG_TRACE"]) {
        crate::runtime::diagnostics::trace_event(
            "method_resolve",
            &format!(
                "\"class\":\"{}\",\"method\":\"{}\",\"provider\":\"unified\",\"method_id\":{}",
                box_type, method_name, mh.method_id
            ),
        );
    }
    Ok(mh.method_id)
}

pub fn resolve_method_handle(
    box_type: &str,
    method_name: &str,
) -> BidResult<crate::runtime::plugin_loader_unified::MethodHandle> {
    if let Some((tid, mid)) = crate::runtime::type_registry::resolve_builtin_method_handle(box_type, method_name) {
        return Ok(crate::runtime::plugin_loader_unified::MethodHandle {
            lib: "builtin".to_string(),
            box_type: box_type.to_string(),
            type_id: tid,
            method_id: mid,
            returns_result: false,
        });
    }
    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
    let guard = host.read().map_err(|_| crate::bid::BidError::PluginError)?;
    let mh = guard.resolve_method(box_type, method_name)?;
    if crate::runtime::env_gate_box::bool_any(&["NYASH_METHOD_REG_TRACE"]) {
        crate::runtime::diagnostics::trace_event(
            "method_resolve",
            &format!(
                "\"class\":\"{}\",\"method\":\"{}\",\"provider\":\"unified\",\"method_id\":{}",
                box_type, method_name, mh.method_id
            ),
        );
    }
    Ok(mh)
}
