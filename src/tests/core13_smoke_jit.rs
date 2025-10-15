#[cfg(feature = "cranelift-jit")]
#[test]
fn core13_jit_array_push_len_get() {
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    // Build: a = new ArrayBox(); a.push(3); ret a.len()+a.get(0)
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let a = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::NewBox { dst: a, box_type: "ArrayBox".into(), args: vec![] , auto_birth: None });
    let three = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: three, value: ConstValue::Integer(3) });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a, three], effects: EffectMask::PURE });
    let ln = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: Some(ln), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.len/0".into())), args: vec![a], effects: EffectMask::PURE });
    let zero = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: zero, value: ConstValue::Integer(0) });
    let g0 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: Some(g0), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.get/1".into())), args: vec![a, zero], effects: EffectMask::PURE });
    let sum = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: sum, op: crate::mir::BinaryOp::Add, lhs: ln, rhs: g0 });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(sum) });
    let mut m = MirModule::new("core13_jit_array_push_len_get".into()); m.add_function(f);
    let jit_out = crate::backend::cranelift_compile_and_execute(&m, "core13_jit_array").expect("JIT exec");
    assert_eq!(jit_out.to_string_box().value, "4");
}

#[cfg(feature = "cranelift-jit")]
#[test]
fn core13_jit_array_set_get() {
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    // Build: a = new ArrayBox(); a.set(0, 9); ret a.get(0)
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let a = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::NewBox { dst: a, box_type: "ArrayBox".into(), args: vec![] , auto_birth: None });
    let zero = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: zero, value: ConstValue::Integer(0) });
    let nine = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: nine, value: ConstValue::Integer(9) });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.set/2".into())), args: vec![a, zero, nine], effects: EffectMask::PURE });
    let z2 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: z2, value: ConstValue::Integer(0) });
    let outv = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: Some(outv), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.get/1".into())), args: vec![a, z2], effects: EffectMask::PURE });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(outv) });
    let mut m = MirModule::new("core13_jit_array_set_get".into()); m.add_function(f);
    let jit_out = crate::backend::cranelift_compile_and_execute(&m, "core13_jit_array2").expect("JIT exec");
    assert_eq!(jit_out.to_string_box().value, "9");
}

