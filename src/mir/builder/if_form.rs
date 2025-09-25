use super::{ConstValue, MirBuilder, MirInstruction, ValueId};
use crate::mir::loop_api::LoopBuilderApi; // for current_block()
use crate::ast::ASTNode;

impl MirBuilder {
    /// Lower an if/else using a structured IfForm (header→then/else→merge).
    /// PHI-off: edge-copy only on predecessors; PHI-on: Phi at merge.
    pub(super) fn lower_if_form(
        &mut self,
        condition: ASTNode,
        then_branch: ASTNode,
        else_branch: Option<ASTNode>,
    ) -> Result<ValueId, String> {
        let condition_val = self.build_expression(condition)?;

        // Create blocks
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();

        // Branch
        self.emit_instruction(MirInstruction::Branch {
            condition: condition_val,
            then_bb: then_block,
            else_bb: else_block,
        })?;

        // Snapshot variables before entering branches
        let pre_if_var_map = self.variable_map.clone();

        // then
        self.current_block = Some(then_block);
        self.ensure_block_exists(then_block)?;
        // Scope enter for then-branch
        self.hint_scope_enter(0);
        let then_ast_for_analysis = then_branch.clone();
        self.variable_map = pre_if_var_map.clone();
        let then_value_raw = self.build_expression(then_branch)?;
        let then_exit_block = self.current_block()?;
        let then_var_map_end = self.variable_map.clone();
        if !self.is_current_block_terminated() {
            // Scope leave for then-branch
            self.hint_scope_leave(0);
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }

        // else
        self.current_block = Some(else_block);
        self.ensure_block_exists(else_block)?;
        // Scope enter for else-branch
        self.hint_scope_enter(0);
        let (else_value_raw, else_ast_for_analysis, else_var_map_end_opt) = if let Some(else_ast) = else_branch {
            self.variable_map = pre_if_var_map.clone();
            let val = self.build_expression(else_ast.clone())?;
            (val, Some(else_ast), Some(self.variable_map.clone()))
        } else {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: void_val, value: ConstValue::Void })?;
            (void_val, None, None)
        };
        let else_exit_block = self.current_block()?;
        if !self.is_current_block_terminated() {
            // Scope leave for else-branch
            self.hint_scope_leave(0);
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }

        // merge: primary result via helper, then delta-based variable merges
        self.current_block = Some(merge_block);
        self.ensure_block_exists(merge_block)?;
        self.push_if_merge(merge_block);

        // Pre-analysis: identify then/else assigned var for skip and hints
        let assigned_then_pre = crate::mir::phi_core::if_phi::extract_assigned_var(&then_ast_for_analysis);
        let assigned_else_pre = else_ast_for_analysis
            .as_ref()
            .and_then(|e| crate::mir::phi_core::if_phi::extract_assigned_var(e));
        let pre_then_var_value = assigned_then_pre
            .as_ref()
            .and_then(|name| pre_if_var_map.get(name).copied());

        let result_val = self.normalize_if_else_phi(
            then_block,
            else_block,
            Some(then_exit_block),
            Some(else_exit_block),
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
            then_exit_block,
            Some(else_exit_block),
            &pre_if_var_map,
            &then_var_map_end,
            &else_var_map_end_opt,
            skip_name,
        )?;

        self.pop_if_merge();
        Ok(result_val)
    }
}
