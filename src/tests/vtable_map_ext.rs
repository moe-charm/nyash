#[test]
fn vtable_map_keys_values_delete_clear() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // keys/values size check
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let m = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: m, box_type: "MapBox".into(), args: vec![],
                auto_birth: None });
    // set two entries
    let k1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: k1,
            value: ConstValue::String("a".into()),
        });
    let v1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: v1,
            value: ConstValue::Integer(1),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m, k1, v1], effects: EffectMask::PURE });
    let k2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: k2,
            value: ConstValue::String("b".into()),
        });
    let v2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: v2,
            value: ConstValue::Integer(2),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m, k2, v2], effects: EffectMask::PURE });
    // keys().len + values().len == 4
    let keys = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(keys), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.keys/0".into())), args: vec![m], effects: EffectMask::PURE });
    let klen = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(klen), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.len/0".into())), args: vec![keys], effects: EffectMask::PURE });
    let vals = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(vals), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.values/0".into())), args: vec![m], effects: EffectMask::PURE });
    let vlen = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(vlen), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("ArrayBox.len/0".into())), args: vec![vals], effects: EffectMask::PURE });
    let sum = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: sum,
            op: crate::mir::BinaryOp::Add,
            lhs: klen,
            rhs: vlen,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sum) });
    let mut m1 = MirModule::new("map_keys_values".into());
    m1.add_function(f);
    let mut vm1 = VM::new();
    let out1 = vm1.execute_module(&m1).expect("vm exec");
    assert_eq!(out1.to_string_box().value, "4");

    // delete + clear → size 0
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let m2v = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: m2v, box_type: "MapBox".into(), args: vec![],
                auto_birth: None });
    let k = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: k,
            value: ConstValue::String("x".into()),
        });
    let v = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: v,
            value: ConstValue::String("y".into()),
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.set/2".into())), args: vec![m2v, k, v], effects: EffectMask::PURE });
    let dk = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: dk,
            value: ConstValue::String("x".into()),
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.delete/1".into())), args: vec![m2v, dk], effects: EffectMask::PURE });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.clear/0".into())), args: vec![m2v], effects: EffectMask::PURE });
    let sz = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(sz), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("MapBox.size/0".into())), args: vec![m2v], effects: EffectMask::PURE });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sz) });
    let mut mm2 = MirModule::new("map_delete_clear".into());
    mm2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&mm2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "0");
}
