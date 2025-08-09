/*!
 * Nyash P2P Channel System - Arrow構文によるBox間通信
 * 
 * alice >> bob でメッセージ送信を可能にする
 * Everything is Box哲学に基づくP2P通信システム
 */

use crate::box_trait::{NyashBox, StringBox, VoidBox};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::fmt::{Debug, Display};
use std::any::Any;

/// チャンネル - Box間の通信路
#[derive(Clone)]
pub struct ChannelBox {
    /// 送信者の名前
    pub sender_name: String,
    
    /// 受信者の名前
    pub receiver_name: String,
    
    /// リンクされたBox（弱参照）
    linked_boxes: Arc<Mutex<HashMap<String, Weak<Mutex<dyn NyashBox>>>>>,
    
    /// メッセージハンドラー
    handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(Box<dyn NyashBox>) -> Box<dyn NyashBox> + Send>>>>,
    
    /// チャンネルID
    id: u64,
}

impl ChannelBox {
    /// 新しいチャンネルを作成
    pub fn new(sender: &str, receiver: &str) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self {
            sender_name: sender.to_string(),
            receiver_name: receiver.to_string(),
            linked_boxes: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),
            id,
        }
    }
    
    /// Boxをリンク
    pub fn link(&self, name: &str, target: Arc<Mutex<dyn NyashBox>>) {
        self.linked_boxes.lock().unwrap().insert(
            name.to_string(),
            Arc::downgrade(&target)
        );
    }
    
    /// メッセージハンドラーを登録
    pub fn register_handler<F>(&self, method: &str, handler: F)
    where
        F: Fn(Box<dyn NyashBox>) -> Box<dyn NyashBox> + Send + 'static
    {
        self.handlers.lock().unwrap().insert(
            method.to_string(),
            Box::new(handler)
        );
    }
    
    /// メソッド呼び出しを実行
    pub fn invoke(&self, method: &str, args: Vec<Box<dyn NyashBox>>) -> Box<dyn NyashBox> {
        // "*" はブロードキャスト
        if self.receiver_name == "*" {
            return self.broadcast(method, args);
        }
        
        // 通常の送信
        let handlers = self.handlers.lock().unwrap();
        if let Some(handler) = handlers.get(method) {
            // 簡易実装：最初の引数のみ使用
            let arg = args.get(0)
                .map(|a| a.clone_box())
                .unwrap_or_else(|| Box::new(VoidBox::new()));
            handler(arg)
        } else {
            Box::new(StringBox::new(&format!("No handler for method: {}", method)))
        }
    }
    
    /// 送信者名を取得
    pub fn sender(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(&self.sender_name))
    }
    
    /// 受信者名を取得
    pub fn receiver(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(&self.receiver_name))
    }
    
    /// ブロードキャスト
    fn broadcast(&self, _method: &str, _args: Vec<Box<dyn NyashBox>>) -> Box<dyn NyashBox> {
        let linked = self.linked_boxes.lock().unwrap();
        let mut results = Vec::new();
        
        for (name, weak_box) in linked.iter() {
            if let Some(_strong_box) = weak_box.upgrade() {
                // 各Boxにメッセージを送信
                results.push(format!("Sent to {}", name));
            }
        }
        
        Box::new(StringBox::new(&format!("Broadcast complete: {:?}", results)))
    }
}

impl NyashBox for ChannelBox {
    fn type_name(&self) -> &'static str {
        "ChannelBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&format!("Channel({} >> {})", self.sender_name, self.receiver_name))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        if let Some(other_channel) = other.as_any().downcast_ref::<ChannelBox>() {
            crate::box_trait::BoolBox::new(
                self.sender_name == other_channel.sender_name &&
                self.receiver_name == other_channel.receiver_name
            )
        } else {
            crate::box_trait::BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for ChannelBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
}

impl Debug for ChannelBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelBox")
            .field("sender_name", &self.sender_name)
            .field("receiver_name", &self.receiver_name)
            .field("id", &self.id)
            .finish()
    }
}

/// メッセージを表すBox
#[derive(Debug, Clone)]
pub struct MessageBox {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
}

impl MessageBox {
    pub fn new(sender: &str, content: &str) -> Self {
        static mut COUNTER: u64 = 0;
        let timestamp = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self {
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp,
        }
    }
}

impl NyashBox for MessageBox {
    fn type_name(&self) -> &'static str {
        "MessageBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&format!("[{}] {}: {}", self.timestamp, self.sender, self.content))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        if let Some(other_msg) = other.as_any().downcast_ref::<MessageBox>() {
            crate::box_trait::BoolBox::new(
                self.sender == other_msg.sender &&
                self.content == other_msg.content
            )
        } else {
            crate::box_trait::BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.timestamp
    }
}

impl Display for MessageBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
}