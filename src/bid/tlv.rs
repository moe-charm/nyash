use super::{BidError, BidResult, BidHandle, BidTag, BID_VERSION};
use std::mem;

/// BID-1 TLV Header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BidTlvHeader {
    pub version: u16,    // BID version (1)
    pub argc: u16,       // Argument count
}

/// TLV Entry structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TlvEntry {
    pub tag: u8,         // Type tag
    pub reserved: u8,    // Reserved for future use (0)
    pub size: u16,       // Payload size
    // Payload follows immediately after
}

/// TLV encoder for BID-1 format
pub struct TlvEncoder {
    buffer: Vec<u8>,
    entry_count: u16,
}

impl TlvEncoder {
    /// Create a new TLV encoder
    pub fn new() -> Self {
        let mut encoder = Self {
            buffer: Vec::with_capacity(256),
            entry_count: 0,
        };
        
        // Reserve space for header
        encoder.buffer.extend_from_slice(&[0; mem::size_of::<BidTlvHeader>()]);
        encoder
    }
    
    /// Encode a boolean value
    pub fn encode_bool(&mut self, value: bool) -> BidResult<()> {
        self.encode_entry(BidTag::Bool, &[if value { 1 } else { 0 }])
    }
    
    /// Encode a 32-bit integer
    pub fn encode_i32(&mut self, value: i32) -> BidResult<()> {
        self.encode_entry(BidTag::I32, &value.to_le_bytes())
    }
    
    /// Encode a 64-bit integer
    pub fn encode_i64(&mut self, value: i64) -> BidResult<()> {
        self.encode_entry(BidTag::I64, &value.to_le_bytes())
    }
    
    /// Encode a 32-bit float
    pub fn encode_f32(&mut self, value: f32) -> BidResult<()> {
        self.encode_entry(BidTag::F32, &value.to_le_bytes())
    }
    
    /// Encode a 64-bit float
    pub fn encode_f64(&mut self, value: f64) -> BidResult<()> {
        self.encode_entry(BidTag::F64, &value.to_le_bytes())
    }
    
    /// Encode a string
    pub fn encode_string(&mut self, value: &str) -> BidResult<()> {
        let bytes = value.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(BidError::invalid_args());
        }
        self.encode_entry(BidTag::String, bytes)
    }
    
    /// Encode binary data
    pub fn encode_bytes(&mut self, value: &[u8]) -> BidResult<()> {
        if value.len() > u16::MAX as usize {
            return Err(BidError::invalid_args());
        }
        self.encode_entry(BidTag::Bytes, value)
    }
    
    /// Encode a handle
    pub fn encode_handle(&mut self, handle: BidHandle) -> BidResult<()> {
        self.encode_entry(BidTag::Handle, &handle.to_u64().to_le_bytes())
    }
    
    /// Encode void (no payload)
    pub fn encode_void(&mut self) -> BidResult<()> {
        self.encode_entry(BidTag::Void, &[])
    }
    
    /// Internal: encode a TLV entry
    fn encode_entry(&mut self, tag: BidTag, payload: &[u8]) -> BidResult<()> {
        let entry = TlvEntry {
            tag: tag as u8,
            reserved: 0,
            size: payload.len() as u16,
        };
        
        // Write entry header
        self.buffer.push(entry.tag);
        self.buffer.push(entry.reserved);
        self.buffer.extend_from_slice(&entry.size.to_le_bytes());
        
        // Write payload
        self.buffer.extend_from_slice(payload);
        
        self.entry_count += 1;
        Ok(())
    }
    
    /// Finalize the encoding and return the buffer
    pub fn finish(mut self) -> Vec<u8> {
        // Update header
        let header = BidTlvHeader {
            version: BID_VERSION,
            argc: self.entry_count,
        };
        
        // Write header at the beginning
        self.buffer[0..2].copy_from_slice(&header.version.to_le_bytes());
        self.buffer[2..4].copy_from_slice(&header.argc.to_le_bytes());
        self.buffer
    }
}

/// TLV decoder for BID-1 format
pub struct TlvDecoder<'a> {
    data: &'a [u8],
    position: usize,
    header: BidTlvHeader,
}

impl<'a> TlvDecoder<'a> {
    /// Create a new TLV decoder
    pub fn new(data: &'a [u8]) -> BidResult<Self> {
        if data.len() < mem::size_of::<BidTlvHeader>() {
            return Err(BidError::invalid_args());
        }
        
        // Read header safely
        let version = u16::from_le_bytes([data[0], data[1]]);
        let argc = u16::from_le_bytes([data[2], data[3]]);
        let header = BidTlvHeader { version, argc };
        
        if header.version != BID_VERSION {
            return Err(BidError::version_mismatch());
        }
        
        Ok(Self {
            data,
            position: mem::size_of::<BidTlvHeader>(),
            header,
        })
    }
    
    /// Get the argument count
    pub fn arg_count(&self) -> u16 {
        self.header.argc
    }
    
    /// Decode the next entry
    pub fn decode_next(&mut self) -> BidResult<Option<(BidTag, &'a [u8])>> {
        if self.position >= self.data.len() {
            return Ok(None);
        }
        
        // Read entry header
        if self.position + mem::size_of::<TlvEntry>() > self.data.len() {
            return Err(BidError::invalid_args());
        }
        
        // Read entry safely
        let tag = self.data[self.position];
        let reserved = self.data[self.position + 1];
        let size = u16::from_le_bytes([
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        let entry = TlvEntry { tag, reserved, size };
        self.position += mem::size_of::<TlvEntry>();
        
        // Read payload
        let payload_end = self.position + entry.size as usize;
        if payload_end > self.data.len() {
            return Err(BidError::invalid_args());
        }
        
        let payload = &self.data[self.position..payload_end];
        self.position = payload_end;
        
        // Convert tag
        let tag = match entry.tag {
            1 => BidTag::Bool,
            2 => BidTag::I32,
            3 => BidTag::I64,
            4 => BidTag::F32,
            5 => BidTag::F64,
            6 => BidTag::String,
            7 => BidTag::Bytes,
            8 => BidTag::Handle,
            9 => BidTag::Void,
            20 => BidTag::Result,
            21 => BidTag::Option,
            22 => BidTag::Array,
            _ => return Err(BidError::invalid_type()),
        };
        
        Ok(Some((tag, payload)))
    }
    
    /// Decode a boolean from payload
    pub fn decode_bool(payload: &[u8]) -> BidResult<bool> {
        if payload.len() != 1 {
            return Err(BidError::invalid_args());
        }
        Ok(payload[0] != 0)
    }
    
    /// Decode an i32 from payload
    pub fn decode_i32(payload: &[u8]) -> BidResult<i32> {
        if payload.len() != 4 {
            return Err(BidError::invalid_args());
        }
        Ok(i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]))
    }
    
    /// Decode an i64 from payload
    pub fn decode_i64(payload: &[u8]) -> BidResult<i64> {
        if payload.len() != 8 {
            return Err(BidError::invalid_args());
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(payload);
        Ok(i64::from_le_bytes(bytes))
    }
    
    /// Decode a handle from payload
    pub fn decode_handle(payload: &[u8]) -> BidResult<BidHandle> {
        if payload.len() != 8 {
            return Err(BidError::invalid_args());
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(payload);
        Ok(BidHandle::from_u64(u64::from_le_bytes(bytes)))
    }
    
    /// Decode an f32 from payload
    pub fn decode_f32(payload: &[u8]) -> BidResult<f32> {
        if payload.len() != 4 {
            return Err(BidError::invalid_args());
        }
        Ok(f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]))
    }
    
    /// Decode an f64 from payload
    pub fn decode_f64(payload: &[u8]) -> BidResult<f64> {
        if payload.len() != 8 {
            return Err(BidError::invalid_args());
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(payload);
        Ok(f64::from_le_bytes(bytes))
    }
    
    /// Decode a string from payload
    pub fn decode_string(payload: &[u8]) -> BidResult<&str> {
        std::str::from_utf8(payload).map_err(|_| BidError::invalid_utf8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_decode_primitives() {
        let mut encoder = TlvEncoder::new();
        encoder.encode_bool(true).unwrap();
        encoder.encode_i32(42).unwrap();
        encoder.encode_i64(9876543210).unwrap();
        encoder.encode_string("Hello Nyash!").unwrap();
        
        let data = encoder.finish();
        let mut decoder = TlvDecoder::new(&data).unwrap();
        
        assert_eq!(decoder.arg_count(), 4);
        
        // Decode bool
        let (tag, payload) = decoder.decode_next().unwrap().unwrap();
        assert_eq!(tag, BidTag::Bool);
        assert_eq!(TlvDecoder::decode_bool(payload).unwrap(), true);
        
        // Decode i32
        let (tag, payload) = decoder.decode_next().unwrap().unwrap();
        assert_eq!(tag, BidTag::I32);
        assert_eq!(TlvDecoder::decode_i32(payload).unwrap(), 42);
        
        // Decode i64
        let (tag, payload) = decoder.decode_next().unwrap().unwrap();
        assert_eq!(tag, BidTag::I64);
        assert_eq!(TlvDecoder::decode_i64(payload).unwrap(), 9876543210);
        
        // Decode string
        let (tag, payload) = decoder.decode_next().unwrap().unwrap();
        assert_eq!(tag, BidTag::String);
        assert_eq!(TlvDecoder::decode_string(payload).unwrap(), "Hello Nyash!");
    }
    
    #[test]
    fn test_encode_decode_handle() {
        let mut encoder = TlvEncoder::new();
        let handle = BidHandle::new(6, 12345);
        encoder.encode_handle(handle).unwrap();
        
        let data = encoder.finish();
        let mut decoder = TlvDecoder::new(&data).unwrap();
        
        let (tag, payload) = decoder.decode_next().unwrap().unwrap();
        assert_eq!(tag, BidTag::Handle);
        assert_eq!(TlvDecoder::decode_handle(payload).unwrap(), handle);
    }
}