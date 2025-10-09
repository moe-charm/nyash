#[test]
fn verify_box_compare_eq_forbidden() {
    use crate::mir::{
        BasicBlock, BasicBlockId, FunctionSignature, MirFunction, MirInstruction as I,
        MirModule, MirType, ValueId, CompareOp,
    };
    use crate::mir::{MirVerifier, verification_types::VerificationError};

    // Build a minimal function with two Box-typed values and a Compare(Eq)
    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(
        FunctionSignature { name: "cmp_box_eq".into(), params: vec![], return_type: MirType::Void, effects: crate::mir::effect::EffectMask::PURE },
        entry,
    );
    let mut b0 = BasicBlock::new(entry);
    let lhs = ValueId::new(1);
    let rhs = ValueId::new(2);
    let dst = ValueId::new(3);
    // Emit a compare instruction (types are provided via metadata below)
    b0.add_instruction(I::Compare { dst, op: CompareOp::Eq, lhs, rhs });
    b0.set_terminator(I::Return { value: None });
    f.add_block(b0);
    // Annotate value types as Box to trigger the verifier rule
    f.metadata.value_types.insert(lhs, MirType::Box("AnyBox".into()));
    f.metadata.value_types.insert(rhs, MirType::Box("AnyBox".into()));

    let mut m = MirModule::new("verify_box_compare".into());
    m.add_function(f);

    // Run verifier on single function
    let func = m.get_function("cmp_box_eq").unwrap();
    let mut verifier = MirVerifier::new();
    let res = verifier.verify_function(func);
    assert!(res.is_err(), "verifier should error on box eq compare");
    let errs = res.err().unwrap();
    assert!(errs.iter().any(|e| matches!(e, VerificationError::BoxCompareForbidden { .. })),
        "expected BoxCompareForbidden error, got: {:?}", errs);
}

