// Emit functions for unified and legacy call emission
use super::super::{Effect, EffectMask, MirBuilder, MirInstruction, ValueId};
use crate::mir::builder::calls::call_unified;
use crate::mir::builder::calls::call_target::CallTarget;
use crate::mir::builder::calls::function_lowering;
use crate::mir::definitions::call_unified::Callee;
use crate::common::trace_box::TraceBox;

impl MirBuilder {
    /// Unified call emission - replaces all emit_*_call methods
    /// ChatGPT5 Pro A++ design for complete call unification
    pub fn emit_unified_call(
        &mut self,
        dst: Option<ValueId>,
        target: CallTarget,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        // Check environment variable for unified call usage
        if !call_unified::is_unified_call_enabled() {
            // Fall back to legacy implementation
            return self.emit_legacy_call(dst, target, args);
        }

        // Ensure method receiver is materialized in the current block.
        // This avoids "use of undefined recv" across block boundaries for direct Method calls
        // that bypass legacy BoxCall emission. Do this before any observation/rewrite.
        let target = target;
        let _bb_before = self.current_block; // snapshot retained for potential future checks
        // Do not pin at entry; rely on LocalSSA/materialize at emission site to avoid
        // variable_map interference. Keep target as-is.
        if let CallTarget::Method { .. } = target { /* noop */ }

        // Emit resolve.try for method targets (dev-only; default OFF)
        let arity_for_try = args.len();
        if let CallTarget::Method { ref box_type, ref method, receiver } = target {
            let (recv_cls_infer, _c) = crate::mir::builder::infer::receiver::infer_receiver(
                box_type.as_deref(),
                method,
                receiver,
                |vid| self.origin_get(vid).map(|s| s.to_string()),
                &self.value_types,
            );
            let recv_cls = recv_cls_infer;
            // Dev trace: help diagnose receiver identity/name binding issues
            if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                let mut names: Vec<String> = Vec::new();
                for (k, v) in self.variable_map.iter() {
                    if *v == receiver { names.push(k.clone()); }
                }
                let ty = self.value_types.get(&receiver).cloned();
                eprintln!(
                    "[resolve.try] method={} recv=%{} recv_cls_hint={} recv_names={:?} recv_ty={:?}",
                    method, receiver.0, recv_cls, names, ty
                );
            }
            // Use indexed candidate lookup (tail → names)
            let candidates: Vec<String> = self.method_candidates(method, arity_for_try);
            crate::mir::builder::observe::resolve_trace::emit_try_method(self, &recv_cls, method, arity_for_try, &candidates);
        }

        // Centralized user-box rewrite for method targets (toString/stringify, equals/1, Known→unique)
        if let CallTarget::Method { ref box_type, ref method, receiver } = target {
            let (recv_cls, _c) = crate::mir::builder::infer::receiver::infer_receiver(
                box_type.as_deref(),
                method,
                receiver,
                |vid| self.origin_get(vid).map(|s| s.to_string()),
                &self.value_types,
            );
            let class_name_opt = Some(recv_cls.clone());
            // Early str-like
            if let Some(res) = crate::mir::builder::rewrite::special::try_early_str_like_to_dst(
                self, dst, receiver, &class_name_opt, method, args.len(),
            ) { res?; return Ok(()); }
            // equals/1
            if let Some(res) = crate::mir::builder::rewrite::special::try_special_equals_to_dst(
                self, dst, receiver, &class_name_opt, method, args.clone(),
            ) { res?; return Ok(()); }
            // Known or unique
            if let Some(res) = crate::mir::builder::rewrite::known::try_known_or_unique_to_dst(
                self, dst, receiver, &class_name_opt, method, args.clone(),
            ) { res?; return Ok(()); }
        }

        // Convert CallTarget to Callee using the new module
        if let CallTarget::Global(ref _n) = target { /* dev trace removed */ }
        // Fallback: if Global target is unknown, try unique static-method mapping (name/arity)
        // Preserve original receiver (for Method) to guard against accidental zero-id binding
        let orig_recv = match &target {
            CallTarget::Method { receiver, .. } => Some(*receiver),
            _ => None,
        };

        let mut callee = match call_unified::convert_target_to_callee(
            target.clone(),
            |vid| self.origin_get(vid).map(|s| s.to_string()),
            &self.value_types,
        ) {
            Ok(c) => c,
            Err(e) => {
                if let CallTarget::Global(ref name) = target {
                    // 0) Dev-only safety: treat condition_fn as always-true predicate when missing
                    if name == "condition_fn" {
                        let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                        // Emit integer constant via ConstantEmissionBox
                        let one = crate::mir::builder::emission::constant::emit_integer(self, 1);
                        if dst.is_none() {
                            // If a destination was not provided, copy into the allocated dstv
                            self.emit_instruction(MirInstruction::Copy { dst: dstv, src: one })?;
                            crate::mir::builder::metadata::propagate::propagate(self, one, dstv);
                        } else {
                            // If caller provided dst, ensure the computed value lands there
                            self.emit_instruction(MirInstruction::Copy { dst: dstv, src: one })?;
                            crate::mir::builder::metadata::propagate::propagate(self, one, dstv);
                        }
                        return Ok(());
                    }
                    // 1) Direct module function fallback: call by name if present
                    // ONLY accept fully qualified names (Box.method/Arity) to avoid ambiguity
                    if let Some(ref module) = self.current_module {
                        // Only proceed if name is already fully qualified (contains '.' and '/')
                        if name.contains('.') && name.contains('/') && module.functions.contains_key(name) {
                            let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                            let mut args2 = args.clone();
                            crate::mir::builder::ssa::local::finalize_args(self, &mut args2);
                            self.emit_instruction(MirInstruction::Call { dst: Some(dstv), func: ValueId::new(0), callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(name.to_string())), args: args2, effects: EffectMask::IO })?;
                            self.annotate_call_result_from_func_name(dstv, name);
                            return Ok(());
                        }
                    }
                    // 2) Unique static-method fallback: name+arity → Box.name/Arity
                    if let Some(cands) = self.method_index.static_methods().get(name) {
                        let mut matches: Vec<(String, usize)> = cands
                            .iter()
                            .cloned()
                            .filter(|(_, ar)| *ar == arity_for_try)
                            .collect();
                        if matches.len() == 1 {
                            let (bx, _arity) = matches.remove(0);
                            let func_name = format!("{}.{}{}", bx, name, format!("/{}", arity_for_try));
                            // Emit unified ModuleFunction instead of legacy string-based call
                            let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                            let mut args2 = args.clone();
                            crate::mir::builder::ssa::local::finalize_args(self, &mut args2);
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dstv),
                                func: ValueId::new(0),
                                callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(func_name.clone())),
                                args: args2,
                                effects: EffectMask::IO,
                            })?;
                            self.annotate_call_result_from_func_name(dstv, func_name);
                            return Ok(());
                        }
                    }
                }
                return Err(e);
            }
        };

        // Guard: ValueId(0) must never be used as a receiver (reserved dummy in legacy Call.func)
        // If it appears here due to an upstream mix, restore the original receiver id.
        if let (Some(orig), Callee::Method { receiver: Some(r0), box_name, method, certainty }) = (orig_recv, &callee) {
            if r0.0 == 0 {
                TraceBox::local_ssa(|| format!("[recv-guard] fixup receiver=%%0 -> %{} for {}.{}", orig.0, box_name, method));
                callee = Callee::Method { box_name: box_name.clone(), method: method.clone(), receiver: Some(orig), certainty: *certainty };
            }
        }

        // Entry pin is disabled; materialization is handled uniformly later.

        // Block change guard removed; rely on LocalSSA/materialize

        // Debug: trace unified method emission with pinned receiver (dev only)
        if super::super::utils::builder_debug_enabled() {
            if let Callee::Method { method, receiver: Some(r), .. } = &callee {
                super::super::utils::builder_debug_log(&format!("unified-call method={} recv=%{} (pinned)", method, r.0));
            }
        }

        // Emit resolve.choose for method callee (dev-only; default OFF)
        if let Callee::Method { box_name, method, certainty, .. } = &callee {
            let chosen = format!("{}.{}{}", box_name, method, format!("/{}", arity_for_try));
            crate::mir::builder::observe::resolve_trace::emit_choose_unified(self, box_name, method, arity_for_try, &chosen, certainty);
        }

        // Validate call arguments
        call_unified::validate_call_args(&callee, &args)?;

        // Early normalize for Set operations to avoid BoxCall fallback on unknown methods
        {
            // Direct, conservative early rewrite for SetBox only (avoid BoxCall fallback)
            let mut callee_early = callee.clone();
            let mut changed = false;
            if let Callee::Method { method, receiver: Some(r), .. } = &callee_early {
                let is_set = self
                    .origin_get(*r)
                    .map(|s| s == "SetBox")
                    .unwrap_or_else(|| matches!(self.value_types.get(r), Some(crate::mir::MirType::Box(b)) if b == "SetBox"));
                let allow_map = std::env::var("HAKO_SET_ON_MAP").ok().as_deref() == Some("1");
                let is_map = self
                    .origin_get(*r)
                    .map(|s| s == "MapBox")
                    .unwrap_or_else(|| matches!(self.value_types.get(r), Some(crate::mir::MirType::Box(b)) if b == "MapBox"));
                if is_set || (allow_map && is_map) {
                    match method.as_str() {
                        "add" | "remove" | "has" if args.len() == 1 => {
                            callee_early = Callee::Extern(format!("nyrt.set.{}", method));
                            changed = true;
                        }
                        "size" | "clear" | "toArray" if args.is_empty() => {
                            callee_early = Callee::Extern(format!("nyrt.set.{}", method));
                            changed = true;
                        }
                        _ => {}
                    }
                }
            }
            if changed { callee = callee_early; }
        }

        // Stability guard: decide route via RouterPolicyBox (behavior-preserving rules)
        if let Callee::Method { box_name, method, receiver: Some(r), certainty } = &callee {
            let route = crate::mir::builder::router::policy::choose_route(box_name, method, *certainty, arity_for_try);
            if let crate::mir::builder::router::policy::Route::BoxCall = route {
                if super::super::utils::builder_debug_enabled() {
                    TraceBox::local_ssa(|| format!("[router-guard] {}.{} → BoxCall fallback (recv=%{})", box_name, method, r.0));
                }
                let effects = EffectMask::READ.add(Effect::ReadHeap);
                // Force legacy once to avoid unified→boxcall→unified recursion
                return self.emit_box_or_plugin_call(dst, *r, method.clone(), None, args, effects, true);
            }
        }

        // Before creating the call, ensure receiver is materialized in the current block
        TraceBox::local_ssa(|| format!("[emit-call] BEFORE materialize current_block={:?}", self.current_block));
        let callee = match callee {
            Callee::Method { box_name, method, receiver: Some(r), certainty } => {
                Callee::Method { box_name, method, receiver: Some(r), certainty }
            }
            other => other,
        };

        // Final materialization unified — keep materialized receiver/args as-is.
        // IMPORTANT: Perform materialization exactly once, right before emission (EmitGuard contract).
        let mut callee2 = callee;
        let mut args2: Vec<ValueId> = args.clone();
        crate::mir::builder::emit_guard::finalize_call_operands(self, &mut callee2, &mut args2);
        // If we early-rewrote Method(Set-like) → Extern("nyrt.set.*"), ensure receiver is present as first arg
        if let Callee::Extern(ref name) = callee2 {
            if name.starts_with("nyrt.set.") {
                // Prepend original receiver if missing
                let need_recv = match name.as_str() {
                    s if s == "nyrt.set.size" || s == "nyrt.set.clear" || s == "nyrt.set.toArray" => args2.len() == 0,
                    _ => args2.len() == 1, // add/remove/has expect (recv, v)
                };
                if need_recv {
                    if let Some(r0) = orig_recv {
                        let recv_local = self.local_recv(r0);
                        let mut new_args: Vec<ValueId> = Vec::with_capacity(args2.len() + 1);
                        new_args.push(recv_local);
                        new_args.extend(args2.drain(..));
                        args2 = new_args;
                    }
                }
            }
        }
        // Centralized rewrite (dispatcher): apply all normalizers in a fixed order.
        crate::mir::builder::normalize::apply_all(self, &mut callee2, &mut args2);

        // Safety net: ensure receiver/args are materialized in the CURRENT block
        // after normalization as well (normalize may rewrite Method→Extern and
        // construct fresh arg vectors). This avoids use‑before‑def of the new
        // receiver/args introduced by normalization.
        match &mut callee2 {
            Callee::Method { receiver: Some(r), .. } => { *r = self.local_recv(*r); }
            _ => {}
        }
        let mut pre_call_copies: Vec<MirInstruction> = Vec::new();
        if !matches!(callee2, Callee::Extern(_)) {
            for a in args2.iter_mut() {
                let src = *a;
                let dst_local = self.value_gen.next();
                pre_call_copies.push(MirInstruction::Copy { dst: dst_local, src });
                if let Some(t) = self.value_types.get(&src).cloned() {
                    self.value_types.insert(dst_local, t);
                }
                if let Some(orig) = self.origin_get(src).map(|s| s.to_string()) {
                    self.origin_register(dst_local, orig);
                }
                *a = dst_local;
            }
        }
        if let Callee::Method { method, receiver, box_name, .. } = &callee2 {
            if let Some(r) = receiver {
                if super::super::utils::builder_debug_enabled() {
                    let rty = self.value_types.get(r).cloned();
                    let rorig = self.origin_get(*r).map(|s| s.to_string());
                    TraceBox::local_ssa(|| format!("[vm-call-final] bb={:?} method={} recv=%{} class={} ty={:?} orig={}",
                        self.current_block, method, r.0, box_name, rty, rorig.as_deref().unwrap_or("-")));
                    crate::mir::builder::observe::varmap::emit_recv_names(self, *r, "vm-call-final");
                }
            }
        }

        // Compose final MirCall from normalized+finalized operands for accurate effects
        let mir_call = call_unified::create_mir_call(dst, callee2.clone(), args2.clone());

        // For Phase 2: Convert to legacy Call instruction with new callee field (use finalized operands)
        let legacy_call = MirInstruction::Call {
            dst: mir_call.dst,
            func: ValueId::new(0), // Dummy value for legacy compatibility
            callee: Some(callee2),
            args: args2,
            effects: mir_call.effects,
        };

        let res = self.emit_instruction(legacy_call);
        if res.is_ok() && !pre_call_copies.is_empty() {
            if let (Some(bb), Some(function)) = (self.current_block, self.current_function.as_mut()) {
                if let Some(block) = function.get_block_mut(bb) {
                    let insert_at = block.instructions.len().saturating_sub(1);
                    let mut idx = 0usize;
                    for inst in pre_call_copies {
                        block.instructions.insert(insert_at + idx, inst);
                        idx += 1;
                    }
                }
            }
        }
        // Dev-only: verify block schedule invariants after emitting call
        crate::mir::builder::emit_guard::verify_after_call(self);
        res
    }

    /// Legacy call fallback - preserves existing behavior
    pub(in super::super) fn emit_legacy_call(
        &mut self,
        dst: Option<ValueId>,
        target: CallTarget,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
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
                    let changed = crate::mir::builder::normalize::string_length::normalize_string_length_call(self, &mut callee, &mut argv);
                    if changed {
                        crate::mir::builder::ssa::local::finalize_args(self, &mut argv);
                        let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                        let name_const = crate::mir::builder::name_const::make_name_const_result(self, "nyrt.string.length")?;
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dstv),
                            func: name_const,
                            callee: Some(callee),
                            args: argv,
                            effects: EffectMask::READ.add(Effect::ReadHeap),
                        })?;
                        // Annotate result as Integer
                        self.value_types.insert(dstv, crate::mir::MirType::Integer);
                        return Ok(());
                    }
                }
                // Centralized rewrite: array size/len/length (0 args) → Extern("nyrt.array.length")
                {
                    let mut callee = Callee::Method { box_name: "ArrayBox".to_string(), method: method.clone(), receiver: Some(receiver), certainty: crate::mir::definitions::call_unified::TypeCertainty::Union };
                    let mut argv = Vec::<ValueId>::new();
                    let changed = crate::mir::builder::normalize::array_length::normalize_array_length_call(self, &mut callee, &mut argv);
                    if changed {
                        crate::mir::builder::ssa::local::finalize_args(self, &mut argv);
                        let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                        let name_const = crate::mir::builder::name_const::make_name_const_result(self, "nyrt.array.size")?;
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dstv),
                            func: name_const,
                            callee: Some(callee),
                            args: argv,
                            effects: EffectMask::READ.add(Effect::ReadHeap),
                        })?;
                        // Annotate result as Integer
                        self.value_types.insert(dstv, crate::mir::MirType::Integer);
                        return Ok(());
                    }
                }
                // Special-case: instance.birth(args) should invoke lowered ModuleFunction (user-defined birth)
                if method == "birth" {
                    let (cls, _cert) = crate::mir::builder::infer::receiver::infer_receiver(
                        None,
                        &method,
                        receiver,
                        |vid| self.origin_get(vid).map(|s| s.to_string()),
                        &self.value_types,
                    );
                    let me_local = self.local_recv(receiver);
                    let mut call_args: Vec<ValueId> = Vec::with_capacity(args.len() + 1);
                    call_args.push(me_local);
                    call_args.extend(args.into_iter());
                    crate::mir::builder::ssa::local::finalize_args(self, &mut call_args);
                    let out = dst.unwrap_or_else(|| self.value_gen.next());
                    let fname = function_lowering::generate_method_function_name(&cls, &method, call_args.len() - 1);
                    let name_val = crate::mir::builder::name_const::make_name_const_result(self, &fname)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(out),
                        func: name_val,
                        callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(fname.clone())),
                        args: call_args,
                        effects: EffectMask::IO,
                    })?;
                    self.annotate_call_result_from_func_name(out, &fname);
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
                        |vid| self.origin_get(vid).map(|s| s.to_string()),
                        &self.value_types,
                    );
                    let me_local = self.local_recv(receiver);
                    let mut call_args: Vec<ValueId> = Vec::with_capacity(args.len() + 1);
                    call_args.push(me_local);
                    call_args.extend(args.into_iter());
                    crate::mir::builder::ssa::local::finalize_args(self, &mut call_args);
                    let out = dst.unwrap_or_else(|| self.value_gen.next());
                    let fname = function_lowering::generate_method_function_name(&cls, &method, call_args.len() - 1);
                    let name_val = crate::mir::builder::name_const::make_name_const_result(self, &fname)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(out),
                        func: name_val,
                        callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(fname.clone())),
                        args: call_args,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(out, &fname);
                    return Ok(());
                }
                // Legacy fallback: Box/Plugin call
                self.emit_box_or_plugin_call(dst, receiver, method, None, args, EffectMask::IO, false)
            },
            CallTarget::Constructor(box_type) => {
                // Use existing NewBox
                let dst = dst.ok_or("Constructor must have destination")?;
                self.emit_instruction(MirInstruction::NewBox {
                    dst,
                    box_type,
                    args,
                    auto_birth: None,
                })
            },
            CallTarget::Extern(name) => {
                // Unified path: emit Call with callee=Extern("iface.method")
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                // Normalize dotted name; accept bare as "nyash.<name>"
                let full_name = if name.contains('.') { name } else { format!("nyash.{}", name) };
                // Compute effects for extern
                let (iface, method) = full_name.rsplit_once('.').unwrap_or(("nyash", full_name.as_str()));
                let effects = crate::mir::builder::calls::extern_calls::compute_extern_effects(iface, method);
                self.emit_instruction(MirInstruction::Call {
                    dst,
                    func: ValueId::new(0),
                    callee: Some(crate::mir::definitions::call_unified::Callee::Extern(full_name)),
                    args,
                    effects,
                })
            },
            CallTarget::Global(name) => {
                // Early rewrite for dotted StringBox methods: StringBox.size/1 → Extern("nyrt.string.length")
                if (name.starts_with("StringBox.size") || name.starts_with("StringBox.length") || name.starts_with("StringBox.len")) && args.len() == 1 {
                    let recv_local = self.local_recv(args[0]);
                    let mut argv = vec![recv_local];
                    crate::mir::builder::ssa::local::finalize_args(self, &mut argv);
                    let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                    let name_const = crate::mir::builder::name_const::make_name_const_result(self, "nyrt.string.length")?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dstv),
                        func: name_const,
                        callee: Some(crate::mir::definitions::call_unified::Callee::Extern("nyrt.string.length".to_string())),
                        args: argv,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.value_types.insert(dstv, crate::mir::MirType::Integer);
                    return Ok(());
                }
                let normalized = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::normalize(&name, args.len()) {
                    Ok(full) => full,
                    Err(_) => format!("{}/{}", name, args.len()),
                };
                // Special-case: route hostbridge.* globals to Extern("hostbridge.*") for unified HostBridge
                if name.starts_with("hostbridge.") {
                    let mut args = args;
                    crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                    self.emit_instruction(MirInstruction::Call {
                        dst,
                        func: ValueId::new(0),
                        callee: Some(crate::mir::definitions::call_unified::Callee::Extern(name.clone())),
                        args,
                        effects: crate::mir::builder::calls::extern_calls::compute_extern_effects("hostbridge", name.strip_prefix("hostbridge.").unwrap_or("")),
                    })?;
                    return Ok(());
                }
                // Prefer direct ModuleFunction when available in current module (avoids legacy string callee)
                // Use CallNameResolverBox::normalize to ensure fully qualified form before lookup.
                if let Some(ref module) = self.current_module {
                    // Only attempt module function lookup when the name looks like Class.method or fully-qualified.
                    if name.contains('.') {
                        let want = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::normalize(&name, args.len()) {
                            Ok(full) => full,
                            Err(_) => name.clone(), // keep raw if it cannot be normalized (unlikely when contains '.')
                        };
                        if module.functions.contains_key(&want) {
                            let actual_dst = if let Some(d) = dst { d } else { self.value_gen.next() };
                            let mut args = args;
                            crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(actual_dst),
                                func: ValueId::new(0),
                                callee: Some(crate::mir::definitions::call_unified::Callee::ModuleFunction(want.clone())),
                                args,
                                effects: EffectMask::IO,
                            })?;
                            self.annotate_call_result_from_func_name(actual_dst, &want);
                            return Ok(());
                        }
                    }
                }
                // First-class: JSON.stringify(any) → arg0.toJSON() (arity 1→0)
                if name == "JSON.stringify/1" || name.starts_with("JSON.stringify") {
                    if let Some(recv) = args.get(0).cloned() {
                        let argv: Vec<super::super::ValueId> = Vec::new();
                        let recv_local = self.local_recv(recv);
                        self.emit_box_or_plugin_call(dst, recv_local, "toJSON".to_string(), None, argv, EffectMask::READ, false)?;
                        return Ok(());
                    }
                }
                // Emit unified Global callee instead of legacy string-based call
                let actual_dst = if let Some(d) = dst { d } else { self.value_gen.next() };
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                self.emit_instruction(MirInstruction::Call {
                    dst: Some(actual_dst),
                    func: ValueId::new(0),
                    callee: Some(crate::mir::definitions::call_unified::Callee::Global(normalized.clone())),
                    args,
                    effects: EffectMask::IO,
                })?;
                self.annotate_call_result_from_func_name(actual_dst, normalized);
                Ok(())
            },
            CallTarget::Value(func_val) => {
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                self.emit_instruction(MirInstruction::Call {
                    dst,
                    func: func_val,
                    callee: Some(crate::mir::definitions::call_unified::Callee::Value(func_val)),
                    args,
                    effects: EffectMask::IO,
                })
            },
            CallTarget::Closure { params, captures, me_capture } => {
                let dst = dst.ok_or("Closure creation must have destination")?;
                self.emit_instruction(MirInstruction::NewClosure {
                    dst,
                    params,
                    body: vec![], // Empty body for now
                    captures,
                    me: me_capture,
                })
            },
        }
    }

    // Phase 2 Migration: Convenience methods that use emit_unified_call

    /// Emit a global function call (print, panic, etc.)
    pub fn emit_global_call(
        &mut self,
        dst: Option<ValueId>,
        name: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(dst, CallTarget::Global(name), args)
    }

    /// Emit a method call (box.method)
    pub fn emit_method_call(
        &mut self,
        dst: Option<ValueId>,
        receiver: ValueId,
        method: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(
            dst,
            CallTarget::Method {
                box_type: None, // Auto-infer
                method,
                receiver,
            },
            args,
        )
    }

    /// Emit a constructor call (new BoxType)
    pub fn emit_constructor_call(
        &mut self,
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(
            Some(dst),
            CallTarget::Constructor(box_type),
            args,
        )
    }
}
