/*! 📡 P2P通信メソッド実装 (NEW ARCHITECTURE)
 * IntentBoxとP2PBoxのNyashインタープリター統合
 * Arc<Mutex>パターン対応版
 */

use crate::interpreter::core::NyashInterpreter;
use crate::interpreter::core::RuntimeError;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::boxes::{IntentBox, P2PBox};
use crate::method_box::MethodBox;

impl NyashInterpreter {
    /// IntentBoxのメソッド実行 (Arc<Mutex>版)
    pub(in crate::interpreter) fn execute_intent_box_method(
        &mut self,
        intent_box: &IntentBox,
        method: &str,
        _arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let data = intent_box.lock().map_err(|_| RuntimeError::UndefinedVariable {
            name: "Failed to lock IntentBox".to_string(),
        })?;
        
        match method {
            // メッセージ名取得
            "getName" | "name" => {
                Ok(Box::new(StringBox::new(data.name.clone())))
            }
            
            // ペイロード取得（JSON文字列として）
            "getPayload" | "payload" => {
                let payload_str = data.payload.to_string();
                Ok(Box::new(StringBox::new(payload_str)))
            }
            
            // 型情報取得
            "getType" | "type" => {
                Ok(Box::new(StringBox::new("IntentBox")))
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("IntentBox method '{}' not found", method),
            })
        }
    }
    
    /// P2PBoxのメソッド実行 (Arc<Mutex>版)
    pub(in crate::interpreter) fn execute_p2p_box_method(
        &mut self,
        p2p_box: &P2PBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let data = p2p_box.lock().map_err(|_| RuntimeError::UndefinedVariable {
            name: "Failed to lock P2PBox".to_string(),
        })?;
        
        match method {
            // ノードID取得
            "getNodeId" | "getId" => {
                Ok(Box::new(StringBox::new(data.get_node_id().to_string())))
            }
            
            // トランスポート種類取得
            "getTransportType" | "transport" => {
                Ok(Box::new(StringBox::new(data.get_transport_type())))
            }
            
            // ノード到達可能性確認
            "isReachable" => {
                if arguments.is_empty() {
                    return Err(RuntimeError::UndefinedVariable {
                        name: "isReachable requires node_id argument".to_string(),
                    });
                }
                
                let node_id_result = self.execute_expression(&arguments[0])?;
                let node_id = node_id_result.to_string_box().value;
                let reachable = data.is_reachable(&node_id);
                Ok(Box::new(BoolBox::new(reachable)))
            }
            
            // send メソッド実装
            "send" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::UndefinedVariable {
                        name: "send requires (to, intent) arguments".to_string(),
                    });
                }
                
                let to_result = self.execute_expression(&arguments[0])?;
                let to = to_result.to_string_box().value;
                
                let intent_result = self.execute_expression(&arguments[1])?;
                
                // IntentBoxかチェック
                if let Some(intent_box) = intent_result.as_any().downcast_ref::<IntentBox>() {
                    match data.send(&to, intent_box.clone()) {
                        Ok(_) => Ok(Box::new(StringBox::new("sent"))),
                        Err(e) => Err(RuntimeError::UndefinedVariable {
                            name: format!("Send failed: {:?}", e),
                        })
                    }
                } else {
                    Err(RuntimeError::UndefinedVariable {
                        name: "Second argument must be an IntentBox".to_string(),
                    })
                }
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("P2PBox method '{}' not found", method),
            })
        }
    }
}