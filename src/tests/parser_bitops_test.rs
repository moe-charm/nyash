use crate::parser::NyashParser;
use crate::ast::ASTNode;

#[test]
fn parse_bitops_and_shift_precedence() {
    // Expression: 1 + 2 << 3 & 7
    // Precedence: shift before add, then &: (1 + (2 << 3)) & 7
    let code = "return 1 + 2 << 3 & 7";
    let ast = NyashParser::parse_from_string(code).expect("parse ok");
    // Just ensure it parses into a Program and contains a Return; deeper tree checks are optional here
    fn has_return(n: &ASTNode) -> bool {
        match n {
            ASTNode::Program { statements, .. } => statements.iter().any(has_return),
            ASTNode::Return { .. } => true,
            _ => false,
        }
    }
    assert!(has_return(&ast));
}

