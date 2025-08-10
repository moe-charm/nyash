//! JSONBox 📋 - JSON解析・生成
// Nyashの箱システムによるJSON解析・生成を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use serde_json::{Value, Error};

#[derive(Debug, Clone)]
pub struct JSONBox {
    pub value: Value,
    id: u64,
}

impl JSONBox {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        let value = serde_json::from_str(s)?;
        Ok(JSONBox { value, id })
    }
    
    pub fn new(value: Value) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        JSONBox { value, id }
    }
    
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl NyashBox for JSONBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.value.to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "JSONBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_json) = other.as_any().downcast_ref::<JSONBox>() {
            BoolBox::new(self.value == other_json.value)
        } else {
            BoolBox::new(false)
        }
    }
}
