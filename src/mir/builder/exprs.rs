// Expression lowering split from builder.rs to keep files lean
use super::{ConstValue, MirInstruction, ValueId};
use crate::ast::{ASTNode, AssignStmt, ReturnStmt, BinaryExpr, CallExpr, MethodCallExpr, FieldAccessExpr};

impl super::MirBuilder {
    // Main expression dispatcher
    pub(super) fn build_expression_impl(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        if matches!(
            ast,
            ASTNode::Program { .. }
                | ASTNode::If { .. }
                | ASTNode::Loop { .. }
                | ASTNode::TryCatch { .. }
                | ASTNode::Throw { .. }
        ) {
            return self.build_expression_impl_legacy(ast);
        }
        match ast {
            ASTNode::Literal { value, .. } => self.build_literal(value),

            node @ ASTNode::BinaryOp { .. } => {
                // Use BinaryExpr for clear destructuring (no behavior change)
                let e = BinaryExpr::try_from(node).expect("ASTNode::BinaryOp must convert");
                self.build_binary_op(*e.left, e.operator, *e.right)
            }

            ASTNode::UnaryOp {
                operator, operand, ..
            } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            }

            ASTNode::Variable { name, .. } => self.build_variable_access(name.clone()),

            ASTNode::Me { .. } => self.build_me_expression(),

            node @ ASTNode::MethodCall { .. } => {
                let m = MethodCallExpr::try_from(node).expect("ASTNode::MethodCall must convert");
                if (m.method == "is" || m.method == "as") && m.arguments.len() == 1 {
                    if let Some(type_name) = Self::extract_string_literal(&m.arguments[0]) {
                        let obj_val = self.build_expression_impl(*m.object.clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if m.method == "is" { crate::mir::TypeOpKind::Check } else { crate::mir::TypeOpKind::Cast };
                        self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
                        return Ok(dst);
                    }
                }
                self.build_method_call(*m.object.clone(), m.method.clone(), m.arguments.clone())
            }

            ASTNode::FromCall {
                parent,
                method,
                arguments,
                ..
            } => self.build_from_expression(parent.clone(), method.clone(), arguments.clone()),

            node @ ASTNode::Assignment { .. } => {
                // Use AssignStmt wrapper for clearer destructuring (no behavior change)
                let stmt = AssignStmt::try_from(node).expect("ASTNode::Assignment must convert");
                if let ASTNode::FieldAccess { object, field, .. } = stmt.target.as_ref() {
                    self.build_field_assignment(*object.clone(), field.clone(), *stmt.value.clone())
                } else if let ASTNode::Variable { name, .. } = stmt.target.as_ref() {
                    self.build_assignment(name.clone(), *stmt.value.clone())
                } else {
                    Err("Complex assignment targets not yet supported".to_string())
                }
            }

            node @ ASTNode::FunctionCall { .. } => {
                let c = CallExpr::try_from(node).expect("ASTNode::FunctionCall must convert");
                self.build_function_call(c.name, c.arguments)
            }

            ASTNode::Call {
                callee, arguments, ..
            } => self.build_indirect_call_expression(*callee.clone(), arguments.clone()),

            ASTNode::QMarkPropagate { expression, .. } => {
                self.build_qmark_propagate_expression(*expression.clone())
            }

            ASTNode::PeekExpr {
                scrutinee,
                arms,
                else_expr,
                ..
            } => self.build_peek_expression(*scrutinee.clone(), arms.clone(), *else_expr.clone()),

            ASTNode::Lambda { params, body, .. } => {
                self.build_lambda_expression(params.clone(), body.clone())
            }

            node @ ASTNode::Return { .. } => {
                // Use ReturnStmt wrapper for consistent access (no behavior change)
                let stmt = ReturnStmt::try_from(node).expect("ASTNode::Return must convert");
                self.build_return_statement(stmt.value.clone())
            }

            // Control flow: break/continue are handled inside LoopBuilder context
            ASTNode::Local {
                variables,
                initial_values,
                ..
            } => self.build_local_statement(variables.clone(), initial_values.clone()),

            ASTNode::BoxDeclaration {
                name,
                methods,
                is_static,
                fields,
                constructors,
                weak_fields,
                ..
            } => {
                if is_static && name == "Main" {
                    self.build_static_main_box(name.clone(), methods.clone())
                } else {
                    self.user_defined_boxes.insert(name.clone());
                    self.build_box_declaration(
                        name.clone(),
                        methods.clone(),
                        fields.clone(),
                        weak_fields.clone(),
                    )?;
                    for (ctor_key, ctor_ast) in constructors.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, .. } = ctor_ast {
                            let func_name = format!("{}.{}", name, ctor_key);
                            self.lower_method_as_function(
                                func_name,
                                name.clone(),
                                params.clone(),
                                body.clone(),
                            )?;
                        }
                    }
                    for (method_name, method_ast) in methods.clone() {
                        if let ASTNode::FunctionDeclaration {
                            params,
                            body,
                            is_static,
                            ..
                        } = method_ast
                        {
                            if !is_static {
                                let func_name = format!(
                                    "{}.{}{}",
                                    name,
                                    method_name,
                                    format!("/{}", params.len())
                                );
                                self.lower_method_as_function(
                                    func_name,
                                    name.clone(),
                                    params.clone(),
                                    body.clone(),
                                )?;
                            }
                        }
                    }
                    let void_val = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_val,
                        value: ConstValue::Void,
                    })?;
                    Ok(void_val)
                }
            }

            node @ ASTNode::FieldAccess { .. } => {
                let f = FieldAccessExpr::try_from(node).expect("ASTNode::FieldAccess must convert");
                self.build_field_access(*f.object.clone(), f.field.clone())
            }

            ASTNode::New {
                class, arguments, ..
            } => self.build_new_expression(class.clone(), arguments.clone()),

            ASTNode::ArrayLiteral { elements, .. } => {
                let arr_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::NewBox {
                    dst: arr_id,
                    box_type: "ArrayBox".to_string(),
                    args: vec![],
                })?;
                for e in elements {
                    let v = self.build_expression_impl(e)?;
                    self.emit_instruction(MirInstruction::BoxCall {
                        dst: None,
                        box_val: arr_id,
                        method: "push".to_string(),
                        method_id: None,
                        args: vec![v],
                        effects: super::EffectMask::MUT,
                    })?;
                }
                Ok(arr_id)
            }
            ASTNode::MapLiteral { entries, .. } => {
                let map_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::NewBox {
                    dst: map_id,
                    box_type: "MapBox".to_string(),
                    args: vec![],
                })?;
                for (k, expr) in entries {
                    // const string key
                    let k_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: k_id,
                        value: ConstValue::String(k),
                    })?;
                    let v_id = self.build_expression_impl(expr)?;
                    self.emit_instruction(MirInstruction::BoxCall {
                        dst: None,
                        box_val: map_id,
                        method: "set".to_string(),
                        method_id: None,
                        args: vec![k_id, v_id],
                        effects: super::EffectMask::MUT,
                    })?;
                }
                Ok(map_id)
            }

            ASTNode::Nowait {
                variable,
                expression,
                ..
            } => self.build_nowait_statement(variable.clone(), *expression.clone()),

            ASTNode::AwaitExpression { expression, .. } => {
                self.build_await_expression(*expression.clone())
            }

            ASTNode::Include { filename, .. } => self.build_include_expression(filename.clone()),

            ASTNode::Program { statements, .. } => self.cf_block(statements.clone()),

            ASTNode::Print { expression, .. } => self.build_print_statement(*expression.clone()),

            ASTNode::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let else_ast = if let Some(else_statements) = else_body {
                    Some(ASTNode::Program {
                        statements: else_statements.clone(),
                        span: crate::ast::Span::unknown(),
                    })
                } else {
                    None
                };
                self.cf_if(
                    *condition.clone(),
                    ASTNode::Program {
                        statements: then_body.clone(),
                        span: crate::ast::Span::unknown(),
                    },
                    else_ast,
                )
            }

            ASTNode::Loop {
                condition, body, ..
            } => self.cf_loop(*condition.clone(), body.clone()),

            ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                ..
            } => self.cf_try_catch(
                try_body.clone(),
                catch_clauses.clone(),
                finally_body.clone(),
            ),

            ASTNode::Throw { expression, .. } => self.cf_throw(*expression.clone()),

            _ => Err(format!("Unsupported AST node type: {:?}", ast)),
        }
    }
}
