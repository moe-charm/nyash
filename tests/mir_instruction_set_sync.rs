use nyash_rust::mir::instruction_introspection;

// MIR14: ensure instruction roster remains stable
#[test]
fn mir14_shape_is_fixed() {
    let impl_names = instruction_introspection::mir14_instruction_names();
    assert_eq!(impl_names.len(), 14, "MIR14 must contain exactly 14 instructions");
    assert!(impl_names.contains(&"UnaryOp"), "MIR14 must include UnaryOp");
}
