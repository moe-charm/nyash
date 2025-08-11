/*! 🌐 IntentBox - 通信世界を定義するBox
 * 
 * ## 📝 概要
 * IntentBoxは「通信世界」を定義する中心的なコンポーネントです。
 * P2PBoxノードが参加する通信環境を抽象化し、
 * プロセス内通信、WebSocket、共有メモリなど
 * 様々な通信方式を統一的に扱います。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `new()` - デフォルト（ローカル）通信世界を作成
 * - `new_with_transport(transport)` - カスタム通信方式で作成
 * - `register_node(node)` - P2PBoxノードを登録
 * - `unregister_node(node_id)` - ノードを登録解除
 * - `get_transport()` - 通信トランスポートを取得
 * 
 * ## 💡 使用例
 * ```nyash
 * // ローカル通信世界
 * local_world = new IntentBox()
 * 
 * // WebSocket通信世界（将来）
 * remote_world = new IntentBox(websocket, {
 *     "url": "ws://example.com/api"
 * })
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::fmt::{self, Debug};

/// 通信方式を抽象化するトレイト
pub trait Transport: Send + Sync {
    /// 特定のノードにメッセージを送信
    fn send(&self, from: &str, to: &str, intent: &str, data: Box<dyn NyashBox>);
    
    /// 全ノードにメッセージをブロードキャスト
    fn broadcast(&self, from: &str, intent: &str, data: Box<dyn NyashBox>);
    
    /// トランスポートの種類を取得
    fn transport_type(&self) -> &str;
}

/// ローカル（プロセス内）通信を実装
pub struct LocalTransport {
    /// メッセージキュー
    message_queue: Arc<Mutex<Vec<Message>>>,
}

/// メッセージ構造体
pub struct Message {
    pub from: String,
    pub to: Option<String>,  // Noneの場合はブロードキャスト
    pub intent: String,
    pub data: Box<dyn NyashBox>,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            from: self.from.clone(),
            to: self.to.clone(),
            intent: self.intent.clone(),
            data: self.data.clone_box(),
        }
    }
}

impl LocalTransport {
    pub fn new() -> Self {
        LocalTransport {
            message_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// メッセージをキューに追加
    pub fn enqueue_message(&self, msg: Message) {
        let mut queue = self.message_queue.lock().unwrap();
        queue.push(msg);
    }
    
    /// キューからメッセージを取得
    pub fn dequeue_messages(&self) -> Vec<Message> {
        let mut queue = self.message_queue.lock().unwrap();
        let messages = queue.drain(..).collect();
        messages
    }
}

impl Transport for LocalTransport {
    fn send(&self, from: &str, to: &str, intent: &str, data: Box<dyn NyashBox>) {
        let msg = Message {
            from: from.to_string(),
            to: Some(to.to_string()),
            intent: intent.to_string(),
            data,
        };
        
        // メッセージをキューに追加
        self.enqueue_message(msg);
    }
    
    fn broadcast(&self, from: &str, intent: &str, data: Box<dyn NyashBox>) {
        let msg = Message {
            from: from.to_string(),
            to: None,
            intent: intent.to_string(),
            data,
        };
        
        // メッセージをキューに追加
        self.enqueue_message(msg);
    }
    
    fn transport_type(&self) -> &str {
        "local"
    }
}

/// IntentBox - 通信世界を定義
#[derive(Clone)]
pub struct IntentBox {
    base: BoxBase,
    transport: Arc<Mutex<Box<dyn Transport>>>,
}

impl Debug for IntentBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntentBox")
            .field("id", &self.base.id)
            .field("transport", &"<Transport>")
            .finish()
    }
}

impl IntentBox {
    /// デフォルト（ローカル）通信世界を作成
    pub fn new() -> Self {
        IntentBox {
            base: BoxBase::new(),
            transport: Arc::new(Mutex::new(Box::new(LocalTransport::new()))),
        }
    }
    
    /// カスタムトランスポートで通信世界を作成
    pub fn new_with_transport(transport: Box<dyn Transport>) -> Self {
        IntentBox {
            base: BoxBase::new(),
            transport: Arc::new(Mutex::new(transport)),
        }
    }
    
    /// メッセージを処理（LocalTransport専用）
    pub fn process_messages(&self) -> Vec<Message> {
        let _transport = self.transport.lock().unwrap();
        // TransportをAnyにキャストしてLocalTransportかチェック
        // 現在はLocalTransportのみサポート
        Vec::new()  // TODO: 実装
    }
    
    /// トランスポートへのアクセス（P2PBoxから使用）
    pub fn get_transport(&self) -> Arc<Mutex<Box<dyn Transport>>> {
        self.transport.clone()
    }
}

impl NyashBox for IntentBox {
    fn to_string_box(&self) -> StringBox {
        let transport = self.transport.lock().unwrap();
        StringBox::new(format!("IntentBox[{}]", transport.transport_type()))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_intent) = other.as_any().downcast_ref::<IntentBox>() {
            BoolBox::new(self.base.id == other_intent.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "IntentBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    
}

impl BoxCore for IntentBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let transport = self.transport.lock().unwrap();
        write!(f, "IntentBox[{}]", transport.transport_type())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for IntentBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

