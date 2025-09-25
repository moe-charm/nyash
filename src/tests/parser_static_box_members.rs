use crate::parser::NyashParser;

fn parse(src: &str) -> crate::ast::ASTNode { NyashParser::parse_from_string(src).expect("parse ok") }

fn no_toplevel_funccall(ast: &crate::ast::ASTNode) -> bool {
    match ast {
        crate::ast::ASTNode::Program { statements, .. } => {
            !statements.iter().any(|n| matches!(n, crate::ast::ASTNode::FunctionCall { .. }))
        }
        _ => true,
    }
}

fn box_has_methods(ast: &crate::ast::ASTNode, box_name: &str, methods: &[&str]) -> bool {
    fn check_box(b: &crate::ast::ASTNode, box_name: &str, methods: &[&str]) -> bool {
        if let crate::ast::ASTNode::BoxDeclaration { name, methods: m, is_static, .. } = b {
            if name == box_name && *is_static {
                return methods.iter().all(|k| {
                    if let Some(node) = m.get(*k) {
                        matches!(node, crate::ast::ASTNode::FunctionDeclaration { name, .. } if name == *k)
                    } else { false }
                });
            }
        }
        false
    }
    match ast {
        crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(|n| check_box(n, box_name, methods)),
        _ => false,
    }
}

#[test]
fn static_box_methods_no_stray_call_compact() {
    let src = r#"
static box S {
  f(a) { return a }
  g(b) { return b }
}
"#;
    let ast = parse(src);
    assert!(no_toplevel_funccall(&ast), "no top-level FunctionCall expected");
    assert!(box_has_methods(&ast, "S", &["f", "g"]), "static box S should have f and g methods");
}

#[test]
fn static_box_methods_no_stray_call_newline_seams() {
    // Newlines between ) and {, and tight seam between } and next method head
    let src = r#"
static box S {
  parse_float(s)
  {
    return s
  }
  is_empty_or_whitespace(s)
  {
    return 0
  }
}
"#;
    let ast = parse(src);
    assert!(no_toplevel_funccall(&ast), "no top-level FunctionCall expected at seams");
    assert!(box_has_methods(&ast, "S", &["parse_float", "is_empty_or_whitespace"]));
}

