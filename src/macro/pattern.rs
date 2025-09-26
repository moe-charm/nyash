use std::collections::HashMap;
use nyash_rust::ASTNode;

/// Minimal pattern trait — MVP
pub trait MacroPattern {
    fn match_ast(&self, node: &ASTNode) -> Option<HashMap<String, ASTNode>>;
}

/// Quote/Unquote placeholders — MVP stubs
pub struct AstBuilder;

impl AstBuilder {
    pub fn new() -> Self { Self }
    pub fn quote(&self, code: &str) -> ASTNode {
        // MVP: parse string into AST using existing parser
        match nyash_rust::parser::NyashParser::parse_from_string(code) {
            Ok(ast) => ast,
            Err(_) => ASTNode::Program { statements: vec![], span: nyash_rust::ast::Span::unknown() },
        }
    }
    pub fn unquote(&self, template: &ASTNode, _bindings: &HashMap<String, ASTNode>) -> ASTNode {
        // Replace Variables named like "$name" with corresponding bound AST
        fn is_placeholder(name: &str) -> Option<&str> {
            if name.starts_with('$') && name.len() > 1 { Some(&name[1..]) } else { None }
        }
        fn is_variadic(name: &str) -> Option<&str> {
            if name.starts_with("$...") && name.len() > 4 { Some(&name[4..]) } else { None }
        }
        fn subst(node: &ASTNode, binds: &HashMap<String, ASTNode>) -> ASTNode {
            match node.clone() {
                ASTNode::Variable { name, .. } => {
                    if let Some(k) = is_placeholder(&name) {
                        if let Some(v) = binds.get(k) { return v.clone(); }
                    }
                    node.clone()
                }
                ASTNode::BinaryOp { operator, left, right, span } => ASTNode::BinaryOp {
                    operator, left: Box::new(subst(&left, binds)), right: Box::new(subst(&right, binds)), span
                },
                ASTNode::UnaryOp { operator, operand, span } => ASTNode::UnaryOp { operator, operand: Box::new(subst(&operand, binds)), span },
                ASTNode::MethodCall { object, method, arguments, span } => {
                    let mut out_args: Vec<ASTNode> = Vec::new();
                    let mut i = 0usize;
                    while i < arguments.len() {
                        if let ASTNode::Variable { name, .. } = &arguments[i] {
                            if let Some(vn) = is_variadic(name) {
                                if let Some(ASTNode::Program { statements, .. }) = binds.get(vn) {
                                    out_args.extend(statements.clone());
                                    i += 1;
                                    continue;
                                }
                            }
                        }
                        out_args.push(subst(&arguments[i], binds));
                        i += 1;
                    }
                    ASTNode::MethodCall { object: Box::new(subst(&object, binds)), method, arguments: out_args, span }
                }
                ASTNode::FunctionCall { name, arguments, span } => {
                    let mut out_args: Vec<ASTNode> = Vec::new();
                    let mut i = 0usize;
                    while i < arguments.len() {
                        if let ASTNode::Variable { name, .. } = &arguments[i] {
                            if let Some(vn) = is_variadic(name) {
                                if let Some(ASTNode::Program { statements, .. }) = binds.get(vn) {
                                    out_args.extend(statements.clone());
                                    i += 1;
                                    continue;
                                }
                            }
                        }
                        out_args.push(subst(&arguments[i], binds));
                        i += 1;
                    }
                    ASTNode::FunctionCall { name, arguments: out_args, span }
                }
                ASTNode::ArrayLiteral { elements, span } => {
                    // Splice variadic placeholder inside arrays
                    let mut out_elems: Vec<ASTNode> = Vec::new();
                    let mut i = 0usize;
                    while i < elements.len() {
                        if let ASTNode::Variable { name, .. } = &elements[i] {
                            if let Some(vn) = is_variadic(name) {
                                if let Some(ASTNode::Program { statements, .. }) = binds.get(vn) {
                                    out_elems.extend(statements.clone());
                                    i += 1;
                                    continue;
                                }
                            }
                        }
                        out_elems.push(subst(&elements[i], binds));
                        i += 1;
                    }
                    ASTNode::ArrayLiteral { elements: out_elems, span }
                }
                ASTNode::MapLiteral { entries, span } => {
                    ASTNode::MapLiteral { entries: entries.into_iter().map(|(k, v)| (k, subst(&v, binds))).collect(), span }
                }
                ASTNode::FieldAccess { object, field, span } => ASTNode::FieldAccess { object: Box::new(subst(&object, binds)), field, span },
                ASTNode::Assignment { target, value, span } => ASTNode::Assignment { target: Box::new(subst(&target, binds)), value: Box::new(subst(&value, binds)), span },
                ASTNode::Return { value, span } => ASTNode::Return { value: value.as_ref().map(|v| Box::new(subst(v, binds))), span },
                ASTNode::If { condition, then_body, else_body, span } => ASTNode::If {
                    condition: Box::new(subst(&condition, binds)),
                    then_body: then_body.into_iter().map(|n| subst(&n, binds)).collect(),
                    else_body: else_body.map(|v| v.into_iter().map(|n| subst(&n, binds)).collect()),
                    span,
                },
                ASTNode::Program { statements, span } => ASTNode::Program { statements: statements.into_iter().map(|n| subst(&n, binds)).collect(), span },
                other => other,
            }
        }
        subst(template, _bindings)
    }
}

/// Simple template-based pattern that uses Variables named "$name" as bind points.
pub struct TemplatePattern { pub template: ASTNode }

impl TemplatePattern { pub fn new(template: ASTNode) -> Self { Self { template } } }

impl MacroPattern for TemplatePattern {
    fn match_ast(&self, node: &ASTNode) -> Option<HashMap<String, ASTNode>> {
        fn is_placeholder(name: &str) -> Option<&str> {
            if name.starts_with('$') && name.len() > 1 { Some(&name[1..]) } else { None }
        }
        fn is_variadic(name: &str) -> Option<&str> {
            if name.starts_with("$...") && name.len() > 4 { Some(&name[4..]) } else { None }
        }
        fn go(tpl: &ASTNode, tgt: &ASTNode, out: &mut HashMap<String, ASTNode>) -> bool {
            match (tpl, tgt) {
                (ASTNode::Variable { name, .. }, v) => {
                    if let Some(k) = is_placeholder(name) { out.insert(k.to_string(), v.clone()); true } else { tpl == tgt }
                }
                (ASTNode::Literal { .. }, _) | (ASTNode::Me { .. }, _) | (ASTNode::This { .. }, _) => tpl == tgt,
                (ASTNode::BinaryOp { operator: op1, left: l1, right: r1, .. }, ASTNode::BinaryOp { operator: op2, left: l2, right: r2, .. }) => {
                    op1 == op2 && go(l1, l2, out) && go(r1, r2, out)
                }
                (ASTNode::UnaryOp { operator: o1, operand: a1, .. }, ASTNode::UnaryOp { operator: o2, operand: a2, .. }) => {
                    o1 == o2 && go(a1, a2, out)
                }
                (ASTNode::MethodCall { object: o1, method: m1, arguments: a1, .. }, ASTNode::MethodCall { object: o2, method: m2, arguments: a2, .. }) => {
                    if m1 != m2 { return false; }
                    if !go(o1, o2, out) { return false; }
                    // Support variadic anywhere in a1
                    let mut varpos: Option<(usize, String)> = None;
                    for (i, arg) in a1.iter().enumerate() {
                        if let ASTNode::Variable { name, .. } = arg { if let Some(vn) = is_variadic(name) { varpos = Some((i, vn.to_string())); break; } }
                    }
                    if let Some((k, vn)) = varpos {
                        let suffix_len = a1.len() - k - 1;
                        if a2.len() < k + suffix_len { return false; }
                        for (x, y) in a1[..k].iter().zip(a2.iter()) { if !go(x, y, out) { return false; } }
                        for (i, x) in a1[a1.len()-suffix_len..].iter().enumerate() {
                            let y = &a2[a2.len()-suffix_len + i];
                            if !go(x, y, out) { return false; }
                        }
                        let tail: Vec<ASTNode> = a2[k..a2.len()-suffix_len].to_vec();
                        out.insert(vn, ASTNode::Program { statements: tail, span: nyash_rust::ast::Span::unknown() });
                        return true;
                    }
                    if a1.len() != a2.len() { return false; }
                    for (x, y) in a1.iter().zip(a2.iter()) { if !go(x, y, out) { return false; } }
                    true
                }
                (ASTNode::FunctionCall { name: n1, arguments: a1, .. }, ASTNode::FunctionCall { name: n2, arguments: a2, .. }) => {
                    if n1 != n2 { return false; }
                    let mut varpos: Option<(usize, String)> = None;
                    for (i, arg) in a1.iter().enumerate() {
                        if let ASTNode::Variable { name, .. } = arg { if let Some(vn) = is_variadic(name) { varpos = Some((i, vn.to_string())); break; } }
                    }
                    if let Some((k, vn)) = varpos {
                        let suffix_len = a1.len() - k - 1;
                        if a2.len() < k + suffix_len { return false; }
                        for (x, y) in a1[..k].iter().zip(a2.iter()) { if !go(x, y, out) { return false; } }
                        for (i, x) in a1[a1.len()-suffix_len..].iter().enumerate() {
                            let y = &a2[a2.len()-suffix_len + i];
                            if !go(x, y, out) { return false; }
                        }
                        let tail: Vec<ASTNode> = a2[k..a2.len()-suffix_len].to_vec();
                        out.insert(vn, ASTNode::Program { statements: tail, span: nyash_rust::ast::Span::unknown() });
                        return true;
                    }
                    if a1.len() != a2.len() { return false; }
                    for (x, y) in a1.iter().zip(a2.iter()) { if !go(x, y, out) { return false; } }
                    true
                }
                (ASTNode::ArrayLiteral { elements: e1, .. }, ASTNode::ArrayLiteral { elements: e2, .. }) => {
                    let mut varpos: Option<(usize, String)> = None;
                    for (i, el) in e1.iter().enumerate() {
                        if let ASTNode::Variable { name, .. } = el { if let Some(vn) = is_variadic(name) { varpos = Some((i, vn.to_string())); break; } }
                    }
                    if let Some((k, vn)) = varpos {
                        let suffix_len = e1.len() - k - 1;
                        if e2.len() < k + suffix_len { return false; }
                        for (x, y) in e1[..k].iter().zip(e2.iter()) { if !go(x, y, out) { return false; } }
                        for (i, x) in e1[e1.len()-suffix_len..].iter().enumerate() {
                            let y = &e2[e2.len()-suffix_len + i];
                            if !go(x, y, out) { return false; }
                        }
                        let tail: Vec<ASTNode> = e2[k..e2.len()-suffix_len].to_vec();
                        out.insert(vn, ASTNode::Program { statements: tail, span: nyash_rust::ast::Span::unknown() });
                        return true;
                    }
                    if e1.len() != e2.len() { return false; }
                    for (x, y) in e1.iter().zip(e2.iter()) { if !go(x, y, out) { return false; } }
                    true
                }
                (ASTNode::MapLiteral { entries: m1, .. }, ASTNode::MapLiteral { entries: m2, .. }) => {
                    if m1.len() != m2.len() { return false; }
                    for ((k1, v1), (k2, v2)) in m1.iter().zip(m2.iter()) {
                        if k1 != k2 { return false; }
                        if !go(v1, v2, out) { return false; }
                    }
                    true
                }
                (ASTNode::FieldAccess { object: o1, field: f1, .. }, ASTNode::FieldAccess { object: o2, field: f2, .. }) => f1 == f2 && go(o1, o2, out),
                (ASTNode::Assignment { target: t1, value: v1, .. }, ASTNode::Assignment { target: t2, value: v2, .. }) => go(t1, t2, out) && go(v1, v2, out),
                (ASTNode::Return { value: v1, .. }, ASTNode::Return { value: v2, .. }) => match (v1, v2) { (Some(a), Some(b)) => go(a, b, out), (None, None) => true, _ => false },
                (ASTNode::If { condition: c1, then_body: t1, else_body: e1, .. }, ASTNode::If { condition: c2, then_body: t2, else_body: e2, .. }) => {
                    if !go(c1, c2, out) || t1.len() != t2.len() { return false; }
                    for (x, y) in t1.iter().zip(t2.iter()) { if !go(x, y, out) { return false; } }
                    match (e1, e2) {
                        (Some(a), Some(b)) => {
                            if a.len() != b.len() { return false; }
                            for (x, y) in a.iter().zip(b.iter()) { if !go(x, y, out) { return false; } }
                            true
                        }
                        (None, None) => true,
                        _ => false,
                    }
                }
                _ => tpl == tgt,
            }
        }
        let mut out = HashMap::new();
        if go(&self.template, node, &mut out) { Some(out) } else { None }
    }
}

/// ORパターン: いずれかのテンプレートにマッチすれば成功
pub struct OrPattern { pub alts: Vec<TemplatePattern> }

impl OrPattern { pub fn new(alts: Vec<TemplatePattern>) -> Self { Self { alts } } }

impl MacroPattern for OrPattern {
    fn match_ast(&self, node: &ASTNode) -> Option<HashMap<String, ASTNode>> {
        for tp in &self.alts {
            if let Some(b) = tp.match_ast(node) { return Some(b); }
        }
        None
    }
}
