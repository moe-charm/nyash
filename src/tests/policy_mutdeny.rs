#[cfg(feature = "cranelift-jit")]
#[test]
#[ignore]
fn jit_readonly_array_push_denied() {
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    // Ensure read-only policy is on
    std::env::set_var("NYASH_JIT_READ_ONLY", "1");

    // Build: a = new ArrayBox(); a.push(3); ret a.len()
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let a = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: a, box_type: "ArrayBox".into(), args: vec![],
                auto_birth: None });
    let three = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: three,
            value: ConstValue::Integer(3),
        });
    // push should be denied under read-only policy, effectively no-op for length
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: a,
            method: "push".into(),
            args: vec![three],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let ln = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(ln),
            box_val: a,
            method: "len".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(ln) });
    let mut m = MirModule::new("jit_readonly_array_push_denied".into());
    m.add_function(f);
    let out = crate::backend::cranelift_compile_and_execute(&m, "jit_readonly_array_push_denied")
        .expect("JIT exec");
    assert_eq!(
        out.to_string_box().value,
        "0",
        "Array.push must be denied under read-only policy"
    );
}

#[cfg(feature = "cranelift-jit")]
#[test]
#[ignore]
fn jit_readonly_map_set_denied() {
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    // Ensure read-only policy is on
    std::env::set_var("NYASH_JIT_READ_ONLY", "1");

    // Build: m = new MapBox(); m.set("a", 2); ret m.size()
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let mbox = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: mbox, box_type: "MapBox".into(), args: vec![],
                auto_birth: None });
    let key = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key,
            value: ConstValue::String("a".into()),
        });
    let val = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: val,
            value: ConstValue::Integer(2),
        });
    // set should be denied under read-only policy
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: mbox,
            method: "set".into(),
            args: vec![key, val],
            method_id: None,
            effects: EffectMask::PURE,
        });
    let sz = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BoxCall {
            dst: Some(sz),
            box_val: mbox,
            method: "size".into(),
            args: vec![],
            method_id: None,
            effects: EffectMask::PURE,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(sz) });
    let mut module = MirModule::new("jit_readonly_map_set_denied".into());
    module.add_function(f);
    let out = crate::backend::cranelift_compile_and_execute(&module, "jit_readonly_map_set_denied")
        .expect("JIT exec");
    assert_eq!(
        out.to_string_box().value,
        "0",
        "Map.set must be denied under read-only policy"
    );
}

// Engine-independent smoke: validate policy denial via host externs
#[cfg(feature = "cranelift-jit")]
#[test]
fn extern_readonly_array_push_denied() {
    use crate::backend::vm::VMValue;
    use crate::boxes::array::ArrayBox;
    use crate::jit::r#extern::collections as c;
    use std::sync::Arc;

    std::env::set_var("NYASH_JIT_READ_ONLY", "1");
    let arr = Arc::new(ArrayBox::new());
    let recv = VMValue::BoxRef(arr.clone());
    let val = VMValue::Integer(3);
    let _ = c::array_push(&[recv.clone(), val]);
    let len = c::array_len(&[recv]);
    assert_eq!(len.to_string(), "0");
}

#[cfg(feature = "cranelift-jit")]
#[test]
fn extern_readonly_map_set_denied() {
    use crate::backend::vm::VMValue;
    use crate::boxes::map_box::MapBox;
    use crate::jit::r#extern::collections as c;
    use std::sync::Arc;

    std::env::set_var("NYASH_JIT_READ_ONLY", "1");
    let map = Arc::new(MapBox::new());
    let recv = VMValue::BoxRef(map);
    let key = VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new("a")));
    let val = VMValue::Integer(2);
    let _ = c::map_set(&[recv.clone(), key, val]);
    let sz = c::map_size(&[recv]);
    assert_eq!(sz.to_string(), "0");
}

#[cfg(feature = "cranelift-jit")]
#[test]
fn extern_readonly_read_ops_allowed() {
    use crate::backend::vm::VMValue;
    use crate::boxes::{array::ArrayBox, map_box::MapBox};
    use crate::jit::r#extern::collections as c;
    use std::sync::Arc;

    std::env::set_var("NYASH_JIT_READ_ONLY", "1");
    // Array: len/get are read-only
    let arr = Arc::new(ArrayBox::new());
    let recv_a = VMValue::BoxRef(arr.clone());
    let len = c::array_len(&[recv_a.clone()]);
    assert_eq!(len.to_string(), "0");
    let zero = VMValue::Integer(0);
    let got = c::array_get(&[recv_a.clone(), zero]);
    assert_eq!(got.to_string(), "void");

    // Map: size is read-only
    let map = Arc::new(MapBox::new());
    let recv_m = VMValue::BoxRef(map);
    let size = c::map_size(&[recv_m]);
    assert_eq!(size.to_string(), "0");
}
