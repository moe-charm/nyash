//! Method call handlers for MIR builder
//!
//! This module contains specialized handlers for different types of method calls,
//! following the Single Responsibility Principle.

use crate::ast::ASTNode;
use crate::mir::builder::{MirBuilder, ValueId};
use crate::mir::builder::builder_calls::CallTarget;
use crate::mir::{MirInstruction, TypeOpKind, MirType};

impl MirBuilder {
    /// Handle static method calls: BoxName.method(args)
    pub(super) fn handle_static_method_call(
        &mut self,
        box_name: &str,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<ValueId, String> {
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // Compose lowered function name: BoxName.method/N
        let func_name = format!("{}.{}/{}", box_name, method, arg_values.len());
        let dst = self.value_gen.next();

        if std::env::var("NYASH_STATIC_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[builder] static-call {}", func_name);
        }

        // Use legacy global-call emission to avoid unified builtin/extern constraints
        self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
        Ok(dst)
    }

    /// Handle TypeOp method calls: value.is("Type") and value.as("Type")
    pub(super) fn handle_typeop_method(
        &mut self,
        object_value: ValueId,
        method: &str,
        type_name: &str,
    ) -> Result<ValueId, String> {
        let mir_ty = Self::parse_type_name_to_mir(type_name);
        let dst = self.value_gen.next();
        let op = if method == "is" {
            TypeOpKind::Check
        } else {
            TypeOpKind::Cast
        };

        self.emit_instruction(MirInstruction::TypeOp {
            dst,
            op,
            value: object_value,
            ty: mir_ty,
        })?;

        Ok(dst)
    }

    /// Check if this is a TypeOp method call
    pub(super) fn is_typeop_method(method: &str, arguments: &[ASTNode]) -> Option<String> {
        if (method == "is" || method == "as") && arguments.len() == 1 {
            Self::extract_string_literal(&arguments[0])
        } else {
            None
        }
    }

    /// Handle me.method() calls within static box context
    pub(super) fn handle_me_method_call(
        &mut self,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Option<ValueId>, String> {
        // Convert slice to Vec for compatibility
        let args_vec = arguments.to_vec();
        // Delegate to existing try_handle_me_direct_call
        match self.try_handle_me_direct_call(method, &args_vec) {
            Some(result) => result.map(Some),
            None => Ok(None),
        }
    }

    /// Handle standard Box/Plugin method calls (fallback)
    pub(super) fn handle_standard_method_call(
        &mut self,
        object_value: ValueId,
        method: String,
        arguments: &[ASTNode],
    ) -> Result<ValueId, String> {
        // Correctness-first: pin receiver so it has a block-local def and can safely
        // flow across branches/merges when method calls are used in conditions.
        let object_value = self
            .pin_to_slot(object_value, "@recv")
            .unwrap_or(object_value);
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // If receiver is a user-defined box, lower to function call: "Box.method/(1+arity)"
        let mut class_name_opt: Option<String> = None;
        // Heuristic guard: if this receiver equals the current function's 'me',
        // prefer the enclosing box name parsed from the function signature.
        if class_name_opt.is_none() {
            if let Some(&me_vid) = self.variable_map.get("me") {
                if me_vid == object_value {
                    if let Some(ref fun) = self.current_function {
                        if let Some(dot) = fun.signature.name.find('.') {
                            class_name_opt = Some(fun.signature.name[..dot].to_string());
                        }
                    }
                }
            }
        }
        if class_name_opt.is_none() {
            if let Some(cn) = self.value_origin_newbox.get(&object_value) { class_name_opt = Some(cn.clone()); }
        }
        if class_name_opt.is_none() {
            if let Some(t) = self.value_types.get(&object_value) {
                if let MirType::Box(bn) = t { class_name_opt = Some(bn.clone()); }
            }
        }
        // Instance→Function rewrite (obj.m(a) → Box.m/Arity(obj,a))
        // Phase 2 policy: Only rewrite when receiver class is Known (from origin propagation).
        let class_known = self.value_origin_newbox.get(&object_value).is_some();
        // Rationale:
        // - Keep language surface idiomatic (obj.method()), while executing
        //   deterministically as a direct function call.
        // - Prod VM forbids user Instance BoxCall fallback by policy; this
        //   rewrite guarantees prod runs without runtime instance-dispatch.
        // Control:
        //   NYASH_BUILDER_REWRITE_INSTANCE={1|true|on}  → force enable
        //   NYASH_BUILDER_REWRITE_INSTANCE={0|false|off} → force disable
        let rewrite_enabled = {
            match std::env::var("NYASH_BUILDER_REWRITE_INSTANCE").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
                Some(ref s) if s == "0" || s == "false" || s == "off" => false,
                Some(ref s) if s == "1" || s == "true" || s == "on" => true,
                _ => {
                    // Default: ON (prod/dev/ci) unless明示OFF。再発防止のため常時関数化を優先。
                    true
                }
            }
        };
        // Emit resolve.try event (dev-only) before making a decision
        if rewrite_enabled {
            if let Some(ref module) = self.current_module {
                let tail = format!(".{}{}", method, format!("/{}", arguments.len()));
                let candidates: Vec<String> = module
                    .functions
                    .keys()
                    .filter(|k| k.ends_with(&tail))
                    .cloned()
                    .collect();
                let recv_cls = class_name_opt.clone().or_else(|| self.value_origin_newbox.get(&object_value).cloned()).unwrap_or_default();
                let meta = serde_json::json!({
                    "recv_cls": recv_cls,
                    "method": method,
                    "arity": arguments.len(),
                    "candidates": candidates,
                });
                let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                let region = self.debug_current_region_id();
                crate::debug::hub::emit(
                    "resolve",
                    "try",
                    fn_name,
                    region.as_deref(),
                    meta,
                );
            }
        }
        // Early special-case: toString → stringify mapping when user function exists
        if method == "toString" && arguments.len() == 0 {
            if let Some(ref module) = self.current_module {
                // Prefer class-qualified stringify if we can infer class
                if let Some(cls_ts) = class_name_opt.clone() {
                    let stringify_name = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls_ts, "stringify", 0);
                    if module.functions.contains_key(&stringify_name) {
                        if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                            super::utils::builder_debug_log(&format!("(early) toString→stringify cls={} fname={}", cls_ts, stringify_name));
                        }
                        // DebugHub emit: resolve.choose (early, class)
                        {
                            let meta = serde_json::json!({
                                "recv_cls": cls_ts,
                                "method": "toString",
                                "arity": 0,
                                "chosen": stringify_name,
                                "reason": "toString-early-class",
                            });
                            let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                            let region = self.debug_current_region_id();
                            crate::debug::hub::emit(
                                "resolve",
                                "choose",
                                fn_name,
                                region.as_deref(),
                                meta,
                            );
                        }
                        let name_const = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Const {
                            dst: name_const,
                            value: crate::mir::builder::ConstValue::String(stringify_name.clone()),
                        })?;
                        let mut call_args = Vec::with_capacity(1);
                        call_args.push(object_value);
                        let dst = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dst),
                            func: name_const,
                            callee: None,
                            args: call_args,
                            effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                        })?;
                        self.annotate_call_result_from_func_name(dst, &stringify_name);
                        return Ok(dst);
                    }
                }
                // Fallback: unique suffix ".stringify/0" in module
                let mut cands: Vec<String> = module
                    .functions
                    .keys()
                    .filter(|k| k.ends_with(".stringify/0"))
                    .cloned()
                    .collect();
                if cands.len() == 1 {
                    let fname = cands.remove(0);
                    if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        super::utils::builder_debug_log(&format!("(early) toString→stringify unique-suffix fname={}", fname));
                    }
                    // DebugHub emit: resolve.choose (early, unique)
                    {
                        let meta = serde_json::json!({
                            "recv_cls": class_name_opt.clone().unwrap_or_default(),
                            "method": "toString",
                            "arity": 0,
                            "chosen": fname,
                            "reason": "toString-early-unique",
                        });
                        let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                        let region = self.debug_current_region_id();
                        crate::debug::hub::emit(
                            "resolve",
                            "choose",
                            fn_name,
                            region.as_deref(),
                            meta,
                        );
                    }
                    let name_const = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: name_const,
                        value: crate::mir::builder::ConstValue::String(fname.clone()),
                    })?;
                    let mut call_args = Vec::with_capacity(1);
                    call_args.push(object_value);
                    let dst = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: name_const,
                        callee: None,
                        args: call_args,
                        effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &fname);
                    return Ok(dst);
                } else if cands.len() > 1 {
                    // Deterministic tie-breaker: prefer JsonNode.stringify/0 over JsonNodeInstance.stringify/0
                    if let Some(pos) = cands.iter().position(|n| n == "JsonNode.stringify/0") {
                        let fname = cands.remove(pos);
                        if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                            super::utils::builder_debug_log(&format!("(early) toString→stringify prefer JsonNode fname={}", fname));
                        }
                        // DebugHub emit: resolve.choose (early, prefer-JsonNode)
                        {
                            let meta = serde_json::json!({
                                "recv_cls": class_name_opt.clone().unwrap_or_default(),
                                "method": "toString",
                                "arity": 0,
                                "chosen": fname,
                                "reason": "toString-early-prefer-JsonNode",
                            });
                            let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                            let region = self.debug_current_region_id();
                            crate::debug::hub::emit(
                                "resolve",
                                "choose",
                                fn_name,
                                region.as_deref(),
                                meta,
                            );
                        }
                        let name_const = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Const {
                            dst: name_const,
                            value: crate::mir::builder::ConstValue::String(fname.clone()),
                        })?;
                        let mut call_args = Vec::with_capacity(1);
                        call_args.push(object_value);
                        let dst = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dst),
                            func: name_const,
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

        if rewrite_enabled && class_known {
            if let Some(cls) = class_name_opt.clone() {
                let from_new_origin = self.value_origin_newbox.get(&object_value).is_some();
                let allow_new_origin = std::env::var("NYASH_DEV_REWRITE_NEW_ORIGIN").ok().as_deref() == Some("1");
                let is_user_box = self.user_defined_boxes.contains(&cls);
            let fname = {
                let arity = arg_values.len();
                crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, &method, arity)
            };
            let module_has = if let Some(ref module) = self.current_module { module.functions.contains_key(&fname) } else { false };
            let allow_userbox_rewrite = std::env::var("NYASH_DEV_REWRITE_USERBOX").ok().as_deref() == Some("1");
            if (is_user_box && (module_has || allow_userbox_rewrite)) || (from_new_origin && allow_new_origin) {
                let arity = arg_values.len(); // function name arity excludes 'me'
                // Special-case: toString → stringify mapping (only when present)
                if method == "toString" && arity == 0 {
                    if let Some(ref module) = self.current_module {
                        let stringify_name = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "stringify", 0);
                        if module.functions.contains_key(&stringify_name) {
                            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                                super::utils::builder_debug_log(&format!("userbox toString→stringify cls={} fname={}", cls, stringify_name));
                            }
                            let name_const = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Const {
                                dst: name_const,
                                value: crate::mir::builder::ConstValue::String(stringify_name.clone()),
                            })?;
                            let mut call_args = Vec::with_capacity(1);
                            call_args.push(object_value);
                            let dst = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dst),
                                func: name_const,
                                callee: None,
                                args: call_args,
                                effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                            })?;
                            self.annotate_call_result_from_func_name(dst, &stringify_name);
                            return Ok(dst);
                        }
                    }
                }

                // Default: unconditionally rewrite to Box.method/Arity. The target
                // may be materialized later during lowering of the box; runtime
                // resolution by name will succeed once the module is finalized.
                let fname = fname.clone();
                if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                    super::utils::builder_debug_log(&format!("userbox method-call cls={} method={} fname={}", cls, method, fname));
                }
                // Dev WARN when the function is not yet present (materialize pending)
                if crate::config::env::cli_verbose() {
                    if let Some(ref module) = self.current_module {
                        if !module.functions.contains_key(&fname) {
                            eprintln!(
                                "[warn] rewrite (materialize pending): {} (class={}, method={}, arity={})",
                                fname, cls, method, arity
                            );
                        }
                    }
                }
                let name_const = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: name_const,
                    value: crate::mir::builder::ConstValue::String(fname.clone()),
                })?;
                let mut call_args = Vec::with_capacity(arity + 1);
                call_args.push(object_value); // 'me'
                call_args.extend(arg_values.into_iter());
                let dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::Call {
                    dst: Some(dst),
                    func: name_const,
                    callee: None,
                    args: call_args,
                    effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                })?;
                // Annotate and emit resolve.choose
                let chosen = format!("{}.{}{}", cls, method, format!("/{}", arity));
                self.annotate_call_result_from_func_name(dst, &chosen);
                let meta = serde_json::json!({
                    "recv_cls": cls,
                    "method": method,
                    "arity": arity,
                    "chosen": chosen,
                    "reason": "userbox-rewrite",
                });
                let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                let region = self.debug_current_region_id();
                crate::debug::hub::emit(
                    "resolve",
                    "choose",
                    fn_name,
                    region.as_deref(),
                    meta,
                );
                return Ok(dst);
            } else {
                // Not a user-defined box; fall through
            }
        }
        }

        // Fallback (narrowed): when exactly one user-defined method matches by
        // name/arity across the module, resolve to that even if class inference
        // failed (defensive for PHI/branch cases). This preserves determinism
        // because we require uniqueness and a user-defined box prefix.
        if rewrite_enabled && class_known {
            if let Some(ref module) = self.current_module {
                let tail = format!(".{}{}", method, format!("/{}", arg_values.len()));
                let mut cands: Vec<String> = module
                    .functions
                    .keys()
                    .filter(|k| k.ends_with(&tail))
                    .cloned()
                    .collect();
                if cands.len() == 1 {
                    let fname = cands.remove(0);
                    // sanity: ensure the box prefix looks like a user-defined box
                    if let Some((bx, _)) = fname.split_once('.') {
                        if self.user_defined_boxes.contains(bx) {
                            let name_const = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Const {
                                dst: name_const,
                                value: crate::mir::builder::ConstValue::String(fname.clone()),
                            })?;
                let mut call_args = Vec::with_capacity(arg_values.len() + 1);
                call_args.push(object_value); // 'me'
                let arity_us = arg_values.len();
                call_args.extend(arg_values.into_iter());
                            let dst = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dst),
                                func: name_const,
                                callee: None,
                                args: call_args,
                                effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                            })?;
                            // Annotate and emit resolve.choose
                            self.annotate_call_result_from_func_name(dst, &fname);
                            let meta = serde_json::json!({
                                "recv_cls": bx,
                                "method": method,
                                "arity": arity_us,
                                "chosen": fname,
                                "reason": "unique-suffix",
                            });
                            let fn_name = self.current_function.as_ref().map(|f| f.signature.name.as_str());
                            let region = self.debug_current_region_id();
                            crate::debug::hub::emit(
                                "resolve",
                                "choose",
                                fn_name,
                                region.as_deref(),
                                meta,
                            );
                            return Ok(dst);
                        }
                    }
                }
            }
        }

        // Else fall back to plugin/boxcall path
        let result_id = self.value_gen.next();
        self.emit_box_or_plugin_call(
            Some(result_id),
            object_value,
            method,
            None,
            arg_values,
            crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
        )?;

        Ok(result_id)
    }
}
