use super::super::{ConstValue, Effect, EffectMask, MirBuilder, MirInstruction};

/// Early special-case: toString/stringify → str（互換）を処理。
/// 戻り値: Some(result_id) なら処理済み。None なら通常経路へ委譲。
pub(crate) fn try_early_str_like(
    builder: &mut MirBuilder,
    object_value: super::super::ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    arity: usize,
) -> Option<Result<super::super::ValueId, String>> {
    if !(method == "toString" || method == "stringify") || arity != 0 {
        return None;
    }
    let module = match &builder.current_module { Some(m) => m, None => return None };
    // Prefer class-qualified str if we can infer class; fallback to stringify for互換
    if let Some(cls) = class_name_opt.clone() {
        let str_name = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "str", 0);
        let compat_stringify = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "stringify", 0);
        let have_str = module.functions.contains_key(&str_name);
        let have_compat = module.functions.contains_key(&compat_stringify);
        if have_str || (!have_str && have_compat) {
            let chosen = if have_str { str_name } else { compat_stringify };
            // emit choose (dev-only)
            let meta = serde_json::json!({
                "recv_cls": cls,
                "method": method,
                "arity": 0,
                "chosen": chosen,
                "reason": if have_str { "toString-early-class-str" } else { "toString-early-class-stringify" },
                "certainty": "Known",
            });
            super::super::observe::resolve::emit_choose(builder, meta);
            let name_const = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(chosen.clone()) }) { return Some(Err(e)); }
            let mut call_args = Vec::with_capacity(1);
            call_args.push(object_value);
            let dst = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Call {
                dst: Some(dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap),
            }) { return Some(Err(e)); }
            builder.annotate_call_result_from_func_name(dst, &chosen);
            return Some(Ok(dst));
        }
    }
    // Unique suffix fallback: prefer *.str/0, else *.stringify/0（両方ある場合は JsonNode.str/0 を優先）
    // Merge candidates from tails: ".str/0" and ".stringify/0"
    let mut cands: Vec<String> = builder.method_candidates_tail(".str/0");
    let mut more = builder.method_candidates_tail(".stringify/0");
    cands.append(&mut more);
    if cands.len() == 1 {
        let fname = cands.remove(0);
        let meta = serde_json::json!({
            "recv_cls": class_name_opt.clone().unwrap_or_default(),
            "method": method,
            "arity": 0,
            "chosen": fname,
            "reason": "toString-early-unique",
            "certainty": "Heuristic",
        });
        super::super::observe::resolve::emit_choose(builder, meta);
        let name_const = builder.value_gen.next();
        if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
        let mut call_args = Vec::with_capacity(1);
        call_args.push(object_value);
        let dst = builder.value_gen.next();
        if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap), }) { return Some(Err(e)); }
        builder.annotate_call_result_from_func_name(dst, &fname);
        return Some(Ok(dst));
    } else if cands.len() > 1 {
        if let Some(pos) = cands.iter().position(|n| n == "JsonNode.str/0") {
            let fname = cands.remove(pos);
            let meta = serde_json::json!({
                "recv_cls": class_name_opt.clone().unwrap_or_default(),
                "method": method,
                "arity": 0,
                "chosen": fname,
                "reason": "toString-early-prefer-JsonNode",
                "certainty": "Heuristic",
            });
            super::super::observe::resolve::emit_choose(builder, meta);
            let name_const = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
            let mut call_args = Vec::with_capacity(1);
            call_args.push(object_value);
            let dst = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap), }) { return Some(Err(e)); }
            builder.annotate_call_result_from_func_name(dst, &fname);
            return Some(Ok(dst));
        }
    }
    None
}

/// Special-case for equals/1: prefer Known rewrite; otherwise allow unique-suffix fallback
/// when it is deterministic (single candidate). This centralizes equals handling.
pub(crate) fn try_special_equals(
    builder: &mut MirBuilder,
    object_value: super::super::ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    args: Vec<super::super::ValueId>,
) -> Option<Result<super::super::ValueId, String>> {
    if method != "equals" || args.len() != 1 { return None; }
    // First, Known rewrite if possible
    if let Some(cls) = class_name_opt.as_ref() {
        if let Some(res) = super::known::try_known_rewrite(builder, object_value, cls, method, args.clone()) {
            return Some(res);
        }
    }
    // Next, try unique-suffix fallback only for equals/1
    super::known::try_unique_suffix_rewrite(builder, object_value, method, args)
}

/// To-dst variant: early str-like with a requested destination
pub(crate) fn try_early_str_like_to_dst(
    builder: &mut MirBuilder,
    want_dst: Option<super::super::ValueId>,
    object_value: super::super::ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    arity: usize,
) -> Option<Result<super::super::ValueId, String>> {
    if !(method == "toString" || method == "stringify") || arity != 0 {
        return None;
    }
    let module = match &builder.current_module { Some(m) => m, None => return None };
    if let Some(cls) = class_name_opt.clone() {
        let str_name = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "str", 0);
        let compat_stringify = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "stringify", 0);
        let have_str = module.functions.contains_key(&str_name);
        let have_compat = module.functions.contains_key(&compat_stringify);
        if have_str || (!have_str && have_compat) {
            let chosen = if have_str { str_name } else { compat_stringify };
            let meta = serde_json::json!({
                "recv_cls": cls,
                "method": method,
                "arity": 0,
                "chosen": chosen,
                "reason": if have_str { "toString-early-class-str" } else { "toString-early-class-stringify" },
                "certainty": "Known",
            });
            super::super::observe::resolve::emit_choose(builder, meta);
            let name_const = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(chosen.clone()) }) { return Some(Err(e)); }
            let mut call_args = Vec::with_capacity(1);
            call_args.push(object_value);
            let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
            if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(actual_dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap), }) { return Some(Err(e)); }
            builder.annotate_call_result_from_func_name(actual_dst, &chosen);
            return Some(Ok(actual_dst));
        }
    }
    let mut cands: Vec<String> = builder.method_candidates_tail(".str/0");
    let mut more = builder.method_candidates_tail(".stringify/0");
    cands.append(&mut more);
    if cands.len() == 1 {
        let fname = cands.remove(0);
        let meta = serde_json::json!({
            "recv_cls": class_name_opt.clone().unwrap_or_default(),
            "method": method,
            "arity": 0,
            "chosen": fname,
            "reason": "toString-early-unique",
            "certainty": "Heuristic",
        });
        super::super::observe::resolve::emit_choose(builder, meta);
        let name_const = builder.value_gen.next();
        if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
        let mut call_args = Vec::with_capacity(1);
        call_args.push(object_value);
        let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
        if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(actual_dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap), }) { return Some(Err(e)); }
        builder.annotate_call_result_from_func_name(actual_dst, &fname);
        return Some(Ok(actual_dst));
    } else if cands.len() > 1 {
        if let Some(pos) = cands.iter().position(|n| n == "JsonNode.str/0") {
            let fname = cands.remove(pos);
            let meta = serde_json::json!({
                "recv_cls": class_name_opt.clone().unwrap_or_default(),
                "method": method,
                "arity": 0,
                "chosen": fname,
                "reason": "toString-early-prefer-JsonNode",
                "certainty": "Heuristic",
            });
            super::super::observe::resolve::emit_choose(builder, meta);
            let name_const = builder.value_gen.next();
            if let Err(e) = builder.emit_instruction(MirInstruction::Const { dst: name_const, value: ConstValue::String(fname.clone()) }) { return Some(Err(e)); }
            let mut call_args = Vec::with_capacity(1);
            call_args.push(object_value);
            let actual_dst = want_dst.unwrap_or_else(|| builder.value_gen.next());
            if let Err(e) = builder.emit_instruction(MirInstruction::Call { dst: Some(actual_dst), func: name_const, callee: None, args: call_args, effects: EffectMask::READ.add(Effect::ReadHeap), }) { return Some(Err(e)); }
            builder.annotate_call_result_from_func_name(actual_dst, &fname);
            return Some(Ok(actual_dst));
        }
    }
    None
}

/// To-dst variant: equals/1 consolidation with requested destination
pub(crate) fn try_special_equals_to_dst(
    builder: &mut MirBuilder,
    want_dst: Option<super::super::ValueId>,
    object_value: super::super::ValueId,
    class_name_opt: &Option<String>,
    method: &str,
    args: Vec<super::super::ValueId>,
) -> Option<Result<super::super::ValueId, String>> {
    if method != "equals" || args.len() != 1 { return None; }
    if let Some(cls) = class_name_opt.as_ref() {
        if let Some(res) = super::known::try_known_rewrite_to_dst(builder, want_dst, object_value, cls, method, args.clone()) {
            return Some(res);
        }
    }
    super::known::try_unique_suffix_rewrite_to_dst(builder, want_dst, object_value, method, args)
}
