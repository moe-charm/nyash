//! ArrayBox 📦 - 配列・リスト操作
// Nyashの箱システムによる配列・リスト操作を提供します。
// RwLockパターンで内部可変性を実現（Phase 9.75-B Arc<Mutex>削除）

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox, BoxCore, BoxBase};
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::fmt::Display;

pub struct ArrayBox {
    pub items: Arc<RwLock<Vec<Box<dyn NyashBox>>>>,  // Arc追加
    base: BoxBase,
}

impl ArrayBox {
    /// 新しいArrayBoxを作成
    pub fn new() -> Self {
        ArrayBox { 
            items: Arc::new(RwLock::new(Vec::new())),  // Arc::new追加
            base: BoxBase::new(),
        }
    }
    
    /// 要素を持つArrayBoxを作成
    pub fn new_with_elements(elements: Vec<Box<dyn NyashBox>>) -> Self {
        ArrayBox { 
            items: Arc::new(RwLock::new(elements)),    // Arc::new追加
            base: BoxBase::new(),
        }
    }
    
    /// 要素を追加
    pub fn push(&self, item: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        self.items.write().unwrap().push(item);
        Box::new(StringBox::new("ok"))
    }
    
    /// 最後の要素を取り出す
    pub fn pop(&self) -> Box<dyn NyashBox> {
        match self.items.write().unwrap().pop() {
            Some(item) => item,
            None => Box::new(crate::boxes::null_box::NullBox::new()),
        }
    }
    
    /// 要素数を取得
    pub fn length(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.items.read().unwrap().len() as i64))
    }

    /// Rust向けヘルパー: 要素数をusizeで取得（テスト用）
    pub fn len(&self) -> usize {
        self.items.read().unwrap().len()
    }
    
    /// インデックスで要素を取得
    pub fn get(&self, index: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let items = self.items.read().unwrap();
            match items.get(idx) {
                Some(item) => {
                    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                    if item.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                        return item.share_box();
                    }
                    item.clone_box()
                }
                None => Box::new(crate::boxes::null_box::NullBox::new()),
            }
        } else {
            Box::new(StringBox::new("Error: get() requires integer index"))
        }
    }
    
    /// インデックスで要素を設定
    pub fn set(&self, index: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let mut items = self.items.write().unwrap();
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
    
    /// 要素を削除
    pub fn remove(&self, index: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(idx_box) = index.as_any().downcast_ref::<IntegerBox>() {
            let idx = idx_box.value as usize;
            let mut items = self.items.write().unwrap();
            if idx < items.len() {
                items.remove(idx)
            } else {
                Box::new(crate::boxes::null_box::NullBox::new())
            }
        } else {
            Box::new(StringBox::new("Error: remove() requires integer index"))
        }
    }
    
    /// 指定された値のインデックスを検索
    pub fn indexOf(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let items = self.items.read().unwrap();
        for (i, item) in items.iter().enumerate() {
            if item.equals(value.as_ref()).value {
                return Box::new(IntegerBox::new(i as i64));
            }
        }
        Box::new(IntegerBox::new(-1))
    }
    
    /// 指定された値が含まれているか確認
    pub fn contains(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let items = self.items.read().unwrap();
        for item in items.iter() {
            if item.equals(value.as_ref()).value {
                return Box::new(BoolBox::new(true));
            }
        }
        Box::new(BoolBox::new(false))
    }
    
    /// 配列を空にする
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.items.write().unwrap().clear();
        Box::new(StringBox::new("ok"))
    }
    
    /// 文字列結合
    pub fn join(&self, delimiter: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(sep_box) = delimiter.as_any().downcast_ref::<StringBox>() {
            let items = self.items.read().unwrap();
            let parts: Vec<String> = items
                .iter()
                .map(|item| item.to_string_box().value)
                .collect();
            Box::new(StringBox::new(&parts.join(&sep_box.value)))
        } else {
            Box::new(StringBox::new("Error: join() requires string separator"))
        }
    }
    
    /// 配列をソート（昇順）
    pub fn sort(&self) -> Box<dyn NyashBox> {
        let mut items = self.items.write().unwrap();
        
        // Numeric values first, then string values
        items.sort_by(|a, b| {
            use std::cmp::Ordering;
            
            // Try to compare as numbers first
            if let (Some(a_int), Some(b_int)) = (
                a.as_any().downcast_ref::<IntegerBox>(),
                b.as_any().downcast_ref::<IntegerBox>()
            ) {
                return a_int.value.cmp(&b_int.value);
            }
            
            // Try FloatBox comparison
            if let (Some(a_float), Some(b_float)) = (
                a.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>(),
                b.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>()
            ) {
                return a_float.value.partial_cmp(&b_float.value).unwrap_or(Ordering::Equal);
            }
            
            // Mixed numeric types
            if let (Some(a_int), Some(b_float)) = (
                a.as_any().downcast_ref::<IntegerBox>(),
                b.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>()
            ) {
                return (a_int.value as f64).partial_cmp(&b_float.value).unwrap_or(Ordering::Equal);
            }
            
            if let (Some(a_float), Some(b_int)) = (
                a.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>(),
                b.as_any().downcast_ref::<IntegerBox>()
            ) {
                return a_float.value.partial_cmp(&(b_int.value as f64)).unwrap_or(Ordering::Equal);
            }
            
            // Fall back to string comparison
            let a_str = a.to_string_box().value;
            let b_str = b.to_string_box().value;
            a_str.cmp(&b_str)
        });
        
        Box::new(StringBox::new("ok"))
    }
    
    /// 配列を反転
    pub fn reverse(&self) -> Box<dyn NyashBox> {
        let mut items = self.items.write().unwrap();
        items.reverse();
        Box::new(StringBox::new("ok"))
    }
    
    /// 部分配列を取得
    pub fn slice(&self, start: Box<dyn NyashBox>, end: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let items = self.items.read().unwrap();
        
        // Extract start and end indices
        let start_idx = if let Some(start_int) = start.as_any().downcast_ref::<IntegerBox>() {
            if start_int.value < 0 {
                0
            } else {
                start_int.value as usize
            }
        } else {
            return Box::new(StringBox::new("Error: slice() start index must be an integer"));
        };
        
        let end_idx = if let Some(end_int) = end.as_any().downcast_ref::<IntegerBox>() {
            if end_int.value < 0 {
                items.len()
            } else {
                (end_int.value as usize).min(items.len())
            }
        } else {
            return Box::new(StringBox::new("Error: slice() end index must be an integer"));
        };
        
        // Validate indices
        if start_idx > items.len() || start_idx > end_idx {
            return Box::new(ArrayBox::new());
        }
        
        // Create slice
        let slice_items: Vec<Box<dyn NyashBox>> = items[start_idx..end_idx]
            .iter()
            .map(|item| {
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if item.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                    return item.share_box();
                }
                item.clone_box()
            })
            .collect();
            
        Box::new(ArrayBox::new_with_elements(slice_items))
    }
}

// Clone implementation for ArrayBox (needed since RwLock doesn't auto-derive Clone)
impl Clone for ArrayBox {
    fn clone(&self) -> Self {
        // ディープコピー（独立インスタンス）
        let items_guard = self.items.read().unwrap();
        let cloned_items: Vec<Box<dyn NyashBox>> = items_guard.iter()
            .map(|item| {
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if item.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                    return item.share_box();
                }
                item.clone_box()
            })  // 要素もディープコピー（ハンドルは共有）
            .collect();
        
        ArrayBox {
            items: Arc::new(RwLock::new(cloned_items)),  // 新しいArc
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for ArrayBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let items = self.items.read().unwrap();
        let strings: Vec<String> = items.iter()
            .map(|item| item.to_string_box().value)
            .collect();
        write!(f, "[{}]", strings.join(", "))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for ArrayBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

impl NyashBox for ArrayBox {
    fn is_identity(&self) -> bool { true }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 🎯 状態共有の核心実装
    fn share_box(&self) -> Box<dyn NyashBox> {
        let new_instance = ArrayBox {
            items: Arc::clone(&self.items),  // Arcクローンで状態共有
            base: BoxBase::new(),            // 新しいID
        };
        Box::new(new_instance)
    }

    fn to_string_box(&self) -> StringBox {
        let items = self.items.read().unwrap();
        let strings: Vec<String> = items.iter()
            .map(|item| item.to_string_box().value)
            .collect();
        StringBox::new(format!("[{}]", strings.join(", ")))
    }


    fn type_name(&self) -> &'static str {
        "ArrayBox"
    }


    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_array) = other.as_any().downcast_ref::<ArrayBox>() {
            let self_items = self.items.read().unwrap();
            let other_items = other_array.items.read().unwrap();
            
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

// Debug implementation for ArrayBox
impl std::fmt::Debug for ArrayBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self.items.read().unwrap();
        f.debug_struct("ArrayBox")
            .field("id", &self.base.id)
            .field("length", &items.len())
            .finish()
    }
}
