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

        // Minimal built-in system handler: auto-respond to sys.ping
        // This enables health checks via ping() without requiring user wiring.
        if attach_cb {
            // capture for receive-side traces
            let last_from = Arc::clone(&p2p.last_from);
            let last_intent = Arc::clone(&p2p.last_intent_name);
            // capture transport Arc to use inside handler
            let transport_arc_outer = Arc::clone(&p2p.transport);
            {
                if let Ok(mut t) = transport_arc_outer.write() {
                    let transport_arc_for_cb = Arc::clone(&transport_arc_outer);
                    t.register_intent_handler("sys.ping", Box::new(move |env| {
                        if let Ok(mut lf) = last_from.write() { *lf = Some(env.from.clone()); }
                        if let Ok(mut li) = last_intent.write() { *li = Some(env.intent.get_name().to_string_box().value); }
                        // Reply asynchronously to avoid deep call stacks
                        let to = env.from.clone();
                        let reply = crate::boxes::IntentBox::new("sys.pong".to_string(), serde_json::json!({}));
                        let transport_arc = Arc::clone(&transport_arc_for_cb);
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                            if let Ok(transport) = transport_arc.read() {
                                let _ = transport.send(&to, reply, Default::default());
                            }
                        });
                    }));
                };
            }
        }

        p2p
    }
    
    /// ノードIDを取得
    pub fn get_node_id(&self) -> Box<dyn NyashBox> {
        let node_id = self.node_id.read().unwrap().clone();
        Box::new(StringBox::new(node_id))
    }

    /// Blocking ping: send sys.ping to target and wait for sys.pong
    /// Returns BoolBox(true) on success within timeout, else false.
    pub fn ping_with_timeout(&self, to: Box<dyn NyashBox>, timeout_ms: u64) -> Box<dyn NyashBox> {
        use std::sync::{mpsc, Arc};
        let to_str = to.to_string_box().value;

        // Create oneshot channel for pong
        let (tx, rx) = mpsc::channel::<()>();
        let active = Arc::new(AtomicBool::new(true));
        let active_cb = Arc::clone(&active);

        // Register temporary transport-level handler for sys.pong
        if let Ok(mut t) = self.transport.write() {
            t.register_intent_handler("sys.pong", Box::new(move |env| {
                if active_cb.load(Ordering::SeqCst) {
                    // record last receive for visibility
                    // Note: we cannot access self here safely; rely on tx notify only
                    let _ = env; // suppress unused
                    let _ = tx.send(());
                }
            }));

            // Send sys.ping
            let ping = IntentBox::new("sys.ping".to_string(), serde_json::json!({}));
            match t.send(&to_str, ping, Default::default()) {
                Ok(()) => { /* proceed to wait */ }
                Err(_) => {
                    return Box::new(BoolBox::new(false));
                }
            }
        } else {
            return Box::new(BoolBox::new(false));
        }

        // Wait for pong with timeout
        let ok = rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)).is_ok();
        active.store(false, Ordering::SeqCst);
        Box::new(BoolBox::new(ok))
    }

    /// Convenience default-timeout ping (200ms)
    pub fn ping(&self, to: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        self.ping_with_timeout(to, 200)
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
                // capture state holders for receive-side tracing
                let last_from = Arc::clone(&self.last_from);
                let last_intent = Arc::clone(&self.last_intent_name);
                t.register_intent_handler(&intent_name, Box::new(move |env| {
                    // flagがtrueのときのみ実行
                    if flag.load(Ordering::SeqCst) {
                        // Update receive-side traces for E2E visibility
                        if let Ok(mut lf) = last_from.write() { *lf = Some(env.from.clone()); }
                        if let Ok(mut li) = last_intent.write() { *li = Some(env.intent.get_name().to_string_box().value); }
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

    /// デバッグ: intentに対する有効ハンドラー数（trueフラグ数）
    pub fn debug_active_handler_count(&self, intent_name: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let name = intent_name.to_string_box().value;
        let flags = self.handler_flags.read().unwrap();
        let cnt = flags.get(&name)
            .map(|v| v.iter().filter(|f| f.load(Ordering::SeqCst)).count())
            .unwrap_or(0);
        Box::new(crate::box_trait::IntegerBox::new(cnt as i64))
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

    /// Internal helper for tests: register raw Rust handler with optional async reply
    impl P2PBox {
        #[allow(dead_code)]
        fn __debug_on_rust(&self, intent: &str, reply_intent: Option<&str>) {
            if let Ok(mut t) = self.transport.write() {
                let intent_name = intent.to_string();
                let last_from = Arc::clone(&self.last_from);
                let last_intent = Arc::clone(&self.last_intent_name);
                // create self clone for reply
                // Avoid deep clone (which re-registers transport). Use transport directly for reply.
                let transport_arc = Arc::clone(&self.transport);
                let reply_name = reply_intent.map(|s| s.to_string());
                t.register_intent_handler(&intent_name, Box::new(move |env| {
                    if let Ok(mut lf) = last_from.write() { *lf = Some(env.from.clone()); }
                    if let Ok(mut li) = last_intent.write() { *li = Some(env.intent.get_name().to_string_box().value); }
                    if let Some(rn) = reply_name.clone() {
                        let to = env.from.clone();
                        let transport_arc = Arc::clone(&transport_arc);
                        std::thread::spawn(move || {
                            // slight delay to avoid lock contention
                            std::thread::sleep(std::time::Duration::from_millis(5));
                            let intent = IntentBox::new(rn, serde_json::json!({}));
                            if let Ok(transport) = transport_arc.read() {
                                let _ = transport.send(&to, intent, Default::default());
                            }
                        });
                    }
                }));
            }
        }
    }

    #[test]
    fn two_node_ping_pong() {
        let alice = P2PBox::new("alice".to_string(), TransportKind::InProcess);
        let bob = P2PBox::new("bob".to_string(), TransportKind::InProcess);
        // bob replies pong to ping
        bob.__debug_on_rust("ping", Some("pong"));
        // alice listens pong
        alice.__debug_on_rust("pong", None);
        // send ping
        let ping = IntentBox::new("ping".to_string(), serde_json::json!({}));
        let _ = alice.send(Box::new(StringBox::new("bob")), Box::new(ping));
        // bob should record ping
        assert_eq!(bob.get_last_intent_name().to_string_box().value, "ping");
        // allow async reply
        std::thread::sleep(std::time::Duration::from_millis(20));
        // alice should record pong
        assert_eq!(alice.get_last_intent_name().to_string_box().value, "pong");
    }

    #[test]
    fn on_once_disables_after_first_delivery() {
        let p = P2PBox::new("alice".to_string(), TransportKind::InProcess);
        // Register one-time handler for 'hello'
        let handler = crate::method_box::MethodBox::new(Box::new(p.clone()), "noop".to_string());
        let _ = p.on_once(Box::new(StringBox::new("hello")), Box::new(handler));
        // Initially active = 1
        let c0 = p.debug_active_handler_count(Box::new(StringBox::new("hello")));
        assert_eq!(c0.to_string_box().value, "1");
        // Send twice to self
        let intent = IntentBox::new("hello".to_string(), serde_json::json!({}));
        let _ = p.send(Box::new(StringBox::new("alice")), Box::new(intent.clone()));
        let _ = p.send(Box::new(StringBox::new("alice")), Box::new(intent));
        // After first delivery, once-flag should be false => active count = 0
        let c1 = p.debug_active_handler_count(Box::new(StringBox::new("hello")));
        assert_eq!(c1.to_string_box().value, "0");
    }

    #[test]
    fn off_clears_handlers() {
        let p = P2PBox::new("bob".to_string(), TransportKind::InProcess);
        let handler = crate::method_box::MethodBox::new(Box::new(p.clone()), "noop".to_string());
        let _ = p.on(Box::new(StringBox::new("bye")), Box::new(handler));
        // Active = 1
        let c0 = p.debug_active_handler_count(Box::new(StringBox::new("bye")));
        assert_eq!(c0.to_string_box().value, "1");
        // Off
        let _ = p.off(Box::new(StringBox::new("bye")));
        let c1 = p.debug_active_handler_count(Box::new(StringBox::new("bye")));
        assert_eq!(c1.to_string_box().value, "0");
    }

    #[test]
    fn ping_success_between_two_nodes() {
        let alice = P2PBox::new("alice".to_string(), TransportKind::InProcess);
        let bob = P2PBox::new("bob".to_string(), TransportKind::InProcess);
        // bob has built-in sys.ping -> sys.pong
        let ok = alice.ping(Box::new(StringBox::new("bob")));
        if let Some(b) = ok.as_any().downcast_ref::<BoolBox>() {
            assert!(b.value);
        } else {
            panic!("ping did not return BoolBox");
        }
    }

    #[test]
    fn ping_timeout_on_missing_node() {
        let alice = P2PBox::new("alice".to_string(), TransportKind::InProcess);
        let ok = alice.ping_with_timeout(Box::new(StringBox::new("nobody")), 20);
        if let Some(b) = ok.as_any().downcast_ref::<BoolBox>() {
            assert!(!b.value);
        } else {
            panic!("ping_with_timeout did not return BoolBox");
        }
    }
}
