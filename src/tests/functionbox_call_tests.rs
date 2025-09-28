#![cfg(feature = "interpreter-legacy")]

use crate::interpreter::NyashInterpreter;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, IntegerBox};

#[test]
fn functionbox_call_via_variable_with_capture() {
    let mut interp = NyashInterpreter::new();
    // local x = 10
    interp.declare_local_variable("x", Box::new(IntegerBox::new(10)));

    // f = function() { return x }
    let lam = ASTNode::Lambda { params: vec![], body: vec![
        ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "x".to_string(), span: crate::ast::Span::unknown() })), span: crate::ast::Span::unknown() }
    ], span: crate::ast::Span::unknown() };
    let assign_f = ASTNode::Assignment {
        target: Box::new(ASTNode::Variable { name: "f".to_string(), span: crate::ast::Span::unknown() }),
        value: Box::new(lam.clone()),
        span: crate::ast::Span::unknown(),
    };
    let _ = interp.execute_statement(&assign_f).expect("assign f");

    // x = 20
    let assign_x = ASTNode::Assignment {
        target: Box::new(ASTNode::Variable { name: "x".to_string(), span: crate::ast::Span::unknown() }),
        value: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Integer(20), span: crate::ast::Span::unknown() }),
        span: crate::ast::Span::unknown(),
    };
    let _ = interp.execute_statement(&assign_x).expect("assign x");

    // return f()
    let call_f = ASTNode::Call { callee: Box::new(ASTNode::Variable { name: "f".to_string(), span: crate::ast::Span::unknown() }), arguments: vec![], span: crate::ast::Span::unknown() };
    let out = interp.execute_expression(&call_f).expect("call f");
    let ib = out.as_any().downcast_ref::<IntegerBox>().expect("integer ret");
    assert_eq!(ib.value, 20);
}

#[test]
fn functionbox_call_via_field() {
    let mut interp = NyashInterpreter::new();
    // obj with field f
    let inst = crate::instance_v2::InstanceBox::from_declaration("C".to_string(), vec!["f".to_string()], std::collections::HashMap::new());
    interp.declare_local_variable("obj", Box::new(inst.clone()));

    // obj.f = function(a){ return a }
    let lam = ASTNode::Lambda { params: vec!["a".to_string()], body: vec![
        ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "a".to_string(), span: crate::ast::Span::unknown() })), span: crate::ast::Span::unknown() }
    ], span: crate::ast::Span::unknown() };
    let assign = ASTNode::Assignment {
        target: Box::new(ASTNode::FieldAccess { object: Box::new(ASTNode::Variable { name: "obj".to_string(), span: crate::ast::Span::unknown() }), field: "f".to_string(), span: crate::ast::Span::unknown() }),
        value: Box::new(lam.clone()),
        span: crate::ast::Span::unknown(),
    };
    let _ = interp.execute_statement(&assign).expect("assign field");

    // return (obj.f)(7)
    let call = ASTNode::Call {
        callee: Box::new(ASTNode::FieldAccess { object: Box::new(ASTNode::Variable { name: "obj".to_string(), span: crate::ast::Span::unknown() }), field: "f".to_string(), span: crate::ast::Span::unknown() }),
        arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::Integer(7), span: crate::ast::Span::unknown() }],
        span: crate::ast::Span::unknown(),
    };
    let out = interp.execute_expression(&call).expect("call obj.f");
    let ib = out.as_any().downcast_ref::<IntegerBox>().expect("integer ret");
    assert_eq!(ib.value, 7);
}
