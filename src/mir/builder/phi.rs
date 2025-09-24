use super::MirBuilder;
use crate::ast::ASTNode;
use crate::mir::{BasicBlockId, MirFunction, MirInstruction, MirType, ValueId};
use std::collections::HashMap;

// PHI-based return type inference helper
pub(super) fn infer_type_from_phi(
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

// Local helper for if-statement analysis (moved from stmts.rs)
pub(super) fn extract_assigned_var(ast: &ASTNode) -> Option<String> {
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
        ASTNode::If {
            then_body,
            else_body,
            ..
        } => {
            // Look into nested if: if both sides assign the same variable, propagate that name upward.
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
            match (tvar, evar) {
                (Some(tv), Some(ev)) if tv == ev => Some(tv),
                _ => None,
            }
        }
        _ => None,
    }
}

impl MirBuilder {
    #[inline]
    #[cfg(debug_assertions)]
    fn debug_verify_phi_inputs(&self, inputs: &Vec<(BasicBlockId, ValueId)>) {
        use std::collections::HashSet;
        if let Some(cur_bb) = self.current_block {
            let mut seen = HashSet::new();
            for (pred, _v) in inputs.iter() {
                debug_assert_ne!(
                    *pred, cur_bb,
                    "PHI incoming predecessor must not be the merge block itself"
                );
                debug_assert!(
                    seen.insert(*pred),
                    "Duplicate PHI incoming predecessor detected: {:?}",
                    pred
                );
            }
            // Ensure all incoming predecessors are known CFG predecessors of the merge block
            if let Some(func) = &self.current_function {
                if let Some(block) = func.blocks.get(&cur_bb) {
                    for (pred, _v) in inputs.iter() {
                        debug_assert!(
                            block.predecessors.contains(pred),
                            "PHI incoming pred {:?} is not a predecessor of merge bb {:?}",
                            pred,
                            cur_bb
                        );
                    }
                }
            }
        }
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    fn debug_verify_phi_inputs(&self, _inputs: &Vec<(BasicBlockId, ValueId)>) {}

    /// Merge all variables modified in then/else relative to pre_if_snapshot.
    /// In PHI-off mode inserts edge copies from branch exits to merge. In PHI-on mode emits Phi.
    /// `skip_var` allows skipping a variable already merged elsewhere (e.g., bound to an expression result).
    pub(super) fn merge_modified_vars(
        &mut self,
        then_block: super::BasicBlockId,
        else_block: super::BasicBlockId,
        then_exit_block: super::BasicBlockId,
        else_exit_block_opt: Option<super::BasicBlockId>,
        pre_if_snapshot: &std::collections::HashMap<String, super::ValueId>,
        then_map_end: &std::collections::HashMap<String, super::ValueId>,
        else_map_end_opt: &Option<std::collections::HashMap<String, super::ValueId>>,
        skip_var: Option<&str>,
    ) -> Result<(), String> {
        use std::collections::HashSet;
        let mut names: HashSet<&str> = HashSet::new();
        for k in then_map_end.keys() { names.insert(k.as_str()); }
        if let Some(emap) = else_map_end_opt.as_ref() {
            for k in emap.keys() { names.insert(k.as_str()); }
        }
        // Only variables that changed against pre_if_snapshot
        let mut changed: Vec<&str> = Vec::new();
        for &name in &names {
            let pre = pre_if_snapshot.get(name);
            let t = then_map_end.get(name);
            let e = else_map_end_opt.as_ref().and_then(|m| m.get(name));
            // changed when either branch value differs from pre
            if (t.is_some() && Some(t.copied().unwrap()) != pre.copied())
                || (e.is_some() && Some(e.copied().unwrap()) != pre.copied())
            {
                changed.push(name);
            }
        }
        for name in changed {
            if skip_var.map(|s| s == name).unwrap_or(false) {
                continue;
            }
            let pre = match pre_if_snapshot.get(name) {
                Some(v) => *v,
                None => continue, // unknown before-if; skip
            };
            let then_v = then_map_end.get(name).copied().unwrap_or(pre);
            let else_v = else_map_end_opt
                .as_ref()
                .and_then(|m| m.get(name).copied())
                .unwrap_or(pre);
            // フェーズM: 常にPHI命令を使用（no_phi_mode撤廃）
            // incoming の predecessor は "実際に merge に遷移してくる出口ブロック" を使用する
            let then_pred = then_exit_block;
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            let merged = self.value_gen.next();
            let inputs = vec![(then_pred, then_v), (else_pred, else_v)];
            self.debug_verify_phi_inputs(&inputs);
            self.emit_instruction(MirInstruction::Phi { dst: merged, inputs })?;
            self.variable_map.insert(name.to_string(), merged);
        }
        Ok(())
    }
    /// Normalize Phi creation for if/else constructs.
    /// This handles variable reassignment patterns and ensures a single exit value.
    pub(super) fn normalize_if_else_phi(
        &mut self,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
        then_exit_block_opt: Option<BasicBlockId>,
        else_exit_block_opt: Option<BasicBlockId>,
        then_value_raw: ValueId,
        else_value_raw: ValueId,
        pre_if_var_map: &HashMap<String, ValueId>,
        then_ast_for_analysis: &ASTNode,
        else_ast_for_analysis: &Option<ASTNode>,
        then_var_map_end: &HashMap<String, ValueId>,
        else_var_map_end_opt: &Option<HashMap<String, ValueId>>,
        pre_then_var_value: Option<ValueId>,
    ) -> Result<ValueId, String> {
        // If only the then-branch assigns a variable (e.g., `if c { x = ... }`) and the else
        // does not assign the same variable, bind that variable to a Phi of (then_value, pre_if_value).
        let assigned_var_then = extract_assigned_var(then_ast_for_analysis);
        let assigned_var_else = else_ast_for_analysis
            .as_ref()
            .and_then(|a| extract_assigned_var(a));
        let result_val = self.value_gen.next();

        // フェーズM: no_phi_mode分岐削除（常にPHI命令を使用）

        if let Some(var_name) = assigned_var_then.clone() {
            let else_assigns_same = assigned_var_else
                .as_ref()
                .map(|s| s == &var_name)
                .unwrap_or(false);
            // Resolve branch-end values for the assigned variable
            let then_value_for_var = then_var_map_end
                .get(&var_name)
                .copied()
                .unwrap_or(then_value_raw);
            let else_value_for_var = if else_assigns_same {
                else_var_map_end_opt
                    .as_ref()
                    .and_then(|m| m.get(&var_name).copied())
                    .unwrap_or(else_value_raw)
            } else {
                // Else doesn't assign: use pre-if value if available
                pre_then_var_value.unwrap_or(else_value_raw)
            };
            // predecessor を then/else の exit ブロックに揃える
            let then_pred = then_exit_block_opt.unwrap_or(then_block);
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            // Emit Phi for the assigned variable and bind it
            let inputs = vec![(then_pred, then_value_for_var), (else_pred, else_value_for_var)];
            self.debug_verify_phi_inputs(&inputs);
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs })?;
            self.variable_map = pre_if_var_map.clone();
            self.variable_map.insert(var_name, result_val);
        } else {
            // No variable assignment pattern detected – just emit Phi for expression result
            let then_pred = then_exit_block_opt.unwrap_or(then_block);
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            let inputs = vec![(then_pred, then_value_raw), (else_pred, else_value_raw)];
            self.debug_verify_phi_inputs(&inputs);
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs })?;
            // Merge variable map conservatively to pre-if snapshot (no new bindings)
            self.variable_map = pre_if_var_map.clone();
        }

        Ok(result_val)
    }
}
