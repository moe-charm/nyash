use nyash_rust::parser::NyashParser;

#[test]
fn macro_derive_injects_equals_and_tostring() {
    // Enable macro engine and default derives
    std::env::set_var("NYASH_MACRO_ENABLE", "1");
    std::env::set_var("NYASH_MACRO_TRACE", "0");
    std::env::remove_var("NYASH_MACRO_DERIVE");

    let code = r#"
box UserBox {
  name: StringBox
  age: IntegerBox
}
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse ok");
    let ast2 = crate::r#macro::maybe_expand_and_dump(&ast, false);

    // Find UserBox and check methods
    let mut found = false;
    if let nyash_rust::ASTNode::Program { statements, .. } = ast2 {
        for st in statements {
            if let nyash_rust::ASTNode::BoxDeclaration { name, methods, .. } = st {
                if name == "UserBox" {
                    assert!(methods.contains_key("equals"), "equals method should be generated");
                    assert!(methods.contains_key("toString"), "toString method should be generated");
                    found = true;
                }
            }
        }
    }
    assert!(found, "UserBox declaration not found after expansion");
}

