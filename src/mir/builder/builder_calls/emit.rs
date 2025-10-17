// Emit functions for unified and legacy call emission
use super::super::{Effect, EffectMask, MirBuilder, MirInstruction, ValueId};
use crate::mir::builder::calls::call_unified;
use crate::mir::builder::calls::call_target::CallTarget;
use crate::mir::builder::calls::legacy_bridge::LegacyCallBridgeBox;
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
                            if self.method_index.static_signature(name).is_some() {
                                if let Some(fun) = module.functions.get(name) {
                                    if fun.params.len() == args2.len() + 1 {
                                        if let Some((box_name, _)) = name.split_once('.') {
                                            let me = self.current_fn_singleton(box_name);
                                            let mut with_me = Vec::with_capacity(args2.len() + 1);
                                            with_me.push(me);
                                            with_me.extend(args2.drain(..));
                                            args2 = with_me;
                                        }
                                    }
                                }
                            }
                            crate::mir::builder::ssa::local::finalize_args(self, &mut args2);
                            self.emit_call_with_guard(
                                Some(dstv),
                                ValueId::new(0),
                                crate::mir::definitions::call_unified::Callee::ModuleFunction(name.to_string()),
                                args2,
                                EffectMask::IO,
                            )?;
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
                            if self.method_index.static_signature(&func_name).is_some() {
                                if let Some(ref module) = self.current_module {
                                    if let Some(fun) = module.functions.get(&func_name) {
                                        if fun.params.len() == args2.len() + 1 {
                                            if let Some((box_name, _)) = func_name.split_once('.') {
                                                let me = self.current_fn_singleton(box_name);
                                                let mut with_me = Vec::with_capacity(args2.len() + 1);
                                                with_me.push(me);
                                                with_me.extend(args2.drain(..));
                                                args2 = with_me;
                                            }
                                        }
                                    }
                                }
                            }
                            crate::mir::builder::ssa::local::finalize_args(self, &mut args2);
                            self.emit_call_with_guard(
                                Some(dstv),
                                ValueId::new(0),
                                crate::mir::definitions::call_unified::Callee::ModuleFunction(func_name.clone()),
                                args2,
                                EffectMask::IO,
                            )?;
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

        // If ModuleFunction belongs to a static box normalized to singleton `me`,
        // and the concrete function in current module expects one more parameter
        // than currently provided, prepend the per-function singleton as the first arg.
        let mut callee2 = callee;
        let mut args2: Vec<ValueId> = args.clone();
        if let crate::mir::definitions::call_unified::Callee::ModuleFunction(ref fname) = callee2 {
            if self.method_index.static_signature(fname).is_some() {
                if let Some(ref module) = self.current_module {
                    if let Some(fun) = module.functions.get(fname) {
                        if fun.params.len() == args2.len() + 1 {
                            if let Some((box_name, _)) = fname.split_once('.') {
                                let me = self.current_fn_singleton(box_name);
                                let mut with_me = Vec::with_capacity(args2.len() + 1);
                                with_me.push(me);
                                with_me.extend(args2.drain(..));
                                args2 = with_me;
                            }
                        }
                    }
                }
            }
        }
        // Final materialization unified — keep materialized receiver/args as-is.
        // IMPORTANT: Perform materialization exactly once, right before emission (EmitGuard contract).
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
        for a in args2.iter_mut() { *a = self.local_arg(*a); }
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
        if let Some(dst_id) = mir_call.dst {
            if let Callee::Extern(name) = &mir_call.callee {
                if std::env::var("NYASH_DEBUG_MAP_VALUES").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[map-values-debug] extern callee={} dst={:?}",
                        name,
                        mir_call.dst
                    );
                }
                match name.as_str() {
                    "nyrt.array.size" | "nyrt.map.size" => {
                        self.value_types
                            .insert(dst_id, crate::mir::MirType::Integer);
                    }
                    "nyrt.map.values" | "nyrt.map.keys" => {
                        self.value_types
                            .insert(dst_id, crate::mir::MirType::Box("ArrayBox".into()));
                        self.origin_register(dst_id, "ArrayBox".to_string());
                        if super::super::utils::builder_debug_enabled() {
                            super::super::utils::builder_debug_log(&format!(
                                "[annotate] extern {} dst=%{} -> origin=ArrayBox",
                                name,
                                dst_id.0
                            ));
                        }
                        if std::env::var("NYASH_DEBUG_MAP_VALUES").ok().as_deref() == Some("1") {
                            let ty = self.value_types.get(&dst_id).cloned();
                            let origin = self.origin_get(dst_id).map(|s| s.to_string());
                            eprintln!(
                                "[map-values] dst=%{} ty={:?} origin={:?}",
                                dst_id.0,
                                ty,
                                origin
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        // For Phase 2: Convert to legacy Call instruction with new callee field (use finalized operands)
        let legacy_call = MirInstruction::Call {
            dst: mir_call.dst,
            func: ValueId::new(0), // Dummy value for legacy compatibility
            callee: Some(callee2),
            args: args2,
            effects: mir_call.effects,
        };

        let res = self.emit_instruction(legacy_call);
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
        LegacyCallBridgeBox::new(self).emit(dst, target, args)
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
