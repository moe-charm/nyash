use nyash_rust::mir::MirBuilder;
use nyash_rust::ast::{ASTNode, LiteralValue, BinaryOperator, Span};

#[test]
fn test_literal_building() {
    let mut builder = MirBuilder::new();
    let ast = ASTNode::Literal {
        value: LiteralValue::Integer(42),
        span: Span::unknown(),
    };
    let result = builder.build_module(ast);
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.function_names().len(), 1);
    assert!(module.get_function("main").is_some());
}

#[test]
fn test_binary_op_building() {
    let mut builder = MirBuilder::new();
    let ast = ASTNode::BinaryOp {
        left: Box::new(ASTNode::Literal {
            value: LiteralValue::Integer(10),
            span: Span::unknown(),
        }),
        operator: BinaryOperator::Add,
        right: Box::new(ASTNode::Literal {
            value: LiteralValue::Integer(32),
            span: Span::unknown(),
        }),
        span: Span::unknown(),
    };
    let result = builder.build_module(ast);
    assert!(result.is_ok());
    let module = result.unwrap();
    let function = module.get_function("main").unwrap();
    let stats = function.stats();
    assert!(stats.instruction_count >= 3);
}

#[test]
fn test_if_statement_building() {
    let mut builder = MirBuilder::new();
    let ast = ASTNode::If {
        condition: Box::new(ASTNode::Literal {
            value: LiteralValue::Bool(true),
            span: Span::unknown(),
        }),
        then_body: vec![ASTNode::Literal {
            value: LiteralValue::Integer(1),
            span: Span::unknown(),
        }],
        else_body: Some(vec![ASTNode::Literal {
            value: LiteralValue::Integer(2),
            span: Span::unknown(),
        }]),
        span: Span::unknown(),
    };
    let result = builder.build_module(ast);
    assert!(result.is_ok());
    let module = result.unwrap();
    let function = module.get_function("main").unwrap();
    assert!(function.blocks.len() >= 3);
    let stats = function.stats();
    assert!(stats.phi_count >= 1);
}

