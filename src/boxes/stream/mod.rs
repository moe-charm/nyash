//! StreamBox 🌊 - ストリーミング処理
// Nyashの箱システムによるストリーミング処理を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::io::{Read, Write, Result};

#[derive(Debug)]
pub struct NyashStreamBox {
    pub buffer: Vec<u8>,
    pub position: usize,
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
            buffer: Vec::new(),
            position: 0,
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
            buffer: data,
            position: 0,
            id,
        }
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let available = self.buffer.len().saturating_sub(self.position);
        let to_read = buf.len().min(available);
        
        if to_read == 0 {
            return Ok(0);
        }
        
        buf[..to_read].copy_from_slice(&self.buffer[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }
    
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(buf);
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn position(&self) -> usize {
        self.position
    }
    
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

impl NyashBox for NyashStreamBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(NyashStreamBox {
            buffer: self.buffer.clone(),
            position: self.position,
            id: self.id,
        })
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("NyashStreamBox({} bytes, pos: {})", self.buffer.len(), self.position))
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
            BoolBox::new(self.buffer == other_stream.buffer && self.position == other_stream.position)
        } else {
            BoolBox::new(false)
        }
    }
}

// Keep the original generic StreamBox for compatibility
pub struct StreamBox<R: Read, W: Write> {
    pub reader: R,
    pub writer: W,
}

impl<R: Read, W: Write> StreamBox<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        StreamBox { reader, writer }
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)
    }
}
