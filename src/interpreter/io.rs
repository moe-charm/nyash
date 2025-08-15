/*!
 * I/O Processing Module
 * 
 * Extracted from core.rs - file operations and communication
 * Handles include system, arrow operators, and I/O-related operations
 * Core philosophy: "Everything is Box" with secure I/O processing
 */

use super::*;
use crate::parser::NyashParser;

impl NyashInterpreter {
    /// include文を実行：ファイル読み込み・パース・実行 - File inclusion system
    pub(super) fn execute_include(&mut self, filename: &str) -> Result<(), RuntimeError> {
        // パス正規化（簡易版）
        let canonical_path = if filename.starts_with("./") || filename.starts_with("../") {
            filename.to_string()
        } else {
            format!("./{}", filename)
        };
        
        // 重複読み込みチェック
        if self.shared.included_files.lock().unwrap().contains(&canonical_path) {
            return Ok(()); // 既に読み込み済み
        }
        
        // ファイル読み込み
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Failed to read file '{}': {}", filename, e),
            })?;
        
        // パース
        let ast = NyashParser::parse_from_string(&content)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Parse error in '{}': {:?}", filename, e),
            })?;
        
        // 重複防止リストに追加
        self.shared.included_files.lock().unwrap().insert(canonical_path);
        
        // 現在の環境で実行
        self.execute(ast)?;
        
        Ok(())
    }
    
    /// Arrow演算子を実行: sender >> receiver - Channel communication
    pub(super) fn execute_arrow(&mut self, sender: &ASTNode, receiver: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 送信者を評価
        let sender_value = self.execute_expression(sender)?;
        
        // 受信者を評価
        let receiver_str = match receiver {
            ASTNode::Variable { name, .. } => name.clone(),
            ASTNode::Literal { value, .. } => {
                // "*" のようなリテラルの場合
                value.to_string()
            }
            _ => {
                // その他の式の場合は評価して文字列化
                let receiver_value = self.execute_expression(receiver)?;
                receiver_value.to_string_box().value
            }
        };
        
        // 送信者の名前を取得
        let sender_name = sender_value.to_string_box().value;
        
        // ChannelBoxを作成して返す
        let channel_box = Box::new(ChannelBox::new(&sender_name, &receiver_str)) as Box<dyn NyashBox>;
        // 🌍 革命的実装：Environment tracking廃止
        Ok(channel_box)
    }
    
    /// nowait文を実行 - 非同期実行（真の非同期実装） - Async execution
    pub(super) fn execute_nowait(&mut self, variable: &str, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        use crate::boxes::FutureBox;
        use std::thread;
        
        // FutureBoxを作成
        let future_box = FutureBox::new();
        let future_box_clone = future_box.clone();
        
        // 式をクローンして別スレッドで実行
        let expr_clone = expression.clone();
        let shared_state = self.shared.clone();
        
        // 別スレッドで非同期実行
        thread::spawn(move || {
            // 新しいインタープリタインスタンスを作成（SharedStateを使用）
            let mut async_interpreter = NyashInterpreter::with_shared(shared_state);
            
            // 式を評価
            match async_interpreter.execute_expression(&expr_clone) {
                Ok(result) => {
                    future_box_clone.set_result(result);
                }
                Err(e) => {
                    // エラーをErrorBoxとして設定
                    let error_box = Box::new(ErrorBox::new("RuntimeError", &format!("{:?}", e)));
                    future_box_clone.set_result(error_box);
                }
            }
        });
        
        // FutureBoxを変数に保存
        let future_box_instance = Box::new(future_box) as Box<dyn NyashBox>;
        self.set_variable(variable, future_box_instance)?;
        
        Ok(Box::new(VoidBox::new()))
    }
}