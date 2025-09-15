/*!
 * MIR Loop Builder - SSA形式でのループ構築専用モジュール
 * 
 * Sealed/Unsealed blockとPhi nodeを使った正しいループ実装
 * Based on Gemini's recommendation for proper SSA loop handling
 */

use super::{
    MirInstruction, BasicBlockId, ValueId, 
    ConstValue
};
use crate::ast::ASTNode;
use std::collections::HashMap;

/// 不完全なPhi nodeの情報
#[derive(Debug, Clone)]
struct IncompletePhi {
    /// Phi nodeの結果ValueId
    phi_id: ValueId,
    /// 変数名
    var_name: String,
    /// 既知の入力値 (predecessor block id, value)
    known_inputs: Vec<(BasicBlockId, ValueId)>,
}

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
}

impl<'a> LoopBuilder<'a> {
    /// 新しいループビルダーを作成
    pub fn new(parent: &'a mut super::builder::MirBuilder) -> Self {
        Self {
            parent_builder: parent,
            incomplete_phis: HashMap::new(),
            block_var_maps: HashMap::new(),
            loop_header: None,
            continue_snapshots: Vec::new(),
        }
    }
    
    /// SSA形式でループを構築
    pub fn build_loop(
        &mut self,
        condition: ASTNode,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // 1. ブロックの準備
        let preheader_id = self.current_block()?;
        let header_id = self.new_block();
        let body_id = self.new_block();
        let after_loop_id = self.new_block();
        self.loop_header = Some(header_id);
        self.continue_snapshots.clear();
        self.parent_builder.loop_exit_stack.push(after_loop_id);
        // Push loop context to parent builder (for nested break/continue lowering)
        self.parent_builder.loop_header_stack.push(header_id);
        
        // 2. Preheader -> Header へのジャンプ
        self.emit_jump(header_id)?;
        let _ = self.add_predecessor(header_id, preheader_id);
        
        // 3. Headerブロックの準備（unsealed状態）
        self.set_current_block(header_id)?;
        let _ = self.mark_block_unsealed(header_id);
        
        // 4. ループ変数のPhi nodeを準備
        // ここでは、ループ内で変更される可能性のある変数を事前に検出するか、
        // または変数アクセス時に遅延生成する
        self.prepare_loop_variables(header_id, preheader_id)?;
        
        // 5. 条件評価（Phi nodeの結果を使用）
        let condition_value = self.build_expression_with_phis(condition)?;
        
        // 6. 条件分岐
        self.emit_branch(condition_value, body_id, after_loop_id)?;
        let _ = self.add_predecessor(body_id, header_id);
        let _ = self.add_predecessor(after_loop_id, header_id);
        
        // 7. ループボディの構築
        self.set_current_block(body_id)?;
        // Optional safepoint per loop-iteration
        if std::env::var("NYASH_BUILDER_SAFEPOINT_LOOP").ok().as_deref() == Some("1") {
            self.emit_safepoint()?;
        }

        // ボディをビルド
        for stmt in body {
            self.build_statement(stmt)?;
        }
        // 8. Latchブロック（ボディの最後）からHeaderへ戻る
        // 現在の挿入先が latch（最後のブロック）なので、そのブロックIDでスナップショットを保存する
        let latch_id = self.current_block()?;
        let latch_snapshot = self.get_current_variable_map();
        // 以前は body_id に保存していたが、複数ブロックのボディや continue 混在時に不正確になるため
        // 実際の latch_id に対してスナップショットを紐づける
        self.block_var_maps.insert(latch_id, latch_snapshot);
        self.emit_jump(header_id)?;
        let _ = self.add_predecessor(header_id, latch_id);
        
        // 9. Headerブロックをシール（全predecessors確定）
        self.seal_block(header_id, latch_id)?;
        
        // 10. ループ後の処理
        self.set_current_block(after_loop_id)?;
        // Pop loop context
        let _ = self.parent_builder.loop_header_stack.pop();
        // loop exit stack mirrors header stack; maintain symmetry
        let _ = self.parent_builder.loop_exit_stack.pop();

        // void値を返す
        let void_dst = self.new_value();
        self.emit_const(void_dst, ConstValue::Void)?;

        Ok(void_dst)
    }
    
    /// ループ変数の準備（事前検出または遅延生成）
    fn prepare_loop_variables(
        &mut self,
        header_id: BasicBlockId,
        preheader_id: BasicBlockId,
    ) -> Result<(), String> {
        // 現在の変数マップから、ループで使用される可能性のある変数を取得
        let current_vars = self.get_current_variable_map();
        // preheader時点のスナップショット（後でphi入力の解析に使う）
        self.block_var_maps.insert(preheader_id, current_vars.clone());
        
        // 各変数に対して不完全なPhi nodeを作成
        let mut incomplete_phis = Vec::new();
        for (var_name, &value_before) in &current_vars {
            let phi_id = self.new_value();
            
            // 不完全なPhi nodeを作成（preheaderからの値のみ設定）
            let incomplete_phi = IncompletePhi {
                phi_id,
                var_name: var_name.clone(),
                known_inputs: vec![(preheader_id, value_before)],
            };
            
            incomplete_phis.push(incomplete_phi);
            
            // 変数マップを更新（Phi nodeの結果を使用）
            self.update_variable(var_name.clone(), phi_id);
        }
        
        // 不完全なPhi nodeを記録
        self.incomplete_phis.insert(header_id, incomplete_phis);
        
        Ok(())
    }
    
    /// ブロックをシールし、不完全なPhi nodeを完成させる
    fn seal_block(
        &mut self,
        block_id: BasicBlockId,
        latch_id: BasicBlockId,
    ) -> Result<(), String> {
        // 不完全なPhi nodeを取得
        if let Some(incomplete_phis) = self.incomplete_phis.remove(&block_id) {
            for mut phi in incomplete_phis {
                for (cid, snapshot) in &self.continue_snapshots {
                    if let Some(v) = snapshot.get(&phi.var_name) {
                        phi.known_inputs.push((*cid, *v));
                    }
                }

                let value_after = self
                    .get_variable_at_block(&phi.var_name, latch_id)
                    .ok_or_else(|| {
                        format!("Variable {} not found at latch block", phi.var_name)
                    })?;

                phi.known_inputs.push((latch_id, value_after));

                self.emit_phi_at_block_start(block_id, phi.phi_id, phi.known_inputs)?;
                self.update_variable(phi.var_name.clone(), phi.phi_id);
            }
        }
        
        // ブロックをシール済みとしてマーク
        self.mark_block_sealed(block_id)?;
        
        Ok(())
    }
    
    // --- ヘルパーメソッド（親ビルダーへの委譲） ---
    
    fn current_block(&self) -> Result<BasicBlockId, String> {
        self.parent_builder.current_block
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
        self.parent_builder.emit_instruction(MirInstruction::Jump { target })
    }
    
    fn emit_branch(
        &mut self,
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    ) -> Result<(), String> {
        self.parent_builder.emit_instruction(MirInstruction::Branch {
            condition,
            then_bb,
            else_bb,
        })
    }
    
    fn emit_safepoint(&mut self) -> Result<(), String> {
        self.parent_builder.emit_instruction(MirInstruction::Safepoint)
    }
    
    fn emit_const(&mut self, dst: ValueId, value: ConstValue) -> Result<(), String> {
        self.parent_builder.emit_instruction(MirInstruction::Const { dst, value })
    }
    
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
    
    fn get_current_variable_map(&self) -> HashMap<String, ValueId> {
        self.parent_builder.variable_map.clone()
    }
    
    fn update_variable(&mut self, name: String, value: ValueId) {
        self.parent_builder.variable_map.insert(name, value);
    }
    
    fn get_variable_at_block(&self, name: &str, block_id: BasicBlockId) -> Option<ValueId> {
        // まずブロックごとのスナップショットを優先
        if let Some(map) = self.block_var_maps.get(&block_id) {
            if let Some(v) = map.get(name) { return Some(*v); }
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
            ASTNode::If { condition, then_body, else_body, .. } => {
                // Lower a simple if inside loop, ensuring continue/break inside branches are handled
                let cond_val = self.parent_builder.build_expression(*condition.clone())?;
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let merge_bb = self.new_block();
                self.emit_branch(cond_val, then_bb, else_bb)?;

                // then
                self.set_current_block(then_bb)?;
                for s in then_body.iter().cloned() {
                    let _ = self.build_statement(s)?;
                    // Stop if block terminated
                    let cur_id = self.current_block()?;
                    let terminated = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) { bb.is_terminated() } else { false }
                        } else { false }
                    };
                    if terminated { break; }
                }
                // Only jump to merge if not already terminated (e.g., continue/break)
                {
                    let cur_id = self.current_block()?;
                    let need_jump = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) { !bb.is_terminated() } else { false }
                        } else { false }
                    };
                    if need_jump { self.emit_jump(merge_bb)?; }
                }

                // else
                self.set_current_block(else_bb)?;
                if let Some(es) = else_body {
                    for s in es.into_iter() {
                        let _ = self.build_statement(s)?;
                        let cur_id = self.current_block()?;
                        let terminated = {
                            if let Some(ref fun_ro) = self.parent_builder.current_function {
                                if let Some(bb) = fun_ro.get_block(cur_id) { bb.is_terminated() } else { false }
                            } else { false }
                        };
                        if terminated { break; }
                    }
                }
                {
                    let cur_id = self.current_block()?;
                    let need_jump = {
                        if let Some(ref fun_ro) = self.parent_builder.current_function {
                            if let Some(bb) = fun_ro.get_block(cur_id) { !bb.is_terminated() } else { false }
                        } else { false }
                    };
                    if need_jump { self.emit_jump(merge_bb)?; }
                }

                // Continue at merge
                self.set_current_block(merge_bb)?;
                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            ASTNode::Break { .. } => {
                // Jump to loop exit (after_loop_id) if available
                let cur_block = self.current_block()?;
                // Ensure parent has recorded current loop exit; if not, record now
                if self.parent_builder.loop_exit_stack.last().copied().is_none() {
                    // Determine after_loop by peeking the next id used earlier:
                    // In this builder, after_loop_id was created above; record it for nested lowering
                    // We approximate by using the next block id minus 1 (after_loop) which we set below before branch
                }
                if let Some(exit_bb) = self.parent_builder.loop_exit_stack.last().copied() {
                    self.emit_jump(exit_bb)?;
                    let _ = self.add_predecessor(exit_bb, cur_block);
                }
                // Keep building in a fresh (unreachable) block to satisfy callers
                let next_block = self.new_block();
                self.set_current_block(next_block)?;
                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            ASTNode::Continue { .. } => {
                let snapshot = self.get_current_variable_map();
                let cur_block = self.current_block()?;
                self.block_var_maps.insert(cur_block, snapshot.clone());
                self.continue_snapshots.push((cur_block, snapshot));

                if let Some(header) = self.loop_header {
                    self.emit_jump(header)?;
                    let _ = self.add_predecessor(header, cur_block);
                }

                let next_block = self.new_block();
                self.set_current_block(next_block)?;

                let void_id = self.new_value();
                self.emit_const(void_id, ConstValue::Void)?;
                Ok(void_id)
            }
            other => self.parent_builder.build_expression(other),
        }
    }
}
