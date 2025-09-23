use crate::ast::{ASTNode, LiteralValue};
use crate::parser::entry_sugar::parse_with_sugar_level;
use crate::syntax::sugar_config::SugarLevel;

#[test]
fn coalesce_peek_rewrite() {
    let code = "x = a ?? b\n";
    let ast = parse_with_sugar_level(code, SugarLevel::Basic).expect("parse ok");

    let program = match ast {
        ASTNode::Program { statements, .. } => statements,
        other => panic!("expected program, got {:?}", other),
    };
    let assign = match &program[0] {
        ASTNode::Assignment { target, value, .. } => (target, value),
        other => panic!("expected assignment, got {:?}", other),
    };
    match assign.0.as_ref() {
        ASTNode::Variable { name, .. } => assert_eq!(name, "x"),
        _ => panic!("target not x"),
    }

    match assign.1.as_ref() {
        ASTNode::MatchExpr {
            scrutinee,
            arms,
            else_expr,
            ..
        } => {
            match scrutinee.as_ref() {
                ASTNode::Variable { name, .. } => assert_eq!(name, "a"),
                _ => panic!("scrutinee not a"),
            }
            assert_eq!(arms.len(), 1);
            assert!(matches!(arms[0].0, LiteralValue::Null));
            match &arms[0].1 {
                ASTNode::Variable { name, .. } => assert_eq!(name, "b"),
                _ => panic!("rhs not b"),
            }
            match else_expr.as_ref() {
                ASTNode::Variable { name, .. } => assert_eq!(name, "a"),
                _ => panic!("else not a"),
            }
        }
        other => panic!("expected MatchExpr, got {:?}", other),
    }
}
