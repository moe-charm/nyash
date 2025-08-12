/*! 🚌 MessageBus - Process-wide Message Routing Singleton
 * 
 * ## 📝 概要
 * MessageBusは、プロセス内でのメッセージルーティングを管理する
 * シングルトンコンポーネントです。すべてのP2PBoxノードが共有し、
 * ローカル通信の高速配送を実現します。
 * 
 * ## 🏗️ 設計
 * - **Singleton Pattern**: プロセス内で唯一のインスタンス
 * - **Node Registry**: 登録されたノードの管理
 * - **Handler Management**: イベントハンドラーの管理
 * - **Async Safe**: Arc<Mutex>による並行アクセス対応
 * 
 * ## 🚀 機能
 * - ノードの登録・解除
 * - メッセージルーティング
 * - イベントハンドラー管理
 * - エラーハンドリング
 */

use crate::boxes::IntentBox;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Intent処理ハンドラーの型
pub type IntentHandler = Box<dyn Fn(IntentBox, &str) + Send + Sync>;

/// バスエンドポイント - ノードの通信インターフェース
#[derive(Clone)]
pub struct BusEndpoint {
    pub node_id: String,
    pub handlers: Arc<Mutex<HashMap<String, Vec<IntentHandler>>>>,
}

impl BusEndpoint {
    pub fn new(node_id: String) -> Self {
        BusEndpoint {
            node_id,
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// イベントハンドラーを追加
    pub fn add_handler(&self, intent_name: &str, handler: IntentHandler) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.entry(intent_name.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }
    
    /// メッセージを配送
    pub fn deliver(&self, intent: IntentBox, from: &str) {
        let handlers = self.handlers.lock().unwrap();
        let intent_data = intent.lock().unwrap();
        let intent_name = &intent_data.name;
        
        if let Some(intent_handlers) = handlers.get(intent_name) {
            for handler in intent_handlers {
                handler(intent.clone(), from);
            }
        }
    }
}

/// MessageBus送信エラー
#[derive(Debug, Clone)]
pub enum SendError {
    NodeNotFound(String),
    MessageDeliveryFailed(String),
    InvalidMessage(String),
    BusError(String),
}

/// MessageBus内部データ
pub struct MessageBusData {
    /// 登録されたノード一覧
    nodes: HashMap<String, BusEndpoint>,
}

impl std::fmt::Debug for MessageBusData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBusData")
            .field("nodes", &format!("{} nodes", self.nodes.len()))
            .finish()
    }
}

/// MessageBus - プロセス内シングルトン
pub type MessageBus = Arc<Mutex<MessageBusData>>;

impl MessageBusData {
    /// 新しいMessageBusDataを作成
    fn new() -> Self {
        MessageBusData {
            nodes: HashMap::new(),
        }
    }
    
    /// ノードを登録
    pub fn register_node(&mut self, id: String, endpoint: BusEndpoint) {
        self.nodes.insert(id, endpoint);
    }
    
    /// ノードを解除
    pub fn unregister_node(&mut self, id: &str) -> bool {
        self.nodes.remove(id).is_some()
    }
    
    /// ノードが存在するかチェック
    pub fn node_exists(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }
    
    /// メッセージをルーティング
    pub fn route(&self, to: &str, intent: IntentBox, from: &str) -> Result<(), SendError> {
        if let Some(endpoint) = self.nodes.get(to) {
            endpoint.deliver(intent, from);
            Ok(())
        } else {
            Err(SendError::NodeNotFound(format!("Node '{}' not found", to)))
        }
    }
    
    /// 登録されたノード一覧を取得
    pub fn get_nodes(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
}

/// グローバルMessageBusシングルトン
static GLOBAL_MESSAGE_BUS: Lazy<MessageBus> = Lazy::new(|| {
    Arc::new(Mutex::new(MessageBusData::new()))
});

impl MessageBusData {
    /// グローバルMessageBusへのアクセス
    pub fn global() -> MessageBus {
        GLOBAL_MESSAGE_BUS.clone()
    }
}