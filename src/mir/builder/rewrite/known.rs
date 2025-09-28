use super::super::{ConstValue, Effect, EffectMask, MirBuilder, MirInstruction, ValueId};

/// Gate: whether instance→function rewrite is enabled.
fn rewrite_enabled() -> bool {
    match std::env::var("NYASH_BUILDER_REWRITE_INSTANCE")
        .ok()
        .as_deref()
        .map(|v| v.to_ascii_lowercase())
    {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => true, // default ON (spec unchanged; dev can opt out)
    }
}

/// Try Known‑route instance→function rewrite.
/// 既存の安全ガード（user_defined/存在確認/ENV）を尊重して関数化する。
pub(crate) fn try_known_rewrite(
    builder: &mut MirBuilder,
    object_value: ValueId,
    cls: &str,
    method: &str,
    mut arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    // Global gate
    if !rewrite_enabled() {
        return None;
    }
    // Receiver must be Known (origin 由来)
    if builder.value_origin_newbox.get(&object_value).is_none() {
        return None;
    }
    // Only user-defined boxes (plugin/core boxesは対象外)
    if !builder.user_defined_boxes.contains(cls) {
        return None;
    }
    // Policy gates（従来互換）
    let allow_userbox_rewrite = std::env::var("NYASH_DEV_REWRITE_USERBOX").ok().as_deref() == Some("1");
    let allow_new_origin = std::env::var("NYASH_DEV_REWRITE_NEW_ORIGIN").ok().as_deref() == Some("1");
    let from_new_origin = builder.value_origin_newbox.get(&object_value).is_some();
    let arity = arg_values.len();
    let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(cls, method, arity);
    let module_has = if let Some(ref module) = builder.current_module { module.functions.contains_key(&fname) } else { false };
    if !( (module_has || allow_userbox_rewrite) || (from_new_origin && allow_new_origin) ) {
        return None;
    }
    // Materialize function call: pass 'me' first, then args
    let name_const = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
    let mut call_args = Vec::with_capacity(arity + 1);
    call_args.push(object_value);
    call_args.append(&mut arg_values);
    let dst = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Call {
        dst: Some(dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap),
    }) { return Some(Err(e)); }
    // Annotate and emit choose
    let chosen = fname.clone();
    builder.annotate_call_result_from_func_name(dst, &chosen);
    let meta = serde_json::json!({
        "recv_cls": cls,
        "method": method,
        "arity": arity,
        "chosen": chosen,
        "reason": "userbox-rewrite",
        "certainty": "Known",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
    Some(Ok(dst))
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
    if builder.value_origin_newbox.get(&object_value).is_none() { return None; }
    if !builder.user_defined_boxes.contains(cls) { return None; }
    let allow_userbox_rewrite = std::env::var("NYASH_DEV_REWRITE_USERBOX").ok().as_deref() == Some("1");
    let allow_new_origin = std::env::var("NYASH_DEV_REWRITE_NEW_ORIGIN").ok().as_deref() == Some("1");
    let from_new_origin = builder.value_origin_newbox.get(&object_value).is_some();
    let arity = arg_values.len();
    let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(cls, method, arity);
    let module_has = if let Some(ref module) = builder.current_module { module.functions.contains_key(&fname) } else { false };
    if !((module_has || allow_userbox_rewrite) || (from_new_origin && allow_new_origin)) { return None; }
    let name_const = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
    let mut call_args = Vec::with_capacity(arity + 1);
    call_args.push(object_value);
    call_args.append(&mut arg_values);
    let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
    if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(actual_dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap) }) { return Some(Err(e)); }
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

/// Fallback: when exactly one user-defined method matches by name/arity across the module,
/// resolve to that even if class inference failed. Deterministic via uniqueness and user-box prefix.
pub(crate) fn try_unique_suffix_rewrite(
    builder: &mut MirBuilder,
    object_value: ValueId,
    method: &str,
    mut arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    if !rewrite_enabled() {
        return None;
    }
    // Only attempt if receiver is Known (keeps behavior stable and avoids surprises)
    if builder.value_origin_newbox.get(&object_value).is_none() {
        return None;
    }
    let mut cands: Vec<String> = builder.method_candidates(method, arg_values.len());
    if cands.len() != 1 {
        return None;
    }
    let fname = cands.remove(0);
    if let Some((bx, _)) = fname.split_once('.') {
        if !builder.user_defined_boxes.contains(bx) {
            return None;
        }
    } else {
        return None;
    }
    let name_const = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
    let mut call_args = Vec::with_capacity(arg_values.len() + 1);
    call_args.push(object_value); // 'me'
    let arity_us = arg_values.len();
    call_args.append(&mut arg_values);
    let dst = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Call {
        dst: Some(dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap),
    }) { return Some(Err(e)); }
    builder.annotate_call_result_from_func_name(dst, &fname);
    let meta = serde_json::json!({
        "recv_cls": builder.value_origin_newbox.get(&object_value).cloned().unwrap_or_default(),
        "method": method,
        "arity": arity_us,
        "chosen": fname,
        "reason": "unique-suffix",
        "certainty": "Heuristic",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
    Some(Ok(dst))
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
    if builder.value_origin_newbox.get(&object_value).is_none() { return None; }
    let mut cands: Vec<String> = builder.method_candidates(method, arg_values.len());
    if cands.len() != 1 { return None; }
    let fname = cands.remove(0);
    if let Some((bx, _)) = fname.split_once('.') { if !builder.user_defined_boxes.contains(bx) { return None; } } else { return None; }
    let name_const = builder.value_gen.next();
    if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
    let mut call_args = Vec::with_capacity(arg_values.len() + 1);
    call_args.push(object_value);
    let arity_us = arg_values.len();
    call_args.append(&mut arg_values);
    let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
    if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(actual_dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap) }) { return Some(Err(e)); }
    builder.annotate_call_result_from_func_name(actual_dst, &fname);
    let meta = serde_json::json!({
        "recv_cls": builder.value_origin_newbox.get(&object_value).cloned().unwrap_or_default(),
        "method": method,
        "arity": arity_us,
        "chosen": fname,
        "reason": "unique-suffix",
        "certainty": "Heuristic",
    });
    super::super::observe::resolve::emit_choose(builder, meta);
    Some(Ok(actual_dst))
}

/// Unified entry: try Known rewrite first, then unique-suffix fallback.
pub(crate) fn try_known_or_unique(
    builder: &mut MirBuilder,
    object_value: ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    arg_values: Vec<ValueId>,
) -> Option<Result<ValueId, String>> {
    if let Some(cls) = class_name_opt.as_ref() {
        if let Some(res) = try_known_rewrite(builder, object_value, cls, method, arg_values.clone()) {
            return Some(res);
        }
    }
    try_unique_suffix_rewrite(builder, object_value, method, arg_values)
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
