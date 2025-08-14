//! StreamBox 🌊 - ストリーミング処理
// Nyashの箱システムによるストリーミング処理を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox, BoxCore, BoxBase};
use crate::boxes::buffer::BufferBox;
use crate::boxes::array::ArrayBox;
use std::any::Any;
use std::sync::RwLock;
use std::io::{Read, Write, Result};

pub struct NyashStreamBox {
    buffer: RwLock<Vec<u8>>,
    position: RwLock<usize>,
    base: BoxBase,
}

impl NyashStreamBox {
    pub fn new() -> Self {
        NyashStreamBox {
            buffer: RwLock::new(Vec::new()),
            position: RwLock::new(0),
            base: BoxBase::new(),
        }
    }
    
    pub fn from_data(data: Vec<u8>) -> Self {
        NyashStreamBox {
            buffer: RwLock::new(data),
            position: RwLock::new(0),
            base: BoxBase::new(),
        }
    }
    
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let buffer = self.buffer.read().unwrap();
        let mut position = self.position.write().unwrap();
        
        let available = buffer.len().saturating_sub(*position);
        let to_read = buf.len().min(available);
        
        if to_read == 0 {
            return Ok(0);
        }
        
        buf[..to_read].copy_from_slice(&buffer[*position..*position + to_read]);
        *position += to_read;
        Ok(to_read)
    }
    
    pub fn write(&self, buf: &[u8]) -> Result<()> {
        let mut buffer = self.buffer.write().unwrap();
        buffer.extend_from_slice(buf);
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.buffer.read().unwrap().len()
    }
    
    pub fn position(&self) -> usize {
        *self.position.read().unwrap()
    }
    
    pub fn reset(&self) {
        *self.position.write().unwrap() = 0;
    }
    
    /// ストリームに書き込み
    pub fn stream_write(&self, data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // BufferBoxから変換
        if let Some(buffer_box) = data.as_any().downcast_ref::<BufferBox>() {
            // BufferBoxのreadAllを使用してデータ取得
            let array_data = buffer_box.readAll();
            // ArrayBoxをバイト配列に変換
            if let Some(array_box) = array_data.as_any().downcast_ref::<ArrayBox>() {
                let items = array_box.items.read().unwrap();
                let mut bytes = Vec::new();
                for item in items.iter() {
                    if let Some(int_box) = item.as_any().downcast_ref::<IntegerBox>() {
                        if int_box.value >= 0 && int_box.value <= 255 {
                            bytes.push(int_box.value as u8);
                        }
                    }
                }
                match self.write(&bytes) {
                    Ok(()) => Box::new(StringBox::new("ok")),
                    Err(e) => Box::new(StringBox::new(&format!("Error writing to stream: {}", e))),
                }
            } else {
                Box::new(StringBox::new("Error: BufferBox data is not an ArrayBox"))
            }
        } else if let Some(string_box) = data.as_any().downcast_ref::<StringBox>() {
            match self.write(string_box.value.as_bytes()) {
                Ok(()) => Box::new(StringBox::new("ok")),
                Err(e) => Box::new(StringBox::new(&format!("Error writing to stream: {}", e))),
            }
        } else {
            Box::new(StringBox::new("Error: write() requires BufferBox or StringBox"))
        }
    }
    
    /// ストリームから読み込み
    pub fn stream_read(&self, count: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(count_int) = count.as_any().downcast_ref::<IntegerBox>() {
            let count_val = count_int.value as usize;
            let mut buf = vec![0u8; count_val];
            
            match self.read(&mut buf) {
                Ok(bytes_read) => {
                    buf.truncate(bytes_read);
                    Box::new(BufferBox::from_vec(buf))
                },
                Err(e) => Box::new(StringBox::new(&format!("Error reading from stream: {}", e))),
            }
        } else {
            Box::new(StringBox::new("Error: read() requires integer count"))
        }
    }
    
    /// 現在位置を取得
    pub fn get_position(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.position() as i64))
    }
    
    /// バッファサイズを取得
    pub fn get_length(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.len() as i64))
    }
    
    /// ストリームをリセット
    pub fn stream_reset(&self) -> Box<dyn NyashBox> {
        self.reset();
        Box::new(StringBox::new("ok"))
    }
}

impl NyashBox for NyashStreamBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let buffer = self.buffer.read().unwrap();
        let position = self.position.read().unwrap();
        StringBox::new(format!("NyashStreamBox({} bytes, pos: {})", buffer.len(), *position))
    }


    fn type_name(&self) -> &'static str {
        "NyashStreamBox"
    }


    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_stream) = other.as_any().downcast_ref::<NyashStreamBox>() {
            let self_buffer = self.buffer.read().unwrap();
            let self_position = self.position.read().unwrap();
            let other_buffer = other_stream.buffer.read().unwrap();
            let other_position = other_stream.position.read().unwrap();
            BoolBox::new(*self_buffer == *other_buffer && *self_position == *other_position)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for NyashStreamBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = self.buffer.read().unwrap();
        let position = self.position.read().unwrap();
        write!(f, "NyashStreamBox({} bytes, pos: {})", buffer.len(), *position)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Clone implementation for NyashStreamBox (needed since RwLock doesn't auto-derive Clone)
impl Clone for NyashStreamBox {
    fn clone(&self) -> Self {
        let buffer = self.buffer.read().unwrap();
        let position = self.position.read().unwrap();
        NyashStreamBox {
            buffer: RwLock::new(buffer.clone()),
            position: RwLock::new(*position),
            base: BoxBase::new(),
        }
    }
}

// Debug implementation for NyashStreamBox
impl std::fmt::Debug for NyashStreamBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = self.buffer.read().unwrap();
        let position = self.position.read().unwrap();
        f.debug_struct("NyashStreamBox")
            .field("id", &self.base.id)
            .field("buffer_len", &buffer.len())
            .field("position", &position)
            .finish()
    }
}

impl std::fmt::Display for NyashStreamBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// Export NyashStreamBox as StreamBox for consistency
pub type StreamBox = NyashStreamBox;
