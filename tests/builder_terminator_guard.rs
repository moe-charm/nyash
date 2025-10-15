use nyash_rust::mir::{
    BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction, MirType,
};

#[test]
fn builder_forbids_emit_after_terminator() {
    let sig = FunctionSignature {
        name: "T.main/0".to_string(),
        params: vec![],
        return_type: MirType::Void,
        effects: EffectMask::PURE,
    };
    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(sig, entry);
    let bb = f.entry_block;
    {
        let block = f.get_block_mut(bb).expect("entry block exists");
        block.add_instruction(MirInstruction::Return { value: None });
    }
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let dst = nyash_rust::mir::ValueId::new(0);
        let block = f.get_block_mut(bb).unwrap();
        block.add_instruction(MirInstruction::Const {
            dst,
            value: ConstValue::Integer(1),
        });
    }));
    assert!(
        res.is_err(),
        "expected panic when emitting after terminator"
    );
}
