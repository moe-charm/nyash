/*! 📡 P2P通信メソッド実装
 * IntentBoxとP2PBoxのNyashインタープリター統合
 */

use crate::interpreter::core::NyashInterpreter;
use crate::interpreter::core::RuntimeError;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::{IntentBox, P2PBox};

impl NyashInterpreter {
    /// IntentBoxのメソッド実行
    pub(in crate::interpreter) fn execute_intent_box_method(
        &mut self,
        intent_box: &IntentBox,
        method: &str,
        _arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            // 基本情報取得
            "getType" | "type" => {
                Ok(Box::new(StringBox::new("IntentBox")))
            }
            
            // メッセージ処理（テスト用）
            "processMessages" => {
                let messages = intent_box.process_messages();
                Ok(Box::new(StringBox::new(format!("Processed {} messages", messages.len()))))
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("IntentBox method '{}' not found", method),
            })
        }
    }
    
    /// P2PBoxのメソッド実行
    pub(in crate::interpreter) fn execute_p2p_box_method(
        &mut self,
        p2p_box: &P2PBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            // ノードID取得
            "getNodeId" | "getId" => {
                Ok(Box::new(StringBox::new(p2p_box.get_node_id())))
            }
            
            // メッセージ送信
            "send" => {
                if arguments.len() < 3 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "send requires 3 arguments: intent, data, target".to_string(),
                    });
                }
                
                let intent = self.execute_expression(&arguments[0])?;
                let data = self.execute_expression(&arguments[1])?;
                let target = self.execute_expression(&arguments[2])?;
                
                if let Some(intent_str) = intent.as_any().downcast_ref::<StringBox>() {
                    if let Some(target_str) = target.as_any().downcast_ref::<StringBox>() {
                        return Ok(p2p_box.send(&intent_str.value, data, &target_str.value));
                    }
                }
                
                Err(RuntimeError::TypeError {
                    message: "send requires string arguments for intent and target".to_string(),
                })
            }
            
            // ブロードキャスト
            "broadcast" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "broadcast requires 2 arguments: intent, data".to_string(),
                    });
                }
                
                let intent = self.execute_expression(&arguments[0])?;
                let data = self.execute_expression(&arguments[1])?;
                
                if let Some(intent_str) = intent.as_any().downcast_ref::<StringBox>() {
                    return Ok(p2p_box.broadcast(&intent_str.value, data));
                }
                
                Err(RuntimeError::TypeError {
                    message: "broadcast requires string argument for intent".to_string(),
                })
            }
            
            // リスナー登録
            "on" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "on requires 2 arguments: intent, callback".to_string(),
                    });
                }
                
                let intent = self.execute_expression(&arguments[0])?;
                let callback = self.execute_expression(&arguments[1])?;
                
                if let Some(intent_str) = intent.as_any().downcast_ref::<StringBox>() {
                    return Ok(p2p_box.on(&intent_str.value, callback));
                }
                
                Err(RuntimeError::TypeError {
                    message: "on requires string argument for intent".to_string(),
                })
            }
            
            // リスナー解除
            "off" => {
                if arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "off requires 1 argument: intent".to_string(),
                    });
                }
                
                let intent = self.execute_expression(&arguments[0])?;
                
                if let Some(intent_str) = intent.as_any().downcast_ref::<StringBox>() {
                    return Ok(p2p_box.off(&intent_str.value));
                }
                
                Err(RuntimeError::TypeError {
                    message: "off requires string argument for intent".to_string(),
                })
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("P2PBox method '{}' not found", method),
            })
        }
    }
}