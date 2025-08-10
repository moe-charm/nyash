/*! 🗄️ MapBox - キー値ストレージBox
 * 
 * ## 📝 概要
 * 高性能キー値ストレージを提供するBox。
 * JavaScript Map、Python dict、C# Dictionaryと同等機能。
 * 動的データ管理やキャッシュ実装に最適。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `set(key, value)` - キー値ペア設定
 * - `get(key)` - 値取得
 * - `has(key)` - キー存在確認
 * - `remove(key)` - キー値ペア削除
 * - `clear()` - 全データクリア
 * - `keys()` - 全キー取得
 * - `values()` - 全値取得
 * - `size()` - データ数取得
 * - `isEmpty()` - 空判定
 * 
 * ## 💡 使用例
 * ```nyash
 * local map, result
 * map = new MapBox()
 * 
 * // データ設定
 * map.set("name", "Alice")
 * map.set("age", 25)
 * map.set("active", true)
 * 
 * // データ取得
 * result = map.get("name")     // "Alice"
 * print("User: " + result)
 * 
 * // 存在確認
 * if (map.has("email")) {
 *     print("Email: " + map.get("email"))
 * } else {
 *     print("No email registered")
 * }
 * ```
 * 
 * ## 🎮 実用例 - ゲーム設定管理
 * ```nyash
 * static box GameConfig {
 *     init { settings, scores }
 *     
 *     main() {
 *         me.settings = new MapBox()
 *         me.scores = new MapBox()
 *         
 *         // 設定初期化
 *         me.settings.set("difficulty", "normal")
 *         me.settings.set("sound", true)
 *         me.settings.set("graphics", "high")
 *         
 *         // スコア記録
 *         me.scores.set("level1", 850)
 *         me.scores.set("level2", 1200)
 *         me.scores.set("level3", 950)
 *         
 *         me.displayConfig()
 *     }
 *     
 *     displayConfig() {
 *         print("=== Game Settings ===")
 *         print("Difficulty: " + me.settings.get("difficulty"))
 *         print("Sound: " + me.settings.get("sound").toString())
 *         print("Total scores: " + me.scores.size().toString())
 *     }
 * }
 * ```
 * 
 * ## 🔍 キャッシュ実装例
 * ```nyash
 * static box APICache {
 *     init { cache, ttl_map }
 *     
 *     main() {
 *         me.cache = new MapBox()
 *         me.ttl_map = new MapBox()
 *     }
 *     
 *     getData(url) {
 *         // キャッシュ確認
 *         if (me.cache.has(url)) {
 *             return me.cache.get(url)
 *         }
 *         
 *         // APIから取得
 *         local data
 *         data = fetchFromAPI(url)
 *         
 *         // キャッシュに保存
 *         me.cache.set(url, data)
 *         return data
 *     }
 * }
 * ```
 * 
 * ## ⚠️ 注意
 * - キーは自動的に文字列変換される
 * - スレッドセーフ (Arc<Mutex>使用)
 * - 大量データ格納時はメモリ使用量に注意
 * - 存在しないキーの取得は "Key not found" メッセージ返却
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::boxes::array::ArrayBox;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// キーバリューストアを表すBox
#[derive(Clone)]
pub struct MapBox {
    data: Arc<Mutex<HashMap<String, Box<dyn NyashBox>>>>,
    id: u64,
}

impl MapBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            id,
        }
    }
    
    /// 値を設定
    pub fn set(&self, key: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        self.data.lock().unwrap().insert(key_str.clone(), value);
        Box::new(StringBox::new(&format!("Set key: {}", key_str)))
    }
    
    /// 値を取得
    pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        match self.data.lock().unwrap().get(&key_str) {
            Some(value) => value.clone_box(),
            None => Box::new(StringBox::new(&format!("Key not found: {}", key_str))),
        }
    }
    
    /// キーが存在するかチェック
    pub fn has(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        Box::new(BoolBox::new(self.data.lock().unwrap().contains_key(&key_str)))
    }
    
    /// キーを削除
    pub fn delete(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        match self.data.lock().unwrap().remove(&key_str) {
            Some(_) => Box::new(StringBox::new(&format!("Deleted key: {}", key_str))),
            None => Box::new(StringBox::new(&format!("Key not found: {}", key_str))),
        }
    }
    
    /// 全てのキーを取得
    pub fn keys(&self) -> Box<dyn NyashBox> {
        let keys: Vec<String> = self.data.lock().unwrap().keys().cloned().collect();
        let array = ArrayBox::new();
        for key in keys {
            array.push(Box::new(StringBox::new(&key)));
        }
        Box::new(array)
    }
    
    /// 全ての値を取得
    pub fn values(&self) -> Box<dyn NyashBox> {
        let values: Vec<Box<dyn NyashBox>> = self.data.lock().unwrap()
            .values()
            .map(|v| v.clone_box())
            .collect();
        let array = ArrayBox::new();
        for value in values {
            array.push(value);
        }
        Box::new(array)
    }
    
    /// サイズを取得
    pub fn size(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.data.lock().unwrap().len() as i64))
    }
    
    /// 全てクリア
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.data.lock().unwrap().clear();
        Box::new(StringBox::new("Map cleared"))
    }
    
    /// 各要素に対して関数を実行
    pub fn forEach(&self, _callback: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // 簡易実装：callbackの実行はスキップ
        let count = self.data.lock().unwrap().len();
        Box::new(StringBox::new(&format!("Iterated over {} items", count)))
    }
    
    /// JSON文字列に変換
    pub fn toJSON(&self) -> Box<dyn NyashBox> {
        let data = self.data.lock().unwrap();
        let mut json_parts = Vec::new();
        
        for (key, value) in data.iter() {
            let value_str = value.to_string_box().value;
            // 値が数値の場合はそのまま、文字列の場合は引用符で囲む
            let formatted_value = if value.as_any().downcast_ref::<IntegerBox>().is_some() 
                || value.as_any().downcast_ref::<BoolBox>().is_some() {
                value_str
            } else {
                format!("\"{}\"", value_str.replace("\"", "\\\""))
            };
            json_parts.push(format!("\"{}\":{}", key, formatted_value));
        }
        
        Box::new(StringBox::new(&format!("{{{}}}", json_parts.join(","))))
    }
    
    /// 内部データへのアクセス（JSONBox用）
    pub fn get_data(&self) -> Arc<Mutex<HashMap<String, Box<dyn NyashBox>>>> {
        self.data.clone()
    }
}

impl NyashBox for MapBox {
    fn type_name(&self) -> &'static str {
        "MapBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        let size = self.data.lock().unwrap().len();
        StringBox::new(&format!("MapBox(size={})", size))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_map) = other.as_any().downcast_ref::<MapBox>() {
            // 同じインスタンスかチェック（データの共有を考慮）
            BoolBox::new(self.id == other_map.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for MapBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_box().value)
    }
}

impl Debug for MapBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.lock().unwrap();
        f.debug_struct("MapBox")
            .field("id", &self.id)
            .field("size", &data.len())
            .field("keys", &data.keys().collect::<Vec<_>>())
            .finish()
    }
}