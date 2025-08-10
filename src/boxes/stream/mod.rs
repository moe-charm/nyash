//! StreamBox 🌊 - ストリーミング処理
// Nyashの箱システムによるストリーミング処理を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox};
use crate::boxes::buffer::BufferBox;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, Result};

#[derive(Debug, Clone)]
pub struct NyashStreamBox {
    buffer: Arc<Mutex<Vec<u8>>>,
    position: Arc<Mutex<usize>>,
    id: u64,
}

impl NyashStreamBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        NyashStreamBox {
            buffer: Arc::new(Mutex::new(Vec::new())),
            position: Arc::new(Mutex::new(0)),
            id,
        }
    }
    
    pub fn from_data(data: Vec<u8>) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        NyashStreamBox {
            buffer: Arc::new(Mutex::new(data)),
            position: Arc::new(Mutex::new(0)),
            id,
        }
    }
    
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let buffer = self.buffer.lock().unwrap();
        let mut position = self.position.lock().unwrap();
        
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
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
    
    pub fn position(&self) -> usize {
        *self.position.lock().unwrap()
    }
    
    pub fn reset(&self) {
        *self.position.lock().unwrap() = 0;
    }
    
    /// ストリームに書き込み
    pub fn stream_write(&self, data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // BufferBoxから変換
        if let Some(buffer_box) = data.as_any().downcast_ref::<BufferBox>() {
            let buffer_data = buffer_box.data.lock().unwrap();
            match self.write(&buffer_data) {
                Ok(()) => Box::new(StringBox::new("ok")),
                Err(e) => Box::new(StringBox::new(&format!("Error writing to stream: {}", e))),
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
        let buffer = self.buffer.lock().unwrap();
        let position = self.position.lock().unwrap();
        StringBox::new(format!("NyashStreamBox({} bytes, pos: {})", buffer.len(), *position))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "NyashStreamBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_stream) = other.as_any().downcast_ref::<NyashStreamBox>() {
            let self_buffer = self.buffer.lock().unwrap();
            let self_position = self.position.lock().unwrap();
            let other_buffer = other_stream.buffer.lock().unwrap();
            let other_position = other_stream.position.lock().unwrap();
            BoolBox::new(*self_buffer == *other_buffer && *self_position == *other_position)
        } else {
            BoolBox::new(false)
        }
    }
}

// Export NyashStreamBox as StreamBox for consistency
pub type StreamBox = NyashStreamBox;
