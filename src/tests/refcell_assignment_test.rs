use crate::interpreter::NyashInterpreter;
use crate::ast::{ASTNode, LiteralValue};
use crate::box_trait::{NyashBox, IntegerBox};

#[test]
fn assign_updates_refcell_variable_inner() {
    let mut interp = NyashInterpreter::new();
    // x = RefCell(1)
    let rc = crate::boxes::ref_cell_box::RefCellBox::new(Box::new(IntegerBox::new(1)));
    interp.declare_local_variable("x", Box::new(rc));

    // Execute: x = 42
    let target = ASTNode::Variable { name: "x".to_string(), span: crate::ast::Span::unknown() };
    let value = ASTNode::Literal { value: LiteralValue::Integer(42), span: crate::ast::Span::unknown() };
    let _ = interp.execute_assignment(&target, &value).expect("assign ok");

    // Verify x is still RefCell and inner == 42
    let xv = interp.resolve_variable("x").expect("x exists");
    let rc = xv.as_any().downcast_ref::<crate::boxes::ref_cell_box::RefCellBox>().expect("x is RefCellBox");
    let inner = rc.borrow();
    let ib = inner.as_any().downcast_ref::<IntegerBox>().expect("inner integer");
    assert_eq!(ib.value, 42);
}

#[test]
fn assign_updates_refcell_field_inner() {
    let mut interp = NyashInterpreter::new();
    // obj with field v = RefCell(5)
    let inst = crate::instance_v2::InstanceBox::from_declaration("Test".to_string(), vec!["v".to_string()], std::collections::HashMap::new());
    let rc = crate::boxes::ref_cell_box::RefCellBox::new(Box::new(IntegerBox::new(5)));
    let _ = inst.set_field("v", std::sync::Arc::from(Box::new(rc) as Box<dyn NyashBox>));
    // bind obj into local
    interp.declare_local_variable("obj", Box::new(inst.clone()));

    // Execute: obj.v = 7
    let target = ASTNode::FieldAccess { object: Box::new(ASTNode::Variable { name: "obj".to_string(), span: crate::ast::Span::unknown() }), field: "v".to_string(), span: crate::ast::Span::unknown() };
    let value = ASTNode::Literal { value: LiteralValue::Integer(7), span: crate::ast::Span::unknown() };
    let _ = interp.execute_assignment(&target, &value).expect("assign ok");

    // Verify obj.v inner == 7
    let sv = inst.get_field("v").expect("field exists");
    let rc = sv.as_any().downcast_ref::<crate::boxes::ref_cell_box::RefCellBox>().expect("v is RefCellBox");
    let inner = rc.borrow();
    let ib = inner.as_any().downcast_ref::<IntegerBox>().expect("inner integer");
    assert_eq!(ib.value, 7);
}

#[test]
fn closure_reads_updated_refcell_capture() {
    let mut interp = NyashInterpreter::new();
    // local x = 10
    interp.declare_local_variable("x", Box::new(IntegerBox::new(10)));
    // Build lambda: () { x }
    let lam = ASTNode::Lambda { params: vec![], body: vec![
        ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "x".to_string(), span: crate::ast::Span::unknown() })), span: crate::ast::Span::unknown() }
    ], span: crate::ast::Span::unknown() };
    // Evaluate lambda to FunctionBox
    let f = interp.execute_expression(&lam).expect("lambda eval");
    // x = 20 (should update RefCell capture)
    let target = ASTNode::Variable { name: "x".to_string(), span: crate::ast::Span::unknown() };
    let value = ASTNode::Literal { value: LiteralValue::Integer(20), span: crate::ast::Span::unknown() };
    let _ = interp.execute_assignment(&target, &value).expect("assign ok");
    // Call f()
    let call = ASTNode::Call { callee: Box::new(lam.clone()), arguments: vec![], span: crate::ast::Span::unknown() };
    let out = interp.execute_expression(&call).expect("call ok");
    let ib = out.as_any().downcast_ref::<IntegerBox>().expect("integer ret");
    assert_eq!(ib.value, 20);
}
