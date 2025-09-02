#[test]
fn vtable_array_and_string_len_get_set() {
    use crate::backend::vm::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // Array: set(0, "x"); len(); get(0)
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::String, effects: EffectMask::PURE };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let arr = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::NewBox { dst: arr, box_type: "ArrayBox".into(), args: vec![] });
    let idx0 = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: idx0, value: ConstValue::Integer(0) });
    let sval = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: sval, value: ConstValue::String("x".into()) });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: None, box_val: arr, method: "set".into(), args: vec![idx0, sval], method_id: None, effects: EffectMask::PURE });
    let lenv = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(lenv), box_val: arr, method: "len".into(), args: vec![], method_id: None, effects: EffectMask::PURE });
    // sanity: len should be 1 (not asserted here, just exercise path)
    let got = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(got), box_val: arr, method: "get".into(), args: vec![idx0], method_id: None, effects: EffectMask::PURE });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(got) });
    let mut m = MirModule::new("tarr".into()); m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "x");

    // String: len()
    let sig2 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let s = f2.next_value_id();
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: s, value: ConstValue::String("abc".into()) });
    let sb = f2.next_value_id();
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::NewBox { dst: sb, box_type: "StringBox".into(), args: vec![s] });
    let ln = f2.next_value_id();
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(ln), box_val: sb, method: "len".into(), args: vec![], method_id: None, effects: EffectMask::PURE });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Return { value: Some(ln) });
    let mut m2 = MirModule::new("tstr".into()); m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "3");
}

