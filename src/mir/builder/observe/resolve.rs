use super::super::MirBuilder;
// DELETED: KPI recording imports/statics/helpers (33 lines) - Phase 15.7 debug feature unused
// use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::OnceLock;

/// Dev‑only: emit a resolve.try event（candidates inspection）。
pub(crate) fn emit_try(builder: &MirBuilder, meta: serde_json::Value) {
    let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
    let region = builder.debug_current_region_id();
    crate::debug::hub::emit("resolve", "try", fn_name, region.as_deref(), meta);
}

/// Dev‑only: emit a resolve.choose event（decision）。
pub(crate) fn emit_choose(builder: &MirBuilder, meta: serde_json::Value) {
    let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
    let region = builder.debug_current_region_id();
    // DELETED: record_kpi(&meta); - KPI recording removed (Phase 15.7 debug feature)
    crate::debug::hub::emit("resolve", "choose", fn_name, region.as_deref(), meta);
}

// DELETED: record_kpi() function (16 lines) - Alternative: use DebugHub JSON output + jq post-analysis
