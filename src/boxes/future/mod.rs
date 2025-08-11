//! FutureBox 🔄 - 非同期処理基盤
// Nyashの箱システムによる非同期処理の基盤を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct NyashFutureBox {
    pub result: Arc<Mutex<Option<Box<dyn NyashBox>>>>,
    pub is_ready: Arc<Mutex<bool>>,
    base: BoxBase,
}

impl Clone for NyashFutureBox {
    fn clone(&self) -> Self {
        Self {
            result: Arc::clone(&self.result),
            is_ready: Arc::clone(&self.is_ready),
            base: BoxBase::new(), // Create a new base with unique ID for the clone
        }
    }
}

impl NyashFutureBox {
    pub fn new() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
            is_ready: Arc::new(Mutex::new(false)),
            base: BoxBase::new(),
        }
    }
    
    /// Set the result of the future
    pub fn set_result(&self, value: Box<dyn NyashBox>) {
        let mut result = self.result.lock().unwrap();
        *result = Some(value);
        let mut ready = self.is_ready.lock().unwrap();
        *ready = true;
    }
    
    /// Get the result (blocks until ready)
    pub fn get(&self) -> Box<dyn NyashBox> {
        // Simple busy wait (could be improved with condvar)
        loop {
            let ready = self.is_ready.lock().unwrap();
            if *ready {
                break;
            }
            drop(ready);
            std::thread::yield_now();
        }
        
        let result = self.result.lock().unwrap();
        result.as_ref().unwrap().clone_box()
    }
    
    /// Check if the future is ready
    pub fn ready(&self) -> bool {
        *self.is_ready.lock().unwrap()
    }
}

impl NyashBox for NyashFutureBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let ready = *self.is_ready.lock().unwrap();
        if ready {
            let result = self.result.lock().unwrap();
            if let Some(value) = result.as_ref() {
                StringBox::new(format!("Future(ready: {})", value.to_string_box().value))
            } else {
                StringBox::new("Future(ready: void)".to_string())
            }
        } else {
            StringBox::new("Future(pending)".to_string())
        }
    }


    fn type_name(&self) -> &'static str {
        "NyashFutureBox"
    }


    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_future) = other.as_any().downcast_ref::<NyashFutureBox>() {
            BoolBox::new(self.base.id == other_future.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for NyashFutureBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ready = *self.is_ready.lock().unwrap();
        if ready {
            let result = self.result.lock().unwrap();
            if let Some(value) = result.as_ref() {
                write!(f, "Future(ready: {})", value.to_string_box().value)
            } else {
                write!(f, "Future(ready: void)")
            }
        } else {
            write!(f, "Future(pending)")
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for NyashFutureBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// Export NyashFutureBox as FutureBox for consistency
pub type FutureBox = NyashFutureBox;

impl FutureBox {
    /// wait_and_get()の実装 - await演算子で使用
    pub fn wait_and_get(&self) -> Result<Box<dyn NyashBox>, String> {
        Ok(self.get())
    }
}
