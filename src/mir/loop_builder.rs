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
}

impl<'a> LoopBuilder<'a> {
    /// 新しいループビルダーを作成
    pub fn new(parent: &'a mut super::builder::MirBuilder) -> Self {
        Self {
            parent_builder: parent,
            incomplete_phis: HashMap::new(),
            block_var_maps: HashMap::new(),
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
        self.emit_safepoint()?;
        
        // ボディをビルド
        for stmt in body {
            self.build_statement(stmt)?;
        }
        
        // 8. Latchブロック（ボディの最後）からHeaderへ戻る
        let latch_id = self.current_block()?;
        self.emit_jump(header_id)?;
        let _ = self.add_predecessor(header_id, latch_id);
        
        // 9. Headerブロックをシール（全predecessors確定）
        self.seal_block(header_id, latch_id)?;
        
        // 10. ループ後の処理
        self.set_current_block(after_loop_id)?;
        
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
                // Latchブロックでの変数の値を取得
                let value_after = self.get_variable_at_block(&phi.var_name, latch_id)
                    .ok_or_else(|| format!("Variable {} not found at latch block", phi.var_name))?;
                
                // Phi nodeの入力を完成させる
                phi.known_inputs.push((latch_id, value_after));
                
                // 完成したPhi nodeを発行
                self.emit_phi_at_block_start(block_id, phi.phi_id, phi.known_inputs)?;
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
    
    fn get_variable_at_block(&self, name: &str, _block_id: BasicBlockId) -> Option<ValueId> {
        // 簡易実装：現在の変数マップから取得
        // TODO: 本来はブロックごとの変数マップを管理すべき
        self.parent_builder.variable_map.get(name).copied()
    }
    
    fn build_expression_with_phis(&mut self, expr: ASTNode) -> Result<ValueId, String> {
        // Phi nodeの結果を考慮しながら式を構築
        self.parent_builder.build_expression(expr)
    }
    
    fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
        self.parent_builder.build_expression(stmt)
    }
}