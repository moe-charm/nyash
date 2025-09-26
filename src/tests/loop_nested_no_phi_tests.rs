use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
use crate::mir::{MirCompiler, MirInstruction};

fn lit_i(i: i64) -> ASTNode {
    ASTNode::Literal { value: LiteralValue::Integer(i), span: Span::unknown() }
}

fn bin(op: BinaryOperator, l: ASTNode, r: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: op, left: Box::new(l), right: Box::new(r), span: Span::unknown() }
}

#[test]
fn nested_loop_with_multi_continue_break_edge_copy_merge() {
    // PHI-off mode
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // i=0; sum=0; loop(i < 10) {
    //   i = i + 1;
    //   if (i == 2 || i == 4) { continue }
    //   if (i == 7) { if (1 == 1) { break } }
    //   if ((i % 3) == 0) { continue }
    //   sum = sum + i;
    // }
    // return sum
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "sum".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::Less, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(10))),
                body: vec![
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Or, bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(2)), bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(4)))), then_body: vec![ ASTNode::Continue { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(7))), then_body: vec![ ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, lit_i(1), lit_i(1))), then_body: vec![ ASTNode::Break { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, bin(BinaryOperator::Modulo, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(3)), lit_i(0))), then_body: vec![ ASTNode::Continue { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "sum".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "sum".into(), span: Span::unknown() }, ASTNode::Variable { name: "i".into(), span: Span::unknown() })), span: Span::unknown() },
                ],
                span: Span::unknown(),
            },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "sum".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Locate return block/value
    let (ret_block, out_v) = f
        .blocks
        .iter()
        .find_map(|(bid, bb)| match &bb.terminator {
            Some(MirInstruction::Return { value: Some(v) }) => Some((*bid, *v)),
            _ => None,
        })
        .expect("ret block");

    // In PHI-off, every predecessor of the ret block should write the merged value via edge Copy
    let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
    assert!(preds.len() >= 1, "ret must have at least one predecessor");
    for p in preds {
        let bb = f.blocks.get(&p).unwrap();
        let has_copy = bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "expected edge Copy to merged value at predecessor {:?}", p);
    }
    // ret block itself must not contain Copy to out_v
    let merge_has_copy = f
        .blocks
        .get(&ret_block)
        .unwrap()
        .instructions
        .iter()
        .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "ret/merge must not contain Copy to merged value");
}

#[test]
fn loop_inner_if_multilevel_edge_copy() {
    // PHI-off
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // j=0; acc=0; loop(j<6){ j=j+1; if(j<3){ if(j%2==0){continue} acc=acc+10 } else { if(j==5){break} acc=acc+1 } } return acc
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "j".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::Less, ASTNode::Variable { name: "j".into(), span: Span::unknown() }, lit_i(6))),
                body: vec![
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "j".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "j".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                    ASTNode::If {
                        condition: Box::new(bin(BinaryOperator::Less, ASTNode::Variable { name: "j".into(), span: Span::unknown() }, lit_i(3))),
                        then_body: vec![
                            ASTNode::If {
                                condition: Box::new(bin(BinaryOperator::Equal, bin(BinaryOperator::Modulo, ASTNode::Variable { name: "j".into(), span: Span::unknown() }, lit_i(2)), lit_i(0))),
                                then_body: vec![ASTNode::Continue { span: Span::unknown() }],
                                else_body: Some(vec![]),
                                span: Span::unknown(),
                            },
                            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "acc".into(), span: Span::unknown() }, lit_i(10))), span: Span::unknown() },
                        ],
                        else_body: Some(vec![
                            ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "j".into(), span: Span::unknown() }, lit_i(5))), then_body: vec![ ASTNode::Break { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "acc".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                        ]),
                        span: Span::unknown(),
                    },
                ],
                span: Span::unknown(),
            },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Find ret block/value
    let (ret_block, out_v) = f
        .blocks
        .iter()
        .find_map(|(bid, bb)| match &bb.terminator {
            Some(MirInstruction::Return { value: Some(v) }) => Some((*bid, *v)),
            _ => None,
        })
        .expect("ret block");

    let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
    assert!(preds.len() >= 1, "ret must have predecessors");
    for p in preds {
        let bb = f.blocks.get(&p).unwrap();
        assert!(bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v)));
    }
}

#[cfg(feature = "phi-legacy")]
#[test]
#[ignore]
fn phi_on_loop_has_phi_in_header() {
    // PHI-on (explicit)
    std::env::set_var("NYASH_MIR_NO_PHI", "0");

    // x=0; loop(x<3){ x=x+1 } return x
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::Less, ASTNode::Variable { name: "x".into(), span: Span::unknown() }, lit_i(3))),
                body: vec![ ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "x".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() } ],
                span: Span::unknown(),
            },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Find any block with Phi(s) — loop header should have them in PHI-on.
    let phi_blocks: Vec<_> = f
        .blocks
        .iter()
        .filter(|(_bid, bb)| bb.phi_instructions().count() > 0)
        .map(|(bid, _)| *bid)
        .collect();

    assert!(!phi_blocks.is_empty(), "expected at least one Phi block in PHI-on");

    // Spot-check: each Phi should have at least 2 inputs (preheader + latch) in a loop
    for bid in phi_blocks.into_iter() {
        let bb = f.blocks.get(&bid).unwrap();
        for inst in bb.phi_instructions() {
            if let MirInstruction::Phi { inputs, .. } = inst {
                assert!(inputs.len() >= 2, "Phi should have at least 2 inputs");
            }
        }
    }
}
