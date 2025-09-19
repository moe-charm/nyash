use std::sync::{Mutex, OnceLock};
use nyash_rust::ASTNode;

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
    for m in guard.iter() {
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
                    _ => { eprintln!("[macro][box] unknown MacroBox '{}', ignoring", name); }
                }
            }
        }
    });
}
