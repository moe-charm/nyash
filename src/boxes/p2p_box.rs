/*! 📡 P2PBox - Modern P2P Communication Node
 * 
 * ## 📝 概要
 * P2PBoxは現代的なP2P通信ノードを表現するBoxです。
 * 新しいアーキテクチャ（IntentBox + MessageBus + Transport）を使用し、
 * 構造化メッセージによる安全で明示的な通信を実現します。
 * 
 * ## 🎯 AI大会議決定事項準拠
 * - **個別送信のみ**: `send(to, message)` 固定API
 * - **ブロードキャスト除外**: 安全性のため完全除外
 * - **明示的API**: 関数オーバーロード不採用
 * - **構造化メッセージ**: IntentBox (name + payload) 使用
 * 
 * ## 🛠️ 利用可能メソッド
 * - `new(node_id, transport)` - ノードを作成
 * - `send(to, intent)` - 特定ノードにメッセージ送信
 * - `on(intent_name, handler)` - イベントリスナー登録
 * - `getNodeId()` - ノードID取得
 * - `isReachable(node_id)` - ノード到達可能性確認
 * 
 * ## 💡 使用例
 * ```nyash
 * // ノード作成
 * local alice = new P2PBox("alice", "inprocess")
 * local bob = new P2PBox("bob", "inprocess")
 * 
 * // 受信ハンドラ登録
 * bob.on("chat.message", function(intent, from) {
 *     print("From " + from + ": " + intent.payload.text)
 * })
 * 
 * // メッセージ送信
 * local msg = new IntentBox("chat.message", { text: "Hello P2P!" })
 * alice.send("bob", msg)
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use crate::boxes::IntentBox;
use crate::transport::{Transport, InProcessTransport, TransportError};
use crate::messaging::IntentHandler;
use std::any::Any;
use std::sync::RwLock;
use std::collections::HashMap;

/// P2PBox - P2P通信ノード (RwLock pattern)
#[derive(Debug)]
pub struct P2PBox {
    base: BoxBase,
    node_id: RwLock<String>,
    transport: RwLock<Box<dyn Transport>>,
    handlers: RwLock<HashMap<String, Box<dyn NyashBox>>>,
}

impl Clone for P2PBox {
    fn clone(&self) -> Self {
        // State-preserving clone implementation following PR #87 pattern
        let node_id_val = self.node_id.read().unwrap().clone();
        // Note: Transport cloning is complex, for now we create a new transport
        // In a full implementation, we'd need to properly handle transport state
        let transport_kind = TransportKind::InProcess; // Default for now
        let new_transport: Box<dyn Transport> = match transport_kind {
            TransportKind::InProcess => Box::new(InProcessTransport::new(node_id_val.clone())),
        };
        let handlers_val = HashMap::new(); // Start fresh for cloned instance
        
        Self {
            base: BoxBase::new(), // New unique ID for clone
            node_id: RwLock::new(node_id_val),
            transport: RwLock::new(new_transport),
            handlers: RwLock::new(handlers_val),
        }
    }
}
#[derive(Debug, Clone)]
pub enum TransportKind {
    InProcess,
    // 将来: WebSocket, WebRTC, etc.
}

impl std::str::FromStr for TransportKind {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inprocess" => Ok(TransportKind::InProcess),
            _ => Err(format!("Unknown transport kind: {}", s)),
        }
    }
}

impl P2PBox {
    /// 新しいP2PBoxを作成
    pub fn new(node_id: String, transport_kind: TransportKind) -> Self {
        let transport: Box<dyn Transport> = match transport_kind {
            TransportKind::InProcess => Box::new(InProcessTransport::new(node_id.clone())),
        };
        
        P2PBox {
            base: BoxBase::new(),
            node_id: RwLock::new(node_id),
            transport: RwLock::new(transport),
            handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// ノードIDを取得
    pub fn get_node_id(&self) -> Box<dyn NyashBox> {
        let node_id = self.node_id.read().unwrap().clone();
        Box::new(StringBox::new(node_id))
    }
    
    /// 特定ノードにメッセージを送信
    pub fn send(&self, to: Box<dyn NyashBox>, intent: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let to_str = to.to_string_box().value;
        
        // Extract IntentBox from the generic Box
        if let Some(intent_box) = intent.as_any().downcast_ref::<IntentBox>() {
            let transport = self.transport.read().unwrap();
            match transport.send(&to_str, intent_box.clone(), Default::default()) {
                Ok(()) => Box::new(BoolBox::new(true)),
                Err(_) => Box::new(BoolBox::new(false)),
            }
        } else {
            Box::new(BoolBox::new(false))
        }
    }
    
    /// イベントハンドラーを登録
    pub fn on(&self, intent_name: Box<dyn NyashBox>, handler: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let intent_str = intent_name.to_string_box().value;
        
        // For now, we'll store a simplified handler representation
        // In a full implementation, this would need proper IntentHandler integration
        let mut handlers = self.handlers.write().unwrap();
        handlers.insert(intent_str, handler);
        Box::new(BoolBox::new(true))
    /// ノードが到達可能かチェック
    pub fn is_reachable(&self, node_id: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let node_str = node_id.to_string_box().value;
        let transport = self.transport.read().unwrap();
        Box::new(BoolBox::new(transport.is_reachable(&node_str)))
    }
    
    /// トランスポート種類を取得
    pub fn get_transport_type(&self) -> Box<dyn NyashBox> {
        let transport = self.transport.read().unwrap();
        Box::new(StringBox::new(transport.transport_type().to_string()))
    }
}



impl NyashBox for P2PBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        let node_id = self.node_id.read().unwrap().clone();
        let transport_type = self.transport.read().unwrap().transport_type().to_string();
        StringBox::new(format!("P2PBox[{}:{}]", node_id, transport_type))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_p2p) = other.as_any().downcast_ref::<P2PBox>() {
            BoolBox::new(self.base.id == other_p2p.base.id)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "P2PBox"
    }
}

impl BoxCore for P2PBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_id = self.node_id.read().unwrap().clone();
        let transport_type = self.transport.read().unwrap().transport_type().to_string();
        write!(f, "P2PBox[{}:{}]", node_id, transport_type)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for P2PBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}