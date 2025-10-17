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

struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &'static str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.previous {
            std::env::set_var(self.key, prev);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn map_host_keys_values_return_arrays() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };

    let _guard = EnvGuard::set("HAKO_MAP_FORCE_HOST", "1");

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let map = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: map,
            box_type: "MapBox".into(),
            args: vec![],
            auto_birth: None,
        });

    let key_a = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key_a,
            value: ConstValue::String("alpha".into()),
        });
    let val_1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: val_1,
            value: ConstValue::Integer(10),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.set/2".into(),
            )),
            args: vec![map, key_a, val_1],
            effects: EffectMask::PURE,
        },
    );

    let key_b = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key_b,
            value: ConstValue::String("beta".into()),
        });
    let val_2 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: val_2,
            value: ConstValue::Integer(20),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.set/2".into(),
            )),
            args: vec![map, key_b, val_2],
            effects: EffectMask::PURE,
        },
    );

    let keys = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(keys),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.keys/0".into(),
            )),
            args: vec![map],
            effects: EffectMask::PURE,
        },
    );
    let keys_len = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(keys_len),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.len/0".into(),
            )),
            args: vec![keys],
            effects: EffectMask::PURE,
        },
    );

    let values = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(values),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.values/0".into(),
            )),
            args: vec![map],
            effects: EffectMask::PURE,
        },
    );
    let values_len = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(values_len),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.len/0".into(),
            )),
            args: vec![values],
            effects: EffectMask::PURE,
        },
    );

    let sum = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::BinOp {
            dst: sum,
            op: crate::mir::BinaryOp::Add,
            lhs: keys_len,
            rhs: values_len,
        },
    );
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(sum),
        });

    let mut module = MirModule::new("map_host_keys_values".into());
    module.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "4");
}

#[test]
fn map_remove_returns_removed_array_len() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;

    let array = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: array,
            box_type: "ArrayBox".into(),
            args: vec![],
            auto_birth: None,
        });
    let element = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: element,
            value: ConstValue::Integer(7),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.push/1".into(),
            )),
            args: vec![array, element],
            effects: EffectMask::PURE,
        },
    );

    let map = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: map,
            box_type: "MapBox".into(),
            args: vec![],
            auto_birth: None,
        });
    let key = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key,
            value: ConstValue::String("items".into()),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.set/2".into(),
            )),
            args: vec![map, key, array],
            effects: EffectMask::PURE,
        },
    );

    let removed = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(removed),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.remove/1".into(),
            )),
            args: vec![map, key],
            effects: EffectMask::PURE,
        },
    );
    let removed_len = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(removed_len),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.len/0".into(),
            )),
            args: vec![removed],
            effects: EffectMask::PURE,
        },
    );
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(removed_len),
        });

    let mut module = MirModule::new("map_remove_returns_array".into());
    module.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "1");
}

#[test]
fn map_keys_values_empty_map_return_empty_arrays() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;

    let map = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: map,
            box_type: "MapBox".into(),
            args: vec![],
            auto_birth: None,
        });

    let keys = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(keys),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.keys/0".into(),
            )),
            args: vec![map],
            effects: EffectMask::PURE,
        },
    );
    let keys_len = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(keys_len),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.len/0".into(),
            )),
            args: vec![keys],
            effects: EffectMask::PURE,
        },
    );

    let values = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(values),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.values/0".into(),
            )),
            args: vec![map],
            effects: EffectMask::PURE,
        },
    );
    let values_len = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(values_len),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.len/0".into(),
            )),
            args: vec![values],
            effects: EffectMask::PURE,
        },
    );

    let sum = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::BinOp {
            dst: sum,
            op: crate::mir::BinaryOp::Add,
            lhs: keys_len,
            rhs: values_len,
        },
    );
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(sum),
        });

    let mut module = MirModule::new("map_empty_arrays".into());
    module.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "0");
}

#[test]
fn map_values_follow_sorted_keys_for_mixed_input() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;

    let map = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: map,
            box_type: "MapBox".into(),
            args: vec![],
            auto_birth: None,
        });

    // set int key 42 -> 100
    let key_int = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key_int,
            value: ConstValue::Integer(42),
        });
    let val_int = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: val_int,
            value: ConstValue::Integer(100),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.set/2".into(),
            )),
            args: vec![map, key_int, val_int],
            effects: EffectMask::PURE,
        },
    );

    // set string key "alpha" -> 200
    let key_str = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: key_str,
            value: ConstValue::String("alpha".into()),
        });
    let val_str = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: val_str,
            value: ConstValue::Integer(200),
        });
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: None,
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.set/2".into(),
            )),
            args: vec![map, key_str, val_str],
            effects: EffectMask::PURE,
        },
    );

    let values = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(values),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.values/0".into(),
            )),
            args: vec![map],
            effects: EffectMask::PURE,
        },
    );

    let idx0 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: idx0,
            value: ConstValue::Integer(0),
        });
    let first_val = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(first_val),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.get/1".into(),
            )),
            args: vec![values, idx0],
            effects: EffectMask::PURE,
        },
    );

    let idx1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: idx1,
            value: ConstValue::Integer(1),
        });
    let second_val = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(second_val),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "ArrayBox.get/1".into(),
            )),
            args: vec![values, idx1],
            effects: EffectMask::PURE,
        },
    );

    let expect_first = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: expect_first,
            value: ConstValue::Integer(100),
        });
    let expect_second = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: expect_second,
            value: ConstValue::Integer(200),
        });

    let diff0 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: diff0,
            op: crate::mir::BinaryOp::Sub,
            lhs: first_val,
            rhs: expect_first,
        });
    let diff1 = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: diff1,
            op: crate::mir::BinaryOp::Sub,
            lhs: second_val,
            rhs: expect_second,
        });
    let total = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::BinOp {
            dst: total,
            op: crate::mir::BinaryOp::Add,
            lhs: diff0,
            rhs: diff1,
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(total),
        });

    let mut module = MirModule::new("map_values_alignment".into());
    module.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "0");
}

#[test]
fn map_remove_missing_key_returns_void() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Void,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;

    let map = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox {
            dst: map,
            box_type: "MapBox".into(),
            args: vec![],
            auto_birth: None,
        });
    let missing_key = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: missing_key,
            value: ConstValue::String("missing".into()),
        });
    let removed = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(
        MirInstruction::Call {
            dst: Some(removed),
            func: crate::mir::ValueId::new(0),
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(
                "MapBox.remove/1".into(),
            )),
            args: vec![map, missing_key],
            effects: EffectMask::PURE,
        },
    );
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return {
            value: Some(removed),
        });

    let mut module = MirModule::new("map_remove_missing".into());
    module.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "void");
}
