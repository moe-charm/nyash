use crate::ast::{ASTNode, LiteralValue, Span, BinaryOperator};
use crate::mir::{MirCompiler, MirInstruction};

fn lit_i(i: i64) -> ASTNode {
    ASTNode::Literal { value: LiteralValue::Integer(i), span: Span::unknown() }
}

fn bool_lt(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::LessThan, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

#[test]
fn mir13_no_phi_if_merge_inserts_edge_copies_for_return() {
    // Force PHI-off mode
    std::env::set_var("NYASH_MIR_NO_PHI", "1");

    // if (1 < 2) { return 40 } else { return 50 }
    let ast = ASTNode::If {
        condition: Box::new(bool_lt(lit_i(1), lit_i(2))),
        then_body: vec![ASTNode::Return { value: Some(Box::new(lit_i(40))), span: Span::unknown() }],
        else_body: Some(vec![ASTNode::Return { value: Some(Box::new(lit_i(50))), span: Span::unknown() }]),
        span: Span::unknown(),
    };

    let mut mc = MirCompiler::with_options(false);
    let cr = mc.compile(ast).expect("compile");
    let f = cr.module.functions.get("main").expect("function main");

    // Find the block that returns a value and capture that return value id
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

    // In PHI-off mode we expect copies into each predecessor of the merge/ret block
    let preds: Vec<_> = f
        .blocks
        .get(&ret_block)
        .expect("ret block present")
        .predecessors
        .iter()
        .copied()
        .collect();
    assert!(preds.len() >= 2, "expected at least two predecessors at merge");

    for p in preds {
        let bb = f.blocks.get(&p).expect("pred block present");
        let has_copy = bb
            .instructions
            .iter()
            .any(|inst| matches!(inst, MirInstruction::Copy { dst, .. } if *dst == out_v));
        assert!(has_copy, "expected Copy to out_v in predecessor {:?}", p);
    }
}

