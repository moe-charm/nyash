use nyash_rust::mir::instruction_introspection;

// Core-15: enforce fixed instruction count at 15 (migration mode; docs may differ)
#[test]
fn mir_core15_instruction_count() {
    let impl_names = instruction_introspection::core15_instruction_names();
    assert_eq!(impl_names.len(), 15, "Core-15 must contain exactly 15 instructions");
}
