// Expression lowering split from builder.rs to keep files lean
use super::{MirInstruction, ValueId};
use crate::ast::{ASTNode, AssignStmt, ReturnStmt, BinaryExpr, CallExpr, MethodCallExpr, FieldAccessExpr};

impl super::MirBuilder {
    // Main expression dispatcher
    pub(super) fn build_expression_impl(&mut self, ast: ASTNode) -> Result<ValueId, String> {
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
                    crate::ast::UnaryOperator::BitNot => "~".to_string(),
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

            ASTNode::ExternCCall { symbol, arguments, .. } => {
                // Build arguments
                let mut args: Vec<ValueId> = Vec::with_capacity(arguments.len());
                for a in arguments {
                    args.push(self.build_expression_impl(a)?);
                }
                // Normalize args into SSA locals
                let full = format!("ffi.dynamic.{}", symbol);
                let effects = crate::mir::builder::calls::extern_calls::compute_extern_effects("ffi.dynamic", &symbol);
                let dst = self.value_gen.next();
                self.emit_call_with_guard(
                    Some(dst),
                    ValueId::new(0),
                    crate::mir::Callee::Extern(full),
                    args,
                    effects,
                )?;
                Ok(dst)
            }

            ASTNode::Call {
                callee, arguments, ..
            } => self.build_indirect_call_expression(*callee.clone(), arguments.clone()),

            ASTNode::QMarkPropagate { expression, .. } => {
                self.build_qmark_propagate_expression(*expression.clone())
            }

            ASTNode::MatchExpr {
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
                // Dev lint: suggest flow for fieldless static boxes (staged)
                if is_static
                    && std::env::var("NYASH_LINT_STATIC_TO_FLOW").ok().as_deref() == Some("1")
                {
                    let has_fields = !fields.is_empty() || !weak_fields.is_empty();
                    let has_ctors = !constructors.is_empty();
                    if !has_fields && !has_ctors && name != "Main" {
                        eprintln!(
                            "[lint][flow] static box '{}' has no fields/ctors; consider 'flow {} {{ ... }}'",
                            name, name
                        );
                    }
                }
                if is_static {
                    // Record static/flow box name for later checks (e.g., forbid new Flow())
                    self.static_box_names.insert(name.clone());
                }
                if is_static && name == "Main" {
                    // Special entry box: materialize main() as Program and lower others as static functions
                    self.build_static_main_box(name.clone(), methods.clone())
                } else if is_static {
                    // Generic static box: lower all static methods into standalone MIR functions (BoxName.method/N)
                    self.user_defined_boxes.insert(name.clone());
                    for (method_name, method_ast) in methods.clone() {
                        if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast {
                            let func_name = format!(
                                "{}.{}{}",
                                name,
                                method_name,
                                format!("/{}", params.len())
                            );
                            self.lower_static_method_as_function(
                                func_name,
                                params.clone(),
                                body.clone(),
                            )?;
                            // Index static method for fallback resolution of bare calls
                            self.method_index.register_static_method(
                                method_name.clone(),
                                name.clone(),
                                params.len()
                            );
                        }
                    }
                    // Return void for declaration context
                    let void_val = crate::mir::builder::emission::constant::emit_void(self);
                    Ok(void_val)
                } else {
                    // Instance box: register type and lower instance methods/ctors as functions
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
                    let void_val = crate::mir::builder::emission::constant::emit_void(self);
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
                self.emit_instruction(MirInstruction::NewBox { dst: arr_id, box_type: "ArrayBox".to_string(), args: vec![], auto_birth: None })?;
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
                self.emit_instruction(MirInstruction::NewBox { dst: map_id, box_type: "MapBox".to_string(), args: vec![], auto_birth: None })?;
                for (k, expr) in entries {
                    // const string key
                    let k_id = crate::mir::builder::emission::constant::emit_string(self, k);
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

            

            ASTNode::Program { statements, .. } => self.cf_block(statements.clone()),
            ASTNode::ScopeBox { body, .. } => self.cf_block(body.clone()),

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
