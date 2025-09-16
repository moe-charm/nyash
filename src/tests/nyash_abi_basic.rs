#[cfg(test)]
mod tests {
    use crate::runtime::type_registry::{known_methods_for, resolve_slot_by_name};

    #[test]
    fn type_registry_resolves_core_slots() {
        // MapBox
        assert_eq!(resolve_slot_by_name("MapBox", "size", 0), Some(200));
        assert_eq!(resolve_slot_by_name("MapBox", "len", 0), Some(201));
        assert_eq!(resolve_slot_by_name("MapBox", "has", 1), Some(202));
        assert_eq!(resolve_slot_by_name("MapBox", "get", 1), Some(203));
        assert_eq!(resolve_slot_by_name("MapBox", "set", 2), Some(204));
        // ArrayBox
        assert_eq!(resolve_slot_by_name("ArrayBox", "get", 1), Some(100));
        assert_eq!(resolve_slot_by_name("ArrayBox", "set", 2), Some(101));
        assert_eq!(resolve_slot_by_name("ArrayBox", "len", 0), Some(102));
        // StringBox
        assert_eq!(resolve_slot_by_name("StringBox", "len", 0), Some(300));

        // Known methods listing should include representative entries
        let mm = known_methods_for("MapBox").expect("map methods");
        assert!(mm.contains(&"size"));
        assert!(mm.contains(&"get"));
        assert!(mm.contains(&"set"));
    }

    #[test]
    #[ignore]
    fn vm_vtable_map_set_get_has() {
        use crate::backend::vm::VM;
        use crate::mir::{
            BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
            MirModule, MirType, ValueId,
        };

        // Enable vtable-preferred path
        std::env::set_var("NYASH_ABI_VTABLE", "1");

        // Program: m = new MapBox(); m.set("k","v"); h = m.has("k"); g = m.get("k"); return g
        let mut m = MirModule::new("nyash_abi_map_get".into());
        let sig = FunctionSignature {
            name: "main".into(),
            params: vec![],
            return_type: MirType::String,
            effects: EffectMask::PURE,
        };
        let mut f = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = f.entry_block;

        let mapv = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::NewBox {
                dst: mapv,
                box_type: "MapBox".into(),
                args: vec![],
            });

        let k = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::Const {
                dst: k,
                value: ConstValue::String("k".into()),
            });
        let v = f.next_value_id();
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

        let k2 = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::Const {
                dst: k2,
                value: ConstValue::String("k".into()),
            });
        let hasv = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::BoxCall {
                dst: Some(hasv),
                box_val: mapv,
                method: "has".into(),
                args: vec![k2],
                method_id: None,
                effects: EffectMask::PURE,
            });

        let k3 = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::Const {
                dst: k3,
                value: ConstValue::String("k".into()),
            });
        let got = f.next_value_id();
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::BoxCall {
                dst: Some(got),
                box_val: mapv,
                method: "get".into(),
                args: vec![k3],
                method_id: None,
                effects: EffectMask::PURE,
            });
        f.get_block_mut(bb)
            .unwrap()
            .add_instruction(MirInstruction::Return { value: Some(got) });

        m.add_function(f);

        let mut vm = VM::new();
        let out = vm.execute_module(&m).expect("vm exec");
        assert_eq!(out.to_string_box().value, "v");
    }

    #[test]
    fn mapbox_keys_values_return_arrays() {
        // Direct Box-level test (not via VM): keys()/values() should return ArrayBox
        use crate::box_trait::{IntegerBox, NyashBox, StringBox};
        use crate::boxes::map_box::MapBox;

        let map = MapBox::new();
        map.set(Box::new(StringBox::new("a")), Box::new(IntegerBox::new(1)));
        map.set(Box::new(StringBox::new("b")), Box::new(IntegerBox::new(2)));

        let keys = map.keys();
        let values = map.values();
        assert_eq!(keys.type_name(), "ArrayBox");
        assert_eq!(values.type_name(), "ArrayBox");
    }
}
