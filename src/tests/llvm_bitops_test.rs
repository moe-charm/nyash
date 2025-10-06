#[test]
fn llvm_bitops_compile_and_exec() {
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, BasicBlockId, ConstValue, MirType, instruction::BinaryOp};
    use crate::backend::VM;

    // Build MIR: compute sum of bitwise/shift ops -> 48
    let sig = FunctionSignature { name: "Main.main".into(), params: vec![], return_type: MirType::Integer, effects: Default::default() };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    // Constants
    let c5 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c5, value: ConstValue::Integer(5) });
    let c3 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c3, value: ConstValue::Integer(3) });
    let c2 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c2, value: ConstValue::Integer(2) });
    let c1 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c1, value: ConstValue::Integer(1) });
    let c32 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c32, value: ConstValue::Integer(32) });
    let c5_sh = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c5_sh, value: ConstValue::Integer(5) });
    let c3_sh = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: c3_sh, value: ConstValue::Integer(3) });

    // a = 5 & 3 -> 1
    let a = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: a, op: BinaryOp::BitAnd, lhs: c5, rhs: c3 });
    // b = 5 | 2 -> 7
    let b = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: b, op: BinaryOp::BitOr, lhs: c5, rhs: c2 });
    // c = 5 ^ 1 -> 4
    let c = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: c, op: BinaryOp::BitXor, lhs: c5, rhs: c1 });
    // d = 1 << 5 -> 32
    let d = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: d, op: BinaryOp::Shl, lhs: c1, rhs: c5_sh });
    // e = 32 >> 3 -> 4
    let e = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: e, op: BinaryOp::Shr, lhs: c32, rhs: c3_sh });

    // sum = a + b + c + d + e
    let t1 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: t1, op: BinaryOp::Add, lhs: a, rhs: b });
    let t2 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: t2, op: BinaryOp::Add, lhs: t1, rhs: c });
    let t3 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: t3, op: BinaryOp::Add, lhs: t2, rhs: d });
    let sum = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: sum, op: BinaryOp::Add, lhs: t3, rhs: e });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(sum) });

    let mut m = MirModule::new("bitops".into()); m.add_function(f);

    // VM executes to 48
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "48");

    // LLVM: ensure lowering/emit succeeds; compile_and_execute should also return 48 (via MIR interpreter fallback)
    #[cfg(feature = "llvm-inkwell-legacy")]
    {
        use crate::backend::llvm;
        let tmp = format!("{}/target/aot_objects/test_bitops", env!("CARGO_MANIFEST_DIR"));
        llvm::compile_to_object(&m, &format!("{}.o", tmp)).expect("llvm emit");
        let out2 = llvm::compile_and_execute(&m, &tmp).expect("llvm compile&exec");
        assert_eq!(out2.to_string_box().value, "48");
    }
}
