#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::collections::HashMap;

    use crate::backend::VM;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, BasicBlockId, MirInstruction, ConstValue, EffectMask, MirType};
    use crate::box_trait::NyashBox;
    use crate::interpreter::RuntimeError;

    // Minimal Person factory: creates InstanceBox with fields [name, age]
    struct PersonFactory;
    impl crate::box_factory::BoxFactory for PersonFactory {
        fn create_box(&self, name: &str, _args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
            if name != "Person" { return Err(RuntimeError::InvalidOperation { message: format!("Unknown Box type: {}", name) }); }
            let fields = vec!["name".to_string(), "age".to_string()];
            let methods: HashMap<String, crate::ast::ASTNode> = HashMap::new();
            let inst = crate::instance_v2::InstanceBox::from_declaration("Person".to_string(), fields, methods);
            Ok(Box::new(inst))
        }
        fn box_types(&self) -> Vec<&str> { vec!["Person"] }
        fn is_builtin_factory(&self) -> bool { true }
    }

    fn build_person_module() -> MirModule {
        let mut module = MirModule::new("identical_person".to_string());
        let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::String, effects: EffectMask::PURE };
        let mut f = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = f.entry_block;

        let person = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::NewBox { dst: person, box_type: "Person".into(), args: vec![] });

        // person.setField("name", "Alice")
        let k_name = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: k_name, value: ConstValue::String("name".into()) });
        let v_alice = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v_alice, value: ConstValue::String("Alice".into()) });
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: None, box_val: person, method: "setField".into(), args: vec![k_name, v_alice], method_id: None, effects: EffectMask::PURE });

        // person.setField("age", 25)
        let k_age = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: k_age, value: ConstValue::String("age".into()) });
        let v_25 = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: v_25, value: ConstValue::Integer(25) });
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: None, box_val: person, method: "setField".into(), args: vec![k_age, v_25], method_id: None, effects: EffectMask::PURE });

        // name = person.getField("name"); return name
        let k_name2 = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: k_name2, value: ConstValue::String("name".into()) });
        let out_name = f.next_value_id();
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::BoxCall { dst: Some(out_name), box_val: person, method: "getField".into(), args: vec![k_name2], method_id: None, effects: EffectMask::PURE });
        f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(out_name) });

        module.add_function(f);
        module
    }

    #[cfg(feature = "cranelift-jit")]
    #[test]
    fn identical_vm_and_jit_person_get_set_slots() {
        // Build runtime with Person factory
        let mut rt_builder = crate::runtime::NyashRuntimeBuilder::new();
        rt_builder = rt_builder.with_factory(Arc::new(PersonFactory));
        let runtime = rt_builder.build();

        // Also register factory globally for JIT path (host-bridge creates via global registry)
        crate::runtime::register_user_defined_factory(Arc::new(PersonFactory));

        // Build module
        let module = build_person_module();

        // VM (VTABLE on)
        std::env::set_var("NYASH_ABI_VTABLE", "1");
        let mut vm = VM::with_runtime(runtime);
        let vm_out = vm.execute_module(&module).expect("VM exec");
        let vm_s = vm_out.to_string_box().value;

        // JIT（host-bridge on）
        std::env::set_var("NYASH_JIT_HOST_BRIDGE", "1");
        let jit_out = crate::backend::cranelift_compile_and_execute(&module, "identical_person").expect("JIT exec");
        let jit_s = jit_out.to_string_box().value;

        assert_eq!(vm_s, jit_s);
        assert_eq!(vm_s, "Alice");
    }
}
