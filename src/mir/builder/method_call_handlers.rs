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
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // If receiver is a user-defined box, lower to function call: "Box.method/(1+arity)"
        let mut class_name_opt: Option<String> = None;
        if let Some(cn) = self.value_origin_newbox.get(&object_value) { class_name_opt = Some(cn.clone()); }
        if class_name_opt.is_none() {
            if let Some(t) = self.value_types.get(&object_value) {
                if let MirType::Box(bn) = t { class_name_opt = Some(bn.clone()); }
            }
        }
        if let Some(cls) = class_name_opt.clone() {
            if self.user_defined_boxes.contains(&cls) {
                let arity = arg_values.len(); // function name arity excludes 'me'
                let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(&cls, &method, arity);
                // Only use userbox path if such a function actually exists in the module
                let has_fn = if let Some(ref module) = self.current_module {
                    module.functions.contains_key(&fname)
                } else { false };
                if has_fn {
                    if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        super::utils::builder_debug_log(&format!("userbox method-call cls={} method={} fname={}", cls, method, fname));
                    }
                    let name_const = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: name_const,
                        value: crate::mir::builder::ConstValue::String(fname),
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
                    return Ok(dst);
                }
            }
        }

        // Fallback: if exactly one user-defined method matches by name/arity across module, resolve to that
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
                            value: crate::mir::builder::ConstValue::String(fname),
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
                        return Ok(dst);
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
