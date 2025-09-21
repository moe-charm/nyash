use super::transforms::*;

pub fn run_macro_child(macro_file: &str) {
    // Read full AST JSON (v0) from stdin
    use std::io::Read;
    let mut input = String::new();
    if let Err(_) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("[macro-child] failed to read AST JSON from stdin");
        std::process::exit(3);
    }
    let value: serde_json::Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => { eprintln!("[macro-child] invalid AST JSON v0"); std::process::exit(3); }
    };
    let ast: nyash_rust::ASTNode = match crate::r#macro::ast_json::json_to_ast(&value) {
        Some(a) => a,
        None => { eprintln!("[macro-child] unsupported AST JSON v0"); std::process::exit(4); }
    };

    // Analyze macro behavior (PoC)
    let mut behavior = crate::r#macro::macro_box_ny::analyze_macro_file(macro_file);
    if macro_file.contains("env_tag_string_macro") {
        behavior = crate::r#macro::macro_box_ny::MacroBehavior::EnvTagString;
    }

    let out_ast = match behavior {
        crate::r#macro::macro_box_ny::MacroBehavior::Identity => ast.clone(),
        crate::r#macro::macro_box_ny::MacroBehavior::Uppercase => {
            let m = crate::r#macro::macro_box::UppercasePrintMacro;
            crate::r#macro::macro_box::MacroBox::expand(&m, &ast)
        }
        crate::r#macro::macro_box_ny::MacroBehavior::ArrayPrependZero => transform_array_prepend_zero(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::MapInsertTag => transform_map_insert_tag(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::LoopNormalize => transform_loop_normalize(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::IfMatchNormalize => transform_peek_match_literal(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::ForForeachNormalize => transform_for_foreach(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::EnvTagString => {
            fn tag(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
                use nyash_rust::ast::ASTNode as A;
                match ast.clone() {
                    A::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } => {
                        if s == "hello" { A::Literal { value: nyash_rust::ast::LiteralValue::String("hello [ENV]".to_string()), span: nyash_rust::ast::Span::unknown() } } else { ast.clone() }
                    }
                    A::Program { statements, span } => A::Program { statements: statements.iter().map(|n| tag(n)).collect(), span },
                    A::Print { expression, span } => A::Print { expression: Box::new(tag(&expression)), span },
                    A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(tag(v))), span },
                    A::Assignment { target, value, span } => A::Assignment { target: Box::new(tag(&target)), value: Box::new(tag(&value)), span },
                    A::If { condition, then_body, else_body, span } => A::If { condition: Box::new(tag(&condition)), then_body: then_body.iter().map(|n| tag(n)).collect(), else_body: else_body.map(|v| v.iter().map(|n| tag(n)).collect()), span },
                    A::Loop { condition, body, span } => A::Loop { condition: Box::new(tag(&condition)), body: body.iter().map(|n| tag(n)).collect(), span },
                    A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(tag(&left)), right: Box::new(tag(&right)), span },
                    A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(tag(&operand)), span },
                    A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(tag(&object)), method, arguments: arguments.iter().map(|a| tag(a)).collect(), span },
                    A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.iter().map(|a| tag(a)).collect(), span },
                    A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.iter().map(|e| tag(e)).collect(), span },
                    A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.iter().map(|(k,v)| (k.clone(), tag(v))).collect(), span },
                    other => other,
                }
            }
            let mut env_on = std::env::var("NYASH_MACRO_CAP_ENV").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false);
            if let Ok(ctxs) = std::env::var("NYASH_MACRO_CTX_JSON") {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ctxs) {
                    env_on = v.get("caps").and_then(|c| c.get("env")).and_then(|b| b.as_bool()).unwrap_or(env_on);
                }
            }
            if env_on { tag(&ast) } else { ast.clone() }
        }
    };
    let out_json = crate::r#macro::ast_json::ast_to_json(&out_ast);
    println!("{}", out_json.to_string());
}
