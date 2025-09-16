use crate::ast::ASTNode;
use crate::parser::entry_sugar::parse_with_sugar_level;
use crate::syntax::sugar_config::SugarLevel;

#[test]
fn range_rewrites_to_function_call() {
    let code = "r = 1 .. 5\n";
    let ast = parse_with_sugar_level(code, SugarLevel::Basic).expect("parse ok");

    let program = match ast {
        ASTNode::Program { statements, .. } => statements,
        other => panic!("expected program, got {:?}", other),
    };
    match &program[0] {
        ASTNode::Assignment { value, .. } => match value.as_ref() {
            ASTNode::FunctionCall {
                name, arguments, ..
            } => {
                assert_eq!(name, "Range");
                assert_eq!(arguments.len(), 2);
            }
            other => panic!("expected FunctionCall, got {:?}", other),
        },
        other => panic!("expected assignment, got {:?}", other),
    }
}
