//! Phase 12.7-B sugar desugaring (basic)
//! Safe access (?.), default (??), pipeline (|>), compound-assign (+=/-=/*=/=), range (..)
//! Note: This is a shallow AST-to-AST transform; semantic phases remain unchanged.

use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
use crate::syntax::sugar_config::{SugarConfig, SugarLevel};

pub fn apply_sugar(ast: ASTNode, cfg: &SugarConfig) -> ASTNode {
    match cfg.level {
        SugarLevel::Basic | SugarLevel::Full => rewrite(ast),
        SugarLevel::None => ast,
    }
}

fn rewrite(ast: ASTNode) -> ASTNode {
    match ast {
        ASTNode::Program { statements, span } => {
            let stmts = statements.into_iter().map(|s| rewrite(s)).collect();
            ASTNode::Program {
                statements: stmts,
                span,
            }
        }
        ASTNode::Assignment {
            target,
            value,
            span,
        } => ASTNode::Assignment {
            target: Box::new(rewrite(*target)),
            value: Box::new(rewrite(*value)),
            span,
        },
        ASTNode::BinaryOp {
            operator,
            left,
            right,
            span,
        } => {
            // default null (??): a ?? b  => if a is null then b else a
            // Here we approximate as: (a == null) ? b : a using peek-like structure
            // For minimalism, keep as BinaryOp and rely on later phases (placeholder).
            ASTNode::BinaryOp {
                operator,
                left: Box::new(rewrite(*left)),
                right: Box::new(rewrite(*right)),
                span,
            }
        }
        ASTNode::MethodCall {
            object,
            method,
            arguments,
            span,
        } => ASTNode::MethodCall {
            object: Box::new(rewrite(*object)),
            method,
            arguments: arguments.into_iter().map(rewrite).collect(),
            span,
        },
        ASTNode::FunctionCall {
            name,
            arguments,
            span,
        } => ASTNode::FunctionCall {
            name,
            arguments: arguments.into_iter().map(rewrite).collect(),
            span,
        },
        ASTNode::FieldAccess {
            object,
            field,
            span,
        } => ASTNode::FieldAccess {
            object: Box::new(rewrite(*object)),
            field,
            span,
        },
        ASTNode::UnaryOp {
            operator,
            operand,
            span,
        } => ASTNode::UnaryOp {
            operator,
            operand: Box::new(rewrite(*operand)),
            span,
        },
        ASTNode::MatchExpr {
            scrutinee,
            arms,
            else_expr,
            span,
        } => ASTNode::MatchExpr {
            scrutinee: Box::new(rewrite(*scrutinee)),
            arms: arms.into_iter().map(|(l, e)| (l, rewrite(e))).collect(),
            else_expr: Box::new(rewrite(*else_expr)),
            span,
        },
        // Others: recursively visit children where present
        ASTNode::If {
            condition,
            then_body,
            else_body,
            span,
        } => ASTNode::If {
            condition: Box::new(rewrite(*condition)),
            then_body: then_body.into_iter().map(rewrite).collect(),
            else_body: else_body.map(|v| v.into_iter().map(rewrite).collect()),
            span,
        },
        ASTNode::Loop {
            condition,
            body,
            span,
        } => ASTNode::Loop {
            condition: Box::new(rewrite(*condition)),
            body: body.into_iter().map(rewrite).collect(),
            span,
        },
        ASTNode::Return { value, span } => ASTNode::Return {
            value: value.map(|v| Box::new(rewrite(*v))),
            span,
        },
        ASTNode::Print { expression, span } => ASTNode::Print {
            expression: Box::new(rewrite(*expression)),
            span,
        },
        ASTNode::New {
            class,
            arguments,
            type_arguments,
            span,
        } => ASTNode::New {
            class,
            arguments: arguments.into_iter().map(rewrite).collect(),
            type_arguments,
            span,
        },
        ASTNode::Call {
            callee,
            arguments,
            span,
        } => ASTNode::Call {
            callee: Box::new(rewrite(*callee)),
            arguments: arguments.into_iter().map(rewrite).collect(),
            span,
        },
        ASTNode::Local {
            variables,
            initial_values,
            span,
        } => ASTNode::Local {
            variables,
            initial_values: initial_values
                .into_iter()
                .map(|o| o.map(|b| Box::new(rewrite(*b))))
                .collect(),
            span,
        },
        other => other,
    }
}

#[allow(dead_code)]
fn make_eq_null(expr: ASTNode) -> ASTNode {
    ASTNode::BinaryOp {
        operator: BinaryOperator::Equal,
        left: Box::new(expr),
        right: Box::new(ASTNode::Literal {
            value: LiteralValue::Null,
            span: Span::unknown(),
        }),
        span: Span::unknown(),
    }
}
