#[test]
fn vtable_string_indexof_replace_trim_upper_lower() {
    use crate::backend::vm::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // indexOf("b") in "abc" == 1
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let s = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s,
            value: ConstValue::String("abc".into()),
        });
    let sb = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: sb, box_type: "StringBox".into(), args: vec![s],
        , auto_birth: None , auto_birth: None });
    let b = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: b,
            value: ConstValue::String("b".into()),
        });
    let idx = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(idx),
            box_val: sb,
            method: "indexOf".into(),
            args: vec![b],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(idx) });
    let mut m = MirModule::new("str_indexof".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "1");

    // replace: "a-b" -> replace("-","+") == "a+b"
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let s2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s2,
            value: ConstValue::String("a-b".into()),
        });
    let sb2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: sb2, box_type: "StringBox".into(), args: vec![s2],
        , auto_birth: None , auto_birth: None });
    let dash = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: dash,
            value: ConstValue::String("-".into()),
        });
    let plus = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: plus,
            value: ConstValue::String("+".into()),
        });
    let rep = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(rep),
            box_val: sb2,
            method: "replace".into(),
            args: vec![dash, plus],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(rep) });
    let mut m2 = MirModule::new("str_replace".into());
    m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "a+b");

    // trim + toUpper + toLower: "  Xy  " -> trim=="Xy" -> upper=="XY" -> lower=="xy"
    let sig3 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f3 = MirFunction::new(sig3, BasicBlockId::new(0));
    let bb3 = f3.entry_block;
    let s3 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s3,
            value: ConstValue::String("  Xy  ".into()),
        });
    let sb3 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: sb3, box_type: "StringBox".into(), args: vec![s3],
        , auto_birth: None , auto_birth: None });
    let t = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(t),
            box_val: sb3,
            method: "trim".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let u = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(u),
            box_val: t,
            method: "toUpper".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let l = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(l),
            box_val: u,
            method: "toLower".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(l) });
    let mut m3 = MirModule::new("str_trim_upper_lower".into());
    m3.add_function(f3);
    let mut vm3 = VM::new();
    let out3 = vm3.execute_module(&m3).expect("vm exec");
    assert_eq!(out3.to_string_box().value, "xy");
}
