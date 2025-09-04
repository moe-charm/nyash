#[test]
fn vtable_string_boundary_cases() {
    use crate::backend::vm::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // Case 1: empty string length == 0
    let sig1 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f1 = MirFunction::new(sig1, BasicBlockId::new(0));
    let bb1 = f1.entry_block;
    let s = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Const { dst: s, value: ConstValue::String("".into()) });
    let sb = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::NewBox { dst: sb, box_type: "StringBox".into(), args: vec![s] });
    let ln = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(ln), box_val: sb, method: "len".into(), args: vec![], method_id: Some(300), effects: EffectMask::PURE });
    f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Return { value: Some(ln) });
    let mut m1 = MirModule::new("str_empty_len".into()); m1.add_function(f1);
    let mut vm1 = VM::new();
    let out1 = vm1.execute_module(&m1).expect("vm exec");
    assert_eq!(out1.to_string_box().value, "0");

    // Case 2: indexOf not found returns -1
    let sig2 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let s2 = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: s2, value: ConstValue::String("abc".into()) });
    let sb2 = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::NewBox { dst: sb2, box_type: "StringBox".into(), args: vec![s2] });
    let z = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: z, value: ConstValue::String("z".into()) });
    let idx = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(idx), box_val: sb2, method: "indexOf".into(), args: vec![z], method_id: Some(303), effects: EffectMask::PURE });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Return { value: Some(idx) });
    let mut m2 = MirModule::new("str_indexof_not_found".into()); m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "-1");

    // Case 3: Unicode substring by character indices: "a😊b"[1..2] == "😊"
    let sig3 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::String, effects: EffectMask::PURE };
    let mut f3 = MirFunction::new(sig3, BasicBlockId::new(0));
    let bb3 = f3.entry_block;
    let s3 = f3.next_value_id(); f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::Const { dst: s3, value: ConstValue::String("a😊b".into()) });
    let sb3 = f3.next_value_id(); f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::NewBox { dst: sb3, box_type: "StringBox".into(), args: vec![s3] });
    let sub = f3.next_value_id(); f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(sub), box_val: sb3, method: "substring".into(), args: vec![
        { let v = f3.next_value_id(); f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::Const { dst: v, value: ConstValue::Integer(1) }); v },
        { let v = f3.next_value_id(); f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::Const { dst: v, value: ConstValue::Integer(2) }); v },
    ], method_id: Some(301), effects: EffectMask::PURE });
    f3.get_block_mut(bb3).unwrap().add_instruction(MirInstruction::Return { value: Some(sub) });
    let mut m3 = MirModule::new("str_unicode_substring".into()); m3.add_function(f3);
    let mut vm3 = VM::new();
    let out3 = vm3.execute_module(&m3).expect("vm exec");
    assert_eq!(out3.to_string_box().value, "😊");
}

