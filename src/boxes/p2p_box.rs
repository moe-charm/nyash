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
use crate::messaging::{IntentHandler, MessageBusData};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// P2PBox内部データ構造
pub struct P2PBoxData {
    base: BoxBase,
    node_id: String,
    transport: Arc<Mutex<Box<dyn Transport>>>,
}

impl std::fmt::Debug for P2PBoxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("P2PBoxData")
            .field("base", &self.base)
            .field("node_id", &self.node_id)
            .field("transport", &"<Transport>")
            .finish()
    }
}

/// P2PBox - P2P通信ノード（Arc<Mutex>統一パターン）
pub type P2PBox = Arc<Mutex<P2PBoxData>>;

/// P2PBox作成時のトランスポート種類
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

impl P2PBoxData {
    /// 新しいP2PBoxを作成
    pub fn new(node_id: String, transport_kind: TransportKind) -> P2PBox {
        let transport: Box<dyn Transport> = match transport_kind {
            TransportKind::InProcess => Box::new(InProcessTransport::new(node_id.clone())),
        };
        
        Arc::new(Mutex::new(P2PBoxData {
            base: BoxBase::new(),
            node_id,
            transport: Arc::new(Mutex::new(transport)),
        }))
    }
    
    /// ノードIDを取得
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
    
    /// 特定ノードにメッセージを送信
    pub fn send(&self, to: &str, intent: IntentBox) -> Result<(), TransportError> {
        let transport = self.transport.lock().unwrap();
        transport.send(to, intent, Default::default())
    }
    
    /// イベントハンドラーを登録
    pub fn on(&self, intent_name: &str, handler: IntentHandler) -> Result<(), String> {
        // InProcessTransportの場合のハンドラー追加
        // 現在は簡略化された実装
        Ok(())
    }
    
    /// ノードが到達可能かチェック
    pub fn is_reachable(&self, node_id: &str) -> bool {
        let transport = self.transport.lock().unwrap();
        transport.is_reachable(node_id)
    }
    
    /// トランスポート種類を取得
    pub fn get_transport_type(&self) -> String {
        let transport = self.transport.lock().unwrap();
        transport.transport_type().to_string()
    }
}



impl NyashBox for P2PBox {
    fn to_string_box(&self) -> StringBox {
        let data = self.lock().unwrap();
        StringBox::new(format!("P2PBox[{}:{}]", data.node_id, data.get_transport_type()))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_p2p) = other.as_any().downcast_ref::<P2PBox>() {
            let self_data = self.lock().unwrap();
            let other_data = other_p2p.lock().unwrap();
            BoolBox::new(self_data.base.id == other_data.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "P2PBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // P2PBoxは共有されるので、新しいインスタンスではなく同じ参照を返す
        Box::new(self.clone())
    }
}

impl BoxCore for P2PBox {
    fn box_id(&self) -> u64 {
        self.lock().unwrap().base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.lock().unwrap().base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock().unwrap();
        write!(f, "P2PBox[{}:{}]", data.node_id, data.get_transport_type())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for P2PBoxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P2PBox[{}:{}]", self.node_id, self.get_transport_type())
    }
}