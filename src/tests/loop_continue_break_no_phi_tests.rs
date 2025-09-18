use crate::ast::{ASTNode, LiteralValue, Span, BinaryOperator};
use crate::mir::{MirCompiler, MirInstruction};

fn lit_i(i: i64) -> ASTNode {
    ASTNode::Literal { value: LiteralValue::Integer(i), span: Span::unknown() }
}

fn bin(op: BinaryOperator, l: ASTNode, r: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: op, left: Box::new(l), right: Box::new(r), span: Span::unknown() }
}

#[test]
fn loop_with_continue_and_break_edge_copy_merge() {
    // PHI-off
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // i=0; sum=0; loop(i < 5) {
    //   i = i + 1;
    //   if (i == 3) { break }
    //   if (i % 2 == 0) { continue }
    //   sum = sum + i;
    // }
    // return sum
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "sum".into(), span: Span::unknown() }), value: Box::new(lit_i(0)), span: Span::unknown() },
            ASTNode::Loop {
                condition: Box::new(bin(BinaryOperator::LessThan, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(5))),
                body: vec![
                    ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "i".into(), span: Span::unknown() }), value: Box::new(bin(BinaryOperator::Add, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(1))), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(3))), then_body: vec![ ASTNode::Break { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
                    ASTNode::If { condition: Box::new(bin(BinaryOperator::Equal, bin(BinaryOperator::Mod, ASTNode::Variable { name: "i".into(), span: Span::unknown() }, lit_i(2)), lit_i(0))), then_body: vec![ ASTNode::Continue { span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
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

    // In PHI-off, the after_loop/ret block should have predecessors with edge copies to the merged 'sum'
    let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
    assert!(preds.len() >= 1, "ret must have at least one predecessor");
    for p in preds {
        let bb = f.blocks.get(&p).unwrap();
        let has_copy = bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "expected edge Copy to merged sum at predecessor {:?}", p);
    }
    // And the ret block itself must not contain an extra Copy to out_v
    let merge_has_copy = f
        .blocks
        .get(&ret_block)
        .unwrap()
        .instructions
        .iter()
        .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "ret/merge must not contain Copy to merged sum");
}

