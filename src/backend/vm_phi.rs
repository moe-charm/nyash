/*!
 * VM Phi Node Handler - SSA形式のPhi nodeをVMで正しく実行するモジュール
 * 
 * MIRのloop_builder.rsに対応するVM側の実装
 * previous_blockを追跡してPhi nodeの正しい値を選択
 */

use super::vm::{VMValue, VMError};
use crate::mir::{BasicBlockId, ValueId, MirInstruction};
use std::collections::HashMap;

/// Phi nodeの実行ヘルパー
pub struct PhiHandler {
    /// 現在のブロックに到達する前のブロック
    previous_block: Option<BasicBlockId>,
    
    /// Phi nodeの値キャッシュ（最適化用）
    phi_cache: HashMap<ValueId, VMValue>,
}

impl PhiHandler {
    /// 新しいPhiハンドラーを作成
    pub fn new() -> Self {
        Self {
            previous_block: None,
            phi_cache: HashMap::new(),
        }
    }
    
    /// ブロック遷移を記録
    pub fn record_block_transition(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.previous_block = Some(from);
        // ブロック遷移時にキャッシュをクリア（新しいイテレーション）
        if self.is_loop_header(to) {
            self.phi_cache.clear();
        }
    }
    
    /// 初期ブロックへのエントリを記録
    pub fn record_entry(&mut self) {
        self.previous_block = None;
        self.phi_cache.clear();
    }
    
    /// Phi命令を実行
    pub fn execute_phi(
        &mut self,
        dst: ValueId,
        inputs: &[(BasicBlockId, ValueId)],
        get_value_fn: impl Fn(ValueId) -> Result<VMValue, VMError>,
    ) -> Result<VMValue, VMError> {
        // キャッシュは使わない - Phi nodeは毎回新しい値を計算する必要がある
        // if let Some(cached) = self.phi_cache.get(&dst) {
        //     return Ok(cached.clone());
        // }
        
        // Phi nodeの入力を選択
        let selected_value = self.select_phi_input(inputs, get_value_fn)?;
        
        // キャッシュに保存（デバッグ用に残すが使わない）
        // self.phi_cache.insert(dst, selected_value.clone());
        
        Ok(selected_value)
    }
    
    /// Phi nodeの適切な入力を選択
    fn select_phi_input(
        &self,
        inputs: &[(BasicBlockId, ValueId)],
        get_value_fn: impl Fn(ValueId) -> Result<VMValue, VMError>,
    ) -> Result<VMValue, VMError> {
        if inputs.is_empty() {
            return Err(VMError::InvalidInstruction("Phi node has no inputs".to_string()));
        }
        
        // previous_blockに基づいて入力を選択
        if let Some(prev_block) = self.previous_block {
            // 対応するブロックからの入力を探す
            for (block_id, value_id) in inputs {
                if *block_id == prev_block {
                    let value = get_value_fn(*value_id)?;
                    return Ok(value);
                }
            }
            
            // フォールバック：見つからない場合は最初の入力を使用
            // これは通常起こらないはずだが、安全のため
        }
        
        // previous_blockがない場合（エントリポイント）は最初の入力を使用
        let (_, value_id) = &inputs[0];
        get_value_fn(*value_id)
    }
    
    /// ループヘッダーかどうかを判定（簡易版）
    fn is_loop_header(&self, _block_id: BasicBlockId) -> bool {
        // TODO: MIR情報からループヘッダーを判定する機能を追加
        // 現在は常にfalse（キャッシュクリアしない）
        false
    }
}

/// ループ実行ヘルパー - ループ特有の処理を管理
pub struct LoopExecutor {
    /// Phiハンドラー
    phi_handler: PhiHandler,
    
    /// ループイテレーション数（デバッグ用）
    iteration_count: HashMap<BasicBlockId, usize>,
}

impl LoopExecutor {
    /// 新しいループ実行ヘルパーを作成
    pub fn new() -> Self {
        Self {
            phi_handler: PhiHandler::new(),
            iteration_count: HashMap::new(),
        }
    }
    
    /// ブロック遷移を記録
    pub fn record_transition(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.phi_handler.record_block_transition(from, to);
        
        // ループイテレーション数を更新（デバッグ用）
        if from > to {  // 単純なバックエッジ検出
            *self.iteration_count.entry(to).or_insert(0) += 1;
        }
    }
    
    /// エントリポイントでの初期化
    pub fn initialize(&mut self) {
        self.phi_handler.record_entry();
        self.iteration_count.clear();
    }
    
    /// Phi命令を実行
    pub fn execute_phi(
        &mut self,
        dst: ValueId,
        inputs: &[(BasicBlockId, ValueId)],
        get_value_fn: impl Fn(ValueId) -> Result<VMValue, VMError>,
    ) -> Result<VMValue, VMError> {
        self.phi_handler.execute_phi(dst, inputs, get_value_fn)
    }
    
    /// デバッグ情報を取得
    pub fn debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str("Loop Executor Debug Info:\n");
        
        if let Some(prev) = self.phi_handler.previous_block {
            info.push_str(&format!("  Previous block: {:?}\n", prev));
        } else {
            info.push_str("  Previous block: None (entry)\n");
        }
        
        if !self.iteration_count.is_empty() {
            info.push_str("  Loop iterations:\n");
            for (block, count) in &self.iteration_count {
                info.push_str(&format!("    Block {:?}: {} iterations\n", block, count));
            }
        }
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phi_selection() {
        let mut handler = PhiHandler::new();
        
        // テスト用の値
        let inputs = vec![
            (BasicBlockId::new(0), ValueId::new(1)),  // エントリブロックからの初期値
            (BasicBlockId::new(2), ValueId::new(2)),  // ループボディからの更新値
        ];
        
        // エントリポイントからの実行
        handler.record_entry();
        let result = handler.execute_phi(
            ValueId::new(3),
            &inputs,
            |id| {
                if id == ValueId::new(1) {
                    Ok(VMValue::Integer(0))
                } else {
                    Ok(VMValue::Integer(10))
                }
            }
        );
        assert_eq!(result.unwrap(), VMValue::Integer(0));
        
        // ループボディからの実行
        handler.record_block_transition(BasicBlockId::new(2), BasicBlockId::new(1));
        handler.phi_cache.clear();  // テスト用にキャッシュクリア
        let result = handler.execute_phi(
            ValueId::new(3),
            &inputs,
            |id| {
                if id == ValueId::new(1) {
                    Ok(VMValue::Integer(0))
                } else {
                    Ok(VMValue::Integer(10))
                }
            }
        );
        assert_eq!(result.unwrap(), VMValue::Integer(10));
    }
}