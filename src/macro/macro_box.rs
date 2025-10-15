use std::sync::{Mutex, OnceLock};
use nyash_rust::ASTNode;
use crate::LiteralValue;
use crate::common::trace_box::TraceBox;

/// MacroBox API — user-extensible macro expansion units (experimental)
///
/// Philosophy:
/// - Deterministic, side-effect free, no IO. Pure AST -> AST transforms.
/// - Prefer operating on public interfaces (Box public fields/methods) and avoid
///   coupling to private internals.
pub trait MacroBox: Send + Sync {
    fn name(&self) -> &'static str;
    fn expand(&self, ast: &ASTNode) -> ASTNode;
}

static REG: OnceLock<Mutex<Vec<&'static dyn MacroBox>>> = OnceLock::new();

fn registry() -> &'static Mutex<Vec<&'static dyn MacroBox>> {
    REG.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register a MacroBox. Intended to be called at startup/init paths.
pub fn register(m: &'static dyn MacroBox) {
    let reg = registry();
    let mut guard = reg.lock().expect("macro registry poisoned");
    // avoid duplicates
    if !guard.iter().any(|e| e.name() == m.name()) {
        guard.push(m);
        TraceBox::macro_trace(|| format!("[macro][box] registered {}", m.name()));
    }
}

/// Gate for MacroBox execution.
///
/// Legacy env `NYASH_MACRO_BOX=1` still forces ON, but by default we
/// synchronize with the macro system gate so user macros run when macros are enabled.
pub fn enabled() -> bool {
    if std::env::var("NYASH_MACRO_BOX").ok().as_deref() == Some("1") { return true; }
    super::enabled()
}

/// Expand AST by applying all registered MacroBoxes in order once.
pub fn expand_all_once(ast: &ASTNode) -> ASTNode {
    if !enabled() { return ast.clone(); }
    let reg = registry();
    let guard = reg.lock().expect("macro registry poisoned");
    let mut cur = ast.clone();
    TraceBox::macro_trace(|| {
        let names: Vec<&str> = guard.iter().map(|m| m.name()).collect();
        format!("[macro][box] registry: {} boxes {:?}", names.len(), names)
    });
    for m in guard.iter() {
        TraceBox::macro_trace(|| format!("[macro][box] applying {}", m.name()));
        let out = m.expand(&cur);
        cur = out;
    }
    cur
}

// ---- Built-in example (optional) ----

pub struct UppercasePrintMacro;

impl MacroBox for UppercasePrintMacro {
    fn name(&self) -> &'static str { "UppercasePrintMacro" }
    fn expand(&self, ast: &ASTNode) -> ASTNode {
        use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
        fn go(n: &A) -> A {
            match n.clone() {
                A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|c| go(&c)).collect(), span },
                A::Print { expression, span } => {
                    match &*expression {
                        A::Literal { value: LiteralValue::String(s), .. } => {
                            // Demo: if string starts with "UPPER:", uppercase the rest.
                            if let Some(rest) = s.strip_prefix("UPPER:") {
                                let up = rest.to_uppercase();
                                A::Print { expression: Box::new(A::Literal { value: LiteralValue::String(up), span: Span::unknown() }), span }
                            } else { A::Print { expression: Box::new(go(&*expression)), span } }
                        }
                        other => A::Print { expression: Box::new(go(other)), span }
                    }
                }
                A::Assignment { target, value, span } => A::Assignment { target: Box::new(go(&*target)), value: Box::new(go(&*value)), span },
                A::If { condition, then_body, else_body, span } => A::If {
                    condition: Box::new(go(&*condition)),
                    then_body: then_body.into_iter().map(|c| go(&c)).collect(),
                    else_body: else_body.map(|v| v.into_iter().map(|c| go(&c)).collect()),
                    span,
                },
                A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(go(v))), span },
                A::FieldAccess { object, field, span } => A::FieldAccess { object: Box::new(go(&*object)), field, span },
                A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(go(&*object)), method, arguments: arguments.into_iter().map(|c| go(&c)).collect(), span },
                A::FunctionCall { name, arguments, span } => A::FunctionCall { name, arguments: arguments.into_iter().map(|c| go(&c)).collect(), span },
                A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(go(&*left)), right: Box::new(go(&*right)), span },
                A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(go(&*operand)), span },
                A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|c| go(&c)).collect(), span },
                A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, go(&v))).collect(), span },
                other => other,
            }
        }
        go(ast)
    }
}

static INIT_FLAG: OnceLock<()> = OnceLock::new();

// ---- Selfhost minimal macros (json/map/arr) ----
pub struct SelfhostMinMacro;

impl MacroBox for SelfhostMinMacro {
    fn name(&self) -> &'static str { "SelfhostMinMacro" }
    fn expand(&self, ast: &ASTNode) -> ASTNode {
        use nyash_rust::ast::ASTNode as A;
        fn go(n: &A, changed: &mut bool) -> A {
            match n.clone() {
                A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|c| go(&c, changed)).collect(), span },
                A::BoxDeclaration { name, fields, public_fields, private_fields, mut methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, is_static, static_init, span } => {
                    let mut new_methods = std::collections::HashMap::new();
                    for (mn, mnode) in methods.drain() {
                        match mnode {
                            A::FunctionDeclaration { name: fn_name, params, body, is_static: m_static, is_override, span: m_span } => {
                                let new_body = body.into_iter().map(|st| go(&st, changed)).collect();
                                new_methods.insert(mn, A::FunctionDeclaration { name: fn_name, params, body: new_body, is_static: m_static, is_override, span: m_span });
                            }
                            other => { new_methods.insert(mn, other); }
                        }
                    }
                    A::BoxDeclaration { name, fields, public_fields, private_fields, methods: new_methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, is_static, static_init, span }
                },
                A::FunctionCall { name, arguments, span } => {
                    let lname = name.to_ascii_lowercase();
                    // json/map passthrough of single map literal
                    if (lname == "json" || lname == "map") && arguments.len() == 1 {
                        if let A::MapLiteral { entries, .. } = &arguments[0] {
                            let mapped = entries.iter().map(|(k, v)| (k.clone(), go(v, changed))).collect();
                            *changed = true;
                            return A::MapLiteral { entries: mapped, span };
                        }
                    }
                    // arr passthrough of single array literal
                    if lname == "arr" && arguments.len() == 1 {
                        if let A::ArrayLiteral { elements, .. } = &arguments[0] {
                            let mapped = elements.iter().map(|e| go(e, changed)).collect();
                            *changed = true;
                            return A::ArrayLiteral { elements: mapped, span };
                        }
                    }
                    // call("Box.method/N", recv, args...) -> MethodCall(recv, method, args)
                    if lname == "call" && !arguments.is_empty() {
                        if let A::Literal { value: LiteralValue::String(s), .. } = &arguments[0] {
                            // Parse "Box.method/N"
                            let parts: Vec<&str> = s.split('/').collect();
                            if parts.len() == 2 {
                                if let Ok(n) = parts[1].parse::<usize>() {
                                    if let Some(dot) = parts[0].rfind('.') {
                                        let _box_name = &parts[0][..dot];
                                        let method_name = &parts[0][dot+1..];
                                        // Arity n includes receiver for method calls
                                        let user_argc = arguments.len().saturating_sub(1);
                                        if user_argc == n && n >= 1 {
                                            if let Some(recv) = arguments.get(1..2) {
                                                let _recv_ast = go(&recv[0], changed);
                                                let _new_args = arguments.iter().skip(2).map(|c| go(c, changed)).collect::<Vec<_>>();
                                                *changed = true;
                                                let norm_box = if _box_name.ends_with("Box") { _box_name.to_string() } else { match _box_name { "String"=>"StringBox".to_string(), "Array"=>"ArrayBox".to_string(), "Map"=>"MapBox".to_string(), other=>format!("{}Box", other) } };
                                                let m_arity = n - 1;
                                                let fname = format!("{}. {}/{}", norm_box, method_name, m_arity).replace(". ", ".");
                                                let call_args = arguments.iter().skip(1).map(|c| go(c, changed)).collect::<Vec<_>>();
                                                *changed = true;
                                                return A::FunctionCall { name: fname, arguments: call_args, span };
                                            }
                                        }
                                    }
                                }
                            }
                            // Fallback: treat as global function name string
                            let rest = arguments.iter().skip(1).map(|c| go(c, changed)).collect::<Vec<_>>();
                            *changed = true;
                            return A::FunctionCall { name: s.clone(), arguments: rest, span };
                        }
                    }
                    A::FunctionCall { name, arguments: arguments.into_iter().map(|c| go(&c, changed)).collect(), span }
                }
                A::Assignment { target, value, span } => A::Assignment { target: Box::new(go(&*target, changed)), value: Box::new(go(&*value, changed)), span },
                A::If { condition, then_body, else_body, span } => A::If { condition: Box::new(go(&*condition, changed)), then_body: then_body.into_iter().map(|c| go(&c, changed)).collect(), else_body: else_body.map(|v| v.into_iter().map(|c| go(&c, changed)).collect()), span },
                A::Return { value, span } => A::Return { value: value.as_ref().map(|v| Box::new(go(v, changed))), span },
                A::Local { variables, initial_values, span } => {
                    let new_inits: Vec<Option<Box<A>>> = initial_values.into_iter().map(|opt| opt.map(|b| Box::new(go(&b, changed)))).collect();
                    A::Local { variables, initial_values: new_inits, span }
                },
                A::FieldAccess { object, field, span } => A::FieldAccess { object: Box::new(go(&*object, changed)), field, span },
                A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(go(&*object, changed)), method, arguments: arguments.into_iter().map(|c| go(&c, changed)).collect(), span },
                A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(go(&*left, changed)), right: Box::new(go(&*right, changed)), span },
                A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(go(&*operand, changed)), span },
                A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|c| go(&c, changed)).collect(), span },
                A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k,v)| (k, go(&v, changed))).collect(), span },
                other => other,
            }
        }
        let mut changed = false;
        let out = go(ast, &mut changed);
        if changed {
            TraceBox::macro_trace(|| "[macro][selfhost_min] replaced json/map/arr calls".to_string());
        }
        out
    }
}

/// Initialize built-in demo MacroBoxes when enabled by env flags.
pub fn init_builtin() {
    INIT_FLAG.get_or_init(|| {
        // Explicit example toggle
        if std::env::var("NYASH_MACRO_BOX_EXAMPLE").ok().as_deref() == Some("1") { register(&UppercasePrintMacro); }
        // Comma-separated names: NYASH_MACRO_BOX_ENABLE="UppercasePrintMacro,Other"
        if let Ok(list) = std::env::var("NYASH_MACRO_BOX_ENABLE") {
            for name in list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                match name {
                    "UppercasePrintMacro" => register(&UppercasePrintMacro),
                    "SelfhostMinMacro" => register(&SelfhostMinMacro),
                    _ => { eprintln!("[macro][box] unknown MacroBox '{}', ignoring", name); }
                }
            }
        }
        // Dev-only minimal macros for selfhost sources (explicitly gated)
        if std::env::var("NYASH_MACRO_SELFHOST_MIN").ok().as_deref() == Some("1") {
            register(&SelfhostMinMacro);
        }
    });
}
