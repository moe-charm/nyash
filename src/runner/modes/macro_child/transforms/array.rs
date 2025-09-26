pub(super) fn transform_array_prepend_zero(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
    match ast {
        A::ArrayLiteral { elements, .. } => {
            let mut new_elems: Vec<A> = Vec::with_capacity(elements.len() + 1);
            let already_zero = elements
                .get(0)
                .and_then(|n| match n { A::Literal { value: LiteralValue::Integer(0), .. } => Some(()), _ => None })
                .is_some();
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

