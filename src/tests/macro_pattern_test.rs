use nyash_rust::ast::{ASTNode, BinaryOperator, Span};
use crate::r#macro::pattern::{TemplatePattern, AstBuilder, MacroPattern};
use std::collections::HashMap;

#[test]
fn template_pattern_matches_and_unquotes() {
    // Build a template: ($x + 1) == $y  (Binary(Equal, Binary(Add, $x, 1), $y))
    let tpl = ASTNode::BinaryOp {
        operator: BinaryOperator::Equal,
        left: Box::new(ASTNode::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(ASTNode::Variable { name: "$x".into(), span: Span::unknown() }),
            right: Box::new(ASTNode::Literal { value: nyash_rust::ast::LiteralValue::Integer(1), span: Span::unknown() }),
            span: Span::unknown(),
        }),
        right: Box::new(ASTNode::Variable { name: "$y".into(), span: Span::unknown() }),
        span: Span::unknown(),
    };
    let pat = TemplatePattern::new(tpl);

    // Target: (a + 1) == b
    let target = ASTNode::BinaryOp {
        operator: BinaryOperator::Equal,
        left: Box::new(ASTNode::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(ASTNode::Variable { name: "a".into(), span: Span::unknown() }),
            right: Box::new(ASTNode::Literal { value: nyash_rust::ast::LiteralValue::Integer(1), span: Span::unknown() }),
            span: Span::unknown(),
        }),
        right: Box::new(ASTNode::Variable { name: "b".into(), span: Span::unknown() }),
        span: Span::unknown(),
    };
    let binds = pat.match_ast(&target).expect("pattern match");
    assert!(binds.contains_key("x"));
    assert!(binds.contains_key("y"));

    // Unquote a template: return $y
    let builder = AstBuilder::new();
    let tpl2 = ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "$y".into(), span: Span::unknown() })), span: Span::unknown() };
    let out = builder.unquote(&tpl2, &binds);
    match out {
        ASTNode::Return { value: Some(v), .. } => match *v {
            ASTNode::Variable { name, .. } => assert_eq!(name, "b"),
            _ => panic!("expected variable"),
        }
        _ => panic!("expected return"),
    }
}

