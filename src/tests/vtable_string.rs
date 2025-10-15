#[test]
fn vtable_string_substring_concat() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // substring: "hello".substring(1,4) == "ell"
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let s = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: s,
            value: ConstValue::String("hello".into()),
        });
    let sb = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: sb, box_type: "StringBox".into(), args: vec![s],
                auto_birth: None });
    let i1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: i1,
            value: ConstValue::Integer(1),
        });
    let i4 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: i4,
            value: ConstValue::Integer(4),
        });
    let sub = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(sub), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("StringBox.substring/2".into())), args: vec![sb, i1, i4], effects: EffectMask::PURE });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sub) });
    let mut m = MirModule::new("str_sub".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "ell");

    // concat: "ab".concat("cd") == "abcd"
    let sig2 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f2 = MirFunction::new(sig2, BasicBlockId::new(0));
    let bb2 = f2.entry_block;
    let a = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: a,
            value: ConstValue::String("ab".into()),
        });
    let ab = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: ab, box_type: "StringBox".into(), args: vec![a],
                auto_birth: None });
    let c = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: c,
            value: ConstValue::String("cd".into()),
        });
    let joined = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(joined), func: crate::mir::ValueId::new(0), callee: Some(crate::mir::definitions::Callee::ModuleFunction("StringBox.concat/1".into())), args: vec![ab, c], effects: EffectMask::PURE });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(joined),
        });
    let mut m2 = MirModule::new("str_concat".into());
    m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "abcd");
}
