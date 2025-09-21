fn subst_var(node: &nyash_rust::ASTNode, name: &str, replacement: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match node.clone() {
        A::Variable { name: n, .. } if n == name => replacement.clone(),
        A::Program { statements, span } => A::Program { statements: statements.iter().map(|s| subst_var(s, name, replacement)).collect(), span },
        A::Print { expression, span } => A::Print { expression: Box::new(subst_var(&expression, name, replacement)), span },
        A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(subst_var(v, name, replacement))), span },
        A::Assignment { target, value, span } => A::Assignment { target: Box::new(subst_var(&target, name, replacement)), value: Box::new(subst_var(&value, name, replacement)), span },
        A::If { condition, then_body, else_body, span } => A::If {
            condition: Box::new(subst_var(&condition, name, replacement)),
            then_body: then_body.iter().map(|s| subst_var(s, name, replacement)).collect(),
            else_body: else_body.map(|v| v.iter().map(|s| subst_var(s, name, replacement)).collect()),
            span,
        },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(subst_var(&left, name, replacement)), right: Box::new(subst_var(&right, name, replacement)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(subst_var(&operand, name, replacement)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(subst_var(&object, name, replacement)), method, arguments: arguments.iter().map(|a| subst_var(a, name, replacement)).collect(), span },
        A::FunctionCall { name: fn_name, arguments, span } => A::FunctionCall { name: fn_name, arguments: arguments.iter().map(|a| subst_var(a, name, replacement)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.iter().map(|e| subst_var(e, name, replacement)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.iter().map(|(k,v)| (k.clone(), subst_var(v, name, replacement))).collect(), span },
        other => other,
    }
}

pub(super) fn transform_for_foreach(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, LiteralValue, Span};
    fn rewrite_stmt_list(list: Vec<A>) -> Vec<A> {
        let mut out: Vec<A> = Vec::new();
        for st in list.into_iter() {
            match st.clone() {
                A::FunctionCall { name, arguments, .. } if (name == "ny_for" || name == "for") && arguments.len() == 4 => {
                    let init = arguments[0].clone();
                    let cond = arguments[1].clone();
                    let step = arguments[2].clone();
                    let body_lam = arguments[3].clone();
                    if let A::Lambda { params, body, .. } = body_lam {
                        if params.is_empty() {
                            match init.clone() {
                                A::Assignment { .. } | A::Local { .. } => out.push(init),
                                A::Lambda { params: p2, body: b2, .. } if p2.is_empty() => { for s in b2 { out.push(transform_for_foreach(&s)); } }
                                _ => {}
                            }
                            let mut loop_body: Vec<A> = body.into_iter().map(|n| transform_for_foreach(&n)).collect();
                            match step.clone() {
                                A::Assignment { .. } => loop_body.push(step),
                                A::Lambda { params: p2, body: b2, .. } if p2.is_empty() => { for s in b2 { loop_body.push(transform_for_foreach(&s)); } }
                                _ => {}
                            }
                            out.push(A::Loop { condition: Box::new(transform_for_foreach(&cond)), body: loop_body, span: Span::unknown() });
                            continue;
                        }
                    }
                    out.push(st);
                }
                A::FunctionCall { name, arguments, .. } if (name == "ny_foreach" || name == "foreach") && arguments.len() == 3 => {
                    let array = arguments[0].clone();
                    let param_name = match &arguments[1] { A::Variable { name, .. } => name.clone(), _ => "it".to_string() };
                    let body_lam = arguments[2].clone();
                    if let A::Lambda { params, body, .. } = body_lam {
                        if params.is_empty() {
                            let iter = A::Variable { name: "__i".to_string(), span: Span::unknown() };
                            let zero = A::Literal { value: LiteralValue::Integer(0), span: Span::unknown() };
                            let one = A::Literal { value: LiteralValue::Integer(1), span: Span::unknown() };
                            let init = A::Local { variables: vec!["__i".to_string()], initial_values: vec![Some(Box::new(zero))], span: Span::unknown() };
                            let len_call = A::MethodCall { object: Box::new(transform_for_foreach(&array)), method: "len".to_string(), arguments: vec![], span: Span::unknown() };
                            let cond = A::BinaryOp { operator: BinaryOperator::Less, left: Box::new(iter.clone()), right: Box::new(len_call), span: Span::unknown() };
                            let get_call = A::MethodCall { object: Box::new(transform_for_foreach(&array)), method: "get".to_string(), arguments: vec![iter.clone()], span: Span::unknown() };
                            let body_stmts: Vec<A> = body.into_iter().map(|s| subst_var(&s, &param_name, &get_call)).collect();
                            let step = A::Assignment { target: Box::new(iter.clone()), value: Box::new(A::BinaryOp { operator: BinaryOperator::Add, left: Box::new(iter), right: Box::new(one), span: Span::unknown() }), span: Span::unknown() };
                            out.push(init);
                            out.push(A::Loop { condition: Box::new(cond), body: { let mut b = Vec::new(); for s in body_stmts { b.push(transform_for_foreach(&s)); } b.push(step); b }, span: Span::unknown() });
                            continue;
                        }
                    }
                    out.push(st);
                }
                other => out.push(transform_for_foreach(&other)),
            }
        }
        out
    }
    // `A` is already imported above
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: rewrite_stmt_list(statements), span },
        A::If { condition, then_body, else_body, span } => A::If { condition: Box::new(transform_for_foreach(&condition)), then_body: rewrite_stmt_list(then_body), else_body: else_body.map(rewrite_stmt_list), span },
        A::Loop { condition, body, span } => A::Loop { condition: Box::new(transform_for_foreach(&condition)), body: rewrite_stmt_list(body), span },
        A::Print { expression, span } => A::Print { expression: Box::new(transform_for_foreach(&expression)), span },
        A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(transform_for_foreach(v))), span },
        A::Assignment { target, value, span } => A::Assignment { target: Box::new(transform_for_foreach(&target)), value: Box::new(transform_for_foreach(&value)), span },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_for_foreach(&left)), right: Box::new(transform_for_foreach(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_for_foreach(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_for_foreach(&object)), method, arguments: arguments.iter().map(|a| transform_for_foreach(a)).collect(), span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.iter().map(|a| transform_for_foreach(a)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.iter().map(|e| transform_for_foreach(e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.iter().map(|(k,v)| (k.clone(), transform_for_foreach(v))).collect(), span },
        other => other,
    }
}
