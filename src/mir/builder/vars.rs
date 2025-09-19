use crate::ast::ASTNode;
use std::collections::HashSet;

/// Collect free variables used in `node` into `used`, excluding names present in `locals`.
/// `locals` is updated as new local declarations are encountered.
#[allow(dead_code)]
pub(super) fn collect_free_vars(
    node: &ASTNode,
    used: &mut HashSet<String>,
    locals: &mut HashSet<String>,
) {
    match node {
        ASTNode::Variable { name, .. } => {
            if name != "me" && name != "this" && !locals.contains(name) {
                used.insert(name.clone());
            }
        }
        ASTNode::Local { variables, .. } => {
            for v in variables {
                locals.insert(v.clone());
            }
        }
        ASTNode::Assignment { target, value, .. } => {
            collect_free_vars(target, used, locals);
            collect_free_vars(value, used, locals);
        }
        ASTNode::BinaryOp { left, right, .. } => {
            collect_free_vars(left, used, locals);
            collect_free_vars(right, used, locals);
        }
        ASTNode::UnaryOp { operand, .. } => {
            collect_free_vars(operand, used, locals);
        }
        ASTNode::MethodCall {
            object, arguments, ..
        } => {
            collect_free_vars(object, used, locals);
            for a in arguments {
                collect_free_vars(a, used, locals);
            }
        }
        ASTNode::FunctionCall { arguments, .. } => {
            for a in arguments {
                collect_free_vars(a, used, locals);
            }
        }
        ASTNode::Call {
            callee, arguments, ..
        } => {
            collect_free_vars(callee, used, locals);
            for a in arguments {
                collect_free_vars(a, used, locals);
            }
        }
        ASTNode::FieldAccess { object, .. } => {
            collect_free_vars(object, used, locals);
        }
        ASTNode::New { arguments, .. } => {
            for a in arguments {
                collect_free_vars(a, used, locals);
            }
        }
        ASTNode::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_free_vars(condition, used, locals);
            for st in then_body {
                collect_free_vars(st, used, locals);
            }
            if let Some(eb) = else_body {
                for st in eb {
                    collect_free_vars(st, used, locals);
                }
            }
        }
        ASTNode::Loop {
            condition, body, ..
        } => {
            collect_free_vars(condition, used, locals);
            for st in body {
                collect_free_vars(st, used, locals);
            }
        }
        ASTNode::TryCatch {
            try_body,
            catch_clauses,
            finally_body,
            ..
        } => {
            for st in try_body {
                collect_free_vars(st, used, locals);
            }
            for c in catch_clauses {
                for st in &c.body {
                    collect_free_vars(st, used, locals);
                }
            }
            if let Some(fb) = finally_body {
                for st in fb {
                    collect_free_vars(st, used, locals);
                }
            }
        }
        ASTNode::Throw { expression, .. } => {
            collect_free_vars(expression, used, locals);
        }
        ASTNode::Print { expression, .. } => {
            collect_free_vars(expression, used, locals);
        }
        ASTNode::Return { value, .. } => {
            if let Some(v) = value {
                collect_free_vars(v, used, locals);
            }
        }
        ASTNode::AwaitExpression { expression, .. } => {
            collect_free_vars(expression, used, locals);
        }
        ASTNode::PeekExpr {
            scrutinee,
            arms,
            else_expr,
            ..
        } => {
            collect_free_vars(scrutinee, used, locals);
            for (_, e) in arms {
                collect_free_vars(e, used, locals);
            }
            collect_free_vars(else_expr, used, locals);
        }
        ASTNode::Program { statements, .. } => {
            for st in statements {
                collect_free_vars(st, used, locals);
            }
        }
        ASTNode::FunctionDeclaration { params, body, .. } => {
            let mut inner = locals.clone();
            for p in params {
                inner.insert(p.clone());
            }
            for st in body {
                collect_free_vars(st, used, &mut inner);
            }
        }
        _ => {}
    }
}
