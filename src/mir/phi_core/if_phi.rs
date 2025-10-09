/*!
 * phi_core::if_phi – if/else PHI helpers (Phase 2)
 *
 * Public thin wrappers that mirror the semantics of existing builder::phi
 * helpers. Implemented locally to avoid depending on private submodules.
 * Behavior is identical to the current in-tree logic.
 */

use crate::ast::ASTNode;
use crate::mir::{MirFunction, MirInstruction, MirType, ValueId};
use std::collections::HashMap;

/// Infer return type by scanning for a Phi that defines `ret_val` and
/// verifying that all incoming values have the same type in `types`.
pub fn infer_type_from_phi(
    function: &MirFunction,
    ret_val: ValueId,
    types: &HashMap<ValueId, MirType>,
) -> Option<MirType> {
    for (_bid, bb) in function.blocks.iter() {
        for inst in bb.instructions.iter() {
            if let MirInstruction::Phi { dst, inputs } = inst {
                if *dst == ret_val {
                    let mut it = inputs.iter().filter_map(|(_, v)| types.get(v));
                    if let Some(first) = it.next() {
                        if it.all(|mt| mt == first) {
                            return Some(first.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract the assigned variable name from an AST fragment commonly used in
/// if/else analysis. Same logic as builder::phi::extract_assigned_var.
pub fn extract_assigned_var(ast: &ASTNode) -> Option<String> {
    match ast {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() {
                Some(name.clone())
            } else {
                None
            }
        }
        ASTNode::Program { statements, .. } => {
            statements.last().and_then(|st| extract_assigned_var(st))
        }
        ASTNode::If { then_body, else_body, .. } => {
            let then_prog = ASTNode::Program {
                statements: then_body.clone(),
                span: crate::ast::Span::unknown(),
            };
            let tvar = extract_assigned_var(&then_prog);
            let evar = else_body.as_ref().and_then(|eb| {
                let ep = ASTNode::Program {
                    statements: eb.clone(),
                    span: crate::ast::Span::unknown(),
                };
                extract_assigned_var(&ep)
            });
            crate::mir::phi_core::common::determine_assigned_name(&tvar, &evar)
        }
        _ => None,
    }
}

/// Collect all variable names that are assigned within the given AST subtree.
/// Useful for computing PHI merge candidates across branches/blocks.
pub fn collect_assigned_vars(ast: &ASTNode, out: &mut std::collections::HashSet<String>) {
    match ast {
        ASTNode::Assignment { target, .. } => {
            if let ASTNode::Variable { name, .. } = target.as_ref() {
                out.insert(name.clone());
            }
        }
        ASTNode::Program { statements, .. } => {
            for s in statements { collect_assigned_vars(s, out); }
        }
        ASTNode::If { then_body, else_body, .. } => {
            let tp = ASTNode::Program { statements: then_body.clone(), span: crate::ast::Span::unknown() };
            collect_assigned_vars(&tp, out);
            if let Some(eb) = else_body {
                let ep = ASTNode::Program { statements: eb.clone(), span: crate::ast::Span::unknown() };
                collect_assigned_vars(&ep, out);
            }
        }
        _ => {}
    }
}

/// Compute the set of variable names whose values changed in either branch
/// relative to the pre-if snapshot.
pub fn compute_modified_names(
    pre_if_snapshot: &HashMap<String, ValueId>,
    then_map_end: &HashMap<String, ValueId>,
    else_map_end_opt: &Option<HashMap<String, ValueId>>,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut names: HashSet<&str> = HashSet::new();
    for k in then_map_end.keys() { names.insert(k.as_str()); }
    if let Some(emap) = else_map_end_opt.as_ref() {
        for k in emap.keys() { names.insert(k.as_str()); }
    }
    let mut changed: Vec<String> = Vec::new();
    for &name in &names {
        let pre = pre_if_snapshot.get(name);
        let t = then_map_end.get(name);
        let e = else_map_end_opt.as_ref().and_then(|m| m.get(name));
        if (t.is_some() && Some(*t.unwrap()) != pre.copied())
            || (e.is_some() && Some(*e.unwrap()) != pre.copied())
        {
            changed.push(name.to_string());
        }
    }
    changed
}

/// Operations required for emitting a PHI or direct binding at a merge point.
pub trait PhiMergeOps {
    fn new_value(&mut self) -> ValueId;
    fn emit_phi_at_block_start(
        &mut self,
        block: crate::mir::BasicBlockId,
        dst: ValueId,
        inputs: Vec<(crate::mir::BasicBlockId, ValueId)>,
    ) -> Result<(), String>;
    fn update_var(&mut self, name: String, value: ValueId);
    fn debug_verify_phi_inputs(
        &mut self,
        _merge_bb: crate::mir::BasicBlockId,
        _inputs: &[(crate::mir::BasicBlockId, ValueId)],
    ) {
    }
}

/// Merge variables modified in branches at the merge block using provided ops.
/// Handles both two-pred and single-pred (reachable) cases gracefully.
pub fn merge_modified_at_merge_with<O: PhiMergeOps>(
    ops: &mut O,
    merge_bb: crate::mir::BasicBlockId,
    _then_block: crate::mir::BasicBlockId,
    else_block: crate::mir::BasicBlockId,
    then_pred_opt: Option<crate::mir::BasicBlockId>,
    else_pred_opt: Option<crate::mir::BasicBlockId>,
    pre_if_snapshot: &HashMap<String, ValueId>,
    then_map_end: &HashMap<String, ValueId>,
    else_map_end_opt: &Option<HashMap<String, ValueId>>,
    skip_var: Option<&str>,
) -> Result<(), String> {
    let trace = std::env::var("NYASH_IF_TRACE").ok().as_deref() == Some("1");
    let changed = compute_modified_names(pre_if_snapshot, then_map_end, else_map_end_opt);
    for name in changed {
        if skip_var.map(|s| s == name).unwrap_or(false) {
            continue;
        }
        let pre_opt = pre_if_snapshot.get(name.as_str()).copied();
        let then_v_opt = then_map_end.get(name.as_str()).copied().or(pre_opt);
        let else_v_opt = else_map_end_opt
            .as_ref()
            .and_then(|m| m.get(name.as_str()).copied())
            .or(pre_opt);

        if trace {
            eprintln!(
                "[if-trace] merge var={} pre={:?} then_v_opt={:?} else_v_opt={:?} then_pred={:?} else_pred={:?}",
                name, pre_opt, then_v_opt, else_v_opt, then_pred_opt, else_pred_opt
            );
        }

        // Build incoming pairs from reachable predecessors only
        let mut inputs: Vec<(crate::mir::BasicBlockId, ValueId)> = Vec::new();
        if let (Some(tp), Some(tv)) = (then_pred_opt, then_v_opt) { inputs.push((tp, tv)); }
        if let Some(ev) = else_v_opt {
            if let Some(ep) = else_pred_opt.or(Some(else_block)) { inputs.push((ep, ev)); }
        }

        match inputs.len() {
            0 => {}
            1 => {
                let (_pred, v) = inputs[0];
                if trace {
                    eprintln!(
                        "[if-trace] merge bind var={} v={:?} (single pred)",
                        name, v
                    );
                }
                ops.update_var(name, v);
            }
            _ => {
                ops.debug_verify_phi_inputs(merge_bb, &inputs);
                let dst = ops.new_value();
                ops.emit_phi_at_block_start(merge_bb, dst, inputs)?;
                if trace {
                    eprintln!(
                        "[if-trace] merge phi var={} dst={:?}",
                        name, dst
                    );
                }
                ops.update_var(name, dst);
            }
        }
    }
    Ok(())
}

/// Convenience wrapper: reset variable map (via a caller-provided closure)
/// then perform merge at the merge block. Keeps caller simple while
/// avoiding tying phi_core to concrete builder internals.
pub fn merge_with_reset_at_merge_with<O: PhiMergeOps>(
    ops: &mut O,
    merge_bb: crate::mir::BasicBlockId,
    then_block: crate::mir::BasicBlockId,
    else_block: crate::mir::BasicBlockId,
    then_pred_opt: Option<crate::mir::BasicBlockId>,
    else_pred_opt: Option<crate::mir::BasicBlockId>,
    pre_if_snapshot: &HashMap<String, ValueId>,
    then_map_end: &HashMap<String, ValueId>,
    else_map_end_opt: &Option<HashMap<String, ValueId>>,
    reset_vars: impl FnOnce(),
    skip_var: Option<&str>,
) -> Result<(), String> {
    reset_vars();
    merge_modified_at_merge_with(
        ops,
        merge_bb,
        then_block,
        else_block,
        then_pred_opt,
        else_pred_opt,
        pre_if_snapshot,
        then_map_end,
        else_map_end_opt,
        skip_var,
    )
}
