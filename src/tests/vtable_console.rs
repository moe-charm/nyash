#[test]
fn vtable_console_log_clear_smoke() {
    use crate::backend::VM;
    use crate::mir::{
        BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
        MirModule, MirType,
    };
    std::env::set_var("NYASH_ABI_VTABLE", "1");

    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    let con = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::NewBox { dst: con, box_type: "ConsoleBox".into(), args: vec![],
                auto_birth: None });
    let msg = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: msg,
            value: ConstValue::String("hi".into()),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".into(), method_name: "log".into(), args: vec![msg], effects: EffectMask::IO });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".into(), method_name: "clear".into(), args: vec![], effects: EffectMask::IO });
    let zero = f.next_value_id();
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Const {
            dst: zero,
            value: ConstValue::Integer(0),
        });
    f.get_block_mut(bb)
        .unwrap()
        .add_instruction(MirInstruction::Return { value: Some(zero) });
    let mut m = MirModule::new("console_smoke".into());
    m.add_function(f);
    let mut vm = VM::new();
    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "0");
}
