/**
 * NewP2PBox - 天才アルゴリズム内蔵P2PBox（同期・シンプル版）
 * 
 * 設計原則（4つの核心）：
 * 1. P2PBoxは、トランスポートがネットでもBusを持ち続ける（ローカル配送・購読・監視用）
 * 2. P2PBoxはMessageIntentBoxを使って送る
 * 3. 送信アルゴリズム：ローカルならBus、それ以外はTransport
 * 4. 受信アルゴリズム：Transport→P2PBox→Bus でローカルハンドラに届く
 * 
 * Everything is Box哲学準拠・同期実装
 */

use std::sync::Arc;
use crate::box_trait::{NyashBox, BoxCore, BoxBase, next_box_id};
use crate::boxes::MessageIntentBox;
use crate::transport_trait::{Transport, TransportKind, create_transport};
use crate::message_bus::{get_global_message_bus, BusMessage, MessageBus};

/// NewP2PBox - 天才アルゴリズム内蔵P2P通信ノード
pub struct NewP2PBox {
    base: BoxBase,
    node_id: String,
    transport: Box<dyn Transport>,
    bus: Arc<MessageBus>,  // ← 常に保持！（ローカル配送・購読・監視用）
}

impl NewP2PBox {
    /// シンプル同期コンストラクタ
    pub fn new(node_id: &str, transport_kind: TransportKind) -> Self {
        let bus = get_global_message_bus();  // シングルトン取得
        let transport = create_transport(transport_kind, node_id);  // 簡単ファクトリ
        
        // 自ノード登録
        bus.register_node(node_id).unwrap();
        
        Self { 
            base: BoxBase {
                id: next_box_id(),
                parent_type_id: None,
            },
            node_id: node_id.to_string(), 
            transport, 
            bus 
        }
    }
    
    /// 購読メソッド - Busに登録
    pub fn on(&self, intent: &str, callback: Box<dyn Fn(&MessageIntentBox) + Send + Sync>) {
        // BusMessageからMessageIntentBoxを抽出するラッパー
        let wrapper = Box::new(move |bus_message: &BusMessage| {
            // BusMessageのdataをMessageIntentBoxにダウンキャスト
            if let Some(intent_box) = bus_message.data.as_any().downcast_ref::<MessageIntentBox>() {
                callback(intent_box);
            }
        });
        self.bus.on(&self.node_id, intent, wrapper).unwrap();
    }
    
    /// 送信メソッド - 天才アルゴリズム内蔵（同期版）
    pub fn send(&self, to: &str, intent_box: &MessageIntentBox) -> Result<(), String> {
        // 1) 宛先が同プロセス（Busが知っている）ならローカル配送
        if self.bus.has_node(to) {
            // MessageIntentBoxからBusMessageに変換
            let message = BusMessage {
                from: self.node_id.clone(),
                to: to.to_string(),
                intent: intent_box.intent.clone(),
                data: intent_box.clone_box(),  // MessageIntentBox全体をデータとして送信
                timestamp: std::time::SystemTime::now(),
            };
            self.bus.route(message)?;  // 爆速ローカル
            return Ok(());
        }

        // 2) ローカルに居ない → Transportで外へ出す
        self.transport.send(to, &intent_box.intent, intent_box.clone_box())
    }
    
    /// ノードID取得
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
}

impl BoxCore for NewP2PBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NewP2PBox(node_id: {}, transport: {})", 
               self.node_id, self.transport.transport_type())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for NewP2PBox {
    fn type_name(&self) -> &'static str {
        "NewP2PBox"
    }
    
    fn to_string_box(&self) -> crate::StringBox {
        crate::StringBox::new(&format!("NewP2PBox({})", self.node_id))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // P2PBoxは基本的にクローンしない（ノードの一意性のため）
        // 必要に応じて別のコンストラクタで同じ設定の新ノードを作成する
        todo!("P2PBox clone not recommended - create new node instead")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::BoolBox {
        if let Some(other_p2p) = other.as_any().downcast_ref::<NewP2PBox>() {
            crate::BoolBox::new(self.node_id == other_p2p.node_id)
        } else {
            crate::BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for NewP2PBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

impl std::fmt::Debug for NewP2PBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NewP2PBox {{ node_id: {:?}, transport: {:?} }}", 
               self.node_id, self.transport.transport_type())
    }
}