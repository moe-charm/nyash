/*! 📡 P2P通信メソッド実装
 * IntentBoxとP2PBoxのNyashインタープリター統合
 */

use crate::interpreter::core::NyashInterpreter;
use crate::interpreter::core::RuntimeError;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::{IntentBox, P2PBox, NewP2PBox, MessageIntentBox};
use crate::method_box::MethodBox;

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
    
    /// NewP2PBoxのメソッド実行（天才アルゴリズム版）
    pub(in crate::interpreter) fn execute_new_p2p_box_method(
        &mut self,
        p2p_box: &NewP2PBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            // ノードID取得
            "getNodeId" | "getId" => {
                Ok(Box::new(StringBox::new(p2p_box.get_node_id())))
            }
            
            // メッセージ送信（天才アルゴリズム）
            "send" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "send requires 2 arguments: target, message_intent_box".to_string(),
                    });
                }
                
                let target = self.execute_expression(&arguments[0])?;
                let message_box = self.execute_expression(&arguments[1])?;
                
                if let Some(target_str) = target.as_any().downcast_ref::<StringBox>() {
                    if let Some(intent_box) = message_box.as_any().downcast_ref::<MessageIntentBox>() {
                        match p2p_box.send(&target_str.value, intent_box) {
                            Ok(()) => Ok(Box::new(StringBox::new("sent"))),
                            Err(e) => Err(RuntimeError::InvalidOperation { message: e }),
                        }
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "send requires MessageIntentBox as second argument".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "send requires string target as first argument".to_string(),
                    })
                }
            }
            
            // リスナー登録（MethodBox版）
            "onMethod" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "onMethod requires 2 arguments: intent, method_box".to_string(),
                    });
                }
                
                let intent = self.execute_expression(&arguments[0])?;
                let method_box = self.execute_expression(&arguments[1])?;
                
                if let Some(intent_str) = intent.as_any().downcast_ref::<StringBox>() {
                    if let Some(method_box) = method_box.as_any().downcast_ref::<MethodBox>() {
                        match p2p_box.on_method(&intent_str.value, method_box.clone()) {
                            Ok(()) => Ok(Box::new(StringBox::new("listener registered"))),
                            Err(e) => Err(RuntimeError::InvalidOperation { message: e }),
                        }
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "onMethod requires MethodBox as second argument".to_string(),
                        })
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "onMethod requires string intent as first argument".to_string(),
                    })
                }
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("NewP2PBox method '{}' not found", method),
            })
        }
    }
    
    /// MessageIntentBoxのメソッド実行
    pub(in crate::interpreter) fn execute_message_intent_box_method(
        &mut self,
        message_box: &mut MessageIntentBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            // intent取得
            "getIntent" | "intent" => {
                Ok(Box::new(StringBox::new(&message_box.intent)))
            }
            
            // データ設定
            "set" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "set requires 2 arguments: key, value".to_string(),
                    });
                }
                
                let key = self.execute_expression(&arguments[0])?;
                let value = self.execute_expression(&arguments[1])?;
                
                if let Some(key_str) = key.as_any().downcast_ref::<StringBox>() {
                    message_box.set(&key_str.value, value);
                    Ok(Box::new(StringBox::new("set")))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "set requires string key as first argument".to_string(),
                    })
                }
            }
            
            // データ取得
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: "get requires 1 argument: key".to_string(),
                    });
                }
                
                let key = self.execute_expression(&arguments[0])?;
                if let Some(key_str) = key.as_any().downcast_ref::<StringBox>() {
                    if let Some(value) = message_box.get(&key_str.value) {
                        Ok(value.clone_box())
                    } else {
                        Ok(Box::new(crate::boxes::NullBox::new()))
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "get requires string key as argument".to_string(),
                    })
                }
            }
            
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("MessageIntentBox method '{}' not found", method),
            })
        }
    }
}