pub(super) fn transform_lift_nested_functions(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    fn gensym(base: &str) -> String { let n = COUNTER.fetch_add(1, Ordering::Relaxed); format!("__ny_lifted_{}_{}", base, n) }
    fn collect_locals(n: &A, set: &mut std::collections::HashSet<String>) {
        match n {
            A::Local { variables, .. } => { for v in variables { set.insert(v.clone()); } }
            A::Program { statements, .. } => for s in statements { collect_locals(s, set); },
            A::FunctionDeclaration { body, .. } => for s in body { collect_locals(s, set); },
            A::If { then_body, else_body, .. } => { for s in then_body { collect_locals(s, set); } if let Some(b) = else_body { for s in b { collect_locals(s, set); } } }
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
                    let mut locals: HashSet<String> = HashSet::new();
                    collect_locals(&A::FunctionDeclaration{ name: name.clone(), params: params.clone(), body: body.clone(), is_static, is_override, span }, &mut locals);
                    let mut used: HashSet<String> = HashSet::new();
                    collect_vars(&A::FunctionDeclaration{ name: name.clone(), params: params.clone(), body: body.clone(), is_static, is_override, span }, &mut used);
                    let params_set: HashSet<String> = params.iter().cloned().collect();
                    let mut extra: HashSet<String> = used.drain().collect();
                    extra.retain(|v| !params_set.contains(v) && !locals.contains(v));
                    if extra.is_empty() {
                        let new_name = gensym(&name);
                        let lifted = A::FunctionDeclaration { name: new_name.clone(), params, body, is_static: true, is_override, span };
                        hoisted.push(lifted);
                        mapping.insert(name, new_name);
                        continue;
                    } else { out.push(st); }
                }
                other => out.push(other),
            }
        }
        out.into_iter().map(|n| rename_calls(&n, mapping)).collect()
    }
    fn walk(n: &A, hoisted: &mut Vec<A>) -> A {
        use nyash_rust::ast::ASTNode as A;
        match n.clone() {
            A::Program { statements, span } => {
                let mut mapping = std::collections::HashMap::new();
                let stmts2 = lift_in_body(statements.into_iter().map(|s| walk(&s, hoisted)).collect(), hoisted, &mut mapping);
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
    if let A::Program { statements, span } = out.clone() {
        let mut ss = statements;
        ss.extend(hoisted.into_iter());
        out = A::Program { statements: ss, span };
    }
    out
}

