/*! 📦 IntentBox - Structured Message Box
 * 
 * ## 📝 概要
 * IntentBoxは構造化メッセージを表現するBoxです。
 * P2P通信において、メッセージの種類(name)と内容(payload)を
 * 明確に分離して管理します。
 * 
 * ## 🏗️ 設計
 * - **name**: メッセージの種類 ("chat.message", "file.share"等)
 * - **payload**: JSON形式の任意データ
 * - **Arc<Mutex>**: 他のBoxと統一されたメモリ管理パターン
 * 
 * ## 🛠️ 利用可能メソッド
 * - `new(name, payload)` - 構造化メッセージを作成
 * - `getName()` - メッセージ名を取得
 * - `getPayload()` - ペイロードを取得
 * - `setPayload(data)` - ペイロードを更新
 * 
 * ## 💡 使用例
 * ```nyash
 * // チャットメッセージ
 * local msg = new IntentBox("chat.message", { 
 *     text: "Hello P2P!", 
 *     from: "alice" 
 * })
 * 
 * // ファイル共有メッセージ
 * local file_msg = new IntentBox("file.share", {
 *     filename: "document.pdf",
 *     size: 1024000
 * })
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::sync::RwLock;
use std::fmt::Debug;

/// IntentBox - 構造化メッセージBox (RwLock pattern)
#[derive(Debug)]
pub struct IntentBox {
    base: BoxBase,
    /// メッセージの種類 ("chat.message", "file.share"等)
    name: RwLock<String>,
    /// 任意のJSONデータ
    payload: RwLock<serde_json::Value>,
}

impl Clone for IntentBox {
    fn clone(&self) -> Self {
        let name_val = self.name.read().unwrap().clone();
        let payload_val = self.payload.read().unwrap().clone();
        
        Self {
            base: BoxBase::new(), // New unique ID for clone
            name: RwLock::new(name_val),
            payload: RwLock::new(payload_val),
        }
    }
}

impl IntentBox {
    /// 新しいIntentBoxを作成
    pub fn new(name: String, payload: serde_json::Value) -> Self {
        IntentBox {
            base: BoxBase::new(),
            name: RwLock::new(name),
            payload: RwLock::new(payload),
        }
    }
    
    /// メッセージ名を取得
    pub fn get_name(&self) -> Box<dyn NyashBox> {
        let name = self.name.read().unwrap().clone();
        Box::new(StringBox::new(name))
    }
    
    /// ペイロードを取得
    pub fn get_payload(&self) -> Box<dyn NyashBox> {
        let payload = self.payload.read().unwrap().clone();
        Box::new(StringBox::new(payload.to_string()))
    }
    
    /// ペイロードを更新
    pub fn set_payload(&self, payload: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let payload_str = payload.to_string_box().value;
        match serde_json::from_str(&payload_str) {
            Ok(json_val) => {
                *self.payload.write().unwrap() = json_val;
                Box::new(BoolBox::new(true))
            },
            Err(_) => Box::new(BoolBox::new(false))
        }
    }
}

impl NyashBox for IntentBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        let name = self.name.read().unwrap().clone();
        StringBox::new(format!("IntentBox[{}]", name))
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
}

impl BoxCore for IntentBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.read().unwrap().clone();
        write!(f, "IntentBox[{}]", name)
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

