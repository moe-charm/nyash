//! Legacy call bridge box consolidates deprecated call emission paths.

use crate::mir::{Callee, Effect, EffectMask, MirInstruction, MirType, ValueId};
use crate::mir::builder::MirBuilder;
use super::call_target::CallTarget;
use crate::mir::builder::calls::function_lowering;

pub struct LegacyCallBridgeBox<'a> {
    builder: &'a mut MirBuilder,
}

impl<'a> LegacyCallBridgeBox<'a> {
    pub fn new(builder: &'a mut MirBuilder) -> Self {
        Self { builder }
    }

    pub fn emit(mut self, dst: Option<ValueId>, target: CallTarget, args: Vec<ValueId>) -> Result<(), String> {
        let builder = &mut self.builder;

        // DEPRECATION (Phase‑in): prefer `emit_unified_call` for new code paths.
        // This legacy path remains for compatibility while we converge all
        // call emission (Global/Extern/Method/Constructor) through the unified
        // callee model and RouterPolicy. Behavior is unchanged.
        match target {
            CallTarget::Method { receiver, method, box_type: _ } => {
                // Centralized rewrite: string size/len/length (0 args) → Extern("nyrt.string.length")
                {
                    let mut callee = Callee::Method { box_name: "StringBox".to_string(), method: method.clone(), receiver: Some(receiver), certainty: crate::mir::definitions::call_unified::TypeCertainty::Union };
                    let mut argv = Vec::<ValueId>::new();
                    let changed = crate::mir::builder::normalize::string_length::normalize_string_length_call(builder, &mut callee, &mut argv);
                    if changed {
                        crate::mir::builder::ssa::local::finalize_args(builder, &mut argv);
                        let dstv = dst.unwrap_or_else(|| builder.value_gen.next());
                        let name_const = crate::mir::builder::name_const::make_name_const_result(builder, "nyrt.string.length")?;
                        emit_call(
                            builder,
                            Some(dstv),
                            name_const,
                            callee,
                            argv,
                            EffectMask::READ.add(Effect::ReadHeap),
                        )?;
                        // Annotate result as Integer
                        builder.value_types.insert(dstv, MirType::Integer);
                        return Ok(());
                    }
                }
                // Centralized rewrite: array size/len/length (0 args) → Extern("nyrt.array.length")
                {
                    let mut callee = Callee::Method { box_name: "ArrayBox".to_string(), method: method.clone(), receiver: Some(receiver), certainty: crate::mir::definitions::call_unified::TypeCertainty::Union };
                    let mut argv = Vec::<ValueId>::new();
                    let changed = crate::mir::builder::normalize::array_length::normalize_array_length_call(builder, &mut callee, &mut argv);
                    if changed {
                        crate::mir::builder::ssa::local::finalize_args(builder, &mut argv);
                        let dstv = dst.unwrap_or_else(|| builder.value_gen.next());
                        let name_const = crate::mir::builder::name_const::make_name_const_result(builder, "nyrt.array.size")?;
                        emit_call(
                            builder,
                            Some(dstv),
                            name_const,
                            callee,
                            argv,
                            EffectMask::READ.add(Effect::ReadHeap),
                        )?;
                        // Annotate result as Integer
                        builder.value_types.insert(dstv, MirType::Integer);
                        return Ok(());
                    }
                }
                // Special-case: instance.birth(args) should invoke lowered ModuleFunction (user-defined birth)
                if method == "birth" {
                    let (cls, _cert) = crate::mir::builder::infer::receiver::infer_receiver(
                        None,
                        &method,
                        receiver,
                        |vid| builder.origin_get(vid).map(|s| s.to_string()),
                        &builder.value_types,
                    );
                    let me_local = builder.local_recv(receiver);
                    let mut call_args: Vec<ValueId> = Vec::with_capacity(args.len() + 1);
                    call_args.push(me_local);
                    call_args.extend(args.into_iter());
                    crate::mir::builder::ssa::local::finalize_args(builder, &mut call_args);
                    let out = dst.unwrap_or_else(|| builder.value_gen.next());
                    let fname = function_lowering::generate_method_function_name(&cls, &method, call_args.len() - 1);
                    let name_val = crate::mir::builder::name_const::make_name_const_result(builder, &fname)?;
                    emit_call(
                        builder,
                        Some(out),
                        name_val,
                        crate::mir::definitions::call_unified::Callee::ModuleFunction(fname.clone()),
                        call_args,
                        EffectMask::IO,
                    )?;
                    builder.annotate_call_result_from_func_name(out, &fname);
                    return Ok(());
                }
                // Prod rewrite: when user instance BoxCall is disallowed by policy,
                // lower `obj.method(a,b)` into module function `Class.method/Arity(me,a,b)`.
                // This preserves stable prod semantics without requiring unified-call flag.
                if !crate::config::env::vm_allow_user_instance_boxcall() {
                    let (cls, _cert) = crate::mir::builder::infer::receiver::infer_receiver(
                        None,
                        &method,
                        receiver,
                        |vid| builder.origin_get(vid).map(|s| s.to_string()),
                        &builder.value_types,
                    );
                    let me_local = builder.local_recv(receiver);
                    let mut call_args: Vec<ValueId> = Vec::with_capacity(args.len() + 1);
                    call_args.push(me_local);
                    call_args.extend(args.into_iter());
                    crate::mir::builder::ssa::local::finalize_args(builder, &mut call_args);
                    let out = dst.unwrap_or_else(|| builder.value_gen.next());
                    let fname = function_lowering::generate_method_function_name(&cls, &method, call_args.len() - 1);
                    let name_val = crate::mir::builder::name_const::make_name_const_result(builder, &fname)?;
                    emit_call(
                        builder,
                        Some(out),
                        name_val,
                        crate::mir::definitions::call_unified::Callee::ModuleFunction(fname.clone()),
                        call_args,
                        EffectMask::READ.add(Effect::ReadHeap),
                    )?;
                    builder.annotate_call_result_from_func_name(out, &fname);
                    return Ok(());
                }
                // Legacy fallback: Box/Plugin call
                emit_boxcall(builder, dst, receiver, method, args, EffectMask::IO)
            },
            CallTarget::Constructor(box_type) => {
                // Use existing NewBox
                let dst = dst.ok_or("Constructor must have destination")?;
                builder.emit_instruction(MirInstruction::NewBox {
                    dst,
                    box_type,
                    args,
                    auto_birth: None,
                })
            },
            CallTarget::Extern(name) => {
                // Unified path: emit Call with callee=Extern("iface.method")
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
                // Normalize dotted name; accept bare as "nyash.<name>"
                let full_name = if name.contains('.') { name } else { format!("nyash.{}", name) };
                // Compute effects for extern
                let (iface, method) = full_name.rsplit_once('.').unwrap_or(("nyash", full_name.as_str()));
                let effects = crate::mir::builder::calls::extern_calls::compute_extern_effects(iface, method);
                emit_call(
                    builder,
                    dst,
                    ValueId::new(0),
                    crate::mir::definitions::call_unified::Callee::Extern(full_name),
                    args,
                    effects,
                )
            },
            CallTarget::Global(name) => {
                // Early rewrite for dotted StringBox methods: StringBox.size/1 → Extern("nyrt.string.length")
                if (name.starts_with("StringBox.size") || name.starts_with("StringBox.length") || name.starts_with("StringBox.len")) && args.len() == 1 {
                    let recv_local = builder.local_recv(args[0]);
                    let mut argv = vec![recv_local];
                    crate::mir::builder::ssa::local::finalize_args(builder, &mut argv);
                    let dstv = dst.unwrap_or_else(|| builder.value_gen.next());
                    let name_const = crate::mir::builder::name_const::make_name_const_result(builder, "nyrt.string.length")?;
                    emit_call(
                        builder,
                        Some(dstv),
                        name_const,
                        crate::mir::definitions::call_unified::Callee::Extern("nyrt.string.length".to_string()),
                        argv,
                        EffectMask::READ.add(Effect::ReadHeap),
                    )?;
                    builder.value_types.insert(dstv, MirType::Integer);
                    return Ok(());
                }
                let normalized = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::normalize(&name, args.len()) {
                    Ok(full) => full,
                    Err(_) => format!("{}/{}", name, args.len()),
                };
                // Special-case: route hostbridge.* globals to Extern("hostbridge.*") for unified HostBridge
                if name.starts_with("hostbridge.") {
                    let mut args = args;
                    crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
                    emit_call(
                        builder,
                        dst,
                        ValueId::new(0),
                        crate::mir::definitions::call_unified::Callee::Extern(name.clone()),
                        args,
                        crate::mir::builder::calls::extern_calls::compute_extern_effects("hostbridge", name.strip_prefix("hostbridge.").unwrap_or("")),
                    )?;
                    return Ok(());
                }
                // Prefer direct ModuleFunction when available in current module (avoids legacy string callee)
                // Use CallNameResolverBox::normalize to ensure fully qualified form before lookup.
                if let Some(ref module) = builder.current_module {
                    // Only attempt module function lookup when the name looks like Class.method or fully-qualified.
                    if name.contains('.') {
                        let want = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::normalize(&name, args.len()) {
                            Ok(full) => full,
                            Err(_) => name.clone(), // keep raw if it cannot be normalized (unlikely when contains '.')
                        };
                        if module.functions.contains_key(&want) {
                            let actual_dst = if let Some(d) = dst { d } else { builder.value_gen.next() };
                            // If the target ModuleFunction expects an implicit receiver (static box normalization),
                            // prepend the per-function singleton `me` to the args.
                            let mut args = args;
                            if builder.method_index.static_signature(&want).is_some() {
                                if let Some(fun) = module.functions.get(&want) {
                                    let expected_params = fun.params.len();
                                    if expected_params == args.len() + 1 {
                                        // Parse box name (prefix before '.') from fully-qualified function name
                                        if let Some((box_name, _)) = want.split_once('.') {
                                            let me = builder.current_fn_singleton(box_name);
                                            let mut with_me = Vec::with_capacity(args.len() + 1);
                                            with_me.push(me);
                                            with_me.extend(args.drain(..));
                                            args = with_me;
                                        }
                                    }
                                }
                            }
                            crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
                            emit_call(
                                builder,
                                Some(actual_dst),
                                ValueId::new(0),
                                crate::mir::definitions::call_unified::Callee::ModuleFunction(want.clone()),
                                args,
                                EffectMask::IO,
                            )?;
                            builder.annotate_call_result_from_func_name(actual_dst, &want);
                            return Ok(());
                        }
                    }
                }
                // First-class: JSON.stringify(any) → arg0.toJSON() (arity 1→0)
                if name == "JSON.stringify/1" || name.starts_with("JSON.stringify") {
                    if let Some(recv) = args.get(0).cloned() {
                        let argv: Vec<ValueId> = Vec::new();
                        emit_boxcall(builder, dst, recv, "toJSON".to_string(), argv, EffectMask::READ)?;
                        return Ok(());
                    }
                }
                // Emit unified Global callee instead of legacy string-based call
                let actual_dst = if let Some(d) = dst { d } else { builder.value_gen.next() };
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
                emit_call(
                    builder,
                    Some(actual_dst),
                    ValueId::new(0),
                    crate::mir::definitions::call_unified::Callee::Global(normalized.clone()),
                    args,
                    EffectMask::IO,
                )?;
                builder.annotate_call_result_from_func_name(actual_dst, normalized);
                Ok(())
            },
            CallTarget::Value(func_val) => {
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
                emit_call(
                    builder,
                    dst,
                    func_val,
                    crate::mir::definitions::call_unified::Callee::Value(func_val),
                    args,
                    EffectMask::IO,
                )
            },
            CallTarget::Closure { params, captures, me_capture } => {
                let dst = dst.ok_or("Closure creation must have destination")?;
                builder.emit_instruction(MirInstruction::NewClosure {
                    dst,
                    params,
                    body: vec![], // Empty body for now
                    captures,
                    me: me_capture,
                })
            },
        }
    }
}

fn emit_call(
    builder: &mut MirBuilder,
    dst: Option<ValueId>,
    func: ValueId,
    callee: Callee,
    args: Vec<ValueId>,
    effects: EffectMask,
) -> Result<(), String> {
    builder.emit_call_with_guard(dst, func, callee, args, effects)
}

fn emit_boxcall(
    builder: &mut MirBuilder,
    dst: Option<ValueId>,
    receiver: ValueId,
    method: String,
    mut args: Vec<ValueId>,
    effects: EffectMask,
) -> Result<(), String> {
    let recv_local = crate::mir::builder::ssa::local::recv(builder, receiver);
    crate::mir::builder::ssa::local::finalize_args(builder, &mut args);
    builder.emit_box_or_plugin_call(dst, recv_local, method, None, args, effects, false)
}
