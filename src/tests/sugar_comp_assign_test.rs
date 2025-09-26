use crate::ast::{ASTNode, BinaryOperator};
use crate::parser::entry_sugar::parse_with_sugar_level;
use crate::syntax::sugar_config::SugarLevel;

#[test]
fn compound_assign_rewrites_to_binaryop() {
    let code = "x = 1\nx += 2\n";
    let ast = parse_with_sugar_level(code, SugarLevel::Basic).expect("parse ok");

    let program = match ast {
        ASTNode::Program { statements, .. } => statements,
        other => panic!("expected program, got {:?}", other),
    };
    assert_eq!(program.len(), 2);

    let assign = match &program[1] {
        ASTNode::Assignment { target, value, .. } => (target, value),
        other => panic!("expected assignment, got {:?}", other),
    };
    match assign.0.as_ref() {
        ASTNode::Variable { name, .. } => assert_eq!(name, "x"),
        other => panic!("expected target var, got {:?}", other),
    }

    match assign.1.as_ref() {
        ASTNode::BinaryOp {
            operator,
            left,
            right,
            ..
        } => {
            assert!(matches!(operator, BinaryOperator::Add));
            match left.as_ref() {
                ASTNode::Variable { name, .. } => assert_eq!(name, "x"),
                other => panic!("expected left x, got {:?}", other),
            }
            match right.as_ref() {
                ASTNode::Literal { .. } => {}
                other => panic!("expected right literal, got {:?}", other),
            }
        }
        other => panic!("expected BinaryOp, got {:?}", other),
    }
}
