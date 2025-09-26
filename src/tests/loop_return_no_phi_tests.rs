use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
use crate::mir::{MirCompiler, MirInstruction};

fn lit_i(i: i64) -> ASTNode {
    ASTNode::Literal { value: LiteralValue::Integer(i), span: Span::unknown() }
}

fn bin(op: BinaryOperator, l: ASTNode, r: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: op, left: Box::new(l), right: Box::new(r), span: Span::unknown() }
}

// PHI-off: mixed break + early return in loop; verify ret merge uses edge copies only
#[test]
fn loop_break_and_early_return_edge_copy() {
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // i=0; acc=0;
    // loop (i < 6) {
    //   i = i + 1;
    //   if (i == 5) { break }
    //   if (i == 3) { return acc }
    //   acc = acc + i;
    // }
    // return acc
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::LessThan, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(6))),
                body: vec![
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(5))), then_body: vec![ ASTNode::Break { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(3))), then_body: vec![ ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() })), span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "acc".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "acc".into(), span: Span::unknown() }, ASTNode::Variable { name: "i".into(), span: Span::unknown() })), span: Span::unknown() },
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

    // Locate the final return block/value (after-loop ret)
    let (ret_block, out_v) = f
        .blocks
        .iter()
        .find_map(|(bid, bb)| match &bb.terminator {
            Some(MirInstruction::Return { value: Some(v) }) => Some((*bid, *v)),
            _ => None,
        })
        .expect("ret block");

    // ret block's predecessors must write the merged destination via edge Copy (PHI-off)
    let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
    assert!(preds.len() >= 1, "ret must have at least one predecessor");
    for p in preds {
        let bb = f.blocks.get(&p).unwrap();
        let has_copy = bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "expected edge Copy to merged value at predecessor {:?}", p);
    }
    // Merge/ret block must not contain self-copy to out_v
    let merge_has_copy = f
        .blocks
        .get(&ret_block)
        .unwrap()
        .instructions
        .iter()
        .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "ret/merge must not contain Copy to merged value");
}

// PHI-off: deeper nested if chain in loop; verify edge copies on ret
#[test]
fn loop_if_three_level_merge_edge_copy() {
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // x=0; i=0;
    // loop(i<7){
    //   i=i+1;
    //   if (i%2==0) {
    //     if (i==4) { continue }
    //     x = x + 2
    //   } else {
    //     if (i==5) { break }
    //     x = x + 1
    //   }
    // }
    // return x
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::LessThan, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(7))),
                body: vec![
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                    ASTNode::If {
                        condition: Box::new(bin(BinaryOperator::Equal, bin(BinaryOperator::Mod, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(2)), lit_i(0))),
                        then_body: vec![
                            ASTNode::If {
                                condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(4))),
                                then_body: vec![ ASTNode::Continue { span: Span::unknown() } ],
                                else_body: Some(vec![]),
                                span: Span::unknown(),
                            },
                            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "x".into(), span: Span::unknown() }, lit_i(2))), span: Span::unknown() },
                        ],
                        else_body: Some(vec![
                            ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(5))), then_body: vec![ ASTNode::Break { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "x".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                        ]),
                        span: Span::unknown(),
                    },
                ],
                span: Span::unknown(),
            },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

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
    let merge_has_copy = f
        .blocks
        .get(&ret_block)
        .unwrap()
        .instructions
        .iter()
        .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "ret/merge must not contain Copy to merged value");
}

