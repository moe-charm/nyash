use crate::mir::{
    BasicBlock, BasicBlockId, MirFunction, MirInstruction, ValueId,
};

fn next_block_id(f: &MirFunction) -> BasicBlockId {
    let mut mx = 0u32;
    for k in f.blocks.keys() {
        if k.0 >= mx {
            mx = k.0 + 1;
        }
    }
    BasicBlockId::new(mx)
}

/// Create a fresh basic block and insert it into the function.
pub(super) fn new_block(f: &mut MirFunction) -> BasicBlockId {
    let id = next_block_id(f);
    f.add_block(BasicBlock::new(id));
    id
}

/// Return true if the block ends with Return or Throw (i.e., does not fall through).
pub(super) fn is_block_ends_with_return_or_throw(
    f: &MirFunction,
    block_id: BasicBlockId,
) -> bool {
    if let Some(block) = f.get_block(block_id) {
        return block.ends_with_return()
            || matches!(block.terminator, Some(MirInstruction::Throw { .. }));
    }
    false
}

/// Merge two incoming values either by inserting Copy on predecessor edges
/// (no_phi mode) or by adding a Phi at the merge block head.
pub(super) fn merge_values(
    f: &mut MirFunction,
    no_phi: bool,
    merge_bb: BasicBlockId,
    pred_a: BasicBlockId,
    val_a: ValueId,
    pred_b: BasicBlockId,
    val_b: ValueId,
) -> ValueId {
    if val_a == val_b {
        return val_a;
    }
    // Reachability guard: if a predecessor cannot reach the merge (ends with return/throw),
    // skip it. If both are unreachable (corner), fall back to the first value.
    let a_reachable = !is_block_ends_with_return_or_throw(f, pred_a);
    let b_reachable = !is_block_ends_with_return_or_throw(f, pred_b);
    match (a_reachable, b_reachable) {
        (false, false) => return val_a,
        (false, true) => return val_b,
        (true, false) => return val_a,
        (true, true) => {}
    }
    let dst = f.next_value_id();
    if no_phi {
        if let Some(bb) = f.get_block_mut(pred_a) {
            bb.add_instruction(MirInstruction::Copy { dst, src: val_a });
        }
        if let Some(bb) = f.get_block_mut(pred_b) {
            bb.add_instruction(MirInstruction::Copy { dst, src: val_b });
        }
    } else if let Some(bb) = f.get_block_mut(merge_bb) {
        bb.insert_instruction_after_phis(MirInstruction::Phi {
            dst,
            inputs: vec![(pred_a, val_a), (pred_b, val_b)],
        });
    }
    dst
}

/// Merge then/else variable maps into `out_vars` using either PHI at merge
/// or edge-copies in no-phi mode.
pub(super) fn merge_var_maps(
    f: &mut MirFunction,
    no_phi: bool,
    merge_bb: BasicBlockId,
    then_end: BasicBlockId,
    else_end: BasicBlockId,
    then_vars: std::collections::HashMap<String, ValueId>,
    else_vars: std::collections::HashMap<String, ValueId>,
    base_vars: std::collections::HashMap<String, ValueId>,
    out_vars: &mut std::collections::HashMap<String, ValueId>,
) {
    use std::collections::HashSet;
    let mut names: HashSet<String> = base_vars.keys().cloned().collect();
    for k in then_vars.keys() {
        names.insert(k.clone());
    }
    for k in else_vars.keys() {
        names.insert(k.clone());
    }
    for name in names {
        let tv = then_vars.get(&name).copied();
        let ev = else_vars.get(&name).copied();
        let exists_base = base_vars.contains_key(&name);
        match (tv, ev, exists_base) {
            (Some(tval), Some(eval), _) => {
                let merged = merge_values(f, no_phi, merge_bb, then_end, tval, else_end, eval);
                out_vars.insert(name, merged);
            }
            (Some(tval), None, true) => {
                if let Some(&bval) = base_vars.get(&name) {
                    let merged = merge_values(f, no_phi, merge_bb, then_end, tval, else_end, bval);
                    out_vars.insert(name, merged);
                }
            }
            (None, Some(eval), true) => {
                if let Some(&bval) = base_vars.get(&name) {
                    let merged = merge_values(f, no_phi, merge_bb, then_end, bval, else_end, eval);
                    out_vars.insert(name, merged);
                }
            }
            _ => {}
        }
    }
}
