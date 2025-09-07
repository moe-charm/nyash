#[cfg(all(test, not(feature = "jit-direct-only")))]
mod tests {
    use crate::backend::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, BasicBlockId, MirInstruction, ConstValue, EffectMask, MirType};

    fn make_string_len() -> MirModule {
        let mut module = MirModule::new("identical_string".to_string());
        let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
        let mut f = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = f.entry_block;
        let s = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: s, value: ConstValue::String("hello".into()) });
        let ln = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(ln), box_val: s, method: "len".into(), args: vec![], method_id: None, effects: EffectMask::PURE });
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(ln) });
        module.add_function(f);
        module
    }

    #[cfg(feature = "cranelift-jit")]
    #[test]
    fn identical_vm_and_jit_string_len() {
        // Prefer vtable on VM and host-bridge on JIT for parity
        std::env::set_var("NYASH_ABI_VTABLE", "1");
        std::env::set_var("NYASH_JIT_HOST_BRIDGE", "1");
        let module = make_string_len();

        // VM
        let mut vm = VM::new();
        let vm_out = vm.execute_module(&module).expect("VM exec");
        let vm_s = vm_out.to_string_box().value;

        // JIT
        let jit_out = crate::backend::cranelift_compile_and_execute(&module, "identical_string").expect("JIT exec");
        let jit_s = jit_out.to_string_box().value;

        assert_eq!(vm_s, jit_s, "VM and JIT results should match for String.len");
    }
}
