use crate::ast::ASTNode;
use crate::parser::entry_sugar::parse_with_sugar_level;
use crate::syntax::sugar_config::SugarLevel;

#[test]
fn safe_access_field_and_method() {
    let code = "a = user?.profile\nb = user?.m(1)\n";
    let ast = parse_with_sugar_level(code, SugarLevel::Basic).expect("parse ok");

    let program = match ast {
        ASTNode::Program { statements, .. } => statements,
        other => panic!("expected program, got {:?}", other),
    };
    assert_eq!(program.len(), 2);

    // a = user?.profile
    match &program[0] {
        ASTNode::Assignment { value, .. } => match value.as_ref() {
            ASTNode::MatchExpr {
                scrutinee,
                else_expr,
                ..
            } => {
                match scrutinee.as_ref() {
                    ASTNode::Variable { name, .. } => assert_eq!(name, "user"),
                    _ => panic!("scrutinee not user"),
                }
                match else_expr.as_ref() {
                    ASTNode::FieldAccess { object, field, .. } => {
                        match object.as_ref() {
                            ASTNode::Variable { name, .. } => assert_eq!(name, "user"),
                            _ => panic!("object not user"),
                        }
                        assert_eq!(field, "profile");
                    }
                    other => panic!("else not field access, got {:?}", other),
                }
            }
            other => panic!("expected MatchExpr, got {:?}", other),
        },
        other => panic!("expected assignment, got {:?}", other),
    }

    // b = user?.m(1)
    match &program[1] {
        ASTNode::Assignment { value, .. } => match value.as_ref() {
            ASTNode::MatchExpr {
                scrutinee,
                else_expr,
                ..
            } => {
                match scrutinee.as_ref() {
                    ASTNode::Variable { name, .. } => assert_eq!(name, "user"),
                    _ => panic!("scrutinee not user"),
                }
                match else_expr.as_ref() {
                    ASTNode::MethodCall {
                        object,
                        method,
                        arguments,
                        ..
                    } => {
                        match object.as_ref() {
                            ASTNode::Variable { name, .. } => assert_eq!(name, "user"),
                            _ => panic!("object not user"),
                        }
                        assert_eq!(method, "m");
                        assert_eq!(arguments.len(), 1);
                    }
                    other => panic!("else not method call, got {:?}", other),
                }
            }
            other => panic!("expected MatchExpr, got {:?}", other),
        },
        other => panic!("expected assignment, got {:?}", other),
    }
}
