#[test]
fn vtable_map_set_and_strict_unknown() {
    use crate::backend::vm::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // Build: new MapBox; call set("k","v"); size(); return size
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let mapv = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: mapv, box_type: "MapBox".into(), args: vec![],
        , auto_birth: None });
    let k = f.next_value_id();
    let v = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: k,
            value: ConstValue::String("k".into()),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: v,
            value: ConstValue::String("v".into()),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: mapv,
            method: "set".into(),
            args: vec![k, v],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let sz = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(sz),
            box_val: mapv,
            method: "size".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sz) });
    let mut m = MirModule::new("t".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    let s = out.to_string_box().value;
    assert_eq!(s, "1");

    // STRICT unknown method on MapBox should error
    std::env::set_var("NYASH_ABI_STRICT", "1");
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Void,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let m2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: m2, box_type: "MapBox".into(), args: vec![],
        , auto_birth: None });
    // Call unknown method
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: m2,
            method: "unknown".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: None });
    let mut mm = MirModule::new("t2".into());
    mm.add_function(f2);
    let mut vm2 = VM::new();
    let res = vm2.execute_module(&mm);
    assert!(res.is_err(), "STRICT should error on unknown vtable method");
}
