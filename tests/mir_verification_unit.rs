use nyash_rust::ast::{ASTNode, LiteralValue, Span};
use nyash_rust::mir::{
    BasicBlock, BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirBuilder, MirFunction,
    MirInstruction, MirPrinter, MirType, MirVerifier, VerificationError,
};

#[test]
fn test_valid_function_verification() {
    let signature = FunctionSignature {
        name: "test".to_string(),
        params: vec![],
        return_type: MirType::Void,
        effects: EffectMask::PURE,
    };

    let entry_block = BasicBlockId::new(0);
    let function = MirFunction::new(signature, entry_block);

    let mut verifier = MirVerifier::new();
    let result = verifier.verify_function(&function);

    assert!(result.is_ok(), "Valid function should pass verification");
}

#[test]
fn test_undefined_value_detection() {
    // Placeholder: Define a minimal function without uses; this test is a scaffold.
    let signature = FunctionSignature {
        name: "undef_sanity".to_string(),
        params: vec![],
        return_type: MirType::Void,
        effects: EffectMask::PURE,
    };
    let entry_block = BasicBlockId::new(0);
    let function = MirFunction::new(signature, entry_block);
    let mut verifier = MirVerifier::new();
    verifier.verify_function(&function).unwrap();
}

#[test]
fn test_if_merge_uses_phi_not_predecessor() {
    // Program:
    // if true { result = "A" } else { result = "B" }
    // result
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::If {
                condition: Box::new(ASTNode::Literal {
                    value: LiteralValue::Bool(true),
                    span: Span::unknown(),
                }),
                then_body: vec![ASTNode::Assignment {
                    target: Box::new(ASTNode::Variable {
                        name: "result".to_string(),
                        span: Span::unknown(),
                    }),
                    value: Box::new(ASTNode::Literal {
                        value: LiteralValue::String("A".to_string()),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }],
                else_body: Some(vec![ASTNode::Assignment {
                    target: Box::new(ASTNode::Variable {
                        name: "result".to_string(),
                        span: Span::unknown(),
                    }),
                    value: Box::new(ASTNode::Literal {
                        value: LiteralValue::String("B".to_string()),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }]),
                span: Span::unknown(),
            },
            ASTNode::Variable {
                name: "result".to_string(),
                span: Span::unknown(),
            },
        ],
        span: Span::unknown(),
    };

    let mut builder = MirBuilder::new();
    let module = builder.build_module(ast).expect("build mir");

    // Verify: should be OK (no MergeUsesPredecessorValue)
    let mut verifier = MirVerifier::new();
    let res = verifier.verify_module(&module);
    if let Err(errs) = &res {
        eprintln!("Verifier errors: {:?}", errs);
    }
    assert!(res.is_ok(), "MIR should pass merge-phi verification");

    // Optional: ensure printer shows a phi in merge and ret returns a defined value
    let printer = MirPrinter::verbose();
    let mir_text = printer.print_module(&module);
    assert!(
        mir_text.contains("phi"),
        "Printed MIR should contain a phi in merge block\n{}",
        mir_text
    );
}

#[test]
fn test_merge_use_before_phi_detected() {
    // Construct a function with a bad merge use (no phi)
    let signature = FunctionSignature {
        name: "merge_bad".to_string(),
        params: vec![],
        return_type: MirType::String,
        effects: EffectMask::PURE,
    };

    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(signature, entry);

    let then_bb = BasicBlockId::new(1);
    let else_bb = BasicBlockId::new(2);
    let merge_bb = BasicBlockId::new(3);

    let cond = f.next_value_id(); // %0
    {
        let b0 = f.get_block_mut(entry).unwrap();
        b0.add_instruction(MirInstruction::Const {
            dst: cond,
            value: ConstValue::Bool(true),
        });
        b0.add_instruction(MirInstruction::Branch {
            condition: cond,
            then_bb,
            else_bb,
        });
    }

    let v1 = f.next_value_id(); // %1
    let mut b1 = BasicBlock::new(then_bb);
    b1.add_instruction(MirInstruction::Const {
        dst: v1,
        value: ConstValue::String("A".to_string()),
    });
    b1.add_instruction(MirInstruction::Jump { target: merge_bb });
    f.add_block(b1);

    let v2 = f.next_value_id(); // %2
    let mut b2 = BasicBlock::new(else_bb);
    b2.add_instruction(MirInstruction::Const {
        dst: v2,
        value: ConstValue::String("B".to_string()),
    });
    b2.add_instruction(MirInstruction::Jump { target: merge_bb });
    f.add_block(b2);

    let mut b3 = BasicBlock::new(merge_bb);
    // Illegal: directly use v1 from predecessor
    b3.add_instruction(MirInstruction::Return { value: Some(v1) });
    f.add_block(b3);

    f.update_cfg();

    let mut verifier = MirVerifier::new();
    let res = verifier.verify_function(&f);
    assert!(
        res.is_err(),
        "Verifier should error on merge use without phi"
    );
    let errs = res.err().unwrap();
    assert!(
        errs.iter().any(|e| matches!(
            e,
            VerificationError::MergeUsesPredecessorValue { .. }
                | VerificationError::DominatorViolation { .. }
        )),
        "Expected merge/dominator error, got: {:?}",
        errs
    );
}

#[test]
fn test_loop_phi_normalization() {
    // Program:
    // local i = 0
    // loop(false) { i = 1 }
    // i
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Local {
                variables: vec!["i".to_string()],
                initial_values: vec![Some(Box::new(ASTNode::Literal {
                    value: LiteralValue::Integer(0),
                    span: Span::unknown(),
                }))],
                span: Span::unknown(),
            },
            ASTNode::Loop {
                condition: Box::new(ASTNode::Literal {
                    value: LiteralValue::Bool(false),
                    span: Span::unknown(),
                }),
                body: vec![ASTNode::Assignment {
                    target: Box::new(ASTNode::Variable {
                        name: "i".to_string(),
                        span: Span::unknown(),
                    }),
                    value: Box::new(ASTNode::Literal {
                        value: LiteralValue::Integer(1),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }],
                span: Span::unknown(),
            },
            ASTNode::Variable {
                name: "i".to_string(),
                span: Span::unknown(),
            },
        ],
        span: Span::unknown(),
    };

    let mut builder = MirBuilder::new();
    let module = builder.build_module(ast).expect("build mir");

    // Verify SSA/dominance: should pass
    let mut verifier = MirVerifier::new();
    let res = verifier.verify_module(&module);
    if let Err(errs) = &res {
        eprintln!("Verifier errors: {:?}", errs);
    }
    assert!(
        res.is_ok(),
        "MIR loop with phi normalization should pass verification"
    );

    // Ensure phi is printed (header phi for variable i)
    let printer = MirPrinter::verbose();
    let mir_text = printer.print_module(&module);
    assert!(
        mir_text.contains("phi"),
        "Printed MIR should contain a phi for loop header\n{}",
        mir_text
    );
}

#[test]
fn test_loop_nested_if_phi() {
    // Program:
    // local x = 0
    // loop(false) { if true { x = 1 } else { x = 2 } }
    // x
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Local {
                variables: vec!["x".to_string()],
                initial_values: vec![Some(Box::new(ASTNode::Literal {
                    value: LiteralValue::Integer(0),
                    span: Span::unknown(),
                }))],
                span: Span::unknown(),
            },
            ASTNode::Loop {
                condition: Box::new(ASTNode::Literal {
                    value: LiteralValue::Bool(false),
                    span: Span::unknown(),
                }),
                body: vec![ASTNode::If {
                    condition: Box::new(ASTNode::Literal {
                        value: LiteralValue::Bool(true),
                        span: Span::unknown(),
                    }),
                    then_body: vec![ASTNode::Assignment {
                        target: Box::new(ASTNode::Variable {
                            name: "x".to_string(),
                            span: Span::unknown(),
                        }),
                        value: Box::new(ASTNode::Literal {
                            value: LiteralValue::Integer(1),
                            span: Span::unknown(),
                        }),
                        span: Span::unknown(),
                    }],
                    else_body: Some(vec![ASTNode::Assignment {
                        target: Box::new(ASTNode::Variable {
                            name: "x".to_string(),
                            span: Span::unknown(),
                        }),
                        value: Box::new(ASTNode::Literal {
                            value: LiteralValue::Integer(2),
                            span: Span::unknown(),
                        }),
                        span: Span::unknown(),
                    }]),
                    span: Span::unknown(),
                }],
                span: Span::unknown(),
            },
            ASTNode::Variable {
                name: "x".to_string(),
                span: Span::unknown(),
            },
        ],
        span: Span::unknown(),
    };

    let mut builder = MirBuilder::new();
    let module = builder.build_module(ast).expect("build mir");

    let mut verifier = MirVerifier::new();
    let res = verifier.verify_module(&module);
    if let Err(errs) = &res {
        eprintln!("Verifier errors: {:?}", errs);
    }
    assert!(
        res.is_ok(),
        "Nested if in loop should pass verification with proper phis"
    );

    let printer = MirPrinter::verbose();
    let mir_text = printer.print_module(&module);
    assert!(
        mir_text.contains("phi"),
        "Printed MIR should contain phi nodes for nested if/loop\n{}",
        mir_text
    );
}
