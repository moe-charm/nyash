use crate::parser::NyashParser;
use crate::ast::ASTNode;

#[test]
fn parse_immediate_lambda_call() {
    let src = "(fn(x){ x }) (1)";
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn has_call(n: &ASTNode) -> bool {
        match n {
            ASTNode::Call { .. } => true,
            ASTNode::Program { statements, .. } => statements.iter().any(has_call),
            _ => false,
        }
    }
    assert!(has_call(&ast));
}

