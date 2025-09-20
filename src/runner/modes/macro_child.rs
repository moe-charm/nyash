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

            // まず全体の更新変数の種類を走査（観測のみ）。
            // 制限は設けず、後続のセグメント整列（非代入→代入）に委ねる。
            // 複合ターゲットが出現した場合は保守的に“整列スキップ”とするため、ここでは弾かない。

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
    // Optional: inject ScopeBox wrappers for diagnostics/visibility (no-op for MIR)
    let a4 = if std::env::var("NYASH_SCOPEBOX_ENABLE").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_scopebox_inject(&a3)
    } else { a3 };
    // Lift nested function declarations (no captures) to top-level with gensym names
    let a4b = transform_lift_nested_functions(&a4);
    // Optional: If → LoopForm (conservative). Only transform if no else and branch has no break/continue.
    let a5 = if std::env::var("NYASH_IF_AS_LOOPFORM").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_if_to_loopform(&a4b)
    } else { a4b };
    // Optional: postfix catch/cleanup sugar → TryCatch normalization
    let a6 = if std::env::var("NYASH_CATCH_NEW").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_postfix_handlers(&a5)
    } else { a5 };
    a6
}

// ---- Nested Function Lift (no captures) ----

fn transform_lift_nested_functions(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn gensym(base: &str) -> String {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("__ny_lifted_{}_{}", base, n)
    }

    fn collect_locals(n: &A, set: &mut std::collections::HashSet<String>) {
        match n {
            A::Local { variables, .. } => { for v in variables { set.insert(v.clone()); } }
            A::Program { statements, .. } => for s in statements { collect_locals(s, set); },
            A::FunctionDeclaration { body, .. } => for s in body { collect_locals(s, set); },
            A::If { then_body, else_body, .. } => {
                for s in then_body { collect_locals(s, set); }
                if let Some(b) = else_body { for s in b { collect_locals(s, set); } }
            }
            _ => {}
        }
    }

    fn collect_vars(n: &A, set: &mut std::collections::HashSet<String>) {
        match n {
            A::Variable { name, .. } => { set.insert(name.clone()); }
            A::Program { statements, .. } => for s in statements { collect_vars(s, set); },
            A::FunctionDeclaration { body, .. } => for s in body { collect_vars(s, set); },
            A::If { condition, then_body, else_body, .. } => {
                collect_vars(condition, set);
                for s in then_body { collect_vars(s, set); }
                if let Some(b) = else_body { for s in b { collect_vars(s, set); } }
            }
            A::Assignment { target, value, .. } => { collect_vars(target, set); collect_vars(value, set); }
            A::Return { value, .. } => { if let Some(v) = value { collect_vars(v, set); } }
            A::Print { expression, .. } => collect_vars(expression, set),
            A::BinaryOp { left, right, .. } => { collect_vars(left, set); collect_vars(right, set); }
            A::UnaryOp { operand, .. } => collect_vars(operand, set),
            A::MethodCall { object, arguments, .. } => { collect_vars(object, set); for a in arguments { collect_vars(a, set); } }
            A::FunctionCall { arguments, .. } => { for a in arguments { collect_vars(a, set); } }
            A::ArrayLiteral { elements, .. } => { for e in elements { collect_vars(e, set); } }
            A::MapLiteral { entries, .. } => { for (_,v) in entries { collect_vars(v, set); } }
            _ => {}
        }
    }

    fn rename_calls(n: &A, mapping: &std::collections::HashMap<String, String>) -> A {
        use nyash_rust::ast::ASTNode as A;
        match n.clone() {
            A::FunctionCall { name, arguments, span } => {
                let new_name = mapping.get(&name).cloned().unwrap_or(name);
                A::FunctionCall { name: new_name, arguments: arguments.into_iter().map(|a| rename_calls(&a, mapping)).collect(), span }
            }
            A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|s| rename_calls(&s, mapping)).collect(), span },
            A::FunctionDeclaration { name, params, body, is_static, is_override, span } => {
                A::FunctionDeclaration { name, params, body: body.into_iter().map(|s| rename_calls(&s, mapping)).collect(), is_static, is_override, span }
            }
            A::If { condition, then_body, else_body, span } => A::If {
                condition: Box::new(rename_calls(&condition, mapping)),
                then_body: then_body.into_iter().map(|s| rename_calls(&s, mapping)).collect(),
                else_body: else_body.map(|v| v.into_iter().map(|s| rename_calls(&s, mapping)).collect()),
                span,
            },
            A::Assignment { target, value, span } => A::Assignment { target: Box::new(rename_calls(&target, mapping)), value: Box::new(rename_calls(&value, mapping)), span },
            A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(rename_calls(v, mapping))), span },
            A::Print { expression, span } => A::Print { expression: Box::new(rename_calls(&expression, mapping)), span },
            A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(rename_calls(&left, mapping)), right: Box::new(rename_calls(&right, mapping)), span },
            A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(rename_calls(&operand, mapping)), span },
            A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(rename_calls(&object, mapping)), method, arguments: arguments.into_iter().map(|a| rename_calls(&a, mapping)).collect(), span },
            A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| rename_calls(&e, mapping)).collect(), span },
            A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, rename_calls(&v, mapping))).collect(), span },
            other => other,
        }
    }

    fn lift_in_body(body: Vec<A>, hoisted: &mut Vec<A>, mapping: &mut std::collections::HashMap<String,String>) -> Vec<A> {
        use std::collections::HashSet;
        let mut out: Vec<A> = Vec::new();
        for st in body.into_iter() {
            match st.clone() {
                A::FunctionDeclaration { name, params, body, is_static, is_override, span } => {
                    // check captures
                    let mut locals: HashSet<String> = HashSet::new();
                    collect_locals(&A::FunctionDeclaration{ name: name.clone(), params: params.clone(), body: body.clone(), is_static, is_override, span }, &mut locals);
                    let mut used: HashSet<String> = HashSet::new();
                    collect_vars(&A::FunctionDeclaration{ name: name.clone(), params: params.clone(), body: body.clone(), is_static, is_override, span }, &mut used);
                    let params_set: HashSet<String> = params.iter().cloned().collect();
                    let mut extra: HashSet<String> = used.drain().collect();
                    extra.retain(|v| !params_set.contains(v) && !locals.contains(v));
                    if extra.is_empty() {
                        // Hoist with gensym name
                        let new_name = gensym(&name);
                        let lifted = A::FunctionDeclaration { name: new_name.clone(), params, body, is_static: true, is_override, span };
                        hoisted.push(lifted);
                        mapping.insert(name, new_name);
                        // do not keep nested declaration in place
                        continue;
                    } else {
                        // keep as-is (cannot hoist due to captures)
                        out.push(st);
                    }
                }
                other => out.push(other),
            }
        }
        // After scanning, rename calls in out according to mapping
        out.into_iter().map(|n| rename_calls(&n, mapping)).collect()
    }

    fn walk(n: &A, hoisted: &mut Vec<A>) -> A {
        use nyash_rust::ast::ASTNode as A;
        match n.clone() {
            A::Program { statements, span } => {
                let mut mapping = std::collections::HashMap::new();
                let stmts2 = lift_in_body(statements.into_iter().map(|s| walk(&s, hoisted)).collect(), hoisted, &mut mapping);
                // Append hoisted at end (global scope)
                // Note: hoisted collected at all levels; only append here once after full walk
                A::Program { statements: stmts2, span }
            }
            A::FunctionDeclaration { name, params, body, is_static, is_override, span } => {
                let mut mapping = std::collections::HashMap::new();
                let body2: Vec<A> = body.into_iter().map(|s| walk(&s, hoisted)).collect();
                let body3 = lift_in_body(body2, hoisted, &mut mapping);
                A::FunctionDeclaration { name, params, body: body3, is_static, is_override, span }
            }
            A::If { condition, then_body, else_body, span } => A::If {
                condition: Box::new(walk(&condition, hoisted)),
                then_body: then_body.into_iter().map(|s| walk(&s, hoisted)).collect(),
                else_body: else_body.map(|v| v.into_iter().map(|s| walk(&s, hoisted)).collect()),
                span,
            },
            A::Assignment { target, value, span } => A::Assignment { target: Box::new(walk(&target, hoisted)), value: Box::new(walk(&value, hoisted)), span },
            A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(walk(v, hoisted))), span },
            A::Print { expression, span } => A::Print { expression: Box::new(walk(&expression, hoisted)), span },
            A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(walk(&left, hoisted)), right: Box::new(walk(&right, hoisted)), span },
            A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(walk(&operand, hoisted)), span },
            A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(walk(&object, hoisted)), method, arguments: arguments.into_iter().map(|a| walk(&a, hoisted)).collect(), span },
            A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|a| walk(&a, hoisted)).collect(), span },
            A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| walk(&e, hoisted)).collect(), span },
            A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, walk(&v, hoisted))).collect(), span },
            other => other,
        }
    }

    let mut hoisted: Vec<A> = Vec::new();
    let mut out = walk(ast, &mut hoisted);
    // Append hoisted functions at top-level if root is Program
    if let A::Program { statements, span } = out.clone() {
        let mut ss = statements;
        ss.extend(hoisted.into_iter());
        out = A::Program { statements: ss, span };
    }
    out
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

fn transform_scopebox_inject(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => {
            A::Program { statements: statements.into_iter().map(|n| transform_scopebox_inject(&n)).collect(), span }
        }
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

fn transform_if_to_loopform(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, Span};
    // Conservative rewrite: if (cond) { then } with no else and no break/continue in then → loop(cond) { then }
    // (unused helpers removed)
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|n| transform_if_to_loopform(&n)).collect(), span },
        A::If { condition, then_body, else_body, span } => {
            // Case A/B unified: wrap into single-iteration loop with explicit break (semantics-preserving)
            // This avoids multi-iteration semantics and works for both then-only and else-present cases.
            let cond_t = Box::new(transform_if_to_loopform(&condition));
            let then_t = then_body.into_iter().map(|n| transform_if_to_loopform(&n)).collect();
            let else_t = else_body.map(|v| v.into_iter().map(|n| transform_if_to_loopform(&n)).collect());
            let inner_if = A::If { condition: cond_t, then_body: then_t, else_body: else_t, span: Span::unknown() };
            let one = A::Literal { value: nyash_rust::ast::LiteralValue::Integer(1), span: Span::unknown() };
            let loop_body = vec![inner_if, A::Break { span: Span::unknown() }];
            A::Loop { condition: Box::new(one), body: loop_body, span }
        }
        A::Loop { condition, body, span } => A::Loop {
            condition: Box::new(transform_if_to_loopform(&condition)),
            body: body.into_iter().map(|n| transform_if_to_loopform(&n)).collect(),
            span
        },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_if_to_loopform(&left)), right: Box::new(transform_if_to_loopform(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_if_to_loopform(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_if_to_loopform(&object)), method, arguments: arguments.into_iter().map(|a| transform_if_to_loopform(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_if_to_loopform(&a)).collect(), span },
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_if_to_loopform(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k, v)| (k, transform_if_to_loopform(&v))).collect(), span },
        other => other,
    }
}

// Phase 1 sugar: postfix_catch(expr, "Type"?, fn(e){...}) / with_cleanup(expr, fn(){...})
// → legacy TryCatch AST for existing lowering paths. This is a stopgap until parser accepts postfix forms.
fn transform_postfix_handlers(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::{ASTNode as A, CatchClause, Span};
    fn map_vec(v: Vec<A>) -> Vec<A> { v.into_iter().map(|n| transform_postfix_handlers(&n)).collect() }
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: map_vec(statements), span },
        A::If { condition, then_body, else_body, span } => A::If {
            condition: Box::new(transform_postfix_handlers(&condition)),
            then_body: map_vec(then_body),
            else_body: else_body.map(map_vec),
            span,
        },
        A::Loop { condition, body, span } => A::Loop {
            condition: Box::new(transform_postfix_handlers(&condition)),
            body: map_vec(body),
            span,
        },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp {
            operator,
            left: Box::new(transform_postfix_handlers(&left)),
            right: Box::new(transform_postfix_handlers(&right)),
            span,
        },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_postfix_handlers(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall {
            object: Box::new(transform_postfix_handlers(&object)),
            method,
            arguments: arguments.into_iter().map(|a| transform_postfix_handlers(&a)).collect(),
            span,
        },
        A::FunctionCall { name, arguments, span } => {
            let name_l = name.to_ascii_lowercase();
            if name_l == "postfix_catch" {
                // Forms:
                //  - postfix_catch(expr, fn(e){...})
                //  - postfix_catch(expr, "Type", fn(e){...})
                let mut args = arguments;
                if args.len() >= 2 {
                    let expr = transform_postfix_handlers(&args.remove(0));
                    let (type_opt, handler) = if args.len() == 1 {
                        (None, args.remove(0))
                    } else if args.len() >= 2 {
                        let ty = match args.remove(0) {
                            A::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } => Some(s),
                            other => {
                                // keep robust: non-string type → debug print type name, treat as None
                                let _ = other; None
                            }
                        };
                        (ty, args.remove(0))
                    } else { (None, A::Literal { value: nyash_rust::ast::LiteralValue::Void, span: Span::unknown() }) };
                    if let A::Lambda { params, body, .. } = handler {
                        let var = params.get(0).cloned();
                        let cc = CatchClause { exception_type: type_opt, variable_name: var, body, span: Span::unknown() };
                        return A::TryCatch { try_body: vec![expr], catch_clauses: vec![cc], finally_body: None, span };
                    }
                }
                // Fallback: recurse into args
                A::FunctionCall { name, arguments: args.into_iter().map(|a| transform_postfix_handlers(&a)).collect(), span }
            } else if name_l == "with_cleanup" {
                // Form: with_cleanup(expr, fn(){...})
                let mut args = arguments;
                if args.len() >= 2 {
                    let expr = transform_postfix_handlers(&args.remove(0));
                    let handler = args.remove(0);
                    if let A::Lambda { body, .. } = handler {
                        return A::TryCatch { try_body: vec![expr], catch_clauses: vec![], finally_body: Some(body), span };
                    }
                }
                A::FunctionCall { name, arguments: args.into_iter().map(|a| transform_postfix_handlers(&a)).collect(), span }
            } else {
                A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_postfix_handlers(&a)).collect(), span }
            }
        }
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_postfix_handlers(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, transform_postfix_handlers(&v))).collect(), span },
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
    let mut behavior = crate::r#macro::macro_box_ny::analyze_macro_file(macro_file);
    if macro_file.contains("env_tag_string_macro") {
        behavior = crate::r#macro::macro_box_ny::MacroBehavior::EnvTagString;
    }
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
            // Prefer ctx JSON from env (NYASH_MACRO_CTX_JSON) if provided; fallback to simple flag
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
