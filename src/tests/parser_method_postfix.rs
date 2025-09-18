use crate::parser::NyashParser;

fn enable_stage3() {
    std::env::set_var("NYASH_PARSER_STAGE3", "1");
    std::env::set_var("NYASH_METHOD_CATCH", "1");
}

#[test]
fn method_postfix_cleanup_only_wraps_trycatch() {
    enable_stage3();
    let src = r#"
box SafeBox {
  value: IntegerBox

  update() {
    value = 41
    return value
  } cleanup {
    value = value + 1
  }
}
"#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    // Find FunctionDeclaration 'update' and ensure its body contains a TryCatch
    fn has_method_trycatch(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::BoxDeclaration { methods, .. } => {
                for (_name, m) in methods {
                    if let crate::ast::ASTNode::FunctionDeclaration { name, body, .. } = m {
                        if name == "update" {
                            return body.iter().any(|n| matches!(n, crate::ast::ASTNode::TryCatch { .. }));
                        }
                    }
                }
                false
            }
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(has_method_trycatch),
            _ => false,
        }
    }
    assert!(has_method_trycatch(&ast), "expected TryCatch inside method body");
}
