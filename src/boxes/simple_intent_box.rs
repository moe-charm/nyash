// シンプルなIntentBox - 最小限の実装

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SimpleIntentBox {
    id: u64,
    // ノードID -> コールバック関数のマップ
    listeners: Arc<Mutex<HashMap<String, Vec<String>>>>, // 仮実装
}

impl SimpleIntentBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        SimpleIntentBox {
            id,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl NyashBox for SimpleIntentBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("IntentBox")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_intent) = other.as_any().downcast_ref::<SimpleIntentBox>() {
            BoolBox::new(self.id == other_intent.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "IntentBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // IntentBoxは共有されるので、新しいインスタンスを作らない
        Box::new(SimpleIntentBox {
            id: self.id,
            listeners: self.listeners.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}