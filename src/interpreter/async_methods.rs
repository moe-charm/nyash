/*!
 * Async Methods Module
 * 
 * Extracted from interpreter/box_methods.rs
 * Contains asynchronous Box type method implementations:
 * 
 * - execute_future_method (FutureBox)
 * - execute_channel_method (ChannelBox)  
 * 
 * These methods handle asynchronous operations, futures, and
 * communication channels in the Nyash interpreter.
 */

use super::*;
use crate::box_trait::StringBox;
use crate::channel_box::{ChannelBox, MessageBox};

impl NyashInterpreter {
    /// FutureBoxのメソッド呼び出しを実行
    /// 
    /// 非同期計算の結果を管理するFutureBoxの基本操作を提供します。
    /// 
    /// サポートメソッド:
    /// - get() -> 計算結果を取得 (ブロッキング)
    /// - ready() -> 計算完了状態をチェック
    /// - equals(other) -> 他のFutureBoxと比較
    pub(super) fn execute_future_method(&mut self, future_box: &FutureBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "get" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(future_box.get())
            }
            "ready" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ready() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(future_box.ready())
            }
            "equals" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other = self.execute_expression(&arguments[0])?;
                Ok(Box::new(future_box.equals(other.as_ref())))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for FutureBox", method),
            })
        }
    }

    /// ChannelBoxのメソッド呼び出しを実行
    /// 
    /// 非同期通信チャンネルを管理するChannelBoxの操作を提供します。
    /// プロセス間通信やイベント駆動プログラミングに使用されます。
    /// 
    /// サポートメソッド:
    /// - sendMessage(content) -> メッセージを送信
    /// - announce(content) -> ブロードキャスト送信  
    /// - toString() -> チャンネル情報を文字列化
    /// - sender() -> 送信者情報を取得
    /// - receiver() -> 受信者情報を取得
    pub(super) fn execute_channel_method(&mut self, channel_box: &ChannelBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "sendMessage" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sendMessage() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                // 簡易実装：メッセージを作成して返す
                let content = arg_values[0].to_string_box().value;
                let msg = MessageBox::new(&channel_box.sender_name, &content);
                Ok(Box::new(msg))
            }
            "announce" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("announce() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let content = arg_values[0].to_string_box().value;
                Ok(Box::new(StringBox::new(&format!("Broadcast from {}: {}", channel_box.sender_name, content))))
            }
            "toString" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(channel_box.to_string_box()))
            }
            "sender" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sender() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(channel_box.sender())
            }
            "receiver" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("receiver() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(channel_box.receiver())
            }
            _ => {
                // その他のメソッドはChannelBoxに委譲
                Ok(channel_box.invoke(method, arg_values))
            }
        }
    }
}