/**
 * MessageIntentBox - メッセージコンテナBox（P2P通信用）
 * 
 * 設計原則：
 * - HashMap<String, Box<dyn NyashBox>>でNyashネイティブデータ保持
 * - 同期・シンプル実装（async対応は将来拡張）
 * - Everything is Box哲学に準拠
 * 
 * 注意: 既存のIntentBox（通信世界）とは別物
 * - IntentBox = 通信世界・環境の定義
 * - MessageIntentBox = 実際のメッセージデータ（これ）
 */

use std::collections::HashMap;
use std::fmt;
use crate::box_trait::{NyashBox, BoxCore, BoxBase, next_box_id};

/// MessageIntentBox - Intent型通信メッセージのコンテナ
pub struct MessageIntentBox {
    base: BoxBase,
    /// Intent種類（"chat.message", "file.transfer"等）
    pub intent: String,
    /// Nyashネイティブデータ保持
    pub payload: HashMap<String, Box<dyn NyashBox>>,
}

impl MessageIntentBox {
    /// 新しいMessageIntentBoxを作成
    pub fn new(intent: &str) -> Self {
        Self {
            base: BoxBase { 
                id: next_box_id(),
                parent_type_id: None,  // ビルトインBox継承なし
            },
            intent: intent.to_string(),
            payload: HashMap::new(),
        }
    }
    
    /// キー-値ペアを設定
    pub fn set(&mut self, key: &str, value: Box<dyn NyashBox>) {
        self.payload.insert(key.to_string(), value);
    }
    
    /// キーに対応する値を取得
    pub fn get(&self, key: &str) -> Option<&Box<dyn NyashBox>> {
        self.payload.get(key)
    }
    
    /// キーに対応する値を削除
    pub fn remove(&mut self, key: &str) -> Option<Box<dyn NyashBox>> {
        self.payload.remove(key)
    }
    
    /// すべてのキーを取得
    pub fn keys(&self) -> Vec<String> {
        self.payload.keys().cloned().collect()
    }
    
    /// ペイロードが空かチェック
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
    
    /// ペイロード要素数を取得
    pub fn len(&self) -> usize {
        self.payload.len()
    }
}

impl BoxCore for MessageIntentBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageIntentBox(intent: {}, payload: {} items)", 
               self.intent, self.payload.len())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NyashBox for MessageIntentBox {
    fn type_name(&self) -> &'static str {
        "MessageIntentBox"
    }
    
    fn to_string_box(&self) -> crate::StringBox {
        crate::StringBox::new(&format!("MessageIntentBox({})", self.intent))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        let mut new_intent = MessageIntentBox::new(&self.intent);
        
        // PayloadをDeepClone
        for (key, value) in &self.payload {
            new_intent.payload.insert(key.clone(), value.clone_box());
        }
        
        Box::new(new_intent)
    }
    
    fn equals(&self, other: &dyn NyashBox) -> crate::BoolBox {
        if let Some(other_intent) = other.as_any().downcast_ref::<MessageIntentBox>() {
            crate::BoolBox::new(self.box_id() == other_intent.box_id())
        } else {
            crate::BoolBox::new(false)
        }
    }
}

impl fmt::Display for MessageIntentBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_box(f)
    }
}

impl fmt::Debug for MessageIntentBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageIntentBox {{ intent: {:?}, payload: {:?} }}", 
               self.intent, self.payload.len())
    }
}