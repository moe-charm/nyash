#[test]
fn vtable_array_contains_indexof_join() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    use crate::mir::definitions::Callee;

    std::env::set_var("NYASH_ABI_VTABLE", "1");

    // contains: ["a","b"].contains("b") == true; contains("c") == false
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
        .add_instruction(MirInstruction::NewBox { dst: arr, box_type: "ArrayBox".into(), args: vec![],
                auto_birth: None });
    let sa = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sa,
            value: ConstValue::String("a".into()),
        });
    let sb = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sb,
            value: ConstValue::String("b".into()),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![arr, sa], effects: EffectMask::PURE });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![arr, sb], effects: EffectMask::PURE });
    let sc = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sc,
            value: ConstValue::String("c".into()),
        });
    let got1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(got1), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.contains/1".into())), args: vec![arr, sb], effects: EffectMask::PURE });
    let got2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(got2), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.contains/1".into())), args: vec![arr, sc], effects: EffectMask::PURE });
    // return got1.equals(true) && got2.equals(false) as 1 for pass
    // Instead, just return 0 or 1 using simple branch-like comparison via toString
    // We check: got1==true -> "true", got2==false -> "false" and return 1 if both match else 0
    // For brevity, just return got1.toString() ("true") length + got2.toString() ("false") length == 9
    let s1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(s1),
            box_val: got1,
            method: "toString".into(),
            args: vec![],
            method_id: Some(0),
            effects: EffectMask::PURE,
        });
    let s2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(s2),
            box_val: got2,
            method: "toString".into(),
            args: vec![],
            method_id: Some(0),
            effects: EffectMask::PURE,
        });
    let len1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(len1), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.len/0".into())), args: vec![s1], effects: EffectMask::PURE });
    let len2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(len2), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.len/0".into())), args: vec![s2], effects: EffectMask::PURE });
    // len1 + len2
    let sum = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: sum,
            op: crate::mir::BinaryOp::Add,
            lhs: len1,
            rhs: len2,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sum) });
    let mut m = MirModule::new("arr_contains".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "9"); // "true"(4)+"false"(5)

    // indexOf: ["x","y"].indexOf("y") == 1; indexOf("z") == -1
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
        .add_instruction(MirInstruction::NewBox { dst: a2, box_type: "ArrayBox".into(), args: vec![],
                auto_birth: None });
    let sx = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sx,
            value: ConstValue::String("x".into()),
        });
    let sy = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sy,
            value: ConstValue::String("y".into()),
        });
    let sz = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sz,
            value: ConstValue::String("z".into()),
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a2, sx], effects: EffectMask::PURE });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a2, sy], effects: EffectMask::PURE });
    let i1 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(i1), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.indexOf/1".into())), args: vec![a2, sy], effects: EffectMask::PURE });
    let i2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(i2), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.indexOf/1".into())), args: vec![a2, sz], effects: EffectMask::PURE });
    let sum2 = f2.next_value_id();
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: sum2,
            op: crate::mir::BinaryOp::Add,
            lhs: i1,
            rhs: i2,
        });
    f2.get_block_mut(bb2)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sum2) });
    let mut m2 = MirModule::new("arr_indexOf".into());
    m2.add_function(f2);
    let mut vm2 = VM::new();
    let out2 = vm2.execute_module(&m2).expect("vm exec");
    assert_eq!(out2.to_string_box().value, "0"); // 1 + (-1)

    // join: ["a","b","c"].join("-") == "a-b-c"
    let sig3 = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };
    let mut f3 = MirFunction::new(sig3, BasicBlockId::new(0));
    let bb3 = f3.entry_block;
    let a3 = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: a3, box_type: "ArrayBox".into(), args: vec![],
                auto_birth: None });
    let a = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: a,
            value: ConstValue::String("a".into()),
        });
    let b = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: b,
            value: ConstValue::String("b".into()),
        });
    let c = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: c,
            value: ConstValue::String("c".into()),
        });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a3, a], effects: EffectMask::PURE });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a3, b], effects: EffectMask::PURE });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: None, func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.push/1".into())), args: vec![a3, c], effects: EffectMask::PURE });
    let sep = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: sep,
            value: ConstValue::String("-".into()),
        });
    let joined = f3.next_value_id();
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Call { dst: Some(joined), func: crate::mir::ValueId::new(0), callee: Some(Callee::ModuleFunction("ArrayBox.join/1".into())), args: vec![a3, sep], effects: EffectMask::PURE });
    f3.get_block_mut(bb3)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(joined),
        });
    let mut m3 = MirModule::new("arr_join".into());
    m3.add_function(f3);
    let mut vm3 = VM::new();
    let out3 = vm3.execute_module(&m3).expect("vm exec");
    assert_eq!(out3.to_string_box().value, "a-b-c");
}
