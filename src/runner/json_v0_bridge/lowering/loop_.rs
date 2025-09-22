use super::{lower_stmt_list_with_vars, new_block, BridgeEnv, LoopContext};
use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use std::collections::HashMap;
use super::super::ast::StmtV0;
use super::super::ast::ExprV0;

pub(super) fn lower_loop_stmt(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    cond: &ExprV0,
    body: &[StmtV0],
    vars: &mut HashMap<String, ValueId>,
    loop_stack: &mut Vec<LoopContext>,
    env: &BridgeEnv,
) -> Result<BasicBlockId, String> {
    let cond_bb = new_block(f);
    let body_bb = new_block(f);
    let exit_bb = new_block(f);
    if let Some(bb) = f.get_block_mut(cur_bb) {
        if !bb.is_terminated() {
            bb.add_instruction(MirInstruction::Jump { target: cond_bb });
        }
    }
    // フェーズM.2: no_phi変数削除
    let base_vars = vars.clone();
    let orig_names: Vec<String> = base_vars.keys().cloned().collect();
    let mut phi_map: HashMap<String, ValueId> = HashMap::new();
    for name in &orig_names {
        if let Some(&bval) = base_vars.get(name) {
            let dst = f.next_value_id();
            // フェーズM.2: PHI統一処理（no_phi分岐削除）
            if let Some(bb) = f.get_block_mut(cond_bb) {
                bb.insert_instruction_after_phis(MirInstruction::Phi {
                    dst,
                    inputs: vec![(cur_bb, bval)],
                });
            }
            phi_map.insert(name.clone(), dst);
        }
    }
    for (name, &phi) in &phi_map {
        vars.insert(name.clone(), phi);
    }
    let (cval, _cend) = super::expr::lower_expr_with_vars(env, f, cond_bb, cond, vars)?;
    if let Some(bb) = f.get_block_mut(cond_bb) {
        bb.set_terminator(MirInstruction::Branch {
            condition: cval,
            then_bb: body_bb,
            else_bb: exit_bb,
        });
    }
    let mut body_vars = vars.clone();
    loop_stack.push(LoopContext { cond_bb, exit_bb });
    let bend_res = lower_stmt_list_with_vars(f, body_bb, body, &mut body_vars, loop_stack, env);
    loop_stack.pop();
    let bend = bend_res?;
    if let Some(bb) = f.get_block_mut(bend) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump { target: cond_bb });
        }
    }
    let backedge_to_cond = matches!(
        f.blocks
            .get(&bend)
            .and_then(|bb| bb.terminator.as_ref()),
        Some(MirInstruction::Jump { target, .. }) if *target == cond_bb
    );
    if backedge_to_cond {
        // フェーズM.2: PHI統一処理（no_phi分岐削除）
        if let Some(bb) = f.get_block_mut(cond_bb) {
            for (name, &phi_dst) in &phi_map {
                if let Some(&latch_val) = body_vars.get(name) {
                    for inst in &mut bb.instructions {
                        if let MirInstruction::Phi { dst, inputs } = inst {
                            if *dst == phi_dst {
                                inputs.push((bend, latch_val));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    for (name, &phi) in &phi_map {
        vars.insert(name.clone(), phi);
    }
    Ok(exit_bb)
}
