/*!
 * Loop Building - Main loop construction and statement/expression building
 */

use super::LoopBuilder;
use super::carrier_analyzer::LoopCarrierAnalyzerBox;
use crate::ast::ASTNode;
use crate::mir::{BasicBlockId, ConstValue, ValueId};
use std::collections::HashMap;

// Import control flow utilities
use crate::mir::utils::{
    is_current_block_terminated,
    capture_actual_predecessor_and_jump,
};

impl<'a> LoopBuilder<'a> {
    /// SSA形式でループを構築
    pub fn build_loop(
        &mut self,
        condition: ASTNode,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Reserve a deterministic loop id for debug region labeling
        let loop_id = self.parent_builder.debug_next_loop_id();
        // Pre-scan body for simple carrier pattern (up to 2 assigned variables, no break/continue)
        let mut assigned_vars: Vec<String> = Vec::new();
        let mut has_ctrl = false;
        for st in &body { crate::mir::phi_core::loop_phi::collect_carrier_assigns(st, &mut assigned_vars, &mut has_ctrl); }
        if !has_ctrl && !assigned_vars.is_empty() && assigned_vars.len() <= 2 {
            // Emit a carrier hint (no-op sink by default; visible with NYASH_MIR_TRACE_HINTS=1)
            self.parent_builder.hint_loop_carrier(assigned_vars.clone());
        }

        // 1. ブロックの準備
        let preheader_id = self.current_block()?;
        // 🔥 ROOT CAUSE FIX: Capture preheader variable map BEFORE jumping to header
        let preheader_vars = self.get_current_variable_map();
        let trace = std::env::var("NYASH_LOOP_TRACE").ok().as_deref() == Some("1");
        let (header_id, body_id, after_loop_id) =
            crate::mir::builder::loops::create_loop_blocks(self.parent_builder);
        if trace {
            eprintln!(
                "[loop] blocks preheader={:?} header={:?} body={:?} exit={:?}",
                preheader_id, header_id, body_id, after_loop_id
            );
        }
        self.loop_header = Some(header_id);
        self.continue_snapshots.clear();

        // 2. Preheader -> Header へのジャンプ
        self.emit_jump(header_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, header_id, preheader_id);

        // 3. Headerブロックの準備（unsealed状態）
        self.set_current_block(header_id)?;
        // Debug region: loop header
        self.parent_builder
            .debug_push_region(format!("loop#{}", loop_id) + "/header");
        // Hint: loop header (no-op sink)
        self.parent_builder.hint_loop_header();
        let _ = self.mark_block_unsealed(header_id);

        // 4. ループ変数のPhi nodeを準備
        // 🔥 ROOT CAUSE FIX: Only create PHIs for true loop-carried variables
        // True loop-carried = (variables in preheader) ∩ (variables assigned in body)
        // 🧹 CLEAN-CLEAN: Use LoopCarrierAnalyzerBox for single-responsibility analysis
        let loop_carried_vars = LoopCarrierAnalyzerBox::analyze(&preheader_vars, &body);
        if trace {
            eprintln!(
                "[loop] loop-carried vars: {:?}",
                loop_carried_vars.iter().collect::<Vec<_>>()
            );
        }
        self.prepare_loop_variables(header_id, preheader_id, &loop_carried_vars)?;

        // 5. 条件評価（Phi nodeの結果を使用）
        // Heuristic pre-pin: if condition is a comparison, evaluate its operands and pin them
        // so that the loop body/next iterations can safely reuse these values across blocks.
        if crate::config::env::mir_pre_pin_compare_operands() {
        if let ASTNode::BinaryOp { operator, left, right, .. } = &condition {
            use crate::ast::BinaryOperator as BO;
            match operator {
                BO::Equal | BO::NotEqual | BO::Less | BO::LessEqual | BO::Greater | BO::GreaterEqual => {
                    if let Ok(lhs_v) = self.parent_builder.build_expression((**left).clone()) {
                        let _ = self.parent_builder.pin_to_slot(lhs_v, "@loop_if_lhs");
                    }
                    if let Ok(rhs_v) = self.parent_builder.build_expression((**right).clone()) {
                        let _ = self.parent_builder.pin_to_slot(rhs_v, "@loop_if_rhs");
                    }
                }
                _ => {}
            }
        }
        }
        // Snapshot variable bindings to prevent short-circuit lowering from mutating loop state.
        // Logical conditions (e.g., `j < end && ...`) may introduce Phi-only copies that should not
        // propagate outside the header. Restoring the snapshot keeps loop-carried variables stable.
        let condition_varmap_snapshot = self.parent_builder.variable_map.clone();
        let condition_value = self.build_expression_with_phis(condition)?;
        self.parent_builder.variable_map = condition_varmap_snapshot;

        // 6. 条件分岐
        let pre_branch_bb = self.current_block()?;
        self.emit_branch(condition_value, body_id, after_loop_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, body_id, header_id);
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, after_loop_id, header_id);
        if trace {
            eprintln!(
                "[loop] header branched to body={:?} and exit={:?}",
                body_id, after_loop_id
            );
        }

        // 7. ループボディの構築
        self.set_current_block(body_id)?;
        // Debug region: loop body
        self.parent_builder
            .debug_replace_region(format!("loop#{}", loop_id) + "/body");
        // Materialize pinned slots at entry via single-pred Phi
        let names: Vec<String> = self.parent_builder.variable_map.keys().cloned().collect();
        for name in names {
            if !name.starts_with("__pin$") { continue; }
            if let Some(&pre_v) = self.parent_builder.variable_map.get(&name) {
                let phi_val = self.new_value();
                self.emit_phi_at_block_start(body_id, phi_val, vec![(pre_branch_bb, pre_v)])?;
                self.update_variable(name, phi_val);
            }
        }
        // Scope enter for loop body
        self.parent_builder.hint_scope_enter(0);
        // Optional safepoint per loop-iteration
        if std::env::var("NYASH_BUILDER_SAFEPOINT_LOOP")
            .ok()
            .as_deref()
            == Some("1")
        {
            self.emit_safepoint()?;
        }

        // ボディをビルド
        for stmt in body {
            self.build_statement(stmt)?;
        }
        // 8. Latchブロック（ボディの最後）からHeaderへ戻る
        // 現在の挿入先が latch（最後のブロック）なので、そのブロックIDでスナップショットを保存する
        let latch_id = self.current_block()?;
        // Hint: loop latch (no-op sink)
        self.parent_builder.hint_loop_latch();
        // Debug region: loop latch (end of body)
        self.parent_builder
            .debug_replace_region(format!("loop#{}", loop_id) + "/latch");
        // Scope leave for loop body
        self.parent_builder.hint_scope_leave(0);
        let latch_snapshot = self.get_current_variable_map();
        // 以前は body_id に保存していたが、複数ブロックのボディや continue 混在時に不正確になるため
        // 実際の latch_id に対してスナップショットを紐づける
        crate::mir::phi_core::loop_phi::save_block_snapshot(
            &mut self.block_var_maps,
            latch_id,
            &latch_snapshot,
        );
        // Only jump back to header if the latch block is not already terminated
        {
            let need_jump = {
                if let Some(ref fun_ro) = self.parent_builder.current_function {
                    if let Some(bb) = fun_ro.get_block(latch_id) {
                        !bb.is_terminated()
                    } else {
                        true
                    }
                } else {
                    true
                }
            };
            if need_jump {
                self.emit_jump(header_id)?;
                let _ = crate::mir::builder::loops::add_predecessor(
                    self.parent_builder,
                    header_id,
                    latch_id,
                );
            }
        }

        // 9. Headerブロックをシール（全predecessors確定）
        self.seal_block(header_id, latch_id)?;
        if trace {
            eprintln!(
                "[loop] sealed header={:?} with latch={:?}",
                header_id, latch_id
            );
        }

        // 10. ループ後の処理 - Exit PHI生成
        self.set_current_block(after_loop_id)?;
        // Debug region: loop exit
        self.parent_builder
            .debug_replace_region(format!("loop#{}", loop_id) + "/exit");

        // Exit PHIの生成 - break時点での変数値を統一
        self.create_exit_phis(header_id, after_loop_id)?;

        // Pop loop context
        crate::mir::builder::loops::pop_loop_context(self.parent_builder);
        // Pop debug region scope
        self.parent_builder.debug_pop_region();

        // void値を返す
        let void_dst = self.new_value();
        self.emit_const(void_dst, ConstValue::Void)?;
        if trace {
            eprintln!("[loop] exit={:?} return void=%{:?}", after_loop_id, void_dst);
        }
        Ok(void_dst)
    }

    /// Lower an if-statement inside a loop, preserving continue/break semantics and emitting PHIs per assigned variable.
    pub(super) fn lower_if_in_loop(
        &mut self,
        condition: ASTNode,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    ) -> Result<ValueId, String> {
        // Reserve a deterministic join id for debug region labeling (nested inside loop)
        let join_id = self.parent_builder.debug_next_join_id();
        // Pre-pin comparison operands to slots so repeated uses across blocks are safe
        if crate::config::env::mir_pre_pin_compare_operands() {
        if let ASTNode::BinaryOp { operator, left, right, .. } = &condition {
            use crate::ast::BinaryOperator as BO;
            match operator {
                BO::Equal | BO::NotEqual | BO::Less | BO::LessEqual | BO::Greater | BO::GreaterEqual => {
                    if let Ok(lhs_v) = self.parent_builder.build_expression((**left).clone()) {
                        let _ = self.parent_builder.pin_to_slot(lhs_v, "@loop_if_lhs");
                    }
                    if let Ok(rhs_v) = self.parent_builder.build_expression((**right).clone()) {
                        let _ = self.parent_builder.pin_to_slot(rhs_v, "@loop_if_rhs");
                    }
                }
                _ => {}
            }
        }
        }
        // Evaluate condition and create blocks
        let cond_val = self.parent_builder.build_expression(condition)?;
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let merge_bb = self.new_block();
        let pre_branch_bb = self.current_block()?;
        self.emit_branch(cond_val, then_bb, else_bb)?;

        // Capture pre-if variable map (used for phi normalization)
        let pre_if_var_map = self.get_current_variable_map();
        let trace_if = std::env::var("NYASH_IF_TRACE").ok().as_deref() == Some("1");
        // (legacy) kept for earlier merge style; now unified helpers compute deltas directly.

        // then branch
        self.set_current_block(then_bb)?;
        // Debug region: join then-branch (inside loop)
        self.parent_builder
            .debug_push_region(format!("join#{}", join_id) + "/then");
        // Materialize all variables at entry via single-pred Phi (correctness-first)
        let names_then: Vec<String> = self
            .parent_builder
            .variable_map
            .keys()
            .filter(|n| !n.starts_with("__pin$"))
            .cloned()
            .collect();
        for name in names_then {
            if let Some(&pre_v) = pre_if_var_map.get(&name) {
                let phi_val = self.new_value();
                self.emit_phi_at_block_start(then_bb, phi_val, vec![(pre_branch_bb, pre_v)])?;
                let name_for_log = name.clone();
                self.update_variable(name, phi_val);
                if trace_if {
                    eprintln!(
                        "[if-trace] then-entry phi var={} pre={:?} -> dst={:?}",
                        name_for_log, pre_v, phi_val
                    );
                }
            }
        }
        for s in then_body.iter().cloned() {
            let _ = self.build_statement(s)?;
            // フェーズS修正：統一終端検出ユーティリティ使用
            if is_current_block_terminated(self.parent_builder)? {
                break;
            }
        }
        let then_var_map_end = self.get_current_variable_map();
        // フェーズS修正：最強モード指摘の「実到達predecessor捕捉」を統一
        let then_pred_to_merge = capture_actual_predecessor_and_jump(
            self.parent_builder,
            merge_bb
        )?;
        // Pop then-branch debug region
        self.parent_builder.debug_pop_region();

        // else branch
        self.set_current_block(else_bb)?;
        // Debug region: join else-branch (inside loop)
        self.parent_builder
            .debug_push_region(format!("join#{}", join_id) + "/else");
        // Materialize all variables at entry via single-pred Phi (correctness-first)
        let names2: Vec<String> = self
            .parent_builder
            .variable_map
            .keys()
            .filter(|n| !n.starts_with("__pin$"))
            .cloned()
            .collect();
        for name in names2 {
            if let Some(&pre_v) = pre_if_var_map.get(&name) {
                let phi_val = self.new_value();
                self.emit_phi_at_block_start(else_bb, phi_val, vec![(pre_branch_bb, pre_v)])?;
                let name_for_log = name.clone();
                self.update_variable(name, phi_val);
                if trace_if {
                    eprintln!(
                        "[if-trace] else-entry phi var={} pre={:?} -> dst={:?}",
                        name_for_log, pre_v, phi_val
                    );
                }
            }
        }
        let mut else_var_map_end_opt: Option<HashMap<String, ValueId>> = None;
        if let Some(es) = else_body.clone() {
            for s in es.into_iter() {
                let _ = self.build_statement(s)?;
                // フェーズS修正：統一終端検出ユーティリティ使用
                if is_current_block_terminated(self.parent_builder)? {
                    break;
                }
            }
            else_var_map_end_opt = Some(self.get_current_variable_map());
        }
        // フェーズS修正：else branchでも統一実到達predecessor捕捉
        let else_pred_to_merge = capture_actual_predecessor_and_jump(
            self.parent_builder,
            merge_bb
        )?;
        // Pop else-branch debug region
        self.parent_builder.debug_pop_region();

        // Continue at merge
        self.set_current_block(merge_bb)?;
        // Debug region: join merge (inside loop)
        self.parent_builder
            .debug_push_region(format!("join#{}", join_id) + "/join");

        let mut vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        let then_prog = ASTNode::Program { statements: then_body.clone(), span: crate::ast::Span::unknown() };
        crate::mir::phi_core::if_phi::collect_assigned_vars(&then_prog, &mut vars);
        if let Some(es) = &else_body {
            let else_prog = ASTNode::Program { statements: es.clone(), span: crate::ast::Span::unknown() };
            crate::mir::phi_core::if_phi::collect_assigned_vars(&else_prog, &mut vars);
        }

        // Reset to pre-if map before rebinding to ensure a clean environment
        self.parent_builder.variable_map = pre_if_var_map.clone();
        // Use shared helper to merge modified variables at merge block
        struct Ops<'b, 'a>(&'b mut LoopBuilder<'a>);
        impl<'b, 'a> crate::mir::phi_core::if_phi::PhiMergeOps for Ops<'b, 'a> {
            fn new_value(&mut self) -> ValueId { self.0.new_value() }
            fn emit_phi_at_block_start(
                &mut self,
                block: BasicBlockId,
                dst: ValueId,
                inputs: Vec<(BasicBlockId, ValueId)>,
            ) -> Result<(), String> { self.0.emit_phi_at_block_start(block, dst, inputs) }
            fn update_var(&mut self, name: String, value: ValueId) {
                // Ensure VarMapGuard is applied for loop-internal if-merge as well
                self.0.update_variable(name, value);
            }
            fn debug_verify_phi_inputs(&mut self, merge_bb: BasicBlockId, inputs: &[(BasicBlockId, ValueId)]) {
                if let Some(ref func) = self.0.parent_builder.current_function {
                    crate::mir::phi_core::common::debug_verify_phi_inputs(func, merge_bb, inputs);
                }
            }
        }
        // Reset to pre-if snapshot, then delegate to shared helper
        self.parent_builder.variable_map = pre_if_var_map.clone();
        let mut ops = Ops(self);
        crate::mir::phi_core::if_phi::merge_modified_at_merge_with(
            &mut ops,
            merge_bb,
            then_bb,
            else_bb,
            then_pred_to_merge,
            else_pred_to_merge,
            &pre_if_var_map,
            &then_var_map_end,
            &else_var_map_end_opt,
            None,
        )?;
        let void_id = self.new_value();
        self.emit_const(void_id, ConstValue::Void)?;
        // Pop merge debug region
        self.parent_builder.debug_pop_region();
        Ok(void_id)
    }
}
