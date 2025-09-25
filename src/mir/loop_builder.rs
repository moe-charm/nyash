/*!
 * MIR Loop Builder - SSA形式でのループ構築専用モジュール
 *
 * Sealed/Unsealed blockとPhi nodeを使った正しいループ実装
 * Based on Gemini's recommendation for proper SSA loop handling
 */

use super::{BasicBlockId, ConstValue, MirInstruction, ValueId};
use crate::mir::phi_core::loop_phi::IncompletePhi;
use crate::ast::ASTNode;
use std::collections::HashMap;

// Phase 15 段階的根治戦略：制御フローユーティリティ
use super::utils::{
    is_current_block_terminated,
    capture_actual_predecessor_and_jump,
};

// IncompletePhi has moved to phi_core::loop_phi

/// ループビルダー - SSA形式でのループ構築を管理
pub struct LoopBuilder<'a> {
    /// 親のMIRビルダーへの参照
    parent_builder: &'a mut super::builder::MirBuilder,

    /// ループ内で追跡する変数の不完全Phi node
    incomplete_phis: HashMap<BasicBlockId, Vec<IncompletePhi>>,

    /// ブロックごとの変数マップ（スコープ管理）
    #[allow(dead_code)]
    block_var_maps: HashMap<BasicBlockId, HashMap<String, ValueId>>,

    /// ループヘッダーID（continueで使用）
    loop_header: Option<BasicBlockId>,

    /// continue文からの変数スナップショット
    continue_snapshots: Vec<(BasicBlockId, HashMap<String, ValueId>)>,

    /// break文からの変数スナップショット（exit PHI生成用）
    exit_snapshots: Vec<(BasicBlockId, HashMap<String, ValueId>)>,

    // フェーズM: no_phi_modeフィールド削除（常にPHI使用）
}


impl<'a> LoopBuilder<'a> {
    // Implement phi_core LoopPhiOps on LoopBuilder for in-place delegation
    
    // =============================================================
    // Control Helpers — break/continue/jumps/unreachable handling
    // =============================================================

    /// Emit a jump to `target` from the current block and record predecessor metadata.
    fn jump_with_pred(&mut self, target: BasicBlockId) -> Result<(), String> {
        let cur_block = self.current_block()?;
        self.emit_jump(target)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, target, cur_block);
        Ok(())
    }

    /// Switch insertion to a fresh (unreachable) block and place a Void const to keep callers satisfied.
    fn switch_to_unreachable_block_with_void(&mut self) -> Result<ValueId, String> {
        let next_block = self.new_block();
        self.set_current_block(next_block)?;
        let void_id = self.new_value();
        self.emit_const(void_id, ConstValue::Void)?;
        Ok(void_id)
    }

    /// Handle a `break` statement: jump to loop exit and continue in a fresh unreachable block.
    fn do_break(&mut self) -> Result<ValueId, String> {
        // Snapshot variables at break point for exit PHI generation
        let snapshot = self.get_current_variable_map();
        let cur_block = self.current_block()?;
        self.exit_snapshots.push((cur_block, snapshot));

        if let Some(exit_bb) = crate::mir::builder::loops::current_exit(self.parent_builder) {
            self.jump_with_pred(exit_bb)?;
        }
        self.switch_to_unreachable_block_with_void()
    }

    /// Handle a `continue` statement: snapshot vars, jump to loop header, then continue in a fresh unreachable block.
    fn do_continue(&mut self) -> Result<ValueId, String> {
        // Snapshot variables at current block to be considered as a predecessor input
        let snapshot = self.get_current_variable_map();
        let cur_block = self.current_block()?;
        self.block_var_maps.insert(cur_block, snapshot.clone());
        self.continue_snapshots.push((cur_block, snapshot));

        if let Some(header) = self.loop_header {
            self.jump_with_pred(header)?;
        }

        self.switch_to_unreachable_block_with_void()
    }

    // =============================================================
    // Lifecycle — create builder, main loop construction
    // =============================================================
    /// 新しいループビルダーを作成
    pub fn new(parent: &'a mut super::builder::MirBuilder) -> Self {
        // フェーズM: no_phi_mode初期化削除
        Self {
            parent_builder: parent,
            incomplete_phis: HashMap::new(),
            block_var_maps: HashMap::new(),
            loop_header: None,
            continue_snapshots: Vec::new(),
            exit_snapshots: Vec::new(),  // exit PHI用のスナップショット
            // フェーズM: no_phi_modeフィールド削除
        }
    }

    /// SSA形式でループを構築
    pub fn build_loop(
        &mut self,
        condition: ASTNode,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
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
        let (header_id, body_id, after_loop_id) =
            crate::mir::builder::loops::create_loop_blocks(self.parent_builder);
        self.loop_header = Some(header_id);
        self.continue_snapshots.clear();

        // 2. Preheader -> Header へのジャンプ
        self.emit_jump(header_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, header_id, preheader_id);

        // 3. Headerブロックの準備（unsealed状態）
        self.set_current_block(header_id)?;
        // Hint: loop header (no-op sink)
        self.parent_builder.hint_loop_header();
        let _ = self.mark_block_unsealed(header_id);

        // 4. ループ変数のPhi nodeを準備
        // ここでは、ループ内で変更される可能性のある変数を事前に検出するか、
        // または変数アクセス時に遅延生成する
        self.prepare_loop_variables(header_id, preheader_id)?;

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
        let condition_value = self.build_expression_with_phis(condition)?;

        // 6. 条件分岐
        let pre_branch_bb = self.current_block()?;
        self.emit_branch(condition_value, body_id, after_loop_id)?;
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, body_id, header_id);
        let _ = crate::mir::builder::loops::add_predecessor(self.parent_builder, after_loop_id, header_id);

        // 7. ループボディの構築
        self.set_current_block(body_id)?;
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

        // 10. ループ後の処理 - Exit PHI生成
        self.set_current_block(after_loop_id)?;

        // Exit PHIの生成 - break時点での変数値を統一
        self.create_exit_phis(header_id, after_loop_id)?;

        // Pop loop context
        crate::mir::builder::loops::pop_loop_context(self.parent_builder);

        // void値を返す
        let void_dst = self.new_value();
        self.emit_const(void_dst, ConstValue::Void)?;

        Ok(void_dst)
    }

    // =============================================================
    // PHI Helpers — prepare/finalize PHIs and block sealing
    // =============================================================
    /// ループ変数の準備（事前検出または遅延生成）
    fn prepare_loop_variables(
        &mut self,
        header_id: BasicBlockId,
        preheader_id: BasicBlockId,
    ) -> Result<(), String> {
        let current_vars = self.get_current_variable_map();
        crate::mir::phi_core::loop_phi::save_block_snapshot(
            &mut self.block_var_maps,
            preheader_id,
            &current_vars,
        );
        let incs = crate::mir::phi_core::loop_phi::prepare_loop_variables_with(
            self,
            header_id,
            preheader_id,
            &current_vars,
        )?;
        self.incomplete_phis.insert(header_id, incs);
        Ok(())
    }

    /// ブロックをシールし、不完全なPhi nodeを完成させる
    fn seal_block(&mut self, block_id: BasicBlockId, latch_id: BasicBlockId) -> Result<(), String> {
        if let Some(incomplete_phis) = self.incomplete_phis.remove(&block_id) {
            let cont_snaps = self.continue_snapshots.clone();
            crate::mir::phi_core::loop_phi::seal_incomplete_phis_with(
                self,
                block_id,
                latch_id,
                incomplete_phis,
                &cont_snaps,
            )?;
        }
        self.mark_block_sealed(block_id)?;
        Ok(())
    }

    /// Exitブロックで変数のPHIを生成（breakポイントでの値を統一）
    fn create_exit_phis(&mut self, header_id: BasicBlockId, exit_id: BasicBlockId) -> Result<(), String> {
        let header_vars = self.get_current_variable_map();
        let exit_snaps = self.exit_snapshots.clone();
        crate::mir::phi_core::loop_phi::build_exit_phis_with(
            self,
            header_id,
            exit_id,
            &header_vars,
            &exit_snaps,
        )
    }

    // --- ヘルパーメソッド（親ビルダーへの委譲） ---

    fn current_block(&self) -> Result<BasicBlockId, String> {
        self.parent_builder
            .current_block
            .ok_or_else(|| "No current block".to_string())
    }

    fn new_block(&mut self) -> BasicBlockId {
        self.parent_builder.block_gen.next()
    }

    fn new_value(&mut self) -> ValueId {
        self.parent_builder.value_gen.next()
    }

    fn set_current_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        self.parent_builder.start_new_block(block_id)
    }

    fn emit_jump(&mut self, target: BasicBlockId) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Jump { target })
    }

    fn emit_branch(
        &mut self,
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    ) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            })
    }

    fn emit_safepoint(&mut self) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Safepoint)
    }

    fn emit_const(&mut self, dst: ValueId, value: ConstValue) -> Result<(), String> {
        self.parent_builder
            .emit_instruction(MirInstruction::Const { dst, value })
    }

    /// ブロック先頭に PHI 命令を挿入（不変条件: PHI は常にブロック先頭）
    fn emit_phi_at_block_start(
        &mut self,
        block_id: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        // Phi nodeをブロックの先頭に挿入
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                // Phi命令は必ずブロックの先頭に配置
                let phi_inst = MirInstruction::Phi { dst, inputs };
                block.instructions.insert(0, phi_inst);
                Ok(())
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    #[allow(dead_code)]
    fn add_predecessor(&mut self, block: BasicBlockId, pred: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block) {
                block.add_predecessor(pred);
                Ok(())
            } else {
                Err(format!("Block {} not found", block))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    fn mark_block_unsealed(&mut self, _block_id: BasicBlockId) -> Result<(), String> {
        // ブロックはデフォルトでunsealedなので、特に何もしない
        // （既にBasicBlock::newでsealed: falseに初期化されている）
        Ok(())
    }

    fn mark_block_sealed(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.parent_builder.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                block.seal();
                Ok(())
            } else {
                Err(format!("Block {} not found", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }

    // =============================================================
    // Variable Map Utilities — snapshots and rebinding
    // =============================================================
    fn get_current_variable_map(&self) -> HashMap<String, ValueId> {
        self.parent_builder.variable_map.clone()
    }

    fn update_variable(&mut self, name: String, value: ValueId) {
        self.parent_builder.variable_map.insert(name, value);
    }

    fn get_variable_at_block(&self, name: &str, block_id: BasicBlockId) -> Option<ValueId> {
        // まずブロックごとのスナップショットを優先
        if let Some(map) = self.block_var_maps.get(&block_id) {
            if let Some(v) = map.get(name) {
                return Some(*v);
            }
        }
        // フォールバック：現在の変数マップ（単純ケース用）
        self.parent_builder.variable_map.get(name).copied()
    }

    fn build_expression_with_phis(&mut self, expr: ASTNode) -> Result<ValueId, String> {
        // Phi nodeの結果を考慮しながら式を構築
        self.parent_builder.build_expression(expr)
    }

    fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
        match stmt {
            // Ensure nested bare blocks inside loops are lowered with loop-aware semantics
            ASTNode::Program { statements, .. } => {
                let mut last = None;
                for s in statements.into_iter() {
                    last = Some(self.build_statement(s)?);
                    // フェーズS修正：統一終端検出ユーティリティ使用
                    if is_current_block_terminated(self.parent_builder)? {
                        break;
                    }
                }
                Ok(last.unwrap_or_else(|| {
                    let void_id = self.new_value();
                    // Emit a void const to keep SSA consistent when block is empty
                    let _ = self.emit_const(void_id, ConstValue::Void);
                    void_id
                }))
            }
            ASTNode::If { condition, then_body, else_body, .. } =>
                self.lower_if_in_loop(*condition, then_body, else_body),
            ASTNode::Break { .. } => self.do_break(),
            ASTNode::Continue { .. } => self.do_continue(),
            other => self.parent_builder.build_expression(other),
        }
    }

    /// Lower an if-statement inside a loop, preserving continue/break semantics and emitting PHIs per assigned variable.
    fn lower_if_in_loop(
        &mut self,
        condition: ASTNode,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    ) -> Result<ValueId, String> {
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
        // (legacy) kept for earlier merge style; now unified helpers compute deltas directly.

        // then branch
        self.set_current_block(then_bb)?;
        // Materialize all variables at entry via single-pred Phi (correctness-first)
        let names_then: Vec<String> = self.parent_builder.variable_map.keys().cloned().collect();
        for name in names_then {
            if let Some(&pre_v) = self.parent_builder.variable_map.get(&name) {
                let phi_val = self.new_value();
                self.emit_phi_at_block_start(then_bb, phi_val, vec![(pre_branch_bb, pre_v)])?;
                self.update_variable(name, phi_val);
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

        // else branch
        self.set_current_block(else_bb)?;
        // Materialize all variables at entry via single-pred Phi (correctness-first)
        let names2: Vec<String> = self.parent_builder.variable_map.keys().cloned().collect();
        for name in names2 {
            if let Some(&pre_v) = self.parent_builder.variable_map.get(&name) {
                let phi_val = self.new_value();
                self.emit_phi_at_block_start(else_bb, phi_val, vec![(pre_branch_bb, pre_v)])?;
                self.update_variable(name, phi_val);
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

        // Continue at merge
        self.set_current_block(merge_bb)?;

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
            fn update_var(&mut self, name: String, value: ValueId) { self.0.parent_builder.variable_map.insert(name, value); }
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
        Ok(void_id)
    }
}

// Implement phi_core LoopPhiOps on LoopBuilder for in-place delegation
impl crate::mir::phi_core::loop_phi::LoopPhiOps for LoopBuilder<'_> {
    fn new_value(&mut self) -> ValueId { self.new_value() }
    fn emit_phi_at_block_start(
        &mut self,
        block: BasicBlockId,
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    ) -> Result<(), String> {
        self.emit_phi_at_block_start(block, dst, inputs)
    }
    fn update_var(&mut self, name: String, value: ValueId) { self.update_variable(name, value) }
    fn get_variable_at_block(&mut self, name: &str, block: BasicBlockId) -> Option<ValueId> {
        // Call the inherent method (immutable borrow) to avoid recursion
        LoopBuilder::get_variable_at_block(self, name, block)
    }
    fn debug_verify_phi_inputs(&mut self, merge_bb: BasicBlockId, inputs: &[(BasicBlockId, ValueId)]) {
        if let Some(ref func) = self.parent_builder.current_function {
            crate::mir::phi_core::common::debug_verify_phi_inputs(func, merge_bb, inputs);
        }
    }
}
