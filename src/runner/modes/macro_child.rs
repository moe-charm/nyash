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

fn transform_peek_to_if_stmt_assign(peek: &nyash_rust::ASTNode, target: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::PeekExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Assignment { target: Box::new(target.clone()), value: Box::new(body), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_to_if_stmt_return(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::PeekExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Return { value: Some(Box::new(body)), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_to_if_stmt_print(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::PeekExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Print { expression: Box::new(body), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_match_literal(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => {
            A::Program { statements: statements.into_iter().map(|n| transform_peek_match_literal(&n)).collect(), span }
        }
        A::If { condition, then_body, else_body, span } => {
            A::If {
                condition: Box::new(transform_peek_match_literal(&condition)),
                then_body: then_body.into_iter().map(|n| transform_peek_match_literal(&n)).collect(),
                else_body: else_body.map(|v| v.into_iter().map(|n| transform_peek_match_literal(&n)).collect()),
                span,
            }
        }
        A::Loop { condition, body, span } => {
            A::Loop {
                condition: Box::new(transform_peek_match_literal(&condition)),
                body: body.into_iter().map(|n| transform_peek_match_literal(&n)).collect(),
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
                        new_inits.push(Some(Box::new(transform_peek_match_literal(&v))));
                    }
                } else {
                    new_inits.push(None);
                }
            }
            A::Local { variables, initial_values: new_inits, span }
        }
        A::Assignment { target, value, span } => {
            if let Some(ifstmt) = transform_peek_to_if_stmt_assign(&value, &target) {
                ifstmt
            } else {
                A::Assignment { target, value: Box::new(transform_peek_match_literal(&value)), span }
            }
        }
        A::Return { value, span } => {
            if let Some(v) = &value {
                if let Some(ifstmt) = transform_peek_to_if_stmt_return(v) {
                    ifstmt
                } else {
                    A::Return { value: Some(Box::new(transform_peek_match_literal(v))), span }
                }
            } else {
                A::Return { value: None, span }
            }
        }
        A::Print { expression, span } => {
            if let Some(ifstmt) = transform_peek_to_if_stmt_print(&expression) {
                ifstmt
            } else {
                A::Print { expression: Box::new(transform_peek_match_literal(&expression)), span }
            }
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

fn transform_loop_normalize(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        // Recurse into container nodes first
        A::Program { statements, span } => {
            A::Program { statements: statements.into_iter().map(|n| transform_loop_normalize(&n)).collect(), span }
        }
        A::If { condition, then_body, else_body, span } => {
            A::If {
                condition: Box::new(transform_loop_normalize(&condition)),
                then_body: then_body.into_iter().map(|n| transform_loop_normalize(&n)).collect(),
                else_body: else_body.map(|v| v.into_iter().map(|n| transform_loop_normalize(&n)).collect()),
                span,
            }
        }
        A::Loop { condition, body, span } => {
            // First, normalize inside children
            let condition = Box::new(transform_loop_normalize(&condition));
            let body_norm: Vec<A> = body.into_iter().map(|n| transform_loop_normalize(&n)).collect();

            // MVP-3: break/continue 最小対応
            // 方針: 本体を control(Break/Continue) でセグメントに分割し、
            // 各セグメント内のみ安全に「非代入→代入」に整列する（順序維持の安定版）。
            // 追加ガード: 代入先は変数に限る。変数の種類は全体で最大2種まで（MVP-2 制約維持）。

            // まず全体の更新変数の種類数を計測（上限2）。
            let mut uniq_targets_overall: Vec<String> = Vec::new();
            for stmt in &body_norm {
                if let A::Assignment { target, .. } = stmt {
                    if let A::Variable { name, .. } = target.as_ref() {
                        if !uniq_targets_overall.iter().any(|s| s == name) {
                            uniq_targets_overall.push(name.clone());
                            if uniq_targets_overall.len() > 2 { // 超過したら全体の並べ替えは不許可
                                return A::Loop { condition, body: body_norm, span };
                            }
                        }
                    } else {
                        // 複合ターゲットを含む場合は保守的にスキップ
                        return A::Loop { condition, body: body_norm, span };
                    }
                }
            }

            // セグメント分解 → セグメント毎に安全整列
            let mut rebuilt: Vec<A> = Vec::with_capacity(body_norm.len());
            let mut seg: Vec<A> = Vec::new();
            let flush_seg = |seg: &mut Vec<A>, out: &mut Vec<A>| {
                // セグメント内で「代入の後に非代入」が存在したら整列しない
                let mut saw_assign = false;
                let mut safe = true;
                for n in seg.iter() {
                    match n {
                        A::Assignment { .. } => { saw_assign = true; }
                        _ => {
                            if saw_assign { safe = false; break; }
                        }
                    }
                }
                if safe {
                    // others → assigns の順で安定整列
                    let mut others: Vec<A> = Vec::new();
                    let mut assigns: Vec<A> = Vec::new();
                    for n in seg.drain(..) {
                        match n {
                            A::Assignment { .. } => assigns.push(n),
                            _ => others.push(n),
                        }
                    }
                    out.extend(others.into_iter());
                    out.extend(assigns.into_iter());
                } else {
                    // そのまま吐き出す
                    out.extend(seg.drain(..));
                }
            };

            for stmt in body_norm.into_iter() {
                match stmt.clone() {
                    A::Break { .. } | A::Continue { .. } => {
                        // control の直前までをフラッシュしてから control を出力
                        flush_seg(&mut seg, &mut rebuilt);
                        rebuilt.push(stmt);
                    }
                    other => seg.push(other),
                }
            }
            // 末尾セグメントをフラッシュ
            flush_seg(&mut seg, &mut rebuilt);

            A::Loop { condition, body: rebuilt, span }
        }
        // Leaf and other nodes: unchanged
        A::Local { variables, initial_values, span } => A::Local { variables, initial_values, span },
        A::Assignment { target, value, span } => A::Assignment { target, value, span },
        A::Return { value, span } => A::Return { value, span },
        A::Print { expression, span } => A::Print { expression, span },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left, right, span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand, span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object, method, arguments, span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments, span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements, span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries, span },
        other => other,
    }
}

// Core normalization pass used by runners (always-on when macros enabled).
// Order matters: for/foreach → match(PeekExpr) → loop tail alignment.
pub fn normalize_core_pass(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    let a1 = transform_for_foreach(ast);
    let a2 = transform_peek_match_literal(&a1);
    let a3 = transform_loop_normalize(&a2);
    a3
}

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

fn transform_for_foreach(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
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
                            // Accept init as Local/Assignment or Lambda(); step as Assignment or Lambda()
                            // Emit init statements (0..n)
                            match init.clone() {
                                A::Assignment { .. } | A::Local { .. } => out.push(init),
                                A::Lambda { params: p2, body: b2, .. } if p2.is_empty() => {
                                    for s in b2 { out.push(transform_for_foreach(&s)); }
                                }
                                _ => {}
                            }
                            let mut loop_body: Vec<A> = body
                                .into_iter()
                                .map(|n| transform_for_foreach(&n))
                                .collect();
                            // Append step statements at tail
                            match step.clone() {
                                A::Assignment { .. } => loop_body.push(step),
                                A::Lambda { params: p3, body: b3, .. } if p3.is_empty() => {
                                    for s in b3 { loop_body.push(transform_for_foreach(&s)); }
                                }
                                _ => {}
                            }
                            out.push(A::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() });
                            continue;
                        }
                    }
                    // Fallback: keep as-is
                    out.push(A::FunctionCall { name, arguments, span: Span::unknown() });
                }
                A::FunctionCall { name, arguments, .. } if (name == "ny_foreach" || name == "foreach") && arguments.len() == 3 => {
                    let arr = arguments[0].clone();
                    let var_name_opt = match &arguments[1] { A::Literal { value: LiteralValue::String(s), .. } => Some(s.clone()), _ => None };
                    let lam = arguments[2].clone();
                    if let (Some(vn), A::Lambda { params, body, .. }) = (var_name_opt, lam) {
                        if params.is_empty() {
                            let idx_name = "__ny_i".to_string();
                            let idx_var = A::Variable { name: idx_name.clone(), span: Span::unknown() };
                            let init_idx = A::Local { variables: vec![idx_name.clone()], initial_values: vec![Some(Box::new(A::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))], span: Span::unknown() };
                            let size_call = A::MethodCall { object: Box::new(arr.clone()), method: "size".to_string(), arguments: vec![], span: Span::unknown() };
                            let cond = A::BinaryOp { operator: BinaryOperator::Less, left: Box::new(idx_var.clone()), right: Box::new(size_call), span: Span::unknown() };
                            let elem = A::MethodCall { object: Box::new(arr.clone()), method: "get".to_string(), arguments: vec![idx_var.clone()], span: Span::unknown() };
                            let mut loop_body: Vec<A> = body.into_iter().map(|n| subst_var(&n, &vn, &elem)).map(|n| transform_for_foreach(&n)).collect();
                            let step = A::Assignment { target: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), value: Box::new(A::BinaryOp { operator: BinaryOperator::Add, left: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), right: Box::new(A::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
                            loop_body.push(step);
                            out.push(init_idx);
                            out.push(A::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() });
                            continue;
                        }
                    }
                    out.push(A::FunctionCall { name, arguments, span: Span::unknown() });
                }
                A::Local { variables, initial_values, .. } => {
                    let mut expanded_any = false;
                    for opt in &initial_values {
                        if let Some(v) = opt {
                            if let A::FunctionCall { name, arguments, .. } = v.as_ref() {
                                if ((name == "ny_for" || name == "for") && arguments.len() == 4)
                                    || ((name == "ny_foreach" || name == "foreach") && arguments.len() == 3)
                                {
                                    expanded_any = true;
                                }
                            }
                        }
                    }
                    if expanded_any {
                        for opt in initial_values {
                            if let Some(v) = opt {
                    match v.as_ref() {
                                    A::FunctionCall { name: _, arguments, .. } if (arguments.len() == 4) => {
                                        // Reuse handling by fabricating a statement call
                                        let fake = A::FunctionCall { name: "for".to_string(), arguments: arguments.clone(), span: Span::unknown() };
                                        // Route into the top arm by re-matching
                                        match fake.clone() {
                                            A::FunctionCall { name: _, arguments, .. } => {
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
                                                            A::Lambda { params: p3, body: b3, .. } if p3.is_empty() => { for s in b3 { loop_body.push(transform_for_foreach(&s)); } }
                                                            _ => {}
                                                        }
                                                        out.push(A::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() });
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    A::FunctionCall { name: _, arguments, .. } if (arguments.len() == 3) => {
                                        let arr = arguments[0].clone();
                                        let var_name_opt = match &arguments[1] { A::Literal { value: LiteralValue::String(s), .. } => Some(s.clone()), _ => None };
                                        let lam = arguments[2].clone();
                                        if let (Some(vn), A::Lambda { params, body, .. }) = (var_name_opt, lam) {
                                            if params.is_empty() {
                                                let idx_name = "__ny_i".to_string();
                                                let idx_var = A::Variable { name: idx_name.clone(), span: Span::unknown() };
                                                let init_idx = A::Local { variables: vec![idx_name.clone()], initial_values: vec![Some(Box::new(A::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))], span: Span::unknown() };
                                                let size_call = A::MethodCall { object: Box::new(arr.clone()), method: "size".to_string(), arguments: vec![], span: Span::unknown() };
                                                let cond = A::BinaryOp { operator: BinaryOperator::Less, left: Box::new(idx_var.clone()), right: Box::new(size_call), span: Span::unknown() };
                                                let elem = A::MethodCall { object: Box::new(arr.clone()), method: "get".to_string(), arguments: vec![idx_var.clone()], span: Span::unknown() };
                                                let mut loop_body: Vec<A> = body.into_iter().map(|n| subst_var(&n, &vn, &elem)).map(|n| transform_for_foreach(&n)).collect();
                                                let step = A::Assignment { target: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), value: Box::new(A::BinaryOp { operator: BinaryOperator::Add, left: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), right: Box::new(A::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
                                                loop_body.push(step);
                                                out.push(init_idx);
                                                out.push(A::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() });
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // Drop original Local that carried macros
                        continue;
                    } else {
                        out.push(A::Local { variables, initial_values, span: Span::unknown() });
                    }
                }
                A::FunctionCall { name, arguments, .. } if name == "foreach_" && arguments.len() == 3 => {
                    let arr = arguments[0].clone();
                    let var_name_opt = match &arguments[1] { A::Literal { value: LiteralValue::String(s), .. } => Some(s.clone()), _ => None };
                    let lam = arguments[2].clone();
                    if let (Some(vn), A::Lambda { params, body, .. }) = (var_name_opt, lam) {
                        if params.is_empty() {
                            // __ny_i = 0; loop(__ny_i < arr.size()) { body[var=arr.get(__ny_i)]; __ny_i = __ny_i + 1 }
                            let idx_name = "__ny_i".to_string();
                            let idx_var = A::Variable { name: idx_name.clone(), span: Span::unknown() };
                            let init_idx = A::Local { variables: vec![idx_name.clone()], initial_values: vec![Some(Box::new(A::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))], span: Span::unknown() };
                            let size_call = A::MethodCall { object: Box::new(arr.clone()), method: "size".to_string(), arguments: vec![], span: Span::unknown() };
                            let cond = A::BinaryOp { operator: BinaryOperator::Less, left: Box::new(idx_var.clone()), right: Box::new(size_call), span: Span::unknown() };
                            let elem = A::MethodCall { object: Box::new(arr.clone()), method: "get".to_string(), arguments: vec![idx_var.clone()], span: Span::unknown() };
                            let mut loop_body: Vec<A> = body.into_iter().map(|n| subst_var(&n, &vn, &elem)).map(|n| transform_for_foreach(&n)).collect();
                            let step = A::Assignment { target: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), value: Box::new(A::BinaryOp { operator: BinaryOperator::Add, left: Box::new(A::Variable { name: idx_name.clone(), span: Span::unknown() }), right: Box::new(A::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
                            loop_body.push(step);
                            out.push(init_idx);
                            out.push(A::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() });
                            continue;
                        }
                    }
                    out.push(A::FunctionCall { name, arguments, span: Span::unknown() });
                }
                // Recurse into container nodes and preserve others
                A::If { condition, then_body, else_body, span } => {
                    out.push(A::If {
                        condition: Box::new(transform_for_foreach(&condition)),
                        then_body: rewrite_stmt_list(then_body),
                        else_body: else_body.map(rewrite_stmt_list),
                        span,
                    });
                }
                A::Loop { condition, body, span } => {
                    out.push(A::Loop {
                        condition: Box::new(transform_for_foreach(&condition)),
                        body: rewrite_stmt_list(body),
                        span,
                    });
                }
                other => out.push(transform_for_foreach(&other)),
            }
        }
        out
    }

    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: rewrite_stmt_list(statements), span },
        A::If { condition, then_body, else_body, span } => A::If {
            condition: Box::new(transform_for_foreach(&condition)),
            then_body: rewrite_stmt_list(then_body),
            else_body: else_body.map(rewrite_stmt_list),
            span,
        },
        A::Loop { condition, body, span } => A::Loop { condition: Box::new(transform_for_foreach(&condition)), body: rewrite_stmt_list(body), span },
        // Leaf and expression nodes: descend but no statement expansion
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
            transform_loop_normalize(&ast)
        }
        crate::r#macro::macro_box_ny::MacroBehavior::IfMatchNormalize => {
            transform_peek_match_literal(&ast)
        }
        crate::r#macro::macro_box_ny::MacroBehavior::ForForeachNormalize => {
            transform_for_foreach(&ast)
        }
    };
    let out_json = crate::r#macro::ast_json::ast_to_json(&out_ast);
    println!("{}", out_json.to_string());
}
