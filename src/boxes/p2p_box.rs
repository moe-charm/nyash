/*! 📡 P2PBox - 通信ノードBox
 * 
 * ## 📝 概要
 * P2PBoxは通信世界（IntentBox）に参加するノードを表します。
 * シンプルなsend/onインターフェースで、他のノードとメッセージを
 * やり取りできます。Arc<Mutex>パターンにより、スレッドセーフな
 * 並行通信を実現します。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `new(node_id, intent_box)` - ノードを作成して通信世界に参加
 * - `send(intent, data, target)` - 特定ノードにメッセージ送信
 * - `broadcast(intent, data)` - 全ノードにブロードキャスト
 * - `on(intent, callback)` - イベントリスナー登録
 * - `off(intent)` - リスナー解除
 * - `get_node_id()` - ノードID取得
 * 
 * ## 💡 使用例
 * ```nyash
 * // 通信世界を作成
 * world = new IntentBox()
 * 
 * // ノードを作成
 * alice = new P2PBox("alice", world)
 * bob = new P2PBox("bob", world)
 * 
 * // リスナー登録
 * bob.on("greeting", |data, from| {
 *     print(from + " says: " + data.get("text"))
 * })
 * 
 * // メッセージ送信
 * alice.send("greeting", { "text": "Hello Bob!" }, "bob")
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::boxes::intent_box::IntentBox;
pub use crate::boxes::intent_box::Message;
use crate::boxes::map_box::MapBox;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// リスナー関数の型（MethodBoxまたはクロージャ）
pub type ListenerFn = Box<dyn NyashBox>;

/// P2PBox内部実装
#[derive(Debug)]
struct P2PBoxInner {
    id: u64,
    node_id: String,
    intent_box: Arc<IntentBox>,
    listeners: Arc<Mutex<HashMap<String, Vec<ListenerFn>>>>,
}

/// P2PBox - 通信ノード（Arc<P2PBoxInner>のラッパー）
#[derive(Debug, Clone)]
pub struct P2PBox {
    inner: Arc<P2PBoxInner>,
}

impl P2PBox {
    /// 新しいP2PBoxノードを作成
    pub fn new(node_id: String, intent_box: Arc<IntentBox>) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        let inner = Arc::new(P2PBoxInner {
            id,
            node_id,
            intent_box: intent_box.clone(),
            listeners: Arc::new(Mutex::new(HashMap::new())),
        });
        
        P2PBox { inner }
    }
    
    /// ノードIDを取得
    pub fn get_node_id(&self) -> String {
        self.inner.node_id.clone()
    }
    
    /// 特定のノードにメッセージを送信
    pub fn send(&self, intent: &str, data: Box<dyn NyashBox>, target: &str) -> Box<dyn NyashBox> {
        let transport = self.inner.intent_box.get_transport();
        let transport = transport.lock().unwrap();
        transport.send(&self.inner.node_id, target, intent, data);
        Box::new(StringBox::new("sent"))
    }
    
    /// 全ノードにメッセージをブロードキャスト
    pub fn broadcast(&self, intent: &str, data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let transport = self.inner.intent_box.get_transport();
        let transport = transport.lock().unwrap();
        transport.broadcast(&self.inner.node_id, intent, data);
        Box::new(StringBox::new("broadcast"))
    }
    
    /// イベントリスナーを登録
    pub fn on(&self, intent: &str, callback: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let mut listeners = self.inner.listeners.lock().unwrap();
        listeners.entry(intent.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
        Box::new(StringBox::new("listener added"))
    }
    
    /// リスナーを解除
    pub fn off(&self, intent: &str) -> Box<dyn NyashBox> {
        let mut listeners = self.inner.listeners.lock().unwrap();
        if listeners.remove(intent).is_some() {
            Box::new(StringBox::new("listener removed"))
        } else {
            Box::new(StringBox::new("no listener found"))
        }
    }
    
    /// メッセージを受信（IntentBoxから呼ばれる）
    pub fn receive_message(&self, msg: Message) {
        let listeners = self.inner.listeners.lock().unwrap();
        
        if let Some(callbacks) = listeners.get(&msg.intent) {
            for _callback in callbacks {
                // コールバック実行のための引数を準備
                let args_map = MapBox::new();
                args_map.set(Box::new(StringBox::new("data")), msg.data.clone_box());
                args_map.set(Box::new(StringBox::new("from")), Box::new(StringBox::new(&msg.from)));
                
                // TODO: インタープリターコンテキストでコールバック実行
                // 現在は単純化のため、メッセージ内容を出力
                println!("P2PBox[{}] received '{}' from {}", self.inner.node_id, msg.intent, msg.from);
            }
        }
    }
}

impl Drop for P2PBox {
    fn drop(&mut self) {
        // TODO: 破棄時にIntentBoxから登録解除
    }
}

impl NyashBox for P2PBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("P2PBox[{}]", self.inner.node_id))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_p2p) = other.as_any().downcast_ref::<P2PBox>() {
            BoolBox::new(self.inner.id == other_p2p.inner.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "P2PBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.inner.id
    }
}