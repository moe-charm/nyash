use super::MirBuilder;
use crate::ast::ASTNode;
use crate::mir::{BasicBlockId, MirInstruction, ValueId};
use std::collections::HashMap;

// Local helper has moved to phi_core::if_phi; keep call sites minimal

impl MirBuilder {
    /// Merge all variables modified in then/else relative to pre_if_snapshot.
    /// In PHI-off mode inserts edge copies from branch exits to merge. In PHI-on mode emits Phi.
    /// `skip_var` allows skipping a variable already merged elsewhere (e.g., bound to an expression result).
    pub(super) fn merge_modified_vars(
        &mut self,
        _then_block: super::BasicBlockId,
        else_block: super::BasicBlockId,
        then_exit_block: super::BasicBlockId,
        else_exit_block_opt: Option<super::BasicBlockId>,
        pre_if_snapshot: &std::collections::HashMap<String, super::ValueId>,
        then_map_end: &std::collections::HashMap<String, super::ValueId>,
        else_map_end_opt: &Option<std::collections::HashMap<String, super::ValueId>>,
        skip_var: Option<&str>,
    ) -> Result<(), String> {
        let changed = crate::mir::phi_core::if_phi::compute_modified_names(
            pre_if_snapshot,
            then_map_end,
            else_map_end_opt,
        );
        use std::collections::HashSet;
        let changed_set: HashSet<String> = changed.iter().cloned().collect();
        for name in changed {
            if skip_var.map(|s| s == name).unwrap_or(false) {
                continue;
            }
            let pre = match pre_if_snapshot.get(name.as_str()) {
                Some(v) => *v,
                None => continue, // unknown before-if; skip
            };
            let then_v = then_map_end.get(name.as_str()).copied().unwrap_or(pre);
            let else_v = else_map_end_opt
                .as_ref()
                .and_then(|m| m.get(name.as_str()).copied())
                .unwrap_or(pre);
            // フェーズM: 常にPHI命令を使用（no_phi_mode撤廃）
            // incoming の predecessor は "実際に merge に遷移してくる出口ブロック" を使用する
            let then_pred = then_exit_block;
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            let merged = self.value_gen.next();
            let inputs = vec![(then_pred, then_v), (else_pred, else_v)];
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: merged, inputs })?;
            self.variable_map.insert(name, merged);
        }

        // Ensure pinned synthetic slots ("__pin$...") have a block-local definition at the merge,
        // even if their values did not change across branches. This avoids undefined uses when
        // subsequent blocks re-use pinned values without modifications.
        for (pin_name, pre_val) in pre_if_snapshot.iter() {
            if !pin_name.starts_with("__pin$") { continue; }
            if skip_var.map(|s| s == pin_name.as_str()).unwrap_or(false) { continue; }
            if changed_set.contains(pin_name) { continue; }
            let then_v = then_map_end.get(pin_name.as_str()).copied().unwrap_or(*pre_val);
            let else_v = else_map_end_opt
                .as_ref()
                .and_then(|m| m.get(pin_name.as_str()).copied())
                .unwrap_or(*pre_val);
            let then_pred = then_exit_block;
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            let merged = self.value_gen.next();
            let inputs = vec![(then_pred, then_v), (else_pred, else_v)];
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: merged, inputs })?;
            self.variable_map.insert(pin_name.clone(), merged);
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
        let assigned_var_then = crate::mir::phi_core::if_phi::extract_assigned_var(then_ast_for_analysis);
        let assigned_var_else = else_ast_for_analysis
            .as_ref()
            .and_then(|a| crate::mir::phi_core::if_phi::extract_assigned_var(a));
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
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs })?;
            self.variable_map = pre_if_var_map.clone();
            self.variable_map.insert(var_name, result_val);
        } else {
            // No variable assignment pattern detected – just emit Phi for expression result
            let then_pred = then_exit_block_opt.unwrap_or(then_block);
            let else_pred = else_exit_block_opt.unwrap_or(else_block);
            let inputs = vec![(then_pred, then_value_raw), (else_pred, else_value_raw)];
            if let (Some(func), Some(cur_bb)) = (&self.current_function, self.current_block) {
                crate::mir::phi_core::common::debug_verify_phi_inputs(func, cur_bb, &inputs);
            }
            self.emit_instruction(MirInstruction::Phi { dst: result_val, inputs })?;
            // Merge variable map conservatively to pre-if snapshot (no new bindings)
            self.variable_map = pre_if_var_map.clone();
        }

        Ok(result_val)
    }
}
