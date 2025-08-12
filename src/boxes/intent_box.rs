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
use std::sync::{Arc, Mutex};
use std::fmt::{self, Debug};

/// IntentBox内部データ構造
#[derive(Debug, Clone)]
pub struct IntentBoxData {
    base: BoxBase,
    /// メッセージの種類 ("chat.message", "file.share"等)
    pub name: String,
    /// 任意のJSONデータ
    pub payload: serde_json::Value,
}

/// IntentBox - 構造化メッセージBox（Arc<Mutex>統一パターン）
pub type IntentBox = Arc<Mutex<IntentBoxData>>;

impl IntentBoxData {
    /// 新しいIntentBoxを作成
    pub fn new(name: String, payload: serde_json::Value) -> IntentBox {
        Arc::new(Mutex::new(IntentBoxData {
            base: BoxBase::new(),
            name,
            payload,
        }))
    }
    
    /// メッセージ名を取得
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// ペイロードを取得
    pub fn get_payload(&self) -> &serde_json::Value {
        &self.payload
    }
    
    /// ペイロードを更新
    pub fn set_payload(&mut self, payload: serde_json::Value) {
        self.payload = payload;
    }
}

impl NyashBox for IntentBox {
    fn to_string_box(&self) -> StringBox {
        let data = self.lock().unwrap();
        StringBox::new(format!("IntentBox[{}]", data.name))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_intent) = other.as_any().downcast_ref::<IntentBox>() {
            let self_data = self.lock().unwrap();
            let other_data = other_intent.lock().unwrap();
            BoolBox::new(self_data.base.id == other_data.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "IntentBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        let data = self.lock().unwrap();
        Box::new(IntentBoxData::new(data.name.clone(), data.payload.clone()))
    }
}

impl BoxCore for IntentBox {
    fn box_id(&self) -> u64 {
        self.lock().unwrap().base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.lock().unwrap().base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock().unwrap();
        write!(f, "IntentBox[{}]", data.name)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for IntentBoxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntentBox[{}]", self.name)
    }
}

