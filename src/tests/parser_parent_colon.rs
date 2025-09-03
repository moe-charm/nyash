use crate::parser::NyashParser;
use crate::ast::ASTNode;

#[test]
fn parse_parent_colon_syntax() {
    let src = "Parent::birth()";
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn is_fromcall(n: &ASTNode) -> bool {
        match n {
            ASTNode::FromCall { parent, method, .. } => parent == "Parent" && method == "birth",
            ASTNode::Program { statements, .. } => statements.iter().any(is_fromcall),
            _ => false,
        }
    }
    assert!(is_fromcall(&ast));
}

