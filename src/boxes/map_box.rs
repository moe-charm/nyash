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
 * - スレッドセーフ (Arc<RwLock>使用)
 * - 大量データ格納時はメモリ使用量に注意
 * - 存在しないキーの取得は null を返す
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use crate::boxes::ArrayBox;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock}; // Arc追加
use std::sync::OnceLock;

/// キーバリューストアを表すBox
pub struct MapBox {
    data: Arc<RwLock<HashMap<String, Box<dyn NyashBox>>>>, // Arc追加
    base: BoxBase,
}

impl MapBox {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())), // Arc::new追加
            base: BoxBase::new(),
        }
    }

    /// 値を設定（戻り値: null）
    pub fn set(&self, key: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        if map_trace_enabled() {
            eprintln!("[MapBox] set: key=\"{}\" value={}", key_str, value.to_string_box().value);
        }
        self.data.write().unwrap().insert(key_str.clone(), value);
        Box::new(crate::boxes::null_box::NullBox::new())
    }

    /// 値を取得（存在しないキーは null）
    pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let guard = self.data.read().unwrap();
        match guard.get(&key_str) {
            Some(value) => {
                if map_trace_enabled() {
                    eprintln!("[MapBox] get: key=\"{}\" -> hit", key_str);
                }
                // Preserve identity for plugin/user InstanceBox to keep internal fields
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if value
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                    .is_some()
                {
                    return value.share_box();
                }
                if value
                    .as_any()
                    .downcast_ref::<crate::instance_v2::InstanceBox>()
                    .is_some()
                {
                    return value.share_box();
                }
                // ArrayBox: share underlying Arc to preserve aliasing
                if value
                    .as_any()
                    .downcast_ref::<crate::boxes::array::ArrayBox>()
                    .is_some()
                {
                    return value.share_box();
                }
                value.clone_box()
            }
            None => {
                if map_trace_enabled() {
                    eprintln!("[MapBox] get: key=\"{}\" -> miss", key_str);
                }
                // Phase 15.7: missing → null（legacy suggestion/error strings removed）
                Box::new(crate::boxes::null_box::NullBox::new())
            }
        }
    }

    /// 空かどうか
    pub fn isEmpty(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(self.data.read().unwrap().is_empty()))
    }

    /// キーが存在するかチェック
    pub fn has(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let guard = self.data.read().unwrap();
        let ok = hako_core_map::has_key_str(&guard, &key_str);
        Box::new(BoolBox::new(ok))
    }

    /// キーを削除（戻り値: 削除された値 / null)
    pub fn delete(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let removed = self.data.write().unwrap().remove(&key_str);
        match removed {
            Some(v) => {
                if v.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() { return v.share_box(); }
                if v.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() { return v.share_box(); }
                if v.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() { return v.share_box(); }
                v.clone_box()
            }
            None => Box::new(crate::boxes::null_box::NullBox::new()),
        }
    }

    /// 全てのキーを取得
    pub fn keys(&self) -> Box<dyn NyashBox> {
        let guard = self.data.read().unwrap();
        let keys: Vec<String> = hako_core_map::keys_sorted_from_map_str(&guard);
        drop(guard);
        let array = ArrayBox::new();
        for key in keys.into_iter() {
            array.push(Box::new(StringBox::new(&key)));
        }
        Box::new(array)
    }

    /// 全ての値を取得
    pub fn values(&self) -> Box<dyn NyashBox> {
        let guard = self.data.read().unwrap();
        let keys: Vec<String> = hako_core_map::keys_sorted_from_map_str(&guard);
        let aligned = hako_core_map::values_for_keys_str(&keys, &guard);
        let array = ArrayBox::new();
        for opt in aligned.into_iter() {
            if let Some(v) = opt {
                let out = if v.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() { v.share_box() } else { v.clone_box() };
                array.push(out);
            }
        }
        Box::new(array)
    }

    /// サイズを取得
    pub fn size(&self) -> Box<dyn NyashBox> {
        let guard = self.data.read().unwrap();
        let n = hako_core_map::size_of_str_map(&guard);
        Box::new(IntegerBox::new(n))
    }

    /// 全てクリア（戻り値: null）
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.data.write().unwrap().clear();
        Box::new(crate::boxes::null_box::NullBox::new())
    }

    /// 各要素に対して関数を実行
    pub fn forEach(&self, _callback: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // 簡易実装：callbackの実行はスキップ
        let count = self.data.read().unwrap().len();
        Box::new(StringBox::new(&format!("Iterated over {} items", count)))
    }

    /// JSON文字列に変換
    pub fn toJSON(&self) -> Box<dyn NyashBox> {
        // Robust stringify via JSON module conversion (handles nested maps/arrays)
        let s = crate::boxes::json::stringify_any(self.clone_box());
        Box::new(StringBox::new(&s))
    }

    /// Debug: dump pretty JSON (best-effort)
    pub fn dump(&self) -> Box<dyn NyashBox> {
        let s = crate::boxes::json::stringify_any(self.clone_box());
        match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(v) => Box::new(StringBox::new(&serde_json::to_string_pretty(&v).unwrap_or(s))),
            Err(_) => Box::new(StringBox::new(&s)),
        }
    }

    /// Debug: verify common pitfalls (empty keys, whitespace, control chars)
    pub fn verify(&self) -> Box<dyn NyashBox> {
        let mut issues: Vec<String> = Vec::new();
        let data = self.data.read().unwrap();
        for k in data.keys() {
            if k.is_empty() { issues.push("empty key".to_string()); }
            if k.trim() != *k { issues.push(format!("surrounding whitespace: '{}'", k)); }
            if k.chars().any(|c| c.is_control()) { issues.push(format!("control chars: '{:?}'", k)); }
        }
        if issues.is_empty() { Box::new(StringBox::new("OK")) } else { Box::new(StringBox::new(&issues.join("; "))) }
    }

    /// Debug: basic stats (size, avg_key_len, type histogram)
    pub fn stats(&self) -> Box<dyn NyashBox> {
        let data = self.data.read().unwrap();
        let size = data.len();
        let mut sum = 0usize;
        let mut hist: HashMap<&'static str, usize> = HashMap::new();
        for (k, v) in data.iter() {
            sum += k.len();
            *hist.entry(v.type_name()).or_insert(0) += 1;
        }
        let avg = if size == 0 { 0.0 } else { (sum as f64)/(size as f64) };
        let mut bins: Vec<String> = hist.iter().map(|(t,c)| format!("{}:{}", t, c)).collect();
        bins.sort();
        let msg = format!("size={} avg_key_len={:.2} types=[{}]", size, avg, bins.join(","));
        Box::new(StringBox::new(&msg))
    }

    /// 内部データへのアクセス（JSONBox用）
    pub fn get_data(&self) -> &RwLock<HashMap<String, Box<dyn NyashBox>>> {
        &self.data
    }
}

#[inline]
fn map_trace_enabled() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| matches!(std::env::var("HAKO_MAP_TRACE").ok().as_deref(), Some("1"|"true"|"on")))
}

// legacy suggestion (did-you-mean) removed in Phase 15.7

// Clone implementation for MapBox (needed since RwLock doesn't auto-derive Clone)
impl Clone for MapBox {
    fn clone(&self) -> Self {
        // ディープコピー（独立インスタンス）
        let data_guard = self.data.read().unwrap();
        let cloned_data: HashMap<String, Box<dyn NyashBox>> = data_guard
            .iter()
            .map(|(k, v)| (k.clone(), v.clone_box())) // 要素もディープコピー
            .collect();
        MapBox {
            data: Arc::new(RwLock::new(cloned_data)), // 新しいArc
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for MapBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let size = self.data.read().unwrap().len();
        write!(f, "MapBox(size={})", size)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for MapBox {
    fn is_identity(&self) -> bool {
        true
    }
    fn type_name(&self) -> &'static str {
        "MapBox"
    }

    fn to_string_box(&self) -> StringBox {
        let size = self.data.read().unwrap().len();
        StringBox::new(&format!("MapBox(size={})", size))
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 🎯 状態共有の核心実装
    fn share_box(&self) -> Box<dyn NyashBox> {
        let new_instance = MapBox {
            data: Arc::clone(&self.data), // Arcクローンで状態共有
            base: BoxBase::new(),         // 新しいID
        };
        Box::new(new_instance)
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_map) = other.as_any().downcast_ref::<MapBox>() {
            // 同じインスタンスかチェック（データの共有を考慮）
            BoolBox::new(self.box_id() == other_map.box_id())
        } else {
            BoolBox::new(false)
        }
    }
}

impl Display for MapBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

impl Debug for MapBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read().unwrap();
        f.debug_struct("MapBox")
            .field("id", &self.base.id)
            .field("size", &data.len())
            .field("keys", &data.keys().collect::<Vec<_>>())
            .finish()
    }
}
