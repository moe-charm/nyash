//! Peek/expr-block lowering for JSON v0 bridge.

use super::merge::new_block;
use super::BridgeEnv;
use crate::mir::{BasicBlockId, CompareOp, ConstValue, MirFunction, MirInstruction, ValueId};
use super::super::ast::{ExprV0, PeekArmV0};
use super::expr::{lower_expr_with_scope, VarScope};

pub(super) fn lower_peek_expr_with_scope<S: VarScope>(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    scrutinee: &ExprV0,
    arms: &[PeekArmV0],
    else_expr: &ExprV0,
    vars: &mut S,
) -> Result<(ValueId, BasicBlockId), String> {
    // Evaluate scrutinee
    let (scr_val, start_bb) = lower_expr_with_scope(env, f, cur_bb, scrutinee, vars)?;

    // Set up blocks
    let dispatch_bb = new_block(f);
    if let Some(bb) = f.get_block_mut(start_bb) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: dispatch_bb });
        }
    }
    let else_bb = new_block(f);
    let merge_bb = new_block(f);

    // Chain dispatch over arms
    let mut cur_dispatch = dispatch_bb;
    let mut phi_inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
    for (i, arm) in arms.iter().enumerate() {
        let then_bb = new_block(f);
        let next_dispatch = if i + 1 < arms.len() { Some(new_block(f)) } else { None };
        let fall_bb = next_dispatch.unwrap_or(else_bb);

        // Pre-allocate ids to avoid double borrow
        let ldst = f.next_value_id();
        let cond = f.next_value_id();
        if let Some(bb) = f.get_block_mut(cur_dispatch) {
            // compare scr_val == label
            bb.add_instruction(MirInstruction::Const { dst: ldst, value: ConstValue::String(arm.label.clone()) });
            bb.add_instruction(MirInstruction::Compare { dst: cond, op: CompareOp::Eq, lhs: scr_val, rhs: ldst });
            bb.set_terminator(MirInstruction::Branch { condition: cond, then_bb, else_bb: fall_bb });
        }

        // Then arm body
        let (tval, tend) = lower_expr_with_scope(env, f, then_bb, &arm.expr, vars)?;
        if let Some(bb) = f.get_block_mut(tend) {
            if !bb.is_terminated() {
                bb.set_terminator(MirInstruction::Jump { target: merge_bb });
            }
        }
        phi_inputs.push((tend, tval));

        cur_dispatch = fall_bb;
    }

    // Else body
    let (eval, eend) = lower_expr_with_scope(env, f, else_bb, else_expr, vars)?;
    if let Some(bb) = f.get_block_mut(eend) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: merge_bb });
        }
    }
    phi_inputs.push((eend, eval));

    // Merge result
    let out = f.next_value_id();
    if env.mir_no_phi {
        for (pred, val) in phi_inputs {
            if let Some(bb) = f.get_block_mut(pred) {
                bb.add_instruction(MirInstruction::Copy { dst: out, src: val });
            }
        }
    } else if let Some(bb) = f.get_block_mut(merge_bb) {
        let mut inputs = phi_inputs;
        inputs.sort_by_key(|(bbid, _)| bbid.0);
        bb.insert_instruction_after_phis(MirInstruction::Phi { dst: out, inputs });
    }
    Ok((out, merge_bb))
}
