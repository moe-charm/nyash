use super::{MirInstruction, ValueId};
use crate::ast::{ASTNode, CallExpr};
use crate::mir::TypeOpKind;
use crate::mir::utils::is_current_block_terminated;

impl super::MirBuilder {
    // Print statement: env.console.log(value) with early TypeOp handling
    pub(super) fn build_print_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        super::utils::builder_debug_log("enter build_print_statement");
        // Prefer wrapper for simple function-call pattern (non-breaking refactor)
        if let Ok(call) = CallExpr::try_from(expression.clone()) {
            if (call.name == "isType" || call.name == "asType") && call.arguments.len() == 2 {
                super::utils::builder_debug_log("pattern: print(FunctionCall isType|asType) [via wrapper]");
                if let Some(type_name) = super::MirBuilder::extract_string_literal(&call.arguments[1]) {
                    super::utils::builder_debug_log(&format!("extract_string_literal OK: {}", type_name));
                    let val = self.build_expression(call.arguments[0].clone())?;
                    let ty = super::MirBuilder::parse_type_name_to_mir(&type_name);
                    let dst = self.safe_next_value();
                    let op = if call.name == "isType" { TypeOpKind::Check } else { TypeOpKind::Cast };
                    super::utils::builder_debug_log(&format!("emit TypeOp {:?} value={} dst= {}", op, val, dst));
                    self.emit_instruction(MirInstruction::TypeOp { dst, op, value: val, ty })?;
                    self.emit_unified_call(
                        None,
                        super::builder_calls::CallTarget::Extern("env.console.log".to_string()),
                        vec![dst],
                    )?;
                    return Ok(dst);
                } else {
                    super::utils::builder_debug_log("extract_string_literal FAIL [via wrapper]");
                }
            }
        }

        match &expression {
            // print(isType(val, "Type")) / print(asType(...))
            ASTNode::FunctionCall {
                name, arguments, ..
            } if (name == "isType" || name == "asType") && arguments.len() == 2 => {
                super::utils::builder_debug_log("pattern: print(FunctionCall isType|asType)");
                if let Some(type_name) = super::MirBuilder::extract_string_literal(&arguments[1]) {
                    super::utils::builder_debug_log(&format!(
                        "extract_string_literal OK: {}",
                        type_name
                    ));
                    let val = self.build_expression(arguments[0].clone())?;
                    let ty = super::MirBuilder::parse_type_name_to_mir(&type_name);
                    let dst = self.safe_next_value();
                    let op = if name == "isType" {
                        TypeOpKind::Check
                    } else {
                        TypeOpKind::Cast
                    };
                    super::utils::builder_debug_log(&format!(
                        "emit TypeOp {:?} value={} dst= {}",
                        op, val, dst
                    ));
                    self.emit_instruction(MirInstruction::TypeOp {
                        dst,
                        op,
                        value: val,
                        ty,
                    })?;
                    self.emit_unified_call(
                        None,
                        super::builder_calls::CallTarget::Extern("env.console.log".to_string()),
                        vec![dst],
                    )?;
                    return Ok(dst);
                } else {
                    super::utils::builder_debug_log("extract_string_literal FAIL");
                }
            }
            // print(obj.is("Type")) / print(obj.as("Type"))
            ASTNode::MethodCall {
                object,
                method,
                arguments,
                ..
            } if (method == "is" || method == "as") && arguments.len() == 1 => {
                super::utils::builder_debug_log("pattern: print(MethodCall is|as)");
                if let Some(type_name) = super::MirBuilder::extract_string_literal(&arguments[0]) {
                    super::utils::builder_debug_log(&format!(
                        "extract_string_literal OK: {}",
                        type_name
                    ));
                    let obj_val = self.build_expression(*object.clone())?;
                    let ty = super::MirBuilder::parse_type_name_to_mir(&type_name);
                    let dst = self.safe_next_value();
                    let op = if method == "is" {
                        TypeOpKind::Check
                    } else {
                        TypeOpKind::Cast
                    };
                    super::utils::builder_debug_log(&format!(
                        "emit TypeOp {:?} obj={} dst= {}",
                        op, obj_val, dst
                    ));
                    self.emit_instruction(MirInstruction::TypeOp {
                        dst,
                        op,
                        value: obj_val,
                        ty,
                    })?;
                    self.emit_unified_call(
                        None,
                        super::builder_calls::CallTarget::Extern("env.console.log".to_string()),
                        vec![dst],
                    )?;
                    return Ok(dst);
                } else {
                    super::utils::builder_debug_log("extract_string_literal FAIL");
                }
            }
            _ => {}
        }

        let value = self.build_expression(expression)?;
        super::utils::builder_debug_log(&format!("fallback print value={}", value));

        // Phase 3.2: Use unified call for print statements
        let use_unified = super::calls::call_unified::is_unified_call_enabled();

        if use_unified {
            // New unified path - treat print as global function
            self.emit_unified_call(
                None,  // print returns nothing
                super::builder_calls::CallTarget::Global("print".to_string()),
                vec![value],
            )?;
        } else {
            // Fallback path (compat): use callee=Extern directly
            self.emit_unified_call(
                None,
                super::builder_calls::CallTarget::Extern("env.console.log".to_string()),
                vec![value],
            )?;
        }
        Ok(value)
    }

    // Block: sequentially build statements and return last value or Void
    pub(super) fn build_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        // Scope hint for bare block (Program)
        let scope_id = self.current_block.map(|bb| bb.as_u32()).unwrap_or(0);
        self.hint_scope_enter(scope_id);
        let mut last_value = None;
        for statement in statements {
            last_value = Some(self.build_expression(statement)?);
            // If the current block was terminated by this statement (e.g., return/throw),
            // do not emit any further instructions for this block.
            if is_current_block_terminated(self)? {
                break;
            }
        }
        let out = last_value.unwrap_or_else(|| {
            // Use ConstantEmissionBox for Void
            crate::mir::builder::emission::constant::emit_void(self)
        });
        // Scope leave only if block not already terminated
        if !self.is_current_block_terminated() {
            self.hint_scope_leave(scope_id);
        }
        Ok(out)
    }


    // Local declarations with optional initializers
    pub(super) fn build_local_statement(
        &mut self,
        variables: Vec<String>,
        initial_values: Vec<Option<Box<ASTNode>>>,
    ) -> Result<ValueId, String> {
        let mut last_value = None;
        for (i, var_name) in variables.iter().enumerate() {
            let var_id = if i < initial_values.len() && initial_values[i].is_some() {
                // Use initializer's ValueId directly to avoid SSA aliasing/undefined use
                let init_expr = initial_values[i].as_ref().unwrap();
                let init_val = self.build_expression(*init_expr.clone())?;
                init_val
            } else {
                // Create a concrete register for uninitialized locals (Void)
                crate::mir::builder::emission::constant::emit_void(self)
            };
            self.variable_map.insert(var_name.clone(), var_id);
            last_value = Some(var_id);
        }
        Ok(last_value.unwrap_or_else(|| self.safe_next_value()))
    }

    // Return statement
    pub(super) fn build_return_statement(
        &mut self,
        value: Option<Box<ASTNode>>,
    ) -> Result<ValueId, String> {
        // Enforce cleanup policy
        if self.in_cleanup_block && !self.cleanup_allow_return {
            return Err("return is not allowed inside cleanup block (enable NYASH_CLEANUP_ALLOW_RETURN=1 to permit)".to_string());
        }

        let return_value = if let Some(expr) = value {
            self.build_expression(*expr)?
        } else {
            crate::mir::builder::emission::constant::emit_void(self)
        };

        if self.return_defer_active {
            // Defer: copy into slot and jump to target
            if let (Some(slot), Some(target)) = (self.return_defer_slot, self.return_defer_target) {
                self.return_deferred_emitted = true;
                self.emit_instruction(MirInstruction::Copy { dst: slot, src: return_value })?;
                crate::mir::builder::metadata::propagate::propagate(self, return_value, slot);
                if !self.is_current_block_terminated() {
                    crate::mir::builder::emission::branch::emit_jump(self, target)?;
                }
                Ok(return_value)
            } else {
                // Fallback: no configured slot/target; emit a real return
                self.emit_instruction(MirInstruction::Return { value: Some(return_value) })?;
                Ok(return_value)
            }
        } else {
            // Normal return
            self.emit_instruction(MirInstruction::Return { value: Some(return_value) })?;
            Ok(return_value)
        }
    }

    // Nowait: prefer env.future.spawn_instance if method call; else FutureNew
    pub(super) fn build_nowait_statement(
        &mut self,
        variable: String,
        expression: ASTNode,
    ) -> Result<ValueId, String> {
        if let ASTNode::MethodCall {
            object,
            method,
            arguments,
            ..
        } = expression.clone()
        {
            let recv_val = self.build_expression(*object)?;
            let mname_id = crate::mir::builder::emission::constant::emit_string(self, method.clone());
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(2 + arguments.len());
            arg_vals.push(recv_val);
            arg_vals.push(mname_id);
            for a in arguments.into_iter() {
                arg_vals.push(self.build_expression(a)?);
            }
            let future_id = self.safe_next_value();
            self.emit_unified_call(
                Some(future_id),
                super::builder_calls::CallTarget::Extern("env.future.spawn_instance".to_string()),
                arg_vals,
            )?;
            self.variable_map.insert(variable.clone(), future_id);
            return Ok(future_id);
        }
        let expression_value = self.build_expression(expression)?;
        let future_id = self.safe_next_value();
        self.emit_instruction(MirInstruction::FutureNew {
            dst: future_id,
            value: expression_value,
        })?;
        self.variable_map.insert(variable.clone(), future_id);
        Ok(future_id)
    }

    // Await: insert Safepoint before/after and emit Await
    pub(super) fn build_await_expression(
        &mut self,
        expression: ASTNode,
    ) -> Result<ValueId, String> {
        let future_value = self.build_expression(expression)?;
        self.emit_instruction(MirInstruction::Safepoint)?;
        let result_id = self.safe_next_value();
        self.emit_instruction(MirInstruction::Await {
            dst: result_id,
            future: future_value,
        })?;
        self.emit_instruction(MirInstruction::Safepoint)?;
        Ok(result_id)
    }

    // me: resolve to param if present; else symbolic const (stable mapping)
    pub(super) fn build_me_expression(&mut self) -> Result<ValueId, String> {
        if let Some(id) = self.variable_map.get("me").cloned() {
            return Ok(id);
        }
        let me_tag = if let Some(ref cls) = self.current_static_box {
            cls.clone()
        } else {
            "__me__".to_string()
        };
        let me_value = crate::mir::builder::emission::constant::emit_string(self, me_tag);
        self.variable_map.insert("me".to_string(), me_value);
        // P0: Known 化 — 分かる範囲で me の起源クラスを付与（挙動不変）。
        super::origin::infer::annotate_me_origin(self, me_value);
        Ok(me_value)
    }
}
