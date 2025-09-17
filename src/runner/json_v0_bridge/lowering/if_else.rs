use super::{lower_stmt_list_with_vars, merge_var_maps, new_block, BridgeEnv, LoopContext};
use super::expr::lower_expr_with_vars;
use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use std::collections::HashMap;
use super::super::ast::StmtV0;
use super::super::ast::ExprV0;

pub(super) fn lower_if_stmt(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    cond: &ExprV0,
    then_body: &[StmtV0],
    else_body: &Option<Vec<StmtV0>>,
    vars: &mut HashMap<String, ValueId>,
    loop_stack: &mut Vec<LoopContext>,
    env: &BridgeEnv,
) -> Result<BasicBlockId, String> {
    let (cval, cur) = super::expr::lower_expr_with_vars(env, f, cur_bb, cond, vars)?;
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
    let base_vars = vars.clone();
    let mut then_vars = base_vars.clone();
    let tend = lower_stmt_list_with_vars(f, then_bb, then_body, &mut then_vars, loop_stack, env)?;
    if let Some(bb) = f.get_block_mut(tend) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: merge_bb });
        }
    }
    let (else_end_pred, else_vars) = if let Some(elses) = else_body {
        let mut ev = base_vars.clone();
        let eend = lower_stmt_list_with_vars(f, else_bb, elses, &mut ev, loop_stack, env)?;
        if let Some(bb) = f.get_block_mut(eend) {
            if !bb.is_terminated() {
                bb.set_terminator(MirInstruction::Jump { target: merge_bb });
            }
        }
        (eend, ev)
    } else {
        if let Some(bb) = f.get_block_mut(else_bb) {
            bb.set_terminator(MirInstruction::Jump { target: merge_bb });
        }
        (else_bb, base_vars.clone())
    };
    merge_var_maps(
        f,
        env.mir_no_phi,
        merge_bb,
        tend,
        else_end_pred,
        then_vars,
        else_vars,
        base_vars,
        vars,
    );
    Ok(merge_bb)
}
