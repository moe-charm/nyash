use super::{ConstValue, Effect, EffectMask, MirInstruction, ValueId};
use crate::ast::{ASTNode, CallExpr};
use crate::mir::TypeOpKind;

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
                    let dst = self.value_gen.next();
                    let op = if call.name == "isType" { TypeOpKind::Check } else { TypeOpKind::Cast };
                    super::utils::builder_debug_log(&format!("emit TypeOp {:?} value={} dst= {}", op, val, dst));
                    self.emit_instruction(MirInstruction::TypeOp { dst, op, value: val, ty })?;
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None,
                        iface_name: "env.console".to_string(),
                        method_name: "log".to_string(),
                        args: vec![dst],
                        effects: EffectMask::PURE.add(Effect::Io),
                    })?;
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
                    let dst = self.value_gen.next();
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
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None,
                        iface_name: "env.console".to_string(),
                        method_name: "log".to_string(),
                        args: vec![dst],
                        effects: EffectMask::PURE.add(Effect::Io),
                    })?;
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
                    let dst = self.value_gen.next();
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
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None,
                        iface_name: "env.console".to_string(),
                        method_name: "log".to_string(),
                        args: vec![dst],
                        effects: EffectMask::PURE.add(Effect::Io),
                    })?;
                    return Ok(dst);
                } else {
                    super::utils::builder_debug_log("extract_string_literal FAIL");
                }
            }
            _ => {}
        }

        let value = self.build_expression(expression)?;
        super::utils::builder_debug_log(&format!("fallback print value={}", value));
        self.emit_instruction(MirInstruction::ExternCall {
            dst: None,
            iface_name: "env.console".to_string(),
            method_name: "log".to_string(),
            args: vec![value],
            effects: EffectMask::PURE.add(Effect::Io),
        })?;
        Ok(value)
    }

    // Block: sequentially build statements and return last value or Void
    pub(super) fn build_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        let mut last_value = None;
        for statement in statements {
            last_value = Some(self.build_expression(statement)?);
            // If the current block was terminated by this statement (e.g., return/throw),
            // do not emit any further instructions for this block.
            if self.is_current_block_terminated() {
                break;
            }
        }
        Ok(last_value.unwrap_or_else(|| {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            })
            .unwrap();
            void_val
        }))
    }

    // control-flow build_* moved to control_flow.rs (use cf_* instead)

    // Local declarations with optional initializers
    pub(super) fn build_local_statement(
        &mut self,
        variables: Vec<String>,
        initial_values: Vec<Option<Box<ASTNode>>>,
    ) -> Result<ValueId, String> {
        let mut last_value = None;
        for (i, var_name) in variables.iter().enumerate() {
            let value_id = if i < initial_values.len() && initial_values[i].is_some() {
                let init_expr = initial_values[i].as_ref().unwrap();
                self.build_expression(*init_expr.clone())?
            } else {
                self.value_gen.next()
            };
            self.variable_map.insert(var_name.clone(), value_id);
            last_value = Some(value_id);
        }
        Ok(last_value.unwrap_or_else(|| self.value_gen.next()))
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
            let void_dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: void_dst, value: ConstValue::Void })?;
            void_dst
        };

        if self.return_defer_active {
            // Defer: copy into slot and jump to target
            if let (Some(slot), Some(target)) = (self.return_defer_slot, self.return_defer_target) {
                self.return_deferred_emitted = true;
                self.emit_instruction(MirInstruction::Copy { dst: slot, src: return_value })?;
                if !self.is_current_block_terminated() {
                    self.emit_instruction(MirInstruction::Jump { target })?;
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
            let mname_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: mname_id,
                value: super::ConstValue::String(method.clone()),
            })?;
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(2 + arguments.len());
            arg_vals.push(recv_val);
            arg_vals.push(mname_id);
            for a in arguments.into_iter() {
                arg_vals.push(self.build_expression(a)?);
            }
            let future_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::ExternCall {
                dst: Some(future_id),
                iface_name: "env.future".to_string(),
                method_name: "spawn_instance".to_string(),
                args: arg_vals,
                effects: crate::mir::effect::EffectMask::PURE.add(crate::mir::effect::Effect::Io),
            })?;
            self.variable_map.insert(variable.clone(), future_id);
            return Ok(future_id);
        }
        let expression_value = self.build_expression(expression)?;
        let future_id = self.value_gen.next();
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
        let result_id = self.value_gen.next();
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
        let me_value = self.value_gen.next();
        let me_tag = if let Some(ref cls) = self.current_static_box {
            cls.clone()
        } else {
            "__me__".to_string()
        };
        self.emit_instruction(MirInstruction::Const {
            dst: me_value,
            value: ConstValue::String(me_tag),
        })?;
        self.variable_map.insert("me".to_string(), me_value);
        Ok(me_value)
    }
}
// use crate::mir::loop_api::LoopBuilderApi; // no longer needed here
