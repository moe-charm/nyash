use nyash_rust::mir::instruction_introspection;

// MIR14: ensure instruction count stays fixed at 14
#[test]
fn mir14_instruction_count() {
    let impl_names = instruction_introspection::mir14_instruction_names();
    assert_eq!(impl_names.len(), 14, "MIR14 must contain exactly 14 instructions");
}
