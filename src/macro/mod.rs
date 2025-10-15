//! Macro System scaffolding (Phase 16 – MVP)
//!
//! Goal: Provide minimal, typed interfaces for AST pattern matching and
//! HIR patch based expansion. Backends (MIR/JIT/LLVM) remain unchanged.

pub mod pattern;
pub mod engine;
pub mod macro_box;
pub mod macro_box_ny;
pub mod ast_json;
pub mod ctx;
pub mod test_harness;

use nyash_rust::ASTNode;
use crate::common::trace_box::TraceBox;

/// Enable/disable macro system via env gate.
pub fn enabled() -> bool {
    // Default ON. Disable with NYASH_MACRO_DISABLE=1 or NYASH_MACRO_ENABLE=0/false/off.
    if let Ok(v) = std::env::var("NYASH_MACRO_DISABLE") { if v == "1" { return false; } }
    if let Ok(v) = std::env::var("NYASH_MACRO_ENABLE") {
        let v = v.to_ascii_lowercase();
        if v == "0" || v == "false" || v == "off" { return false; }
        return true;
    }
    true
}

/// A hook to dump AST for `--expand` (pre/post). Expansion is no-op for now.
pub fn maybe_expand_and_dump(ast: &ASTNode, _dump_only: bool) -> ASTNode {
    if !enabled() { return ast.clone(); }
    // Initialize user macro boxes (if any, behind env gates)
    self::macro_box::init_builtin();
    self::macro_box_ny::init_from_env();
    TraceBox::macro_trace(|| format!("[macro] input AST: {:?}", ast));
    let mut eng = self::engine::MacroEngine::new();
    let (out, _patches) = eng.expand(ast);
    let out2 = maybe_inject_test_harness(&out);
    TraceBox::macro_trace(|| {
        fn count_calls(n: &nyash_rust::ASTNode, acc: &mut std::collections::HashMap<String, usize>) {
            use nyash_rust::ast::ASTNode as A;
            match n.clone() {
                A::Program { statements, .. } => { for s in statements { count_calls(&s, acc); } }
                A::FunctionCall { name, arguments, .. } => { *acc.entry(name).or_insert(0) += 1; for a in arguments { count_calls(&a, acc); } }
                A::MethodCall { object, arguments, .. } => { count_calls(&object, acc); for a in arguments { count_calls(&a, acc); } }
                A::ArrayLiteral { elements, .. } => { for e in elements { count_calls(&e, acc); } }
                A::MapLiteral { entries, .. } => { for (_k, v) in entries { count_calls(&v, acc); } }
                A::BinaryOp { left, right, .. } => { count_calls(&left, acc); count_calls(&right, acc); }
                A::UnaryOp { operand, .. } => { count_calls(&operand, acc); }
                A::Assignment { target, value, .. } => { count_calls(&target, acc); count_calls(&value, acc); }
                A::If { condition, then_body, else_body, .. } => { count_calls(&condition, acc); for s in then_body { count_calls(&s, acc); } if let Some(b) = else_body { for s in b { count_calls(&s, acc); } } }
                _ => {}
            }
        }
        let mut acc = std::collections::HashMap::new();
        count_calls(&out2, &mut acc);
        let mut msg = String::new();
        if !acc.is_empty() {
            msg.push_str(&format!("[macro] call census: {:?}\n", acc));
        }
        msg.push_str(&format!("[macro] output AST: {:?}", out2));
        msg
    });
    out2
}

fn maybe_inject_test_harness(ast: &ASTNode) -> ASTNode {
    test_harness::maybe_inject_test_harness(ast)
}
