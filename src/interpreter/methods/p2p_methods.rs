/*! 📡 P2P通信メソッド実装 (NEW ARCHITECTURE)
 * IntentBoxとP2PBoxのNyashインタープリター統合
 * Arc<Mutex>パターン対応版
 */

use crate::interpreter::NyashInterpreter;
use crate::interpreter::RuntimeError;
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::{IntentBox, P2PBox};
use crate::box_trait::BoolBox;

impl NyashInterpreter {
    /// IntentBoxのメソッド実行 (RwLock版)
    pub(in crate::interpreter) fn execute_intent_box_method(
        &mut self,
        intent_box: &IntentBox,
        method: &str,
        _arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {        
        match method {
            // メッセージ名取得
            "getName" | "name" => {
                Ok(intent_box.get_name())
            }
            
            // ペイロード取得（JSON文字列として）
            "getPayload" | "payload" => {
                Ok(intent_box.get_payload())
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
    
    // P2PBoxのメソッド実装（RwLockベース）
    pub(in crate::interpreter) fn execute_p2p_box_method(
        &mut self,
        p2p_box: &P2PBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        if crate::interpreter::utils::debug_on() || std::env::var("NYASH_DEBUG_P2P").unwrap_or_default() == "1" {
            eprintln!("[Interp:P2P] {}(..) called with {} args", method, arguments.len());
        }
        match method {
            // ノードID取得
            "getNodeId" | "getId" => Ok(p2p_box.get_node_id()),

            // トランスポート種類取得
            "getTransportType" | "transport" => Ok(p2p_box.get_transport_type()),

            // ノード到達可能性確認
            "isReachable" => {
                if arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: "isReachable requires node_id argument".to_string() });
                }
                let node_id_result = self.execute_expression(&arguments[0])?;
                Ok(p2p_box.is_reachable(node_id_result))
            }

            // send メソッド実装（ResultBox返却）
            "send" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidOperation { message: "send requires (to, intent) arguments".to_string() });
                }
                let to_result = self.execute_expression(&arguments[0])?;
                let intent_result = self.execute_expression(&arguments[1])?;
                Ok(p2p_box.send(to_result, intent_result))
            }

            // ping: health check using sys.ping/sys.pong
            "ping" => {
                if arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: "ping requires (to [, timeout_ms]) arguments".to_string() });
                }
                let to_result = self.execute_expression(&arguments[0])?;
                if arguments.len() >= 2 {
                    let tmo_val = self.execute_expression(&arguments[1])?;
                    let tmo_ms = tmo_val.to_string_box().value.parse::<u64>().unwrap_or(200);
                    Ok(p2p_box.ping_with_timeout(to_result, tmo_ms))
                } else {
                    Ok(p2p_box.ping(to_result))
                }
            }

            // on メソッド実装（ResultBox返却）
            "on" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidOperation { message: "on requires (intentName, handler) arguments".to_string() });
                }
                let name_val = self.execute_expression(&arguments[0])?;
                let handler_val = self.execute_expression(&arguments[1])?;
                Ok(p2p_box.on(name_val, handler_val))
            }

            // 最後の受信情報（ループバック検証用）
            "getLastFrom" => Ok(p2p_box.get_last_from()),
            "getLastIntentName" => Ok(p2p_box.get_last_intent_name()),
            "debug_nodes" | "debugNodes" => Ok(p2p_box.debug_nodes()),
            "debug_bus_id" | "debugBusId" => Ok(p2p_box.debug_bus_id()),

            // onOnce / off
            "onOnce" | "on_once" => {
                if arguments.len() < 2 {
                    return Err(RuntimeError::InvalidOperation { message: "onOnce requires (intentName, handler) arguments".to_string() });
                }
                let name_val = self.execute_expression(&arguments[0])?;
                let handler_val = self.execute_expression(&arguments[1])?;
                Ok(p2p_box.on_once(name_val, handler_val))
            }
            "off" => {
                if arguments.len() < 1 {
                    return Err(RuntimeError::InvalidOperation { message: "off requires (intentName) argument".to_string() });
                }
                let name_val = self.execute_expression(&arguments[0])?;
                Ok(p2p_box.off(name_val))
            }

            _ => Err(RuntimeError::UndefinedVariable { name: format!("P2PBox method '{}' not found", method) }),
        }
    }
}
