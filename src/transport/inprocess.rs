/*! 🏠 InProcessTransport - Local Process Communication
 * 
 * ## 📝 概要
 * InProcessTransportは、同一プロセス内でのP2P通信を実装します。
 * MessageBusを使用して高速なローカルメッセージ配送を行います。
 * 
 * ## 🏗️ 設計
 * - **MessageBus Integration**: グローバルMessageBusを使用
 * - **Zero-Copy**: プロセス内での直接参照渡し
 * - **Event-Driven**: コールバックベースの受信処理
 * - **Thread-Safe**: 並行アクセス対応
 */

use super::{Transport, IntentEnvelope, SendOpts, TransportError};
use crate::messaging::{MessageBus, MessageBusData, BusEndpoint, SendError, IntentHandler};
use crate::boxes::IntentBox;
use std::sync::{Arc, Mutex};

/// InProcessTransport - プロセス内通信実装
pub struct InProcessTransport {
    node_id: String,
    bus: MessageBus,
    endpoint: BusEndpoint,
    receive_callback: Arc<Mutex<Option<Box<dyn Fn(IntentEnvelope) + Send + Sync>>>>,
}

impl std::fmt::Debug for InProcessTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InProcessTransport")
            .field("node_id", &self.node_id)
            .field("bus", &"MessageBus")
            .field("endpoint", &"BusEndpoint")
            .field("receive_callback", &"<callback>")
            .finish()
    }
}

impl InProcessTransport {
    /// 新しいInProcessTransportを作成
    pub fn new(node_id: String) -> Self {
        let bus = MessageBusData::global();
        let endpoint = BusEndpoint::new(node_id.clone());
        
        // ノードをバスに登録
        {
            let mut bus_data = bus.lock().unwrap();
            bus_data.register_node(node_id.clone(), endpoint.clone());
        }
        
        InProcessTransport {
            node_id,
            bus,
            endpoint,
            receive_callback: Arc::new(Mutex::new(None)),
        }
    }
    
    /// イベントハンドラーを追加
    pub fn add_handler(&self, intent_name: &str, handler: IntentHandler) {
        self.endpoint.add_handler(intent_name, handler);
    }
}

impl Transport for InProcessTransport {
    fn node_id(&self) -> &str {
        &self.node_id
    }
    
    fn send(&self, to: &str, intent: IntentBox, _opts: SendOpts) -> Result<(), TransportError> {
        let bus = self.bus.lock().unwrap();
        
        match bus.route(to, intent.clone(), &self.node_id) {
            Ok(_) => {
                // 受信コールバックがある場合は実行
                if let Some(callback) = self.receive_callback.lock().unwrap().as_ref() {
                    let envelope = IntentEnvelope {
                        from: self.node_id.clone(),
                        to: to.to_string(),
                        intent,
                        timestamp: std::time::Instant::now(),
                    };
                    callback(envelope);
                }
                Ok(())
            }
            Err(SendError::NodeNotFound(msg)) => Err(TransportError::NodeNotFound(msg)),
            Err(SendError::MessageDeliveryFailed(msg)) => Err(TransportError::NetworkError(msg)),
            Err(SendError::InvalidMessage(msg)) => Err(TransportError::SerializationError(msg)),
            Err(SendError::BusError(msg)) => Err(TransportError::NetworkError(msg)),
        }
    }
    
    fn on_receive(&mut self, callback: Box<dyn Fn(IntentEnvelope) + Send + Sync>) {
        let mut receive_callback = self.receive_callback.lock().unwrap();
        *receive_callback = Some(callback);
    }
    
    fn is_reachable(&self, node_id: &str) -> bool {
        let bus = self.bus.lock().unwrap();
        bus.node_exists(node_id)
    }
    
    fn transport_type(&self) -> &'static str {
        "inprocess"
    }
}

impl Drop for InProcessTransport {
    fn drop(&mut self) {
        // ノードをバスから解除
        let mut bus = self.bus.lock().unwrap();
        bus.unregister_node(&self.node_id);
    }
}