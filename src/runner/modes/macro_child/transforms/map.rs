pub(super) fn transform_map_insert_tag(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
    match ast {
        A::MapLiteral { entries, .. } => {
            let mut new_entries: Vec<(String, A)> = Vec::with_capacity(entries.len() + 1);
            let already_tagged = entries.get(0).map(|(k, _)| k == "__macro").unwrap_or(false);
            if already_tagged {
                for (k, v) in entries { new_entries.push((k.clone(), transform_map_insert_tag(v))); }
            } else {
                new_entries.push((
                    "__macro".to_string(),
                    A::Literal { value: LiteralValue::String("on".to_string()), span: Span::unknown() },
                ));
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
        other => other.clone(),
    }
}

