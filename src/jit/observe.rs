//! Observe facade: centralize compile/runtime/trace output rules.
//! Thin wrappers around jit::events and shim_trace to keep callsites tidy.

pub fn lower_plugin_invoke(
    box_type: &str,
    method: &str,
    type_id: u32,
    method_id: u32,
    argc: usize,
) {
    crate::jit::events::emit_lower(
        serde_json::json!({
            "id": format!("plugin:{}:{}", box_type, method),
            "decision":"allow","reason":"plugin_invoke","argc": argc,
            "type_id": type_id, "method_id": method_id
        }),
        "plugin",
        "<jit>",
    );
}

pub fn lower_hostcall(symbol: &str, argc: usize, arg_types: &[&str], decision: &str, reason: &str) {
    crate::jit::events::emit_lower(
        serde_json::json!({
            "id": symbol,
            "decision": decision,
            "reason": reason,
            "argc": argc,
            "arg_types": arg_types
        }),
        "hostcall",
        "<jit>",
    );
}

pub fn runtime_plugin_shim_i64(type_id: i64, method_id: i64, argc: i64, inst: u32) {
    crate::jit::events::emit_runtime(
        serde_json::json!({
            "id": "plugin_invoke.i64",
            "type_id": type_id,
            "method_id": method_id,
            "argc": argc,
            "inst": inst
        }),
        "plugin",
        "<jit>",
    );
}

pub fn trace_push(msg: String) {
    crate::jit::shim_trace::push(msg);
}
pub fn trace_enabled() -> bool {
    crate::jit::shim_trace::is_enabled()
}
