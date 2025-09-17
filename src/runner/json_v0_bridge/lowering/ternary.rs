//! Ternary lowering (skeleton)
//!
//! NOTE: This module is introduced as part of the helper split.
//! It is not wired yet and should not alter behavior.

use super::merge::new_block;
use super::BridgeEnv;
use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use super::super::ast::ExprV0;

use super::expr::{lower_expr_with_scope, VarScope};

#[allow(dead_code)]
pub(super) fn lower_ternary_expr_with_scope<S: VarScope>(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    cond: &ExprV0,
    then_e: &ExprV0,
    else_e: &ExprV0,
    vars: &mut S,
) -> Result<(ValueId, BasicBlockId), String> {
    let (cval, cur) = lower_expr_with_scope(env, f, cur_bb, cond, vars)?;
    let then_bb = new_block(f);
    let else_bb = new_block(f);
    let merge_bb = new_block(f);
    if let Some(bb) = f.get_block_mut(cur) {
        bb.set_terminator(MirInstruction::Branch {
            condition: cval,
            then_bb,
            else_bb,
        });
    }
    let (tval, tend) = lower_expr_with_scope(env, f, then_bb, then_e, vars)?;
    if let Some(bb) = f.get_block_mut(tend) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: merge_bb });
        }
    }
    let (eval, eend) = lower_expr_with_scope(env, f, else_bb, else_e, vars)?;
    if let Some(bb) = f.get_block_mut(eend) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: merge_bb });
        }
    }
    let out = f.next_value_id();
    if env.mir_no_phi {
        if let Some(bb) = f.get_block_mut(tend) {
            bb.add_instruction(MirInstruction::Copy { dst: out, src: tval });
        }
        if let Some(bb) = f.get_block_mut(eend) {
            bb.add_instruction(MirInstruction::Copy { dst: out, src: eval });
        }
    } else if let Some(bb) = f.get_block_mut(merge_bb) {
        let mut inputs = vec![(tend, tval), (eend, eval)];
        inputs.sort_by_key(|(bbid, _)| bbid.0);
        bb.insert_instruction_after_phis(MirInstruction::Phi { dst: out, inputs });
    }
    Ok((out, merge_bb))
}
