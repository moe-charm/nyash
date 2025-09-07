mod tests {
    use crate::mir::{
        MirModule, MirFunction, FunctionSignature, MirType, BasicBlock, BasicBlockId, ValueId,
        MirInstruction as I,
    };
    use crate::mir::optimizer::MirOptimizer;

    fn mk_func(name: &str) -> (MirFunction, BasicBlockId) {
        let sig = FunctionSignature { name: name.to_string(), params: vec![], return_type: MirType::Void, effects: crate::mir::effect::EffectMask::PURE };
        let entry = BasicBlockId::new(0);
        (MirFunction::new(sig, entry), entry)
    }

    #[test]
    fn core13_normalizes_array_get_set_to_boxcall() {
        // Build function with ArrayGet/ArraySet, then optimize under Core-13
        let (mut f, bb0) = mk_func("core13_array_norm");
        let mut b0 = BasicBlock::new(bb0);
        let arr = ValueId::new(0);
        let idx = ValueId::new(1);
        let val = ValueId::new(2);
        let dst = ValueId::new(3);
        b0.add_instruction(I::ArrayGet { dst, array: arr, index: idx });
        b0.add_instruction(I::ArraySet { array: arr, index: idx, value: val });
        b0.add_instruction(I::Return { value: None });
        f.add_block(b0);
        let mut m = MirModule::new("test_core13_array".into());
        m.add_function(f);

        let mut opt = MirOptimizer::new();
        let _ = opt.optimize_module(&mut m);

        let func = m.get_function("core13_array_norm").unwrap();
        let block = func.get_block(bb0).unwrap();
        // Expect only BoxCall in place of ArrayGet/ArraySet
        let mut saw_get = false;
        let mut saw_set = false;
        for inst in block.all_instructions() {
            match inst {
                I::BoxCall { method, .. } if method == "get" => { saw_get = true; },
                I::BoxCall { method, .. } if method == "set" => { saw_set = true; },
                I::ArrayGet { .. } | I::ArraySet { .. } => panic!("legacy Array* must be normalized under Core-13"),
                _ => {}
            }
        }
        assert!(saw_get && saw_set, "expected BoxCall(get) and BoxCall(set) present");
    }

    #[test]
    fn core13_normalizes_ref_get_set_to_boxcall() {
        // Build function with RefGet/RefSet, then optimize under Core-13
        let (mut f, bb0) = mk_func("core13_ref_norm");
        let mut b0 = BasicBlock::new(bb0);
        let r = ValueId::new(10);
        let v = ValueId::new(11);
        let dst = ValueId::new(12);
        b0.add_instruction(I::RefGet { dst, reference: r, field: "foo".to_string() });
        b0.add_instruction(I::RefSet { reference: r, field: "bar".to_string(), value: v });
        b0.add_instruction(I::Return { value: None });
        f.add_block(b0);
        let mut m = MirModule::new("test_core13_ref".into());
        m.add_function(f);

        let mut opt = MirOptimizer::new();
        let _ = opt.optimize_module(&mut m);

        let func = m.get_function("core13_ref_norm").unwrap();
        let block = func.get_block(bb0).unwrap();
        // Expect BoxCall(getField/setField), a Barrier(Write) before setField, and no RefGet/RefSet remain
        let mut saw_get_field = false;
        let mut saw_set_field = false;
        for inst in block.all_instructions() {
            match inst {
                I::BoxCall { method, .. } if method == "getField" => { saw_get_field = true; },
                I::BoxCall { method, .. } if method == "setField" => { saw_set_field = true; },
                I::RefGet { .. } | I::RefSet { .. } => panic!("legacy Ref* must be normalized under Core-13"),
                _ => {}
            }
        }
        assert!(saw_get_field && saw_set_field, "expected BoxCall(getField) and BoxCall(setField) present");
    }
}

