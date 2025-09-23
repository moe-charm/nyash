use crate::mir::{MirCompiler, MirPrinter};
use crate::ast::{ASTNode, LiteralValue, Span};

#[test]
fn mir_lowering_of_peek_expr() {
    // Build AST: peek 2 { 1 => 10, 2 => 20, else => 30 }
    let ast = ASTNode::MatchExpr {
        scrutinee: Box::new(ASTNode::Literal { value: LiteralValue::Integer(2), span: Span::unknown() }),
        arms: vec![
            (LiteralValue::Integer(1), ASTNode::Literal { value: LiteralValue::Integer(10), span: Span::unknown() }),
            (LiteralValue::Integer(2), ASTNode::Literal { value: LiteralValue::Integer(20), span: Span::unknown() }),
        ],
        else_expr: Box::new(ASTNode::Literal { value: LiteralValue::Integer(30), span: Span::unknown() }),
        span: Span::unknown(),
    };

    let mut compiler = MirCompiler::new();
    let res = compiler.compile(ast).expect("compile ok");
    let dump = MirPrinter::new().print_module(&res.module);
    assert!(dump.contains("br "), "expected branches in MIR:\n{}", dump);
    assert!(dump.contains("phi"), "expected phi merge in MIR:\n{}", dump);
}

