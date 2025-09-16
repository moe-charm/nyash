#[test]
fn vtable_array_sort_reverse_slice() {
    use crate::backend::vm::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // sort: push 3,1,2 -> sort() -> get(0) == 1
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let arr = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: arr,
            box_type: "ArrayBox".into(),
            args: vec![],
        });
    let c3 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: c3,
            value: ConstValue::Integer(3),
        });
    let c1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: c1,
            value: ConstValue::Integer(1),
        });
    let c2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: c2,
            value: ConstValue::Integer(2),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: arr,
            method: "push".into(),
            args: vec![c3],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: arr,
            method: "push".into(),
            args: vec![c1],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: arr,
            method: "push".into(),
            args: vec![c2],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: arr,
            method: "sort".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let idx0 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: idx0,
            value: ConstValue::Integer(0),
        });
    let got = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(got),
            box_val: arr,
            method: "get".into(),
            args: vec![idx0],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(got) });
    let mut m = MirModule::new("arr_sort".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "1");

    // reverse: push 1,2 -> reverse() -> get(0) == 2
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let a2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: a2,
            box_type: "ArrayBox".into(),
            args: vec![],
        });
    let i1 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: i1,
            value: ConstValue::Integer(1),
        });
    let i2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: i2,
            value: ConstValue::Integer(2),
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a2,
            method: "push".into(),
            args: vec![i1],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a2,
            method: "push".into(),
            args: vec![i2],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a2,
            method: "reverse".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let z0 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: z0,
            value: ConstValue::Integer(0),
        });
    let g2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(g2),
            box_val: a2,
            method: "get".into(),
            args: vec![z0],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(g2) });
    let mut m2 = MirModule::new("arr_reverse".into());
    m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "2");

    // slice: push "a","b","c" -> slice(0,2) -> len()==2
    let sig3 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f3 = MirFunction::new(sig3, BasicBlockId::new(0));
    let bb3 = f3.entry_block;
    let a3 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: a3,
            box_type: "ArrayBox".into(),
            args: vec![],
        });
    let sa = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sa,
            value: ConstValue::String("a".into()),
        });
    let sb = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sb,
            value: ConstValue::String("b".into()),
        });
    let sc = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sc,
            value: ConstValue::String("c".into()),
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a3,
            method: "push".into(),
            args: vec![sa],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a3,
            method: "push".into(),
            args: vec![sb],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a3,
            method: "push".into(),
            args: vec![sc],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let s0 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s0,
            value: ConstValue::Integer(0),
        });
    let s2 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s2,
            value: ConstValue::Integer(2),
        });
    let sub = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(sub),
            box_val: a3,
            method: "slice".into(),
            args: vec![s0, s2],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let ln = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(ln),
            box_val: sub,
            method: "len".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(ln) });
    let mut m3 = MirModule::new("arr_slice".into());
    m3.add_function(f3);
    let mut vm3 = VM::new();
    let out3 = vm3.execute_module(&m3).expect("vm exec");
    assert_eq!(out3.to_string_box().value, "2");
}
