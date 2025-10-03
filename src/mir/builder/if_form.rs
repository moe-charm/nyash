use super::{MirBuilder, MirInstruction, ValueId};
use crate::mir::loop_api::LoopBuilderApi; // for current_block()
use crate::ast::{ASTNode, BinaryOperator};

impl MirBuilder {
    /// Lower an if/else using a structured IfForm (header→then/else→merge).
    /// PHI-off: edge-copy only on predecessors; PHI-on: Phi at merge.
    pub(super) fn lower_if_form(
        &mut self,
        condition: ASTNode,
        then_branch: ASTNode,
        else_branch: Option<ASTNode>,
    ) -> Result<ValueId, String> {
        // Reserve a deterministic join id for debug region labeling
        let join_id = self.debug_next_join_id();
        // Heuristic pre-pin: if condition is a comparison, evaluate its operands now and pin them
        // so that subsequent branches can safely reuse these values across blocks.
        // This leverages existing variable_map merges (PHI) at the merge block.
        if crate::config::env::mir_pre_pin_compare_operands() {
        if let ASTNode::BinaryOp { operator, left, right, .. } = &condition {
            match operator {
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::Less
                | BinaryOperator::LessEqual
                | BinaryOperator::Greater
                | BinaryOperator::GreaterEqual => {
                    if let Ok(lhs_v) = self.build_expression((**left).clone()) {
                        let _ = self.pin_to_slot(lhs_v, "@if_lhs");
                    }
                    if let Ok(rhs_v) = self.build_expression((**right).clone()) {
                        let _ = self.pin_to_slot(rhs_v, "@if_rhs");
                    }
                }
                _ => {}
            }
        }
        }

        let condition_val = self.build_expression(condition)?;
        let condition_val = self.local_cond(condition_val);

        // Create blocks
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();

        // Branch
        let pre_branch_bb = self.current_block()?;
        let mut condition_val = condition_val;
        crate::mir::builder::ssa::local::finalize_branch_cond(self, &mut condition_val);
        crate::mir::builder::emission::branch::emit_conditional(self, condition_val, then_block, else_block)?;

        // Snapshot variables before entering branches
        let pre_if_var_map = self.variable_map.clone();

        let trace_if = std::env::var("NYASH_IF_TRACE").ok().as_deref() == Some("1");

        // then
        self.start_new_block(then_block)?;
        // Debug region: join then-branch
        self.debug_push_region(format!("join#{}", join_id) + "/then");
        // Scope enter for then-branch
        self.hint_scope_enter(0);
        let then_ast_for_analysis = then_branch.clone();
        self.variable_map = pre_if_var_map.clone();
        // Materialize all variables at block entry via single-pred Phi (correctness-first)
        for (name, &pre_v) in pre_if_var_map.iter() {
            let phi_val = self.value_gen.next();
            let inputs = vec![(pre_branch_bb, pre_v)];
            self.emit_instruction(MirInstruction::Phi { dst: phi_val, inputs })?;
            self.variable_map.insert(name.clone(), phi_val);
            if trace_if {
                eprintln!(
                    "[if-trace] then-entry phi var={} pre={:?} -> dst={:?}",
                    name, pre_v, phi_val
                );
            }
        }
        let then_value_raw = self.build_expression(then_branch)?;
        let then_exit_block = self.current_block()?;
        let then_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() {
            // Scope leave for then-branch
            self.hint_scope_leave(0);
            crate::mir::builder::emission::branch::emit_jump(self, merge_block)?;
        }
        // Pop then-branch debug region
        self.debug_pop_region();

        // else
        self.start_new_block(else_block)?;
        // Debug region: join else-branch
        self.debug_push_region(format!("join#{}", join_id) + "/else");
        // Scope enter for else-branch
        self.hint_scope_enter(0);
        // Materialize all variables at block entry via single-pred Phi (correctness-first)
        for (name, &pre_v) in pre_if_var_map.iter() {
            let phi_val = self.value_gen.next();
            let inputs = vec![(pre_branch_bb, pre_v)];
            self.emit_instruction(MirInstruction::Phi { dst: phi_val, inputs })?;
            self.variable_map.insert(name.clone(), phi_val);
            if trace_if {
                eprintln!(
                    "[if-trace] else-entry phi var={} pre={:?} -> dst={:?}",
                    name, pre_v, phi_val
                );
            }
        }
        let (else_value_raw, else_ast_for_analysis, else_var_map_end_opt) = if let Some(else_ast) = else_branch {
            self.variable_map = pre_if_var_map.clone();
            let val = self.build_expression(else_ast.clone())?;
            (val, Some(else_ast), Some(self.variable_map.clone()))
        } else {
            let void_val = crate::mir::builder::emission::constant::emit_void(self);
            (void_val, None, None)
        };
        let else_exit_block = self.current_block()?;
        if !self.is_current_block_terminated() {
            // Scope leave for else-branch
            self.hint_scope_leave(0);
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }
        // Pop else-branch debug region
        self.debug_pop_region();

        // merge: primary result via helper, then delta-based variable merges
        // Ensure PHIs are first in the block by suppressing entry pin copies here
        self.suppress_next_entry_pin_copy();
        self.start_new_block(merge_block)?;
        // Debug region: join merge
        self.debug_push_region(format!("join#{}", join_id) + "/join");
        self.push_if_merge(merge_block);

        // Pre-analysis: identify then/else assigned var for skip and hints
        let assigned_then_pre = crate::mir::phi_core::if_phi::extract_assigned_var(&then_ast_for_analysis);
        let assigned_else_pre = else_ast_for_analysis
            .as_ref()
            .and_then(|e| crate::mir::phi_core::if_phi::extract_assigned_var(e));
        let pre_then_var_value = assigned_then_pre
            .as_ref()
            .and_then(|name| pre_if_var_map.get(name).copied());

        // Check if branches are terminated (return/throw) to exclude them from PHI predecessors
        let (then_pred_opt, else_pred_opt) =
            super::phi_merge_helper::PhiMergeHelper::compute_if_merge_preds(self, then_exit_block, else_exit_block);

        let result_val = self.normalize_if_else_phi(
            then_block,
            else_block,
            then_pred_opt,
            else_pred_opt,
            then_value_raw,
            else_value_raw,
            &pre_if_var_map,
            &then_ast_for_analysis,
            &else_ast_for_analysis,
            &then_var_map_end,
            &else_var_map_end_opt,
            pre_then_var_value,
        )?;

        // Hint: join result variable(s)
        // 1) Primary: if both branches assign to the same variable name, emit a hint for that name
        if let (Some(tn), Some(en)) = (assigned_then_pre.as_deref(), assigned_else_pre.as_deref()) {
            if tn == en { self.hint_join_result(tn); }
        }
        // 2) Secondary: if both branches assign multiple variables, hint全件（制限なし）
        if let Some(ref else_map_end) = else_var_map_end_opt {
            for name in then_var_map_end.keys() {
                if Some(name.as_str()) == assigned_then_pre.as_deref() { continue; }
                if else_map_end.contains_key(name) {
                    self.hint_join_result(name.as_str());
                }
            }
        }

        // Merge other modified variables (skip the primary assignment if any)
        let skip_name = assigned_then_pre.as_deref();
        self.merge_modified_vars(
            then_block,
            else_block,
            then_pred_opt,  // Use computed then_pred_opt (None if terminated)
            else_pred_opt,  // Use computed else_pred_opt (None if terminated)
            &pre_if_var_map,
            &then_var_map_end,
            &else_var_map_end_opt,
            skip_name,
        )?;

        self.pop_if_merge();
        // Pop merge debug region
        self.debug_pop_region();
        Ok(result_val)
    }
}
