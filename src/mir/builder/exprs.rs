// Expression lowering split from builder.rs to keep files lean
use super::{MirInstruction, ConstValue, BasicBlockId, ValueId};
use crate::ast::{ASTNode, LiteralValue};

impl super::MirBuilder {
    // Main expression dispatcher
    pub(super) fn build_expression_impl(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        match ast {
            ASTNode::Literal { value, .. } => self.build_literal(value),

            ASTNode::BinaryOp { left, operator, right, .. } =>
                self.build_binary_op(*left, operator, *right),

            ASTNode::UnaryOp { operator, operand, .. } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            }

            ASTNode::Variable { name, .. } => self.build_variable_access(name.clone()),

            ASTNode::Me { .. } => self.build_me_expression(),

            ASTNode::MethodCall { object, method, arguments, .. } => {
                if (method == "is" || method == "as") && arguments.len() == 1 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                        let obj_val = self.build_expression_impl(*object.clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if method == "is" { crate::mir::TypeOpKind::Check } else { crate::mir::TypeOpKind::Cast };
                        self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
                        return Ok(dst);
                    }
                }
                self.build_method_call(*object.clone(), method.clone(), arguments.clone())
            }

            ASTNode::FromCall { parent, method, arguments, .. } =>
                self.build_from_expression(parent.clone(), method.clone(), arguments.clone()),

            ASTNode::Assignment { target, value, .. } => {
                if let ASTNode::FieldAccess { object, field, .. } = target.as_ref() {
                    self.build_field_assignment(*object.clone(), field.clone(), *value.clone())
                } else if let ASTNode::Variable { name, .. } = target.as_ref() {
                    self.build_assignment(name.clone(), *value.clone())
                } else {
                    Err("Complex assignment targets not yet supported".to_string())
                }
            }

            ASTNode::FunctionCall { name, arguments, .. } =>
                self.build_function_call(name.clone(), arguments.clone()),

            ASTNode::Call { callee, arguments, .. } => {
                let mut arg_ids: Vec<ValueId> = Vec::new();
                let callee_id = self.build_expression_impl(*callee.clone())?;
                for a in arguments { arg_ids.push(self.build_expression_impl(a)?); }
                let dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::Call { dst: Some(dst), func: callee_id, args: arg_ids, effects: crate::mir::EffectMask::PURE })?;
                Ok(dst)
            }

            ASTNode::QMarkPropagate { expression, .. } => {
                let res_val = self.build_expression_impl(*expression.clone())?;
                let ok_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::PluginInvoke { dst: Some(ok_id), box_val: res_val, method: "isOk".to_string(), args: vec![], effects: crate::mir::EffectMask::PURE })?;
                let then_block = self.block_gen.next();
                let else_block = self.block_gen.next();
                self.emit_instruction(MirInstruction::Branch { condition: ok_id, then_bb: then_block, else_bb: else_block })?;
                self.start_new_block(then_block)?;
                self.emit_instruction(MirInstruction::Return { value: Some(res_val) })?;
                self.start_new_block(else_block)?;
                let val_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::PluginInvoke { dst: Some(val_id), box_val: res_val, method: "getValue".to_string(), args: vec![], effects: crate::mir::EffectMask::PURE })?;
                Ok(val_id)
            }

            ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                let scr_val = self.build_expression_impl(*scrutinee.clone())?;
                let merge_block: BasicBlockId = self.block_gen.next();
                let result_val = self.value_gen.next();
                let mut phi_inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
                let mut next_block = self.block_gen.next();
                self.start_new_block(next_block)?;
                for (i, (label, arm_expr)) in arms.iter().enumerate() {
                    let then_block = self.block_gen.next();
                    if let LiteralValue::String(ref s) = label {
                        let lit_id = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Const { dst: lit_id, value: ConstValue::String(s.clone()) })?;
                        let cond_id = self.value_gen.next();
                        self.emit_instruction(crate::mir::MirInstruction::Compare { dst: cond_id, op: crate::mir::CompareOp::Eq, lhs: scr_val, rhs: lit_id })?;
                        self.emit_instruction(crate::mir::MirInstruction::Branch { condition: cond_id, then_bb: then_block, else_bb: next_block })?;
                        self.start_new_block(then_block)?;
                        let then_val = self.build_expression_impl(arm_expr.clone())?;
                        phi_inputs.push((then_block, then_val));
                        self.emit_instruction(crate::mir::MirInstruction::Jump { target: merge_block })?;
                        if i < arms.len() - 1 { let b = self.block_gen.next(); self.start_new_block(b)?; next_block = b; }
                    }
                }
                let else_block_id = next_block; self.start_new_block(else_block_id)?;
                let else_val = self.build_expression_impl(*else_expr.clone())?;
                phi_inputs.push((else_block_id, else_val));
                self.emit_instruction(crate::mir::MirInstruction::Jump { target: merge_block })?;
                self.start_new_block(merge_block)?;
                self.emit_instruction(crate::mir::MirInstruction::Phi { dst: result_val, inputs: phi_inputs })?;
                Ok(result_val)
            }

            ASTNode::Lambda { params, body, .. } => {
                use std::collections::HashSet;
                let mut used: HashSet<String> = HashSet::new();
                let mut locals: HashSet<String> = HashSet::new(); for p in &params { locals.insert(p.clone()); }
                fn collect_vars(ast: &ASTNode, used: &mut std::collections::HashSet<String>, locals: &mut std::collections::HashSet<String>) {
                    match ast {
                        ASTNode::Variable { name, .. } => { if !locals.contains(name) { used.insert(name.clone()); } }
                        ASTNode::Assignment { target, value, .. } => { collect_vars(target, used, locals); collect_vars(value, used, locals); }
                        ASTNode::BinaryOp { left, right, .. } => { collect_vars(left, used, locals); collect_vars(right, used, locals); }
                        ASTNode::UnaryOp { operand, .. } => { collect_vars(operand, used, locals); }
                        ASTNode::MethodCall { object, arguments, .. } => { collect_vars(object, used, locals); for a in arguments { collect_vars(a, used, locals); } }
                        ASTNode::FunctionCall { arguments, .. } => { for a in arguments { collect_vars(a, used, locals); } }
                        ASTNode::Call { callee, arguments, .. } => { collect_vars(callee, used, locals); for a in arguments { collect_vars(a, used, locals); } }
                        ASTNode::FieldAccess { object, .. } => { collect_vars(object, used, locals); }
                        ASTNode::New { arguments, .. } => { for a in arguments { collect_vars(a, used, locals); } }
                        ASTNode::If { condition, then_body, else_body, .. } => {
                            collect_vars(condition, used, locals);
                            for st in then_body { collect_vars(st, used, locals); }
                            if let Some(eb) = else_body { for st in eb { collect_vars(st, used, locals); } }
                        }
                        ASTNode::Loop { condition, body, .. } => { collect_vars(condition, used, locals); for st in body { collect_vars(st, used, locals); } }
                        ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                            for st in try_body { collect_vars(st, used, locals); }
                            for c in catch_clauses { for st in &c.body { collect_vars(st, used, locals); } }
                            if let Some(fb) = finally_body { for st in fb { collect_vars(st, used, locals); } }
                        }
                        ASTNode::Throw { expression, .. } => { collect_vars(expression, used, locals); }
                        ASTNode::Print { expression, .. } => { collect_vars(expression, used, locals); }
                        ASTNode::Return { value, .. } => { if let Some(v) = value { collect_vars(v, used, locals); } }
                        ASTNode::AwaitExpression { expression, .. } => { collect_vars(expression, used, locals); }
                        ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                            collect_vars(scrutinee, used, locals);
                            for (_, e) in arms { collect_vars(e, used, locals); }
                            collect_vars(else_expr, used, locals);
                        }
                        ASTNode::Program { statements, .. } => { for st in statements { collect_vars(st, used, locals); } }
                        ASTNode::FunctionDeclaration { params, body, .. } => {
                            let mut inner = locals.clone();
                            for p in params { inner.insert(p.clone()); }
                            for st in body { collect_vars(st, used, &mut inner); }
                        }
                        _ => {}
                    }
                }
                for st in body.iter() { collect_vars(st, &mut used, &mut locals); }
                let mut captures: Vec<(String, ValueId)> = Vec::new();
                for name in used.into_iter() {
                    if let Some(&vid) = self.variable_map.get(&name) { captures.push((name, vid)); }
                }
                let me = self.variable_map.get("me").copied();
                let dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::FunctionNew { dst, params: params.clone(), body: body.clone(), captures, me })?;
                self.value_types.insert(dst, crate::mir::MirType::Box("FunctionBox".to_string()));
                Ok(dst)
            }

            ASTNode::Return { value, .. } => self.build_return_statement(value.clone()),

            ASTNode::Local { variables, initial_values, .. } =>
                self.build_local_statement(variables.clone(), initial_values.clone()),

            ASTNode::BoxDeclaration { name, methods, is_static, fields, constructors, weak_fields, .. } => {
                if is_static && name == "Main" {
                    self.build_static_main_box(name.clone(), methods.clone())
                } else {
                    self.user_defined_boxes.insert(name.clone());
                    self.build_box_declaration(name.clone(), methods.clone(), fields.clone(), weak_fields.clone())?;
                    for (ctor_key, ctor_ast) in constructors.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, .. } = ctor_ast {
                            let func_name = format!("{}.{}", name, ctor_key);
                            self.lower_method_as_function(func_name, name.clone(), params.clone(), body.clone())?;
                        }
                    }
                    for (method_name, method_ast) in methods.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, is_static, .. } = method_ast {
                            if !is_static {
                                let func_name = format!("{}.{}{}", name, method_name, format!("/{}", params.len()));
                                self.lower_method_as_function(func_name, name.clone(), params.clone(), body.clone())?;
                            }
                        }
                    }
                    let void_val = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const { dst: void_val, value: ConstValue::Void })?;
                    Ok(void_val)
                }
            }

            ASTNode::FieldAccess { object, field, .. } =>
                self.build_field_access(*object.clone(), field.clone()),

            ASTNode::New { class, arguments, .. } =>
                self.build_new_expression(class.clone(), arguments.clone()),

            ASTNode::Nowait { variable, expression, .. } =>
                self.build_nowait_statement(variable.clone(), *expression.clone()),

            ASTNode::AwaitExpression { expression, .. } =>
                self.build_await_expression(*expression.clone()),

            ASTNode::Include { filename, .. } => {
                let mut path = super::utils::resolve_include_path_builder(&filename);
                if std::path::Path::new(&path).is_dir() {
                    path = format!("{}/index.nyash", path.trim_end_matches('/'));
                } else if std::path::Path::new(&path).extension().is_none() {
                    path.push_str(".nyash");
                }
                if self.include_loading.contains(&path) {
                    return Err(format!("Circular include detected: {}", path));
                }
                if let Some(name) = self.include_box_map.get(&path).cloned() {
                    return self.build_new_expression(name, vec![]);
                }
                self.include_loading.insert(path.clone());
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Include read error '{}': {}", filename, e))?;
                let included_ast = crate::parser::NyashParser::parse_from_string(&content)
                    .map_err(|e| format!("Include parse error '{}': {:?}", filename, e))?;
                let mut box_name: Option<String> = None;
                if let ASTNode::Program { statements, .. } = &included_ast {
                    for st in statements {
                        if let ASTNode::BoxDeclaration { name, is_static, .. } = st { if *is_static { box_name = Some(name.clone()); break; } }
                    }
                }
                let bname = box_name.ok_or_else(|| format!("Include target '{}' has no static box", filename))?;
                let _ = self.build_expression_impl(included_ast)?;
                self.include_loading.remove(&path);
                self.include_box_map.insert(path.clone(), bname.clone());
                self.build_new_expression(bname, vec![])
            }

            _ => Err(format!("Unsupported AST node type: {:?}", ast)),
        }
    }
}
