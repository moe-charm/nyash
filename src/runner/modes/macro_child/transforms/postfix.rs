pub(super) fn transform_postfix_handlers(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, CatchClause, Span};
    fn map_vec(v: Vec<A>) -> Vec<A> { v.into_iter().map(|n| transform_postfix_handlers(&n)).collect() }
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: map_vec(statements), span },
        A::If { condition, then_body, else_body, span } => A::If { condition: Box::new(transform_postfix_handlers(&condition)), then_body: map_vec(then_body), else_body: else_body.map(map_vec), span },
        A::Loop { condition, body, span } => A::Loop { condition: Box::new(transform_postfix_handlers(&condition)), body: map_vec(body), span },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_postfix_handlers(&left)), right: Box::new(transform_postfix_handlers(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_postfix_handlers(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_postfix_handlers(&object)), method, arguments: arguments.into_iter().map(|a| transform_postfix_handlers(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => {
            let name_l = name.to_ascii_lowercase();
            if name_l == "postfix_catch" {
                let mut args = arguments;
                if args.len() >= 2 {
                    let expr = transform_postfix_handlers(&args.remove(0));
                    let (type_opt, handler) = if args.len() == 1 {
                        (None, args.remove(0))
                    } else if args.len() >= 2 {
                        let ty = match args.remove(0) { A::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } => Some(s), _ => None };
                        (ty, args.remove(0))
                    } else {
                        (None, A::Literal { value: nyash_rust::ast::LiteralValue::Integer(0), span: Span::unknown() })
                    };
                    if let A::Lambda { params, body, .. } = handler {
                        if params.len() == 1 {
                            let cc = CatchClause {
                                exception_type: type_opt,
                                variable_name: Some(params[0].clone()),
                                body: body.into_iter().map(|n| transform_postfix_handlers(&n)).collect(),
                                span: Span::unknown(),
                            };
                            return A::TryCatch {
                                try_body: vec![expr],
                                catch_clauses: vec![cc],
                                finally_body: None,
                                span: Span::unknown(),
                            };
                        }
                    }
                }
                A::FunctionCall { name, arguments: args.into_iter().map(|n| transform_postfix_handlers(&n)).collect(), span }
            } else if name_l == "with_cleanup" {
                let mut args = arguments;
                if args.len() >= 2 {
                    let expr = transform_postfix_handlers(&args.remove(0));
                    let cleanup = args.remove(0);
                    if let A::Lambda { params, body, .. } = cleanup {
                        if params.is_empty() {
                            return A::TryCatch {
                                try_body: vec![expr],
                                catch_clauses: vec![],
                                finally_body: Some(body.into_iter().map(|n| transform_postfix_handlers(&n)).collect()),
                                span: Span::unknown(),
                            };
                        }
                    }
                }
                A::FunctionCall { name, arguments: args.into_iter().map(|n| transform_postfix_handlers(&n)).collect(), span }
            } else {
                A::FunctionCall { name, arguments: arguments.into_iter().map(|n| transform_postfix_handlers(&n)).collect(), span }
            }
        }
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_postfix_handlers(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, transform_postfix_handlers(&v))).collect(), span },
        other => other,
    }
}

