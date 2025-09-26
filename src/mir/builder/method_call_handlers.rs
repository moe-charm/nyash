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
        // Optional dev/ci gate: enable builder-side instance→function rewrite by default
        // in dev/ci profiles, keep OFF in prod. Allow explicit override via env:
        //   NYASH_BUILDER_REWRITE_INSTANCE={1|true|on}  → force enable
        //   NYASH_BUILDER_REWRITE_INSTANCE={0|false|off} → force disable
        let rewrite_enabled = {
            match std::env::var("NYASH_BUILDER_REWRITE_INSTANCE").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
                Some(ref s) if s == "0" || s == "false" || s == "off" => false,
                Some(ref s) if s == "1" || s == "true" || s == "on" => true,
                _ => {
                    // Default: ON for dev/ci, OFF for prod
                    !crate::config::env::using_is_prod()
                }
            }
        };
        if rewrite_enabled {
        if let Some(cls) = class_name_opt.clone() {
            if self.user_defined_boxes.contains(&cls) {
                let arity = arg_values.len(); // function name arity excludes 'me'
                let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, &method, arity);
                // Gate: only rewrite when the lowered function actually exists (prevents false rewrites like JsonScanner.length/0)
                let exists = if let Some(ref module) = self.current_module {
                    module.functions.contains_key(&fname)
                } else { false };
                if exists {
                    if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        super::utils::builder_debug_log(&format!("userbox method-call cls={} method={} fname={}", cls, method, fname));
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
                    // Annotate return type/origin from lowered function signature
                    self.annotate_call_result_from_func_name(dst, &format!("{}.{}{}", cls, method, format!("/{}", arity)));
                    return Ok(dst);
                } else {
                    // Special-case: treat toString as stringify when method not present
                    if method == "toString" && arity == 0 {
                        if let Some(ref module) = self.current_module {
                            let stringify_name = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, "stringify", 0);
                            if module.functions.contains_key(&stringify_name) {
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
                    // Try alternate naming: <Class>Instance.method/Arity
                    let alt_cls = format!("{}Instance", cls);
                    let alt_fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(&alt_cls, &method, arity);
                    let alt_exists = if let Some(ref module) = self.current_module {
                        module.functions.contains_key(&alt_fname)
                    } else { false };
                    if alt_exists {
                        if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                            super::utils::builder_debug_log(&format!("userbox method-call alt cls={} method={} fname={}", alt_cls, method, alt_fname));
                        }
                        let name_const = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Const {
                            dst: name_const,
                            value: crate::mir::builder::ConstValue::String(alt_fname.clone()),
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
                        self.annotate_call_result_from_func_name(dst, &alt_fname);
                        return Ok(dst);
                    } else if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        super::utils::builder_debug_log(&format!("skip rewrite (no fn): cls={} method={} fname={} alt={} (missing)", cls, method, fname, alt_fname));
                    }
                }
            }
        }
        }

        // Fallback (narrowed): only when receiver class is known, and exactly one
        // user-defined method matches by name/arity across module, resolve to that.
        if rewrite_enabled && class_name_opt.is_some() {
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
                        call_args.extend(arg_values.into_iter());
                        let dst = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dst),
                            func: name_const,
                            callee: None,
                            args: call_args,
                            effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                        })?;
                        // Annotate from signature if present
                        self.annotate_call_result_from_func_name(dst, &fname);
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
