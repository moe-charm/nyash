use crate::ast::{ASTNode, LiteralValue, Span, BinaryOperator};
use crate::mir::{MirCompiler, MirInstruction};

fn lit_i(i: i64) -> ASTNode {
    ASTNode::Literal { value: LiteralValue::Integer(i), span: Span::unknown() }
}

fn bool_lt(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::LessThan, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

#[test]
fn ifform_no_phi_one_sided_merge_uses_edge_copies_only() {
    // Force PHI-off mode
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // Program:
    // local x = 1;
    // if (1 < 2) { x = 10 } else { }
    // return x
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(lit_i(1)), span: Span::unknown() },
            ASTNode::If { condition: Box::new(bool_lt(lit_i(1), lit_i(2))), then_body: vec![ ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() }), value: Box::new(lit_i(10)), span: Span::unknown() } ], else_body: Some(vec![]), span: Span::unknown() },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "x".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Find return block and value id
    let mut ret_block_id = None;
    let mut ret_val = None;
    for (bid, bb) in &f.blocks {
        if let Some(MirInstruction::Return { value: Some(v) }) = bb.terminator.clone() {
            ret_block_id = Some(*bid);
            ret_val = Some(v);
            break;
        }
    }
    let ret_block = ret_block_id.expect("ret block");
    let out_v = ret_val.expect("ret value");

    // Preds should have Copy to out_v; merge/ret should not have Copy to out_v
    let preds: Vec<_> = f.blocks.get(&ret_block).unwrap().predecessors.iter().copied().collect();
    assert!(preds.len() >= 2, "expected at least two predecessors");
    for p in preds {
        let bb = f.blocks.get(&p).unwrap();
        let has_copy = bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "missing Copy to merged value in predecessor {:?}", p);
    }
    let merge_bb = f.blocks.get(&ret_block).unwrap();
    let merge_has_copy = merge_bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "merge/ret block must not contain Copy to merged value");
}

#[test]
fn ifform_nested_no_merge_block_copies() {
    std::env::set_var("NYASH_MIR_NO_PHI", "1");
    // if (1<2) { if (1<2) { y = 3 } else { y = 4 } } else { y = 5 }; return y
    let inner_if = ASTNode::If {
        condition: Box::new(bool_lt(lit_i(1), lit_i(2))),
        then_body: vec![ ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "y".into(), span: Span::unknown() }), value: Box::new(lit_i(3)), span: Span::unknown() } ],
        else_body: Some(vec![ ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "y".into(), span: Span::unknown() }), value: Box::new(lit_i(4)), span: Span::unknown() } ]),
        span: Span::unknown(),
    };
    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::If {
                condition: Box::new(bool_lt(lit_i(1), lit_i(2))),
                then_body: vec![ inner_if ],
                else_body: Some(vec![ ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "y".into(), span: Span::unknown() }), value: Box::new(lit_i(5)), span: Span::unknown() } ]),
                span: Span::unknown(),
            },
            ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: "y".into(), span: Span::unknown() })), span: Span::unknown() }
        ],
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Find return block/value
    let (ret_block, out_v) = f
        .blocks
        .iter()
        .find_map(|(bid, bb)| match &bb.terminator {
            Some(MirInstruction::Return { value: Some(v) }) => Some((*bid, *v)),
            _ => None,
        })
        .expect("ret block");

    // Preds must have Copy to merged value; merge block must not
    for p in f.blocks.get(&ret_block).unwrap().predecessors.iter().copied() {
        let bb = f.blocks.get(&p).unwrap();
        let has_copy = bb.instructions.iter().any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "missing Copy in pred {:?}", p);
    }
    let merge_has_copy = f
        .blocks
        .get(&ret_block)
        .unwrap()
        .instructions
        .iter()
        .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
    assert!(!merge_has_copy, "merge/ret block must not contain Copy to merged value");
}

