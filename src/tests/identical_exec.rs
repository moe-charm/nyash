#[cfg(test)]
mod tests {
    use crate::mir::{MirModule, MirFunction, FunctionSignature};
    use crate::mir::{BasicBlockId, MirInstruction, ConstValue, EffectMask, MirType, BinaryOp};
    use crate::backend::VM;

    fn make_add_main(a: i64, b: i64) -> MirModule {
        let sig = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Integer,
            effects: EffectMask::PURE,
        };
        let mut func = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = func.entry_block;
        let v0 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v0, value: ConstValue::Integer(a) });
        let v1 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v1, value: ConstValue::Integer(b) });
        let v2 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: v2, op: BinaryOp::Add, lhs: v0, rhs: v1 });
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(v2) });
        let mut module = MirModule::new("identical".to_string());
        module.add_function(func);
        module
    }

    #[cfg(feature = "cranelift-jit")]
    #[test]
    fn identical_vm_and_jit_add() {
        let module = make_add_main(7, 35);
        // Run VM
        let mut vm = VM::new();
        let vm_out = vm.execute_module(&module).expect("VM exec");
        let vm_s = vm_out.to_string_box().value;

        // Run JIT (Cranelift minimal)
        let jit_out = crate::backend::cranelift_compile_and_execute(&module, "identical_jit").expect("JIT exec");
        let jit_s = jit_out.to_string_box().value;

        assert_eq!(vm_s, jit_s, "VM and JIT results should match");
    }
}

