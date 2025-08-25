#[test]
fn vm_compare_integerbox_boxref_lt() {
    use crate::backend::vm::VM;
    use crate::backend::vm::VMValue;
    use crate::box_trait::IntegerBox;
    use std::sync::Arc;
    
    let vm = VM::new();
    let left = VMValue::BoxRef(Arc::new(IntegerBox::new(0)));
    let right = VMValue::BoxRef(Arc::new(IntegerBox::new(3)));
    let out = vm.execute_compare_op(&crate::mir::CompareOp::Lt, &left, &right).unwrap();
    assert!(out, "0 < 3 should be true");
}
