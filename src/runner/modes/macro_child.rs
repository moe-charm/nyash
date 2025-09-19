use serde_json::Value;

fn map_expr_to_stmt(e: nyash_rust::ASTNode) -> nyash_rust::ASTNode { e }

fn transform_peek_to_if_expr(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::PeekExpr { scrutinee, arms, else_expr, .. } = peek {
        // only support literal-only arms conservatively
        let mut conds_bodies: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms {
            conds_bodies.push((lit.clone(), (*body).clone()));
        }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in conds_bodies.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![map_expr_to_stmt(body)];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_match_literal_local_init(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => {
            A::Program { statements: statements.into_iter().map(|n| transform_peek_match_literal_local_init(&n)).collect(), span }
        }
        A::If { condition, then_body, else_body, span } => {
            A::If {
                condition: Box::new(transform_peek_match_literal_local_init(&condition)),
                then_body: then_body.into_iter().map(|n| transform_peek_match_literal_local_init(&n)).collect(),
                else_body: else_body.map(|v| v.into_iter().map(|n| transform_peek_match_literal_local_init(&n)).collect()),
                span,
            }
        }
        A::Loop { condition, body, span } => {
            A::Loop {
                condition: Box::new(transform_peek_match_literal_local_init(&condition)),
                body: body.into_iter().map(|n| transform_peek_match_literal_local_init(&n)).collect(),
                span,
            }
        }
        A::Local { variables, initial_values, span } => {
            let mut new_inits: Vec<Option<Box<A>>> = Vec::with_capacity(initial_values.len());
            for opt in initial_values {
                if let Some(v) = opt {
                    if let Some(ifexpr) = transform_peek_to_if_expr(&v) {
                        new_inits.push(Some(Box::new(ifexpr)));
                    } else {
                        new_inits.push(Some(Box::new(transform_peek_match_literal_local_init(&v))));
                    }
                } else {
                    new_inits.push(None);
                }
            }
            A::Local { variables, initial_values: new_inits, span }
        }
        other => other,
    }
}

fn transform_array_prepend_zero(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
    match ast {
        A::ArrayLiteral { elements, .. } => {
            // Idempotent: only prepend if first element is not int 0
            let mut new_elems: Vec<A> = Vec::with_capacity(elements.len() + 1);
            let already_zero = elements.get(0).and_then(|n| if let A::Literal { value: LiteralValue::Integer(0), .. } = n { Some(()) } else { None }).is_some();
            if already_zero {
                for e in elements { new_elems.push(transform_array_prepend_zero(e)); }
            } else {
                new_elems.push(A::Literal { value: LiteralValue::Integer(0), span: Span::unknown() });
                for e in elements { new_elems.push(transform_array_prepend_zero(e)); }
            }
            A::ArrayLiteral { elements: new_elems, span: Span::unknown() }
        }
        A::Program { statements, .. } => A::Program { statements: statements.iter().map(transform_array_prepend_zero).collect(), span: Span::unknown() },
        A::Print { expression, .. } => A::Print { expression: Box::new(transform_array_prepend_zero(expression)), span: Span::unknown() },
        A::Return { value, .. } => A::Return { value: value.as_ref().map(|v| Box::new(transform_array_prepend_zero(v))), span: Span::unknown() },
        A::Assignment { target, value, .. } => A::Assignment { target: Box::new(transform_array_prepend_zero(target)), value: Box::new(transform_array_prepend_zero(value)), span: Span::unknown() },
        A::If { condition, then_body, else_body, .. } => A::If {
            condition: Box::new(transform_array_prepend_zero(condition)),
            then_body: then_body.iter().map(transform_array_prepend_zero).collect(),
            else_body: else_body.as_ref().map(|v| v.iter().map(transform_array_prepend_zero).collect()),
            span: Span::unknown(),
        },
        A::BinaryOp { operator, left, right, .. } => A::BinaryOp { operator: operator.clone(), left: Box::new(transform_array_prepend_zero(left)), right: Box::new(transform_array_prepend_zero(right)), span: Span::unknown() },
        A::UnaryOp { operator, operand, .. } => A::UnaryOp { operator: operator.clone(), operand: Box::new(transform_array_prepend_zero(operand)), span: Span::unknown() },
        A::MethodCall { object, method, arguments, .. } => A::MethodCall { object: Box::new(transform_array_prepend_zero(object)), method: method.clone(), arguments: arguments.iter().map(transform_array_prepend_zero).collect(), span: Span::unknown() },
        A::FunctionCall { name, arguments, .. } => A::FunctionCall { name: name.clone(), arguments: arguments.iter().map(transform_array_prepend_zero).collect(), span: Span::unknown() },
        A::MapLiteral { entries, .. } => A::MapLiteral { entries: entries.iter().map(|(k,v)| (k.clone(), transform_array_prepend_zero(v))).collect(), span: Span::unknown() },
        other => other.clone(),
    }
}

fn transform_map_insert_tag(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
    match ast {
        A::MapLiteral { entries, .. } => {
            // Idempotent: only insert if first key is not "__macro"
            let mut new_entries: Vec<(String, A)> = Vec::with_capacity(entries.len() + 1);
            let already_tagged = entries.get(0).map(|(k, _)| k == "__macro").unwrap_or(false);
            if already_tagged {
                for (k, v) in entries { new_entries.push((k.clone(), transform_map_insert_tag(v))); }
            } else {
                new_entries.push(("__macro".to_string(), A::Literal { value: LiteralValue::String("on".to_string()), span: Span::unknown() }));
                for (k, v) in entries { new_entries.push((k.clone(), transform_map_insert_tag(v))); }
            }
            A::MapLiteral { entries: new_entries, span: Span::unknown() }
        }
        A::Program { statements, .. } => A::Program { statements: statements.iter().map(transform_map_insert_tag).collect(), span: Span::unknown() },
        A::Print { expression, .. } => A::Print { expression: Box::new(transform_map_insert_tag(expression)), span: Span::unknown() },
        A::Return { value, .. } => A::Return { value: value.as_ref().map(|v| Box::new(transform_map_insert_tag(v))), span: Span::unknown() },
        A::Assignment { target, value, .. } => A::Assignment { target: Box::new(transform_map_insert_tag(target)), value: Box::new(transform_map_insert_tag(value)), span: Span::unknown() },
        A::If { condition, then_body, else_body, .. } => A::If {
            condition: Box::new(transform_map_insert_tag(condition)),
            then_body: then_body.iter().map(transform_map_insert_tag).collect(),
            else_body: else_body.as_ref().map(|v| v.iter().map(transform_map_insert_tag).collect()),
            span: Span::unknown(),
        },
        A::BinaryOp { operator, left, right, .. } => A::BinaryOp { operator: operator.clone(), left: Box::new(transform_map_insert_tag(left)), right: Box::new(transform_map_insert_tag(right)), span: Span::unknown() },
        A::UnaryOp { operator, operand, .. } => A::UnaryOp { operator: operator.clone(), operand: Box::new(transform_map_insert_tag(operand)), span: Span::unknown() },
        A::MethodCall { object, method, arguments, .. } => A::MethodCall { object: Box::new(transform_map_insert_tag(object)), method: method.clone(), arguments: arguments.iter().map(transform_map_insert_tag).collect(), span: Span::unknown() },
        A::FunctionCall { name, arguments, .. } => A::FunctionCall { name: name.clone(), arguments: arguments.iter().map(transform_map_insert_tag).collect(), span: Span::unknown() },
        A::ArrayLiteral { elements, .. } => A::ArrayLiteral { elements: elements.iter().map(transform_map_insert_tag).collect(), span: Span::unknown() },
        other => other.clone(),
    }
}

pub fn run_macro_child(macro_file: &str) {
    // Read stdin all
    use std::io::Read;
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("[macro-child] read stdin error: {}", e);
        std::process::exit(2);
    }
    let v: Value = match serde_json::from_str(&input) {
        Ok(x) => x,
        Err(e) => { eprintln!("[macro-child] invalid JSON: {}", e); std::process::exit(3); }
    };
    let ast = match crate::r#macro::ast_json::json_to_ast(&v) {
        Some(a) => a,
        None => { eprintln!("[macro-child] unsupported AST JSON v0"); std::process::exit(4); }
    };
    // Analyze macro behavior (PoC)
    let behavior = crate::r#macro::macro_box_ny::analyze_macro_file(macro_file);
    let out_ast = match behavior {
        crate::r#macro::macro_box_ny::MacroBehavior::Identity => ast.clone(),
        crate::r#macro::macro_box_ny::MacroBehavior::Uppercase => {
            // Apply built-in Uppercase transformation
            let m = crate::r#macro::macro_box::UppercasePrintMacro;
            crate::r#macro::macro_box::MacroBox::expand(&m, &ast)
        }
        crate::r#macro::macro_box_ny::MacroBehavior::ArrayPrependZero => transform_array_prepend_zero(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::MapInsertTag => transform_map_insert_tag(&ast),
        crate::r#macro::macro_box_ny::MacroBehavior::LoopNormalize => {
            // MVP: identity (future: normalize Loop into carrier-based form)
            ast.clone()
        }
        crate::r#macro::macro_box_ny::MacroBehavior::IfMatchNormalize => {
            transform_peek_match_literal_local_init(&ast)
        }
    };
    let out_json = crate::r#macro::ast_json::ast_to_json(&out_ast);
    println!("{}", out_json.to_string());
}
