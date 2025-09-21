pub(super) fn transform_if_to_loopform(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, Span};
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|n| transform_if_to_loopform(&n)).collect(), span },
        A::If { condition, then_body, else_body, span } => {
            let cond_t = Box::new(transform_if_to_loopform(&condition));
            let then_t = then_body.into_iter().map(|n| transform_if_to_loopform(&n)).collect();
            let else_t = else_body.map(|v| v.into_iter().map(|n| transform_if_to_loopform(&n)).collect());
            let inner_if = A::If { condition: cond_t, then_body: then_t, else_body: else_t, span: Span::unknown() };
            let one = A::Literal { value: nyash_rust::ast::LiteralValue::Integer(1), span: Span::unknown() };
            let loop_body = vec![inner_if, A::Break { span: Span::unknown() }];
            A::Loop { condition: Box::new(one), body: loop_body, span }
        }
        A::Loop { condition, body, span } => A::Loop { condition: Box::new(transform_if_to_loopform(&condition)), body: body.into_iter().map(|n| transform_if_to_loopform(&n)).collect(), span },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_if_to_loopform(&left)), right: Box::new(transform_if_to_loopform(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_if_to_loopform(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_if_to_loopform(&object)), method, arguments: arguments.into_iter().map(|a| transform_if_to_loopform(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_if_to_loopform(&a)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_if_to_loopform(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k, v)| (k, transform_if_to_loopform(&v))).collect(), span },
        other => other,
    }
}

