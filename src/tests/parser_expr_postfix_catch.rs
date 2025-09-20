use crate::parser::NyashParser;

fn enable_stage3() {
    std::env::set_var("NYASH_PARSER_STAGE3", "1");
}

#[test]
fn expr_postfix_catch_basic() {
    enable_stage3();
    let src = r#"
function main(args) {
  f(1) catch(e) { print(e) }
}
"#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn has_try(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::TryCatch { .. } => true,
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(has_try),
            crate::ast::ASTNode::FunctionDeclaration { body, .. } => body.iter().any(has_try),
            _ => false,
        }
    }
    assert!(has_try(&ast), "expected TryCatch from expr‑postfix catch");
}

#[test]
fn expr_postfix_catch_on_method_chain() {
    enable_stage3();
    let src = r#"
function main(args) {
  obj.m1().m2() catch { print("x") }
}
"#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn has_try(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::TryCatch { .. } => true,
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(has_try),
            crate::ast::ASTNode::FunctionDeclaration { body, .. } => body.iter().any(has_try),
            _ => false,
        }
    }
    assert!(has_try(&ast), "expected TryCatch wrapping method chain");
}

