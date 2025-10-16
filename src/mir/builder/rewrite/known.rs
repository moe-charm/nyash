use super::super::{Effect, EffectMask, MirBuilder, ValueId};

/// Gate: whether instance→function rewrite is enabled.
fn rewrite_enabled() -> bool {
    // New primary flag (P4): NYASH_REWRITE_KNOWN_DEFAULT (default ON; allow explicit OFF)
    if let Ok(v) = std::env::var("NYASH_REWRITE_KNOWN_DEFAULT") {
        let s = v.to_ascii_lowercase();
        if s == "0" || s == "false" || s == "off" { return false; }
        if s == "1" || s == "true" || s == "on" { return true; }
        // fallthrough to legacy if malformed
    }
    // Legacy flag (kept for compatibility): NYASH_BUILDER_REWRITE_INSTANCE (default ON)
    match std::env::var("NYASH_BUILDER_REWRITE_INSTANCE").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => true, // default ON (spec unchanged; can opt out by setting ...=0)
    }
}

/// Variant: try Known rewrite but honor a requested destination.
pub(crate) fn try_known_rewrite_to_dst(
    builder: &mut MirBuilder,
    want_dst: Option<ValueId>,
    object_value: ValueId,
    cls: &str,
    method: &str,
    mut arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    if !rewrite_enabled() { return None; }
    if cls == "StringBox" && matches!(method, "length" | "len" | "substring" | "indexOf" | "lastIndexOf") { return None; }
    let arity = arg_values.len();
    if !crate::mir::builder::rewrite::gate::should_rewrite(builder, cls, method, arity) { return None; }
    let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(cls, method, arity);
    // Proceed for user-defined boxes regardless of module registration timing.
    let name_const = match crate::mir::builder::name_const::make_name_const_result(builder, &fname) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };
    let mut call_args = Vec::with_capacity(arity + 1);
    call_args.push(object_value);
    call_args.append(&mut arg_values);
    let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
    if let Err(e) = builder.emit_call_with_guard(
        Some(actual_dst),
        name_const,
        crate::mir::Callee::ModuleFunction(fname.clone()),
        call_args,
        EffectMask::READ.add(Effect::ReadHeap),
    ) {
        return Some(Err(e));
    }
    builder.annotate_call_result_from_func_name(actual_dst, &fname);
    let meta = serde_json::json!({
        "recv_cls": cls,
        "method": method,
        "arity": arity,
        "chosen": fname,
        "reason": "userbox-rewrite",
        "certainty": "Known",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
    Some(Ok(actual_dst))
}

/// Variant: unique-suffix rewrite honoring requested destination.
pub(crate) fn try_unique_suffix_rewrite_to_dst(
    builder: &mut MirBuilder,
    want_dst: Option<ValueId>,
    object_value: ValueId,
    method: &str,
    mut arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    if !rewrite_enabled() { return None; }
    if builder.origin_get(object_value).is_none() { return None; }
    let mut cands: Vec<String> = builder.method_candidates(method, arg_values.len());
    if cands.len() != 1 { return None; }
    let fname = cands.remove(0);
    if let Some((bx, _)) = fname.split_once('.') { if !builder.user_defined_boxes.contains(bx) { return None; } } else { return None; }
    let mut call_args = Vec::with_capacity(arg_values.len() + 1);
    call_args.push(object_value);
    let arity_us = arg_values.len();
    call_args.append(&mut arg_values);
    crate::mir::builder::ssa::local::finalize_args(builder, &mut call_args);
    let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
    if let Err(e) = builder.emit_unified_call(
        Some(actual_dst),
        crate::mir::builder::builder_calls::CallTarget::Global(fname.clone()),
        call_args,
    ) { return Some(Err(e)); }
    builder.annotate_call_result_from_func_name(actual_dst, &fname);
    let meta = serde_json::json!({
        "recv_cls": builder.origin_get(object_value).unwrap_or_default().to_string(),
        "method": method,
        "arity": arity_us,
        "chosen": fname,
        "reason": "unique-suffix",
        "certainty": "Heuristic",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
    Some(Ok(actual_dst))
}

/// Variant: honor requested destination
pub(crate) fn try_known_or_unique_to_dst(
    builder: &mut MirBuilder,
    want_dst: Option<ValueId>,
    object_value: ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    if let Some(cls) = class_name_opt.as_ref() {
        if let Some(res) = try_known_rewrite_to_dst(builder, want_dst, object_value, cls, method, arg_values.clone()) {
            return Some(res);
        }
    }
    try_unique_suffix_rewrite_to_dst(builder, want_dst, object_value, method, arg_values)
}
