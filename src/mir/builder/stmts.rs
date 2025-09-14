use super::{MirInstruction, EffectMask, Effect, ConstValue, ValueId};
use crate::mir::TypeOpKind;
use crate::mir::loop_builder::LoopBuilder;
use crate::ast::ASTNode;

impl super::MirBuilder {
    // Print statement: env.console.log(value) with early TypeOp handling
    pub(super) fn build_print_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        super::utils::builder_debug_log("enter build_print_statement");
        match &expression {
            // print(isType(val, "Type")) / print(asType(...))
            ASTNode::FunctionCall { name, arguments, .. }
                if (name == "isType" || name == "asType") && arguments.len() == 2 =>
            {
                super::utils::builder_debug_log("pattern: print(FunctionCall isType|asType)");
                if let Some(type_name) = super::MirBuilder::extract_string_literal(&arguments[1]) {
                    super::utils::builder_debug_log(&format!("extract_string_literal OK: {}", type_name));
                    let val = self.build_expression(arguments[0].clone())?;
                    let ty = super::MirBuilder::parse_type_name_to_mir(&type_name);
                    let dst = self.value_gen.next();
                    let op = if name == "isType" { TypeOpKind::Check } else { TypeOpKind::Cast };
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
                    super::utils::builder_debug_log("extract_string_literal FAIL");
                }
            }
            // print(obj.is("Type")) / print(obj.as("Type"))
            ASTNode::MethodCall { object, method, arguments, .. }
                if (method == "is" || method == "as") && arguments.len() == 1 =>
            {
                super::utils::builder_debug_log("pattern: print(MethodCall is|as)");
                if let Some(type_name) = super::MirBuilder::extract_string_literal(&arguments[0]) {
                    super::utils::builder_debug_log(&format!("extract_string_literal OK: {}", type_name));
                    let obj_val = self.build_expression(*object.clone())?;
                    let ty = super::MirBuilder::parse_type_name_to_mir(&type_name);
                    let dst = self.value_gen.next();
                    let op = if method == "is" { TypeOpKind::Check } else { TypeOpKind::Cast };
                    super::utils::builder_debug_log(&format!("emit TypeOp {:?} obj={} dst= {}", op, obj_val, dst));
                    self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
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
            self.emit_instruction(MirInstruction::Const { dst: void_val, value: ConstValue::Void }).unwrap();
            void_val
        }))
    }

    // If: lower to Branch + Phi, bind reassigned var name if identical
    pub(super) fn build_if_statement(
        &mut self,
        condition: ASTNode,
        then_branch: ASTNode,
        else_branch: Option<ASTNode>,
    ) -> Result<ValueId, String> {
        let condition_val = self.build_expression(condition)?;
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();
        self.emit_instruction(MirInstruction::Branch { condition: condition_val, then_bb: then_block, else_bb: else_block })?;

        // Snapshot variable map before entering branches to avoid cross-branch pollution
        let pre_if_var_map = self.variable_map.clone();
        // Pre-analysis: detect then-branch assigned var and capture its pre-if value
        let assigned_then_pre = extract_assigned_var(&then_branch);
        let pre_then_var_value: Option<ValueId> = assigned_then_pre
            .as_ref()
            .and_then(|name| pre_if_var_map.get(name).copied());

        // then
        self.current_block = Some(then_block);
        self.ensure_block_exists(then_block)?;
        let then_ast_for_analysis = then_branch.clone();
        // Build then with a clean snapshot of pre-if variables
        self.variable_map = pre_if_var_map.clone();
        let then_value_raw = self.build_expression(then_branch)?;
        let then_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() { self.emit_instruction(MirInstruction::Jump { target: merge_block })?; }

        // else
        self.current_block = Some(else_block);
        self.ensure_block_exists(else_block)?;
        // Build else with a clean snapshot of pre-if variables
        let (mut else_value_raw, else_ast_for_analysis, else_var_map_end_opt) = if let Some(else_ast) = else_branch {
            self.variable_map = pre_if_var_map.clone();
            let val = self.build_expression(else_ast.clone())?;
            (val, Some(else_ast), Some(self.variable_map.clone()))
        } else {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: void_val, value: ConstValue::Void })?;
            (void_val, None, None)
        };
        if !self.is_current_block_terminated() { self.emit_instruction(MirInstruction::Jump { target: merge_block })?; }

        // merge + phi
        self.current_block = Some(merge_block);
        self.ensure_block_exists(merge_block)?;
        // If only the then-branch assigns a variable (e.g., `if c { x = ... }`) and the else
        // does not assign the same variable, bind that variable to a Phi of (then_value, pre_if_value).
        let assigned_var_then = extract_assigned_var(&then_ast_for_analysis);
        let assigned_var_else = else_ast_for_analysis.as_ref().and_then(|a| extract_assigned_var(a));
        let result_val = self.value_gen.next();
        if let Some(var_name) = assigned_var_then.clone() {
            let else_assigns_same = assigned_var_else.as_ref().map(|s| s == &var_name).unwrap_or(false);
            // Resolve branch-end values for the assigned variable
            let then_value_for_var = then_var_map_end.get(&var_name).copied().unwrap_or(then_value_raw);
            let else_value_for_var = if else_assigns_same {
                else_var_map_end_opt.as_ref().and_then(|m| m.get(&var_name).copied()).unwrap_or(else_value_raw)
            } else {
                // Else doesn't assign: use pre-if value if available
                pre_then_var_value.unwrap_or(else_value_raw)
            };
            // Emit Phi for the assigned variable and bind it
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs: vec![(then_block, then_value_for_var), (else_block, else_value_for_var)] })?;
            self.variable_map = pre_if_var_map.clone();
            self.variable_map.insert(var_name, result_val);
        } else {
            // No variable assignment pattern detected – just emit Phi for expression result
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs: vec![(then_block, then_value_raw), (else_block, else_value_raw)] })?;
            // Merge variable map conservatively to pre-if snapshot (no new bindings)
            self.variable_map = pre_if_var_map.clone();
        }

        Ok(result_val)
    }

    // Loop: delegate to LoopBuilder
    pub(super) fn build_loop_statement(&mut self, condition: ASTNode, body: Vec<ASTNode>) -> Result<ValueId, String> {
        let mut loop_builder = LoopBuilder::new(self);
        loop_builder.build_loop(condition, body)
    }

    // Try/Catch/Finally lowering with basic Extern semantics when disabled
    pub(super) fn build_try_catch_statement(
        &mut self,
        try_body: Vec<ASTNode>,
        catch_clauses: Vec<crate::ast::CatchClause>,
        finally_body: Option<Vec<ASTNode>>,
    ) -> Result<ValueId, String> {
        if std::env::var("NYASH_BUILDER_DISABLE_TRYCATCH").ok().as_deref() == Some("1") {
            let try_ast = ASTNode::Program { statements: try_body, span: crate::ast::Span::unknown() };
            let result = self.build_expression(try_ast)?;
            return Ok(result);
        }
        let try_block = self.block_gen.next();
        let catch_block = self.block_gen.next();
        let finally_block = if finally_body.is_some() { Some(self.block_gen.next()) } else { None };
        let exit_block = self.block_gen.next();

        if let Some(catch_clause) = catch_clauses.first() {
            if std::env::var("NYASH_DEBUG_TRYCATCH").ok().as_deref() == Some("1") {
                eprintln!("[BUILDER] Emitting catch handler for {:?}", catch_clause.exception_type);
            }
            let exception_value = self.value_gen.next();
            self.emit_instruction(MirInstruction::Catch { exception_type: catch_clause.exception_type.clone(), exception_value, handler_bb: catch_block })?;
        }

        self.emit_instruction(MirInstruction::Jump { target: try_block })?;
        self.start_new_block(try_block)?;
        let try_ast = ASTNode::Program { statements: try_body, span: crate::ast::Span::unknown() };
        let _try_result = self.build_expression(try_ast)?;
        if !self.is_current_block_terminated() { let next_target = finally_block.unwrap_or(exit_block); self.emit_instruction(MirInstruction::Jump { target: next_target })?; }

        self.start_new_block(catch_block)?;
        if std::env::var("NYASH_DEBUG_TRYCATCH").ok().as_deref() == Some("1") { eprintln!("[BUILDER] Enter catch block {:?}", catch_block); }
        if let Some(catch_clause) = catch_clauses.first() {
            if std::env::var("NYASH_DEBUG_TRYCATCH").ok().as_deref() == Some("1") { eprintln!("[BUILDER] Emitting catch handler for {:?}", catch_clause.exception_type); }
            let catch_ast = ASTNode::Program { statements: catch_clause.body.clone(), span: crate::ast::Span::unknown() };
            self.build_expression(catch_ast)?;
        }
        if !self.is_current_block_terminated() { let next_target = finally_block.unwrap_or(exit_block); self.emit_instruction(MirInstruction::Jump { target: next_target })?; }

        if let (Some(finally_block_id), Some(finally_statements)) = (finally_block, finally_body) {
            self.start_new_block(finally_block_id)?;
            let finally_ast = ASTNode::Program { statements: finally_statements, span: crate::ast::Span::unknown() };
            self.build_expression(finally_ast)?;
            self.emit_instruction(MirInstruction::Jump { target: exit_block })?;
        }

        self.start_new_block(exit_block)?;
        let result = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const { dst: result, value: ConstValue::Void })?;
        Ok(result)
    }

    // Throw: emit Throw or fallback to env.debug.trace when disabled
    pub(super) fn build_throw_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        if std::env::var("NYASH_BUILDER_DISABLE_THROW").ok().as_deref() == Some("1") {
            let v = self.build_expression(expression)?;
            self.emit_instruction(MirInstruction::ExternCall {
                dst: None,
                iface_name: "env.debug".to_string(),
                method_name: "trace".to_string(),
                args: vec![v],
                effects: EffectMask::PURE.add(Effect::Debug),
            })?;
            return Ok(v);
        }
        let exception_value = self.build_expression(expression)?;
        self.emit_instruction(MirInstruction::Throw { exception: exception_value, effects: EffectMask::PANIC })?;
        Ok(exception_value)
    }

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
    pub(super) fn build_return_statement(&mut self, value: Option<Box<ASTNode>>) -> Result<ValueId, String> {
        let return_value = if let Some(expr) = value {
            self.build_expression(*expr)?
        } else {
            let void_dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: void_dst, value: ConstValue::Void })?;
            void_dst
        };
        self.emit_instruction(MirInstruction::Return { value: Some(return_value) })?;
        Ok(return_value)
    }

    // Nowait: prefer env.future.spawn_instance if method call; else FutureNew
    pub(super) fn build_nowait_statement(&mut self, variable: String, expression: ASTNode) -> Result<ValueId, String> {
        if let ASTNode::MethodCall { object, method, arguments, .. } = expression.clone() {
            let recv_val = self.build_expression(*object)?;
            let mname_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: mname_id, value: super::ConstValue::String(method.clone()) })?;
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(2 + arguments.len());
            arg_vals.push(recv_val);
            arg_vals.push(mname_id);
            for a in arguments.into_iter() { arg_vals.push(self.build_expression(a)?); }
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
        self.emit_instruction(MirInstruction::FutureNew { dst: future_id, value: expression_value })?;
        self.variable_map.insert(variable.clone(), future_id);
        Ok(future_id)
    }

    // Await: insert Safepoint before/after and emit Await
    pub(super) fn build_await_expression(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        let future_value = self.build_expression(expression)?;
        self.emit_instruction(MirInstruction::Safepoint)?;
        let result_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Await { dst: result_id, future: future_value })?;
        self.emit_instruction(MirInstruction::Safepoint)?;
        Ok(result_id)
    }

    // me: resolve to param if present; else symbolic const (stable mapping)
    pub(super) fn build_me_expression(&mut self) -> Result<ValueId, String> {
        if let Some(id) = self.variable_map.get("me").cloned() { return Ok(id); }
        let me_value = self.value_gen.next();
        let me_tag = if let Some(ref cls) = self.current_static_box { cls.clone() } else { "__me__".to_string() };
        self.emit_instruction(MirInstruction::Const { dst: me_value, value: ConstValue::String(me_tag) })?;
        self.variable_map.insert("me".to_string(), me_value);
        Ok(me_value)
    }
}

// Local helper for if-statement analysis
fn extract_assigned_var(ast: &ASTNode) -> Option<String> {
    match ast {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() { Some(name.clone()) } else { None }
        }
        ASTNode::Program { statements, .. } => statements.last().and_then(|st| extract_assigned_var(st)),
        ASTNode::If { then_body, else_body, .. } => {
            // Look into nested if: if both sides assign the same variable, propagate that name upward.
            let then_prog = ASTNode::Program { statements: then_body.clone(), span: crate::ast::Span::unknown() };
            let tvar = extract_assigned_var(&then_prog);
            let evar = else_body.as_ref().and_then(|eb| {
                let ep = ASTNode::Program { statements: eb.clone(), span: crate::ast::Span::unknown() };
                extract_assigned_var(&ep)
            });
            match (tvar, evar) {
                (Some(tv), Some(ev)) if tv == ev => Some(tv),
                _ => None,
            }
        }
        _ => None,
    }
}
