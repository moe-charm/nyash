#[cfg(feature = "cranelift-jit")]
#[test]
fn core13_jit_map_set_get_size() {
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue, MirType};
    // Build: m = new MapBox(); m.set("k", 11); r = m.size()+m.get("k"); return r
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let m = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::NewBox { dst: m, box_type: "MapBox".into(), args: vec![] , auto_birth: None , auto_birth: None });
    // set("k", 11)
    let k = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: k, value: ConstValue::String("k".into()) });
    let v = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v, value: ConstValue::Integer(11) });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: None, box_val: m, method: "set".into(), args: vec![k, v], method_id: None, effects: EffectMask::PURE });
    // size()
    let sz = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(sz), box_val: m, method: "size".into(), args: vec![], method_id: None, effects: EffectMask::PURE });
    // get("k")
    let k2 = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: k2, value: ConstValue::String("k".into()) });
    let gk = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(gk), box_val: m, method: "get".into(), args: vec![k2], method_id: None, effects: EffectMask::PURE });
    // sum
    let sum = f.next_value_id(); f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BinOp { dst: sum, op: crate::mir::BinaryOp::Add, lhs: sz, rhs: gk });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(sum) });
    let mut module = MirModule::new("core13_jit_map_set_get_size".into()); module.add_function(f);
    let out = crate::backend::cranelift_compile_and_execute(&module, "core13_jit_map").expect("JIT exec");
    assert_eq!(out.to_string_box().value, "12");
}

