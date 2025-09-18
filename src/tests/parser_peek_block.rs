use crate::parser::NyashParser;

#[test]
fn parse_match_with_block_arm() {
    let src = r#"
        local x = 2
        local y = match x {
            1 => { local a = 10 a }
            2 => { 20 }
            _ => { 30 }
        }
    "#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    // Quick structural check: ensure AST contains PeekExpr and Program nodes inside arms
    fn find_peek(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::PeekExpr { arms, else_expr, .. } => {
                // Expect at least one Program arm
                let has_block = arms.iter().any(|(_, e)| matches!(e, crate::ast::ASTNode::Program { .. }));
                let else_is_block = matches!(**else_expr, crate::ast::ASTNode::Program { .. });
                has_block && else_is_block
            }
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(find_peek),
            _ => false,
        }
    }
    assert!(find_peek(&ast), "expected peek with block arms in AST");
}
