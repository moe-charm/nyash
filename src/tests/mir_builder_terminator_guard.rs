use crate::mir::{FunctionSignature, MirFunction, BasicBlockId, MirType, EffectMask, MirInstruction, ConstValue};
use crate::mir::builder::MirBuilder;

#[test]
fn builder_forbids_emit_after_terminator() {
    // Setup a minimal builder with a single function and entry block
    let mut b = MirBuilder::new();
    let sig = FunctionSignature { name: "T.main/0".to_string(), params: vec![], return_type: MirType::Void, effects: EffectMask::PURE };
    let entry = BasicBlockId::new(0);
    let f = MirFunction::new(sig, entry);
    b.current_function = Some(f);
    b.current_block = Some(entry);

    // Emit a Return terminator
    let ret_res = b.emit_instruction(MirInstruction::Return { value: None });
    assert!(ret_res.is_ok(), "Return should be accepted");

    // Try to emit a non-terminator after Return; must fail fast
    let dst = crate::mir::ValueId::new(0);
    let err = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(1) })
        .expect_err("emitting after terminator must error");
    assert!(err.contains("forbidden") || err.contains("after terminator"), "unexpected error message: {}", err);
}
