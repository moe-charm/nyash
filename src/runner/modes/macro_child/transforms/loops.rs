pub(super) fn transform_loop_normalize(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|n| transform_loop_normalize(&n)).collect(), span },
        A::If { condition, then_body, else_body, span } => A::If {
            condition: Box::new(transform_loop_normalize(&condition)),
            then_body: then_body.into_iter().map(|n| transform_loop_normalize(&n)).collect(),
            else_body: else_body.map(|v| v.into_iter().map(|n| transform_loop_normalize(&n)).collect()),
            span,
        },
        A::Loop { condition, body, span } => A::Loop {
            condition: Box::new(transform_loop_normalize(&condition)),
            body: body.into_iter().map(|n| transform_loop_normalize(&n)).collect(),
            span,
        },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_loop_normalize(&left)), right: Box::new(transform_loop_normalize(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_loop_normalize(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_loop_normalize(&object)), method, arguments: arguments.into_iter().map(|a| transform_loop_normalize(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_loop_normalize(&a)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_loop_normalize(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, transform_loop_normalize(&v))).collect(), span },
        other => other,
    }
}

