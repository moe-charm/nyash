//! FutureBox 🔄 - 非同期処理基盤
// Nyashの箱システムによる非同期処理の基盤を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct NyashFutureBox {
    pub result: Arc<Mutex<Option<Box<dyn NyashBox>>>>,
    pub is_ready: Arc<Mutex<bool>>,
    id: u64,
}

impl Clone for NyashFutureBox {
    fn clone(&self) -> Self {
        Self {
            result: Arc::clone(&self.result),
            is_ready: Arc::clone(&self.is_ready),
            id: self.id,
        }
    }
}

impl NyashFutureBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        Self {
            result: Arc::new(Mutex::new(None)),
            is_ready: Arc::new(Mutex::new(false)),
            id,
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "NyashFutureBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_future) = other.as_any().downcast_ref::<NyashFutureBox>() {
            BoolBox::new(self.id == other_future.id)
        } else {
            BoolBox::new(false)
        }
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
