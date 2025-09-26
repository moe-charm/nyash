use crate::ast::ASTNode;
use crate::parser::entry_sugar::parse_with_sugar_level;
use crate::syntax::sugar_config::SugarLevel;

#[test]
fn pipeline_rewrites_function_and_method_calls() {
    let code = "result = data |> normalize(1) |> obj.m(2)\n";
    let ast = parse_with_sugar_level(code, SugarLevel::Basic).expect("parse ok");

    // Program with one assignment
    let program = match ast {
        ASTNode::Program { statements, .. } => statements,
        other => panic!("expected program, got {:?}", other),
    };
    assert_eq!(program.len(), 1);
    let assign = match &program[0] {
        ASTNode::Assignment { target, value, .. } => (target, value),
        other => panic!("expected assignment, got {:?}", other),
    };

    // target = result
    match assign.0.as_ref() {
        ASTNode::Variable { name, .. } => assert_eq!(name, "result"),
        other => panic!("expected target var, got {:?}", other),
    }

    // value should be obj.m( normalize(data,1), 2 )
    let (obj_name, method_name, args) = match assign.1.as_ref() {
        ASTNode::MethodCall {
            object,
            method,
            arguments,
            ..
        } => {
            let obj_name = match object.as_ref() {
                ASTNode::Variable { name, .. } => name.clone(),
                other => panic!("expected obj var, got {:?}", other),
            };
            (obj_name, method.clone(), arguments.clone())
        }
        other => panic!("expected method call, got {:?}", other),
    };
    assert_eq!(obj_name, "obj");
    assert_eq!(method_name, "m");
    assert_eq!(args.len(), 2);

    // first arg should be normalize(data,1)
    match &args[0] {
        ASTNode::FunctionCall {
            name, arguments, ..
        } => {
            assert_eq!(name, "normalize");
            assert_eq!(arguments.len(), 2);
            match &arguments[0] {
                ASTNode::Variable { name, .. } => assert_eq!(name, "data"),
                other => panic!("expected var data, got {:?}", other),
            }
        }
        other => panic!("expected function call, got {:?}", other),
    }
}
