//! ArrayBox 📦 - 配列・リスト操作
// Nyashの箱システムによる配列・リスト操作を提供します。
// Arc<Mutex>パターンで内部可変性を実現

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ArrayBox {
    pub items: Arc<Mutex<Vec<Box<dyn NyashBox>>>>,
    id: u64,
}

impl ArrayBox {
    /// 新しいArrayBoxを作成
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
    
    /// 要素を追加
    pub fn push(&self, item: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        self.items.lock().unwrap().push(item);
        Box::new(StringBox::new("ok"))
    }
    
    /// 最後の要素を取り出す
    pub fn pop(&self) -> Box<dyn NyashBox> {
        match self.items.lock().unwrap().pop() {
            Some(item) => item,
            None => Box::new(crate::boxes::null_box::NullBox::new()),
        }
    }
    
    /// 要素数を取得
    pub fn length(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.items.lock().unwrap().len() as i64))
    }
    
    /// インデックスで要素を取得
    pub fn get(&self, index: usize) -> Option<Box<dyn NyashBox>> {
        self.items.lock().unwrap().get(index).map(|item| item.clone_box())
    }
    
    /// インデックスで要素を設定
    pub fn set(&self, index: usize, value: Box<dyn NyashBox>) -> Result<(), String> {
        let mut items = self.items.lock().unwrap();
        if index < items.len() {
            items[index] = value;
            Ok(())
        } else {
            Err(format!("Index {} out of bounds", index))
        }
    }
    
    /// 要素を削除
    pub fn remove(&self, index: usize) -> Option<Box<dyn NyashBox>> {
        let mut items = self.items.lock().unwrap();
        if index < items.len() {
            Some(items.remove(index))
        } else {
            None
        }
    }
    
    /// 指定された値のインデックスを検索
    pub fn indexOf(&self, value: &dyn NyashBox) -> Box<dyn NyashBox> {
        let items = self.items.lock().unwrap();
        for (i, item) in items.iter().enumerate() {
            if item.equals(value).value {
                return Box::new(IntegerBox::new(i as i64));
            }
        }
        Box::new(IntegerBox::new(-1))
    }
    
    /// 指定された値が含まれているか確認
    pub fn contains(&self, value: &dyn NyashBox) -> Box<dyn NyashBox> {
        let items = self.items.lock().unwrap();
        for item in items.iter() {
            if item.equals(value).value {
                return Box::new(BoolBox::new(true));
            }
        }
        Box::new(BoolBox::new(false))
    }
    
    /// 配列を空にする
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.items.lock().unwrap().clear();
        Box::new(StringBox::new("ok"))
    }
    
    /// 文字列結合
    pub fn join(&self, delimiter: &str) -> Box<dyn NyashBox> {
        let items = self.items.lock().unwrap();
        let strings: Vec<String> = items.iter()
            .map(|item| item.to_string_box().value)
            .collect();
        Box::new(StringBox::new(&strings.join(delimiter)))
    }
}

impl NyashBox for ArrayBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let items = self.items.lock().unwrap();
        let strings: Vec<String> = items.iter()
            .map(|item| item.to_string_box().value)
            .collect();
        StringBox::new(format!("[{}]", strings.join(", ")))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "ArrayBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_array) = other.as_any().downcast_ref::<ArrayBox>() {
            let self_items = self.items.lock().unwrap();
            let other_items = other_array.items.lock().unwrap();
            
            if self_items.len() != other_items.len() {
                return BoolBox::new(false);
            }
            
            for (a, b) in self_items.iter().zip(other_items.iter()) {
                if !a.equals(b.as_ref()).value {
                    return BoolBox::new(false);
                }
            }
            
            BoolBox::new(true)
        } else {
            BoolBox::new(false)
        }
    }
}