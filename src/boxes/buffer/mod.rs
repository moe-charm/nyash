//! BufferBox 📊 - バイナリデータ処理
// Nyashの箱システムによるバイナリデータ処理を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct BufferBox {
    pub data: Vec<u8>,
    id: u64,
}

impl BufferBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        BufferBox { 
            data: Vec::new(),
            id,
        }
    }
    
    pub fn from_vec(data: Vec<u8>) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        BufferBox { data, id }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl NyashBox for BufferBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("BufferBox({} bytes)", self.data.len()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "BufferBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_buffer) = other.as_any().downcast_ref::<BufferBox>() {
            BoolBox::new(self.data == other_buffer.data)
        } else {
            BoolBox::new(false)
        }
    }
}
