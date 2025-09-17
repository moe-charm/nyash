use super::{
    lower_stmt_list_with_vars, new_block, BridgeEnv, LoopContext,
};
use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use std::collections::HashMap;
use super::super::ast::{StmtV0, CatchV0};

pub(super) fn lower_try_stmt(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    try_body: &[StmtV0],
    catches: &[CatchV0],
    finally: &[StmtV0],
    vars: &mut HashMap<String, ValueId>,
    loop_stack: &mut Vec<LoopContext>,
    env: &BridgeEnv,
) -> Result<BasicBlockId, String> {
    let try_enabled = std::env::var("NYASH_BRIDGE_TRY_ENABLE").ok().as_deref() == Some("1");
    if !try_enabled || catches.is_empty() || catches.len() > 1 {
        let mut tmp_vars = vars.clone();
        let mut next_bb = lower_stmt_list_with_vars(f, cur_bb, try_body, &mut tmp_vars, loop_stack, env)?;
        if !finally.is_empty() {
            next_bb = lower_stmt_list_with_vars(
                f,
                next_bb,
                finally,
                &mut tmp_vars,
                loop_stack,
                env,
            )?;
        }
        *vars = tmp_vars;
        return Ok(next_bb);
    }

    let base_vars = vars.clone();
    let try_bb = new_block(f);
    let catch_clause = &catches[0];
    let catch_bb = new_block(f);
    let finally_bb = if !finally.is_empty() {
        let id = new_block(f);
        Some(id)
    } else {
        None
    };
    let exit_bb = new_block(f);
    let handler_target = finally_bb.unwrap_or(exit_bb);
    let exception_value = f.next_value_id();
    if let Some(bb) = f.get_block_mut(cur_bb) {
        bb.add_instruction(MirInstruction::Catch {
            exception_type: catch_clause.type_hint.clone(),
            exception_value,
            handler_bb: catch_bb,
        });
        bb.set_terminator(MirInstruction::Jump { target: try_bb });
    }
    let mut try_vars = vars.clone();
    let try_end = lower_stmt_list_with_vars(f, try_bb, try_body, &mut try_vars, loop_stack, env)?;
    if let Some(bb) = f.get_block_mut(try_end) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump {
                target: handler_target,
            });
        }
    }
    let try_branch_vars = try_vars.clone();

    let mut catch_vars = base_vars.clone();
    if let Some(param) = &catch_clause.param {
        catch_vars.insert(param.clone(), exception_value);
    }
    let catch_end = lower_stmt_list_with_vars(
        f,
        catch_bb,
        &catch_clause.body,
        &mut catch_vars,
        loop_stack,
        env,
    )?;
    if let Some(param) = &catch_clause.param {
        catch_vars.remove(param);
    }
    if let Some(bb) = f.get_block_mut(catch_end) {
        if !bb.is_terminated() {
            bb.set_terminator(MirInstruction::Jump {
                target: handler_target,
            });
        }
    }
    let catch_branch_vars = catch_vars.clone();

    use std::collections::HashSet;
    let mut branch_vars = vec![(try_end, try_branch_vars), (catch_end, catch_branch_vars)];
    if let Some(finally_block) = finally_bb {
        let names: HashSet<String> = {
            let mut set: HashSet<String> = base_vars.keys().cloned().collect();
            for (_, map) in &branch_vars {
                set.extend(map.keys().cloned());
            }
            set
        };
        let mut merged_vars = base_vars.clone();
        let mut phi_entries: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = Vec::new();
        for name in names {
            let mut inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
            for (bbid, map) in &branch_vars {
                if let Some(&val) = map.get(&name) {
                    inputs.push((*bbid, val));
                }
            }
            if inputs.is_empty() {
                if let Some(&base_val) = base_vars.get(&name) {
                    merged_vars.insert(name.clone(), base_val);
                }
                continue;
            }
            let unique: HashSet<ValueId> = inputs.iter().map(|(_, v)| *v).collect();
            if unique.len() == 1 {
                merged_vars.insert(name.clone(), inputs[0].1);
                continue;
            }
            let dst = f.next_value_id();
            inputs.sort_by_key(|(bbid, _)| bbid.0);
            phi_entries.push((dst, inputs));
            merged_vars.insert(name.clone(), dst);
        }
        if let Some(bb) = f.get_block_mut(finally_block) {
            for (dst, inputs) in phi_entries {
                bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs });
            }
        }
        let mut finally_vars = merged_vars.clone();
        let final_end = lower_stmt_list_with_vars(
            f,
            finally_block,
            finally,
            &mut finally_vars,
            loop_stack,
            env,
        )?;
        if let Some(bb) = f.get_block_mut(final_end) {
            if !bb.is_terminated() {
                bb.set_terminator(MirInstruction::Jump { target: exit_bb });
            }
        }
        *vars = finally_vars;
        Ok(exit_bb)
    } else {
        let names: HashSet<String> = {
            let mut set: HashSet<String> = base_vars.keys().cloned().collect();
            for (_, map) in &branch_vars {
                set.extend(map.keys().cloned());
            }
            set
        };
        let mut merged_vars = base_vars.clone();
        let mut phi_entries: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = Vec::new();
        for name in names {
            let mut inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
            for (bbid, map) in &branch_vars {
                if let Some(&val) = map.get(&name) {
                    inputs.push((*bbid, val));
                }
            }
            if inputs.is_empty() {
                if let Some(&base_val) = base_vars.get(&name) {
                    merged_vars.insert(name.clone(), base_val);
                }
                continue;
            }
            let unique: HashSet<ValueId> = inputs.iter().map(|(_, v)| *v).collect();
            if unique.len() == 1 {
                merged_vars.insert(name.clone(), inputs[0].1);
                continue;
            }
            let dst = f.next_value_id();
            inputs.sort_by_key(|(bbid, _)| bbid.0);
            phi_entries.push((dst, inputs));
            merged_vars.insert(name.clone(), dst);
        }
        if let Some(bb) = f.get_block_mut(exit_bb) {
            for (dst, inputs) in phi_entries {
                bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs });
            }
        }
        *vars = merged_vars;
        Ok(exit_bb)
    }
}

