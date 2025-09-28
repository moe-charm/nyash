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
        // 安全策: レシーバをピン留めしてブロック境界での未定義参照を防ぐ
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
        // レガシー経路（BoxCall/Plugin）へ送る（安定優先・挙動不変）。
        let result_id = self.value_gen.next();
        self.emit_box_or_plugin_call(
            Some(result_id),
            object_value,
            method.clone(),
            None,
            arg_values,
            super::EffectMask::READ.add(super::Effect::ReadHeap),
        )?;
        Ok(result_id)
    }
}
