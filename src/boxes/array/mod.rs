/*! 📦 ArrayBox - 配列・リスト操作Box
 * 
 * ## 📝 概要
 * 順序付きコレクションを扱うためのBox。
 * JavaScript風の配列操作APIで直感的なデータ管理が可能。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `push(item)` - 要素を末尾に追加
 * - `pop()` - 末尾の要素を削除して返す
 * - `get(index)` - 指定インデックスの要素を取得
 * - `set(index, value)` - 指定インデックスに要素を設定
 * - `length()` - 配列の長さを取得
 * - `remove(index)` - 指定インデックスの要素を削除
 * - `indexOf(item)` - 要素のインデックスを検索
 * - `contains(item)` - 要素が含まれているか確認
 * - `clear()` - すべての要素を削除
 * - `join(separator)` - 文字列として結合
 * 
 * ## 💡 使用例
 * ```nyash
 * local arr, item
 * arr = new ArrayBox()
 * 
 * // 要素の追加
 * arr.push("Apple")
 * arr.push("Banana")
 * arr.push("Cherry")
 * 
 * // 要素へのアクセス
 * print(arr.get(0))           // "Apple"
 * print(arr.length())         // 3
 * 
 * // 要素の削除
 * item = arr.pop()            // "Cherry"
 * arr.remove(0)               // "Apple"削除
 * 
 * // 文字列結合
 * print(arr.join(", "))       // "Banana"
 * ```
 * 
 * ## 🎮 実用例 - TodoList
 * ```nyash
 * static box TodoList {
 *     init { items, console }
 *     
 *     main() {
 *         me.items = new ArrayBox()
 *         me.console = new ConsoleBox()
 *         
 *         me.addTask("Nyash開発")
 *         me.addTask("ドキュメント作成")
 *         me.addTask("テスト実行")
 *         
 *         me.showTasks()
 *     }
 *     
 *     addTask(task) {
 *         me.items.push(task)
 *         me.console.log("✅ タスク追加: " + task)
 *     }
 *     
 *     showTasks() {
 *         me.console.log("=== Todo List ===")
 *         local i
 *         i = 0
 *         loop(i < me.items.length()) {
 *             me.console.log((i + 1) + ". " + me.items.get(i))
 *             i = i + 1
 *         }
 *     }
 * }
 * ```
 * 
 * ## ⚠️ 注意
 * - インデックスは0から開始
 * - 範囲外のインデックスアクセスはNullBoxを返す
 * - 異なる型の要素を混在可能（Everything is Box）
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::boxes::null_box::NullBox;
use std::any::Any;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ArrayBox {
    items: Arc<Mutex<Vec<Box<dyn NyashBox>>>>,
    id: u64,
}

impl ArrayBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        ArrayBox { 
            items: Arc::new(Mutex::new(Vec::new())),
            id,
        }
    }
    
    /// 要素を末尾に追加
    pub fn push(&self, item: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        self.items.lock().unwrap().push(item);
        Box::new(StringBox::new("ok"))
    }
    
    /// 末尾の要素を削除して返す
    pub fn pop(&self) -> Box<dyn NyashBox> {
        match self.items.lock().unwrap().pop() {
            Some(item) => item,
            None => Box::new(NullBox::new()),
        }
    }
    
    /// 要素数を取得
    pub fn length(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.items.lock().unwrap().len() as i64))
    }
    
    /// 指定インデックスの要素を取得
    pub fn get(&self, index: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let items = self.items.lock().unwrap();
            match items.get(idx) {
                Some(item) => item.clone_box(),
                None => Box::new(NullBox::new()),
            }
        } else {
            Box::new(StringBox::new("Error: get() requires integer index"))
        }
    }
    
    /// 指定インデックスに要素を設定
    pub fn set(&self, index: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let mut items = self.items.lock().unwrap();
            if idx < items.len() {
                items[idx] = value;
                Box::new(StringBox::new("ok"))
            } else {
                Box::new(StringBox::new("Error: index out of bounds"))
            }
        } else {
            Box::new(StringBox::new("Error: set() requires integer index"))
        }
    }
    
    /// 指定インデックスの要素を削除
    pub fn remove(&self, index: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let mut items = self.items.lock().unwrap();
            if idx < items.len() {
                items.remove(idx)
            } else {
                Box::new(NullBox::new())
            }
        } else {
            Box::new(StringBox::new("Error: remove() requires integer index"))
        }
    }
    
    /// 要素のインデックスを検索
    pub fn indexOf(&self, item: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let items = self.items.lock().unwrap();
        for (i, element) in items.iter().enumerate() {
            if element.equals(item.as_ref()).value {
                return Box::new(IntegerBox::new(i as i64));
            }
        }
        Box::new(IntegerBox::new(-1))
    }
    
    /// 要素が含まれているか確認
    pub fn contains(&self, item: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let items = self.items.lock().unwrap();
        for element in items.iter() {
            if element.equals(item.as_ref()).value {
                return Box::new(BoolBox::new(true));
            }
        }
        Box::new(BoolBox::new(false))
    }
    
    /// すべての要素を削除
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.items.lock().unwrap().clear();
        Box::new(StringBox::new("ok"))
    }
    
    /// 文字列として結合
    pub fn join(&self, separator: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(sep_box) = separator.as_any().downcast_ref::<StringBox>() {
            let items = self.items.lock().unwrap();
            let parts: Vec<String> = items
                .iter()
                .map(|item| item.to_string_box().value)
                .collect();
            Box::new(StringBox::new(parts.join(&sep_box.value)))
        } else {
            Box::new(StringBox::new("Error: join() requires string separator"))
        }
    }
}

impl NyashBox for ArrayBox {
    fn type_name(&self) -> &'static str {
        "ArrayBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        let items = self.items.lock().unwrap();
        let elements: Vec<String> = items
            .iter()
            .map(|item| item.to_string_box().value)
            .collect();
        StringBox::new(format!("[{}]", elements.join(", ")))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_array) = other.as_any().downcast_ref::<ArrayBox>() {
            let self_items = self.items.lock().unwrap();
            let other_items = other_array.items.lock().unwrap();
            
            if self_items.len() != other_items.len() {
                return BoolBox::new(false);
            }
            
            for (a, b) in self_items.iter().zip(other_items.iter()) {
                if !a.equals(&**b).value {
                    return BoolBox::new(false);
                }
            }
            
            BoolBox::new(true)
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

impl Display for ArrayBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self.items.lock().unwrap();
        let elements: Vec<String> = items
            .iter()
            .map(|item| item.to_string_box().value)
            .collect();
        write!(f, "[{}]", elements.join(", "))
    }
}
