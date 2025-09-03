use crate::parser::NyashParser;
use crate::ast::ASTNode;

#[test]
fn parse_lambda_fn_block() {
    let src = "local f = fn() { return 1 }";
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn has_lambda(n: &ASTNode) -> bool {
        match n {
            ASTNode::Lambda { .. } => true,
            ASTNode::Program { statements, .. } => statements.iter().any(has_lambda),
            _ => false,
        }
    }
    assert!(has_lambda(&ast));
}

