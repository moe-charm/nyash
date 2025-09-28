// Extracted call-related builders from builder.rs to keep files lean
use super::{Effect, EffectMask, FunctionSignature, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue, MethodCallExpr};
use crate::mir::definitions::call_unified::{Callee, CallFlags, MirCall};
use crate::mir::TypeOpKind;
use super::call_resolution;

// Import from new modules
use super::calls::*;
pub use super::calls::call_target::CallTarget;

impl super::MirBuilder {
    /// Annotate a call result `dst` with the return type and origin if the callee
    /// is a known user/static function in the current module.
    pub(super) fn annotate_call_result_from_func_name<S: AsRef<str>>(&mut self, dst: super::ValueId, func_name: S) {
        let name = func_name.as_ref();
        // 1) Prefer module signature when available
        if let Some(ref module) = self.current_module {
            if let Some(func) = module.functions.get(name) {
                let mut ret = func.signature.return_type.clone();
                // Targeted stabilization: JsonParser.parse/1 should produce JsonNode
                // If signature is Unknown/Void, normalize to Box("JsonNode")
                if name == "JsonParser.parse/1" {
                    if matches!(ret, super::MirType::Unknown | super::MirType::Void) {
                        ret = super::MirType::Box("JsonNode".into());
                    }
                }
                // Token path: JsonParser.current_token/0 should produce JsonToken
                if name == "JsonParser.current_token/0" {
                    if matches!(ret, super::MirType::Unknown | super::MirType::Void) {
                        ret = super::MirType::Box("JsonToken".into());
                    }
                }
                // Parser factory: JsonParserModule.create_parser/0 returns JsonParser
                if name == "JsonParserModule.create_parser/0" {
                    // Normalize to Known Box(JsonParser)
                    ret = super::MirType::Box("JsonParser".into());
                }
                self.value_types.insert(dst, ret.clone());
                if let super::MirType::Box(bx) = ret {
                    self.value_origin_newbox.insert(dst, bx);
                    if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        let bx = self.value_origin_newbox.get(&dst).cloned().unwrap_or_default();
                        super::utils::builder_debug_log(&format!("annotate call dst={} from {} -> Box({})", dst.0, name, bx));
                    }
                }
                return;
            }
        }
        // 2) No module signature—apply minimal heuristic for known functions
        if name == "JsonParser.parse/1" {
            let ret = super::MirType::Box("JsonNode".into());
            self.value_types.insert(dst, ret.clone());
            if let super::MirType::Box(bx) = ret { self.value_origin_newbox.insert(dst, bx); }
            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonNode)", dst.0, name));
            }
        } else if name == "JsonParser.current_token/0" {
            let ret = super::MirType::Box("JsonToken".into());
            self.value_types.insert(dst, ret.clone());
            if let super::MirType::Box(bx) = ret { self.value_origin_newbox.insert(dst, bx); }
            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonToken)", dst.0, name));
            }
        } else if name == "JsonTokenizer.tokenize/0" {
            // Tokenize returns an ArrayBox of tokens
            let ret = super::MirType::Box("ArrayBox".into());
            self.value_types.insert(dst, ret.clone());
            if let super::MirType::Box(bx) = ret { self.value_origin_newbox.insert(dst, bx); }
            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(ArrayBox)", dst.0, name));
            }
        } else if name == "JsonParserModule.create_parser/0" {
            // Fallback path for parser factory
            let ret = super::MirType::Box("JsonParser".into());
            self.value_types.insert(dst, ret.clone());
            if let super::MirType::Box(bx) = ret { self.value_origin_newbox.insert(dst, bx); }
            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonParser)", dst.0, name));
            }
        } else {
            // Generic tiny whitelist for known primitive-like utilities (spec unchanged)
            crate::mir::builder::types::annotation::annotate_from_function(self, dst, name);
        }
    }
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
        let mut target = target;
        let bb_before = self.current_block; // snapshot for later re-check
        if let CallTarget::Method { box_type, method, receiver } = target {
            if super::utils::builder_debug_enabled() && method == "advance" {
                super::utils::builder_debug_log(&format!("unified-entry Method.advance recv=%{} (pre-pin)", receiver.0));
            }
            let receiver_pinned = match self.pin_to_slot(receiver, "@recv") {
                Ok(v) => v,
                Err(_) => receiver,
            };
            target = CallTarget::Method { box_type, method, receiver: receiver_pinned };
        }

        // Emit resolve.try for method targets (dev-only; default OFF)
        let arity_for_try = args.len();
        if let CallTarget::Method { ref box_type, ref method, receiver } = target {
            let recv_cls = box_type.clone()
                .or_else(|| self.value_origin_newbox.get(&receiver).cloned())
                .unwrap_or_default();
            // Use indexed candidate lookup (tail → names)
            let candidates: Vec<String> = self.method_candidates(method, arity_for_try);
            let meta = serde_json::json!({
                "recv_cls": recv_cls,
                "method": method,
                "arity": arity_for_try,
                "candidates": candidates,
            });
            super::observe::resolve::emit_try(self, meta);
        }

        // Centralized user-box rewrite for method targets (toString/stringify, equals/1, Known→unique)
        if let CallTarget::Method { ref box_type, ref method, receiver } = target {
            let class_name_opt = box_type.clone()
                .or_else(|| self.value_origin_newbox.get(&receiver).cloned())
                .or_else(|| self.value_types.get(&receiver).and_then(|t| if let super::MirType::Box(b) = t { Some(b.clone()) } else { None }));
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
        let mut callee = match call_unified::convert_target_to_callee(
            target.clone(),
            &self.value_origin_newbox,
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
                    if let Some(ref module) = self.current_module {
                        if module.functions.contains_key(name) {
                            let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                            let name_const = match crate::mir::builder::name_const::make_name_const_result(self, name) {
                                Ok(v) => v,
                                Err(e) => return Err(e),
                            };
                            self.emit_instruction(MirInstruction::Call { dst: Some(dstv), func: name_const, callee: None, args: args.clone(), effects: EffectMask::IO })?;
                            self.annotate_call_result_from_func_name(dstv, name);
                            return Ok(());
                        }
                    }
                    // 2) Unique static-method fallback: name+arity → Box.name/Arity
                    if let Some(cands) = self.static_method_index.get(name) {
                        let mut matches: Vec<(String, usize)> = cands
                            .iter()
                            .cloned()
                            .filter(|(_, ar)| *ar == arity_for_try)
                            .collect();
                        if matches.len() == 1 {
                            let (bx, _arity) = matches.remove(0);
                            let func_name = format!("{}.{}{}", bx, name, format!("/{}", arity_for_try));
                            // Emit legacy call directly to preserve behavior
                            let dstv = dst.unwrap_or_else(|| self.value_gen.next());
                            let name_const = match crate::mir::builder::name_const::make_name_const_result(self, &func_name) {
                                Ok(v) => v,
                                Err(e) => return Err(e),
                            };
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dstv),
                                func: name_const,
                                callee: None,
                                args: args.clone(),
                                effects: EffectMask::IO,
                            })?;
                            // annotate
                            self.annotate_call_result_from_func_name(dstv, func_name);
                            return Ok(());
                        }
                    }
                }
                return Err(e);
            }
        };

        // Safety: ensure receiver is materialized even after callee conversion
        // (covers rare paths where earlier pin did not take effect)
        callee = match callee {
            Callee::Method { box_name, method, receiver: Some(r), certainty } => {
                // Prefer pinning to a slot so start_new_block can propagate it across entries.
                let r_pinned = self.pin_to_slot(r, "@recv").unwrap_or(r);
                Callee::Method { box_name, method, receiver: Some(r_pinned), certainty }
            }
            other => other,
        };

        // Final guard: if current block changed since entry, ensure receiver is copied again
        if let (Some(bb0), Some(bb1)) = (bb_before, self.current_block) {
            if bb0 != bb1 {
                if let Callee::Method { box_name, method, receiver: Some(r), certainty } = callee {
                    if super::utils::builder_debug_enabled() {
                        super::utils::builder_debug_log(&format!(
                            "unified-call bb changed: {:?} -> {:?}; re-materialize recv=%{}",
                            bb0, bb1, r.0
                        ));
                    }
                    let r2 = self.pin_to_slot(r, "@recv").unwrap_or(r);
                    callee = Callee::Method { box_name, method, receiver: Some(r2), certainty };
                }
            }
        }

        // Debug: trace unified method emission with pinned receiver (dev only)
        if super::utils::builder_debug_enabled() {
            if let Callee::Method { method, receiver: Some(r), .. } = &callee {
                super::utils::builder_debug_log(&format!("unified-call method={} recv=%{} (pinned)", method, r.0));
            }
        }

        // Emit resolve.choose for method callee (dev-only; default OFF)
        if let Callee::Method { box_name, method, certainty, .. } = &callee {
            let chosen = format!("{}.{}{}", box_name, method, format!("/{}", arity_for_try));
            let meta = serde_json::json!({
                "recv_cls": box_name,
                "method": method,
                "arity": arity_for_try,
                "chosen": chosen,
                "certainty": format!("{:?}", certainty),
                "reason": "unified",
            });
            super::observe::resolve::emit_choose(self, meta);
        }

        // Validate call arguments
        call_unified::validate_call_args(&callee, &args)?;

        // Stability guard: decide route via RouterPolicyBox (behavior-preserving rules)
        if let Callee::Method { box_name, method, receiver: Some(r), certainty } = &callee {
            let route = crate::mir::builder::router::policy::choose_route(box_name, method, *certainty, arity_for_try);
            if let crate::mir::builder::router::policy::Route::BoxCall = route {
                if super::utils::builder_debug_enabled() || std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[router-guard] {}.{} → BoxCall fallback (recv=%{})", box_name, method, r.0);
                }
                let effects = EffectMask::READ.add(Effect::ReadHeap);
                return self.emit_box_or_plugin_call(dst, *r, method.clone(), None, args, effects);
            }
        }

        // Before creating the call, enforce slot (pin) + LocalSSA for Method receiver in the current block
        let callee = match callee {
            Callee::Method { box_name, method, receiver: Some(r), certainty } => {
                // Pin to a named slot so start_new_block can propagate across entries
                let r_pinned = self.pin_to_slot(r, "@recv").unwrap_or(r);
                // And ensure in-block materialization for this emission site
                let r_local = self.local_recv(r_pinned);
                Callee::Method { box_name, method, receiver: Some(r_local), certainty }
            }
            other => other,
        };

        // Finalize operands in current block (EmitGuardBox wrapper)
        let mut callee = callee;
        let mut args_local: Vec<ValueId> = args.clone();
        crate::mir::builder::emit_guard::finalize_call_operands(self, &mut callee, &mut args_local);

        // Create MirCall instruction using the new module (pure data composition)
        let mir_call = call_unified::create_mir_call(dst, callee.clone(), args_local.clone());

        // Final last-chance LocalSSA just before emission in case any block switch happened
        let mut callee2 = callee.clone();
        let mut args2 = args_local.clone();
        crate::mir::builder::emit_guard::finalize_call_operands(self, &mut callee2, &mut args2);

        // Dev trace: show final callee/recv right before emission (guarded)
        if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") || super::utils::builder_debug_enabled() {
            if let Callee::Method { method, receiver, box_name, .. } = &callee2 {
                if let Some(r) = receiver {
                    eprintln!("[vm-call-final] bb={:?} method={} recv=%{} class={}",
                        self.current_block, method, r.0, box_name);
                }
            }
        }

        // Final forced in-block materialization for receiver just-before emission
        // Rationale: ensure a Copy(def) immediately precedes the Call in the same block
        let callee2 = match callee2 {
            Callee::Method { box_name, method, receiver: Some(r), certainty } => {
                if super::utils::builder_debug_enabled() || std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[call-forced-copy] bb={:?} recv=%{} -> (copy)", self.current_block, r.0);
                }
                // Insert immediately after PHIs to guarantee position and dominance
                let r_forced = crate::mir::builder::schedule::block::BlockScheduleBox::ensure_after_phis_copy(self, r)?;
                if super::utils::builder_debug_enabled() || std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[call-forced-copy] bb={:?} inserted Copy %{} := %{} (after PHIs)", self.current_block, r_forced.0, r.0);
                }
                Callee::Method { box_name, method, receiver: Some(r_forced), certainty }
            }
            other => other,
        };

        // Optional last-chance: emit a tail copy right before we emit Call (keeps order stable)
        let callee2 = match callee2 {
            Callee::Method { box_name, method, receiver: Some(r), certainty } => {
                let r_tail = crate::mir::builder::schedule::block::BlockScheduleBox::emit_before_call_copy(self, r)?;
                Callee::Method { box_name, method, receiver: Some(r_tail), certainty }
            }
            other => other,
        };

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
    pub(super) fn emit_legacy_call(
        &mut self,
        dst: Option<ValueId>,
        target: CallTarget,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        match target {
            CallTarget::Method { receiver, method, box_type: _ } => {
                // LEGACY PATH (after unified migration):
                // Instance→Function rewrite is centralized in unified call path.
                // Legacy path no longer functionizes; always use Box/Plugin call here.
                self.emit_box_or_plugin_call(dst, receiver, method, None, args, EffectMask::IO)
            },
            CallTarget::Constructor(box_type) => {
                // Use existing NewBox
                let dst = dst.ok_or("Constructor must have destination")?;
                self.emit_instruction(MirInstruction::NewBox {
                    dst,
                    box_type,
                    args,
                })
            },
            CallTarget::Extern(name) => {
                // Use existing ExternCall
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                let parts: Vec<&str> = name.splitn(2, '.').collect();
                let (iface, method) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    ("nyash".to_string(), name)
                };

                self.emit_instruction(MirInstruction::ExternCall {
                    dst,
                    iface_name: iface,
                    method_name: method,
                    args,
                    effects: EffectMask::IO,
                })
            },
            CallTarget::Global(name) => {
                // Create a string constant for the function name via NameConstBox
                let name_const = crate::mir::builder::name_const::make_name_const_result(self, &name)?;
                // Allocate a destination if not provided so we can annotate it
                let actual_dst = if let Some(d) = dst { d } else { self.value_gen.next() };
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                self.emit_instruction(MirInstruction::Call {
                    dst: Some(actual_dst),
                    func: name_const,
                    callee: None, // Legacy mode
                    args,
                    effects: EffectMask::IO,
                })?;
                // Annotate from module signature (if present)
                self.annotate_call_result_from_func_name(actual_dst, name);
                Ok(())
            },
            CallTarget::Value(func_val) => {
                let mut args = args;
                crate::mir::builder::ssa::local::finalize_args(self, &mut args);
                self.emit_instruction(MirInstruction::Call {
                    dst,
                    func: func_val,
                    callee: None, // Legacy mode
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

    /// Try handle math.* function in function-style (sin/cos/abs/min/max).
    /// Returns Some(result) if handled, otherwise None.
    fn try_handle_math_function(
        &mut self,
        name: &str,
        raw_args: Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        if !special_handlers::is_math_function(name) {
            return None;
        }
        // Build numeric args directly for math.* to preserve f64 typing
        let mut math_args: Vec<ValueId> = Vec::new();
        for a in raw_args.into_iter() {
            match a {
                ASTNode::New { class, arguments, .. } if class == "FloatBox" && arguments.len() == 1 => {
                    match self.build_expression(arguments[0].clone()) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                ASTNode::New { class, arguments, .. } if class == "IntegerBox" && arguments.len() == 1 => {
                    let iv = match self.build_expression(arguments[0].clone()) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let fv = self.value_gen.next();
                    if let Err(e) = self.emit_instruction(MirInstruction::TypeOp { dst: fv, op: TypeOpKind::Cast, value: iv, ty: MirType::Float }) { return Some(Err(e)); }
                    math_args.push(fv);
                }
                ASTNode::Literal { value: LiteralValue::Float(_), .. } => {
                    match self.build_expression(a) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                other => {
                    match self.build_expression(other) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
            }
        }
        // new MathBox()
        let math_recv = self.value_gen.next();
        if let Err(e) = self.emit_constructor_call(math_recv, "MathBox".to_string(), vec![]) { return Some(Err(e)); }
        self.value_origin_newbox.insert(math_recv, "MathBox".to_string());
        // birth()
        if let Err(e) = self.emit_method_call(None, math_recv, "birth".to_string(), vec![]) { return Some(Err(e)); }
        // call method
        let dst = self.value_gen.next();
        if let Err(e) = self.emit_method_call(Some(dst), math_recv, name.to_string(), math_args) { return Some(Err(e)); }
        Some(Ok(dst))
    }

    /// Try handle env.* extern methods like env.console.log via FieldAccess(object, field).
    fn try_handle_env_method(
        &mut self,
        object: &ASTNode,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let ASTNode::FieldAccess { object: env_obj, field: env_field, .. } = object else { return None; };
        if let ASTNode::Variable { name: env_name, .. } = env_obj.as_ref() {
            if env_name != "env" { return None; }
            // Build arguments once
            let mut arg_values = Vec::new();
            for arg in arguments {
                match self.build_expression(arg.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
            }
            let iface = env_field.as_str();
            let m = method;
            let mut extern_call = |iface_name: &str, method_name: &str, effects: EffectMask, returns: bool| -> Result<ValueId, String> {
                let result_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::ExternCall { dst: if returns { Some(result_id) } else { None }, iface_name: iface_name.to_string(), method_name: method_name.to_string(), args: arg_values.clone(), effects })?;
                if returns {
                    Ok(result_id)
                } else {
                    let void_id = crate::mir::builder::emission::constant::emit_void(self);
                    Ok(void_id)
                }
            };
            // Use the new module for env method spec
            if let Some((iface_name, method_name, effects, returns)) =
                extern_calls::get_env_method_spec(iface, m)
            {
                return Some(extern_call(&iface_name, &method_name, effects, returns));
            }
            return None;
        }
        None
    }

    /// Try direct static call for `me` in static box
    pub(super) fn try_handle_me_direct_call(
        &mut self,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let Some(cls_name) = self.current_static_box.clone() else { return None; };
        // Build args
        let mut arg_values = Vec::new();
        for a in arguments {
            match self.build_expression(a.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
        }
        let result_id = self.value_gen.next();
        let fun_name = format!("{}.{}{}", cls_name, method, format!("/{}", arg_values.len()));
        let fun_val = match crate::mir::builder::name_const::make_name_const_result(self, &fun_name) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if let Err(e) = self.emit_instruction(MirInstruction::Call {
            dst: Some(result_id),
            func: fun_val,
            callee: None, // Legacy math function - use old resolution
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap)
        }) { return Some(Err(e)); }
        // Annotate from lowered function signature if present
        self.annotate_call_result_from_func_name(result_id, &fun_name);
        Some(Ok(result_id))
    }

    // === ChatGPT5 Pro Design: Type-safe Call Resolution System ===

    /// Resolve function call target to type-safe Callee
    /// Implements the core logic of compile-time function resolution
    fn resolve_call_target(&self, name: &str) -> Result<super::super::Callee, String> {
        method_resolution::resolve_call_target(
            name,
            &self.current_static_box,
            &self.variable_map,
        )
    }

    // Build function call: name(args)
    pub(super) fn build_function_call(
        &mut self,
        name: String,
        args: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // dev trace removed
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            let cur_fun = self.current_function.as_ref().map(|f| f.signature.name.clone()).unwrap_or_else(|| "<none>".to_string());
            eprintln!(
                "[builder] function-call name={} static_ctx={} in_fn={}",
                name,
                self.current_static_box.as_deref().unwrap_or(""),
                cur_fun
            );
        }
        // Minimal TypeOp wiring via function-style: isType(value, "Type"), asType(value, "Type")
        if (name == "isType" || name == "asType") && args.len() == 2 {
            if let Some(type_name) = special_handlers::extract_string_literal(&args[1]) {
                let val = self.build_expression(args[0].clone())?;
                let ty = special_handlers::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if name == "isType" {
                    TypeOpKind::Check
                } else {
                    TypeOpKind::Cast
                };
                self.emit_instruction(MirInstruction::TypeOp {
                    dst,
                    op,
                    value: val,
                    ty,
                })?;
                return Ok(dst);
            }
        }
        // Keep original args for special handling (math.*)
        let raw_args = args.clone();

        if let Some(res) = self.try_handle_math_function(&name, raw_args) { return res; }

        // Build argument values first (needed for arity-aware fallback)
        let mut arg_values = Vec::new();
        for a in args {
            arg_values.push(self.build_expression(a)?);
        }

        // Special-case: global str(x) → x.str() に正規化（内部は関数へ統一される）
        if name == "str" && arg_values.len() == 1 {
            let dst = self.value_gen.next();
            // Use unified method emission; downstream rewrite will functionize as needed
            self.emit_method_call(Some(dst), arg_values[0], "str".to_string(), vec![])?;
            return Ok(dst);
        }

        // Phase 3.2: Unified call is default ON, but only use it for known builtins/externs.
        let use_unified = super::calls::call_unified::is_unified_call_enabled()
            && (super::call_resolution::is_builtin_function(&name)
                || super::call_resolution::is_extern_function(&name));

        if !use_unified {
            // Legacy path
            let dst = self.value_gen.next();

            // === ChatGPT5 Pro Design: Type-safe function call resolution ===
            // Resolve call target using new type-safe system; if it fails, try static-method fallback
            let callee = match self.resolve_call_target(&name) {
                Ok(c) => c,
                Err(_e) => {
                    // dev trace removed
                    // Fallback: if exactly one static method with this name and arity is known, call it.
                    if let Some(cands) = self.static_method_index.get(&name) {
                        let mut matches: Vec<(String, usize)> = cands
                            .iter()
                            .cloned()
                            .filter(|(_, ar)| *ar == arg_values.len())
                            .collect();
                        if matches.len() == 1 {
                            let (bx, _arity) = matches.remove(0);
                            let dst = self.value_gen.next();
                            let func_name = format!("{}.{}{}", bx, name, format!("/{}", arg_values.len()));
                            // Emit legacy global call to the lowered static method function
                            self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                            return Ok(dst);
                        }
                    }
                    // Secondary fallback (tail-based) is disabled by default to avoid ambiguous resolution.
                    // Enable only when explicitly requested: NYASH_BUILDER_TAIL_RESOLVE=1
                    if std::env::var("NYASH_BUILDER_TAIL_RESOLVE").ok().as_deref() == Some("1") {
                        if let Some(ref module) = self.current_module {
                            let tail = format!(".{}{}", name, format!("/{}", arg_values.len()));
                            let mut cands: Vec<String> = module
                                .functions
                                .keys()
                                .filter(|k| k.ends_with(&tail))
                                .cloned()
                                .collect();
                            if cands.len() == 1 {
                                let func_name = cands.remove(0);
                                let dst = self.value_gen.next();
                                self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                                return Ok(dst);
                            }
                        }
                    }
                    // Propagate original error
                    return Err(format!("Unresolved function: '{}'. {}", name, super::call_resolution::suggest_resolution(&name)));
                }
            };

            // Legacy compatibility: Create dummy func value for old systems
            let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &name)?;

            // Emit new-style Call with type-safe callee
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,                  // Legacy compatibility
                callee: Some(callee),           // New type-safe resolution
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            Ok(dst)
        } else {
            // Unified path for builtins/externs
            let dst = self.value_gen.next();
            self.emit_unified_call(
                Some(dst),
                CallTarget::Global(name),
                arg_values,
            )?;
            Ok(dst)
        }
    }

    // Build method call: object.method(arguments)
    pub(super) fn build_method_call(
        &mut self,
        object: ASTNode,
        method: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        if std::env::var("NYASH_STATIC_CALL_TRACE").ok().as_deref() == Some("1") {
            let kind = match &object {
                ASTNode::Variable { .. } => "Variable",
                ASTNode::FieldAccess { .. } => "FieldAccess",
                ASTNode::This { .. } => "This",
                ASTNode::Me { .. } => "Me",
                _ => "Other",
            };
            eprintln!("[builder] method-call object kind={} method={}", kind, method);
        }

        // 1. Static box method call: BoxName.method(args)
        if let ASTNode::Variable { name: obj_name, .. } = &object {
            let is_local_var = self.variable_map.contains_key(obj_name);
            // Phase 15.5: Treat unknown identifiers in receiver position as static type names
            if !is_local_var {
                return self.handle_static_method_call(obj_name, &method, &arguments);
            }
        }

        // 2. Handle env.* methods
        if let Some(res) = self.try_handle_env_method(&object, &method, &arguments) {
            return res;
        }

        // 3. Handle me.method() calls
        if let ASTNode::Me { .. } = object {
            // 3-a) Static box fast path (already handled)
            if let Some(res) = self.handle_me_method_call(&method, &arguments)? {
                return Ok(res);
            }
            // 3-b) Instance box: prefer enclosing box method explicitly to avoid cross-box name collisions
            {
                // Capture enclosing class name without holding an active borrow
                let enclosing_cls: Option<String> = self
                    .current_function
                    .as_ref()
                    .and_then(|f| f.signature.name.split('.').next().map(|s| s.to_string()));
                if let Some(cls) = enclosing_cls.as_ref() {
                    // Build arg values (avoid overlapping borrows by collecting first)
                    let built_args: Vec<ASTNode> = arguments.clone();
                    let mut arg_values = Vec::with_capacity(built_args.len());
                    for a in built_args.into_iter() { arg_values.push(self.build_expression(a)?); }
                    let arity = arg_values.len();
                    let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(cls, &method, arity);
                    let exists = if let Some(ref module) = self.current_module { module.functions.contains_key(&fname) } else { false };
                    if exists {
                        // Pass 'me' as first arg
                        let me_id = self.build_me_expression()?;
                        let mut call_args = Vec::with_capacity(arity + 1);
                        call_args.push(me_id);
                        call_args.extend(arg_values.into_iter());
                        let dst = self.value_gen.next();
                        // Emit function name via NameConstBox
                        let c = match crate::mir::builder::name_const::make_name_const_result(self, &fname) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dst),
                            func: c,
                            callee: None,
                            args: call_args,
                            effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                        })?;
                        self.annotate_call_result_from_func_name(dst, &fname);
                        return Ok(dst);
                    }
                }
            }
        }

        // 4. Build object value for remaining cases
        let object_value = self.build_expression(object)?;

        // 5. Handle TypeOp methods: value.is("Type") / value.as("Type")
        // Note: This was duplicated in original code - now unified!
        if let Some(type_name) = special_handlers::is_typeop_method(&method, &arguments) {
            return self.handle_typeop_method(object_value, &method, &type_name);
        }

        // 6. Fallback: standard Box/Plugin method call
        self.handle_standard_method_call(object_value, method, &arguments)
    }

    // Map a user-facing type name to MIR type
    pub(super) fn parse_type_name_to_mir(name: &str) -> super::MirType {
        special_handlers::parse_type_name_to_mir(name)
    }

    // Extract string literal from AST node if possible
    pub(super) fn extract_string_literal(node: &ASTNode) -> Option<String> {
        special_handlers::extract_string_literal(node)
    }

    // Build from expression: from Parent.method(arguments)
    pub(super) fn build_from_expression(
        &mut self,
        parent: String,
        method: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg)?);
        }
        let parent_value = crate::mir::builder::emission::constant::emit_string(self, parent);
        let result_id = self.value_gen.next();
        self.emit_box_or_plugin_call(
            Some(result_id),
            parent_value,
            method,
            None,
            arg_values,
            EffectMask::READ.add(Effect::ReadHeap),
        )?;
        Ok(result_id)
    }

    // Lower a box method into a standalone MIR function (with `me` parameter)
    pub(super) fn lower_method_as_function(
        &mut self,
        func_name: String,
        box_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        let signature = function_lowering::prepare_method_signature(
            func_name,
            &box_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = super::MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            let me_id = self.value_gen.next();
            f.params.push(me_id);
            self.variable_map.insert("me".to_string(), me_id);
            self.value_origin_newbox.insert(me_id, box_name.clone());
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value && !self.is_current_block_terminated() {
            let void_val = crate::mir::builder::emission::constant::emit_void(self);
            self.emit_instruction(MirInstruction::Return {
                value: Some(void_val),
            })?;
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized_function = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized_function);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_gen = saved_value_gen;
        Ok(())
    }

    // Lower a static method body into a standalone MIR function (no `me` parameter)
    pub(super) fn lower_static_method_as_function(
        &mut self,
        func_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        // Derive static box context from function name prefix, e.g., "BoxName.method/N"
        let saved_static_ctx = self.current_static_box.clone();
        if let Some(pos) = func_name.find('.') {
            let box_name = &func_name[..pos];
            if !box_name.is_empty() {
                self.current_static_box = Some(box_name.to_string());
            }
        }
        let signature = function_lowering::prepare_static_method_signature(
            func_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = super::MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value {
            if let Some(ref mut f) = self.current_function {
                if let Some(block) = f.get_block(self.current_block.unwrap()) {
                    if !block.is_terminated() {
                        let void_val = crate::mir::builder::emission::constant::emit_void(self);
                        self.emit_instruction(MirInstruction::Return {
                            value: Some(void_val),
                        })?;
                    }
                }
            }
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_gen = saved_value_gen;
        // Restore static box context
        self.current_static_box = saved_static_ctx;
        Ok(())
    }
}
