#[test]
fn vtable_map_boundary_cases() {
    use crate::backend::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // Case 1: empty-string key set/get/has
    let sig1 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f1 = MirFunction::new(sig1, BasicBlockId::new(0));
    let bb1 = f1.entry_block;
    let m = f1.next_value_id();
    f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::NewBox { dst: m, box_type: "MapBox".into(), args: vec![] , auto_birth: None });
    // set("", 1)
    let k_empty = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Const { dst: k_empty, value: ConstValue::String("".into()) });
    let v1 = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Const { dst: v1, value: ConstValue::Integer(1) });
    f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m, k_empty, v1], effects: EffectMask::PURE });
    // has("") -> true
    let h = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Call { dst: Some(h), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.has/1".into())), args: vec![m, k_empty], effects: EffectMask::PURE });
    // get("") -> 1
    let g = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Call { dst: Some(g), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.get/1".into())), args: vec![m, k_empty], effects: EffectMask::PURE });
    // return has + get (true->1) + size == 1 + 1 + 1 = 3 (coerce Bool true to 1 via toString parse in BinOp fallback)
    let sz = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Call { dst: Some(sz), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.size/0".into())), args: vec![m], effects: EffectMask::PURE });
    let tmp = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::BinOp { dst: tmp, op: crate::mir::BinaryOp::Add, lhs: h, rhs: g });
    let sum = f1.next_value_id(); f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::BinOp { dst: sum, op: crate::mir::BinaryOp::Add, lhs: tmp, rhs: sz });
    f1.get_block_mut(bb1).unwrap().add_instruction(MirInstruction::Return { value: Some(sum) });
    let mut m1 = MirModule::new("map_boundary_empty_key".into()); m1.add_function(f1);
    let mut vm1 = VM::new();
    let out1 = vm1.execute_module(&m1).expect("vm exec");
    // Expect 3 as described above
    assert_eq!(out1.to_string_box().value, "3");

    // Case 2: duplicate key overwrite, missing key get message shape, and delete using slot 205
    let sig2 = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let m2 = f2.next_value_id();
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::NewBox { dst: m2, box_type: "MapBox".into(), args: vec![] , auto_birth: None });
    // set("k", 1); set("k", 2)
    let k = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: k, value: ConstValue::String("k".into()) });
    let one = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: one, value: ConstValue::Integer(1) });
    let two = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: two, value: ConstValue::Integer(2) });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m2, k, one], effects: EffectMask::PURE });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m2, k, two], effects: EffectMask::PURE });
    // get("k") should be 2
    let g2 = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Call { dst: Some(g2), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.get/1".into())), args: vec![m2, k], effects: EffectMask::PURE });
    // delete("missing") using method name; ensure no panic and still size==1
    let missing = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Const { dst: missing, value: ConstValue::String("missing".into()) });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.delete/1".into())), args: vec![m2, missing], effects: EffectMask::PURE });
    // size()
    let sz2 = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Call { dst: Some(sz2), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.size/0".into())), args: vec![m2], effects: EffectMask::PURE });
    let sum2 = f2.next_value_id(); f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::BinOp { dst: sum2, op: crate::mir::BinaryOp::Add, lhs: g2, rhs: sz2 });
    f2.get_block_mut(bb2).unwrap().add_instruction(MirInstruction::Return { value: Some(sum2) });
    let mut m2m = MirModule::new("map_boundary_overwrite_delete".into()); m2m.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2m).expect("vm exec");
    // get("k") == 2 and size()==1 => 3
    assert_eq!(out2.to_string_box().value, "3");
}

