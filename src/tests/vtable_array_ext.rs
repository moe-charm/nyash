#[test]
fn vtable_array_push_get_len_pop_clear() {
    use crate::backend::vm::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // Case 1: push("x"); get(0)
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let arr = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: arr, box_type: "ArrayBox".into(), args: vec![],
        , auto_birth: None , auto_birth: None });
    let sval = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sval,
            value: ConstValue::String("x".into()),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: arr,
            method: "push".into(),
            args: vec![sval],
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
    let mut m = MirModule::new("arr_push_get".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "x");

    // Case 2: push("y"); pop() -> "y"
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let a2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: a2, box_type: "ArrayBox".into(), args: vec![],
        , auto_birth: None , auto_birth: None });
    let y = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: y,
            value: ConstValue::String("y".into()),
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a2,
            method: "push".into(),
            args: vec![y],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let popped = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(popped),
            box_val: a2,
            method: "pop".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(popped),
        });
    let mut m2 = MirModule::new("arr_pop".into());
    m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "y");

    // Case 3: push("z"); clear(); len() -> 0
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
        .add_instruction(MirInstruction::NewBox { dst: a3, box_type: "ArrayBox".into(), args: vec![],
        , auto_birth: None , auto_birth: None });
    let z = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: z,
            value: ConstValue::String("z".into()),
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a3,
            method: "push".into(),
            args: vec![z],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a3,
            method: "clear".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let ln = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(ln),
            box_val: a3,
            method: "len".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(ln) });
    let mut m3 = MirModule::new("arr_clear_len".into());
    m3.add_function(f3);
    let mut vm3 = VM::new();
    let out3 = vm3.execute_module(&m3).expect("vm exec");
    assert_eq!(out3.to_string_box().value, "0");
}
