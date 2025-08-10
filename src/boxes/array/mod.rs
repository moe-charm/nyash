//! ArrayBox 📦 - 配列・リスト操作（両者一致！）
// Nyashの箱システムによる配列・リスト操作を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;

#[derive(Debug)]
pub struct ArrayBox {
    pub items: Vec<Box<dyn NyashBox>>,
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
            items: Vec::new(),
            id,
        }
    }
    
    /// 要素を追加
    pub fn push(&mut self, item: Box<dyn NyashBox>) {
        self.items.push(item);
    }
    
    /// 要素数を取得
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// 要素を取得
    pub fn get(&self, index: usize) -> Option<&Box<dyn NyashBox>> {
        self.items.get(index)
    }
    
    /// 要素を削除
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn NyashBox>> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }
}

impl NyashBox for ArrayBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        let mut new_array = ArrayBox::new();
        for item in &self.items {
            new_array.push(item.clone_box());
        }
        Box::new(new_array)
    }

    fn to_string_box(&self) -> StringBox {
        let elements: Vec<String> = self.items.iter()
            .map(|item| item.to_string_box().value)
            .collect();
        StringBox::new(format!("[{}]", elements.join(", ")))
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
            if self.items.len() != other_array.items.len() {
                return BoolBox::new(false);
            }
            for (a, b) in self.items.iter().zip(other_array.items.iter()) {
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
