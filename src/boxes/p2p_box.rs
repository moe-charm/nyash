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
use crate::method_box::MethodBox;
use crate::boxes::result::{ResultBox, NyashResultBox};
use crate::transport::{Transport, InProcessTransport};
use std::any::Any;
use std::sync::{RwLock, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;

/// P2PBox - P2P通信ノード (RwLock pattern)
#[derive(Debug)]
pub struct P2PBox {
    base: BoxBase,
    node_id: RwLock<String>,
    transport: Arc<RwLock<Box<dyn Transport>>>,
    handlers: Arc<RwLock<HashMap<String, Box<dyn NyashBox>>>>,
    handler_flags: Arc<RwLock<HashMap<String, Vec<Arc<AtomicBool>>>>>,
    // Minimal receive cache for loopback smoke tests
    last_from: Arc<RwLock<Option<String>>>,
    last_intent_name: Arc<RwLock<Option<String>>>,
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
        let last_from_val = self.last_from.read().unwrap().clone();
        let last_intent_val = self.last_intent_name.read().unwrap().clone();
        
        Self {
            base: BoxBase::new(), // New unique ID for clone
            node_id: RwLock::new(node_id_val),
            transport: Arc::new(RwLock::new(new_transport)),
            handlers: Arc::new(RwLock::new(handlers_val)),
            handler_flags: Arc::new(RwLock::new(HashMap::new())),
            last_from: Arc::new(RwLock::new(last_from_val)),
            last_intent_name: Arc::new(RwLock::new(last_intent_val)),
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
        // Create transport and attach receive callback before boxing
        let (transport_boxed, attach_cb): (Box<dyn Transport>, bool) = match transport_kind {
            TransportKind::InProcess => {
                let mut t = InProcessTransport::new(node_id.clone());
                // We'll attach callback below after P2PBox struct is created
                (Box::new(t), true)
            }
        };

        let p2p = P2PBox {
            base: BoxBase::new(),
            node_id: RwLock::new(node_id),
            transport: Arc::new(RwLock::new(transport_boxed)),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            handler_flags: Arc::new(RwLock::new(HashMap::new())),
            last_from: Arc::new(RwLock::new(None)),
            last_intent_name: Arc::new(RwLock::new(None)),
        };

        // Note: InProcess callback registration is postponed until a unified
        // Transport subscription API is provided. For now, loopback tracing is
        // handled in send() when sending to self.

        p2p
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
                Ok(()) => {
                    // Minimal loopback trace without relying on transport callbacks
                    let self_id = self.node_id.read().unwrap().clone();
                    if to_str == self_id {
                        if let Ok(mut lf) = self.last_from.write() { *lf = Some(self_id.clone()); }
                        if let Ok(mut li) = self.last_intent_name.write() { *li = Some(intent_box.get_name().to_string_box().value); }
                    }
                    Box::new(ResultBox::new_ok(Box::new(BoolBox::new(true))))
                },
                Err(e) => Box::new(ResultBox::new_err(Box::new(StringBox::new(format!("{:?}", e))))),
            }
        } else {
            Box::new(ResultBox::new_err(Box::new(StringBox::new("Second argument must be IntentBox"))))
        }
    }
    
    /// イベントハンドラーを登録
    fn register_handler_internal(&self, intent_str: &str, handler: &Box<dyn NyashBox>, once: bool) -> Box<dyn NyashBox> {
        // 保存
        {
            let mut handlers = self.handlers.write().unwrap();
            handlers.insert(intent_str.to_string(), handler.clone_box());
        }

        // フラグ登録
        let flag = Arc::new(AtomicBool::new(true));
        {
            let mut flags = self.handler_flags.write().unwrap();
            flags.entry(intent_str.to_string()).or_default().push(flag.clone());
        }

        // 可能ならTransportにハンドラ登録（InProcessなど）
        if let Ok(mut t) = self.transport.write() {
            if let Some(method_box) = handler.as_any().downcast_ref::<MethodBox>() {
                let method_clone = method_box.clone();
                let intent_name = intent_str.to_string();
                t.register_intent_handler(&intent_name, Box::new(move |env| {
                    // flagがtrueのときのみ実行
                    if flag.load(Ordering::SeqCst) {
                        let _ = method_clone.invoke(vec![
                            Box::new(env.intent.clone()),
                            Box::new(StringBox::new(env.from.clone())),
                        ]);
                        if once {
                            flag.store(false, Ordering::SeqCst);
                        }
                    }
                }));
            }
        }
        Box::new(ResultBox::new_ok(Box::new(BoolBox::new(true))))
    }

    /// イベントハンドラーを登録
    pub fn on(&self, intent_name: Box<dyn NyashBox>, handler: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let intent_str = intent_name.to_string_box().value;
        self.register_handler_internal(&intent_str, &handler, false)
    }

    /// 一度だけのハンドラー登録
    pub fn on_once(&self, intent_name: Box<dyn NyashBox>, handler: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let intent_str = intent_name.to_string_box().value;
        self.register_handler_internal(&intent_str, &handler, true)
    }

    /// ハンドラー解除（intentの全ハンドラー無効化）
    pub fn off(&self, intent_name: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let intent_str = intent_name.to_string_box().value;
        if let Ok(mut flags) = self.handler_flags.write() {
            if let Some(v) = flags.get_mut(&intent_str) {
                for f in v.iter() { f.store(false, Ordering::SeqCst); }
                v.clear();
            }
        }
        // 登録ハンドラ保存も削除
        let _ = self.handlers.write().unwrap().remove(&intent_str);
        Box::new(ResultBox::new_ok(Box::new(BoolBox::new(true))))
    }
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

    /// デバッグ: 既知ノード一覧（InProcessのみ対応）
    pub fn debug_nodes(&self) -> Box<dyn NyashBox> {
        let transport = self.transport.read().unwrap();
        if let Some(list) = transport.debug_list_nodes() {
            Box::new(StringBox::new(list.join(",")))
        } else {
            Box::new(StringBox::new("<unsupported>"))
        }
    }

    pub fn debug_bus_id(&self) -> Box<dyn NyashBox> {
        let transport = self.transport.read().unwrap();
        if let Some(id) = transport.debug_bus_id() {
            Box::new(StringBox::new(id))
        } else {
            Box::new(StringBox::new("<unsupported>"))
        }
    }

    /// 最後に受信したfromを取得（ループバック検証用）
    pub fn get_last_from(&self) -> Box<dyn NyashBox> {
        let v = self.last_from.read().unwrap().clone().unwrap_or_default();
        Box::new(StringBox::new(v))
    }

    /// 最後に受信したIntent名を取得（ループバック検証用）
    pub fn get_last_intent_name(&self) -> Box<dyn NyashBox> {
        let v = self.last_intent_name.read().unwrap().clone().unwrap_or_default();
        Box::new(StringBox::new(v))
    }
}



impl NyashBox for P2PBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        // Share underlying transport and state via Arc clones
        let node_id_val = self.node_id.read().unwrap().clone();
        Box::new(P2PBox {
            base: BoxBase::new(),
            node_id: RwLock::new(node_id_val),
            transport: Arc::clone(&self.transport),
            handlers: Arc::clone(&self.handlers),
            handler_flags: Arc::clone(&self.handler_flags),
            last_from: Arc::clone(&self.last_from),
            last_intent_name: Arc::clone(&self.last_intent_name),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_ping_sets_last_fields() {
        let p = P2PBox::new("alice".to_string(), TransportKind::InProcess);
        let intent = IntentBox::new("ping".to_string(), serde_json::json!({}));
        let res = p.send(Box::new(StringBox::new("alice".to_string())), Box::new(intent));
        // Ensure Ok
        if let Some(r) = res.as_any().downcast_ref::<ResultBox>() {
            assert!(matches!(r, ResultBox::Ok(_)));
        } else {
            panic!("send did not return ResultBox");
        }
        assert_eq!(p.get_last_from().to_string_box().value, "alice".to_string());
        assert_eq!(p.get_last_intent_name().to_string_box().value, "ping".to_string());
    }
}
