use super::ValueId;
use crate::ast::ASTNode;

impl super::MirBuilder {
    // Lambda lowering to NewClosure
    pub(super) fn build_lambda_expression(
        &mut self,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        use std::collections::HashSet;
        let mut used: HashSet<String> = HashSet::new();
        let mut locals: HashSet<String> = HashSet::new();
        for p in &params {
            locals.insert(p.clone());
        }
        fn collect_vars(
            ast: &ASTNode,
            used: &mut std::collections::HashSet<String>,
            locals: &mut std::collections::HashSet<String>,
        ) {
            match ast {
                ASTNode::Variable { name, .. } => {
                    if !locals.contains(name) {
                        used.insert(name.clone());
                    }
                }
                ASTNode::Assignment { target, value, .. } => {
                    collect_vars(target, used, locals);
                    collect_vars(value, used, locals);
                }
                ASTNode::BinaryOp { left, right, .. } => {
                    collect_vars(left, used, locals);
                    collect_vars(right, used, locals);
                }
                ASTNode::UnaryOp { operand, .. } => {
                    collect_vars(operand, used, locals);
                }
                ASTNode::MethodCall {
                    object, arguments, ..
                } => {
                    collect_vars(object, used, locals);
                    for a in arguments {
                        collect_vars(a, used, locals);
                    }
                }
                ASTNode::FunctionCall { arguments, .. } => {
                    for a in arguments {
                        collect_vars(a, used, locals);
                    }
                }
                ASTNode::Call {
                    callee, arguments, ..
                } => {
                    collect_vars(callee, used, locals);
                    for a in arguments {
                        collect_vars(a, used, locals);
                    }
                }
                ASTNode::FieldAccess { object, .. } => {
                    collect_vars(object, used, locals);
                }
                ASTNode::New { arguments, .. } => {
                    for a in arguments {
                        collect_vars(a, used, locals);
                    }
                }
                ASTNode::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    collect_vars(condition, used, locals);
                    for st in then_body {
                        collect_vars(st, used, locals);
                    }
                    if let Some(eb) = else_body {
                        for st in eb {
                            collect_vars(st, used, locals);
                        }
                    }
                }
                ASTNode::Loop {
                    condition, body, ..
                } => {
                    collect_vars(condition, used, locals);
                    for st in body {
                        collect_vars(st, used, locals);
                    }
                }
                ASTNode::TryCatch {
                    try_body,
                    catch_clauses,
                    finally_body,
                    ..
                } => {
                    for st in try_body {
                        collect_vars(st, used, locals);
                    }
                    for c in catch_clauses {
                        for st in &c.body {
                            collect_vars(st, used, locals);
                        }
                    }
                    if let Some(fb) = finally_body {
                        for st in fb {
                            collect_vars(st, used, locals);
                        }
                    }
                }
                ASTNode::Throw { expression, .. } => {
                    collect_vars(expression, used, locals);
                }
                ASTNode::Print { expression, .. } => {
                    collect_vars(expression, used, locals);
                }
                ASTNode::Return { value, .. } => {
                    if let Some(v) = value {
                        collect_vars(v, used, locals);
                    }
                }
                ASTNode::AwaitExpression { expression, .. } => {
                    collect_vars(expression, used, locals);
                }
                ASTNode::MatchExpr {
                    scrutinee,
                    arms,
                    else_expr,
                    ..
                } => {
                    collect_vars(scrutinee, used, locals);
                    for (_, e) in arms {
                        collect_vars(e, used, locals);
                    }
                    collect_vars(else_expr, used, locals);
                }
                ASTNode::Program { statements, .. } => {
                    for st in statements {
                        collect_vars(st, used, locals);
                    }
                }
                ASTNode::FunctionDeclaration { params, body, .. } => {
                    let mut inner = locals.clone();
                    for p in params {
                        inner.insert(p.clone());
                    }
                    for st in body {
                        collect_vars(st, used, &mut inner);
                    }
                }
                _ => {}
            }
        }
        for st in body.iter() {
            collect_vars(st, &mut used, &mut locals);
        }
        let mut captures: Vec<(String, ValueId)> = Vec::new();
        for name in used.into_iter() {
            if let Some(&vid) = self.variable_map.get(&name) {
                captures.push((name, vid));
            }
        }
        let me = self.variable_map.get("me").copied();
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::NewClosure {
            dst,
            params: params.clone(),
            body: body.clone(),
            captures,
            me,
        })?;
        self.value_types
            .insert(dst, crate::mir::MirType::Box("FunctionBox".to_string()));
        Ok(dst)
    }
}
