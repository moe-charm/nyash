//! PoC tests for MIR unified ops and VM execution

#[cfg(test)]
mod tests {
    use crate::backend::vm::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature};
    use crate::mir::{BasicBlockId, MirInstruction, ConstValue, EffectMask, Effect, MirType};

    fn make_main() -> MirFunction {
        let sig = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        MirFunction::new(sig, BasicBlockId::new(0))
    }

    #[test]
    fn vm_exec_typeop_check_and_cast() {
        let mut func = make_main();
        let bb = func.entry_block;

        let v0 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v0, value: ConstValue::Integer(42) });

        let v1 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::TypeOp { dst: v1, op: crate::mir::TypeOpKind::Check, value: v0, ty: MirType::Integer });

        // Print result (should be true)
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Print { value: v1, effects: EffectMask::IO });

        // Cast (no-op for PoC semantics)
        let v2 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::TypeOp { dst: v2, op: crate::mir::TypeOpKind::Cast, value: v0, ty: MirType::Integer });

        // Return void
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: None });

        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut vm = VM::new();
        let _ = vm.execute_module(&module).expect("VM should execute module");
    }

    #[test]
    fn vm_exec_legacy_typecheck_cast() {
        let mut func = make_main();
        let bb = func.entry_block;

        let v0 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v0, value: ConstValue::Integer(7) });

        let v1 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::TypeCheck { dst: v1, value: v0, expected_type: "IntegerBox".to_string() });
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Print { value: v1, effects: EffectMask::IO });

        let v2 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Cast { dst: v2, value: v0, target_type: MirType::Integer });

        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: None });

        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut vm = VM::new();
        let _ = vm.execute_module(&module).expect("VM should execute module");
    }

    #[test]
    fn vm_exec_unified_weakref_and_barrier() {
        let mut func = make_main();
        let bb = func.entry_block;

        let v0 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v0, value: ConstValue::Integer(1) });

        let v1 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::WeakRef { dst: v1, op: crate::mir::WeakRefOp::New, value: v0 });

        let v2 = func.next_value_id();
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::WeakRef { dst: v2, op: crate::mir::WeakRefOp::Load, value: v1 });

        // Optional barriers (no-op semantics)
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Barrier { op: crate::mir::BarrierOp::Read, ptr: v2 });
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Barrier { op: crate::mir::BarrierOp::Write, ptr: v2 });

        // Print loaded value
        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Print { value: v2, effects: EffectMask::IO });

        func.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: None });

        let mut module = MirModule::new("test".to_string());
        module.add_function(func);

        let mut vm = VM::new();
        let _ = vm.execute_module(&module).expect("VM should execute module");
    }
}
