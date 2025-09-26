pub(super) fn transform_scopebox_inject(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|n| transform_scopebox_inject(&n)).collect(), span },
        A::If { condition, then_body, else_body, span } => {
            let cond = Box::new(transform_scopebox_inject(&condition));
            let then_wrapped = vec![A::ScopeBox { body: then_body.into_iter().map(|n| transform_scopebox_inject(&n)).collect(), span: nyash_rust::ast::Span::unknown() }];
            let else_wrapped = else_body.map(|v| vec![A::ScopeBox { body: v.into_iter().map(|n| transform_scopebox_inject(&n)).collect(), span: nyash_rust::ast::Span::unknown() }]);
            A::If { condition: cond, then_body: then_wrapped, else_body: else_wrapped, span }
        }
        A::Loop { condition, body, span } => {
            let cond = Box::new(transform_scopebox_inject(&condition));
            let body_wrapped = vec![A::ScopeBox { body: body.into_iter().map(|n| transform_scopebox_inject(&n)).collect(), span: nyash_rust::ast::Span::unknown() }];
            A::Loop { condition: cond, body: body_wrapped, span }
        }
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_scopebox_inject(&left)), right: Box::new(transform_scopebox_inject(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_scopebox_inject(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_scopebox_inject(&object)), method, arguments: arguments.into_iter().map(|a| transform_scopebox_inject(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_scopebox_inject(&a)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_scopebox_inject(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k, v)| (k, transform_scopebox_inject(&v))).collect(), span },
        other => other,
    }
}

