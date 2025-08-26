//! Common FFI helpers for Plugin system
//! Minimal TLV utilities extracted for unified facade usage.

/// Encode empty TLV arguments: version=1, argc=0
pub fn encode_empty_args() -> Vec<u8> { vec![1u8, 0, 0, 0] }

/// Encode TLV header with argc (no payload entries encoded here)
pub fn encode_tlv_header(argc: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&argc.to_le_bytes());
    buf
}

/// Simple helpers for common primitive returns
pub mod decode {
    /// Try to parse a u32 instance id from an output buffer (little-endian).
    pub fn instance_id(buf: &[u8]) -> Option<u32> {
        if buf.len() < 4 { return None; }
        Some(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
    }

    /// Parse TLV header from buffer; returns (tag, size, payload_slice)
    pub fn tlv_first(buf: &[u8]) -> Option<(u8, usize, &[u8])> {
        if buf.len() < 8 { return None; }
        let tag = buf[4];
        let size = u16::from_le_bytes([buf[6], buf[7]]) as usize;
        if buf.len() < 8 + size { return None; }
        Some((tag, size, &buf[8..8 + size]))
    }
    /// Decode i32 payload (size must be 4)
    pub fn i32(payload: &[u8]) -> Option<i32> {
        if payload.len() != 4 { return None; }
        let mut b = [0u8;4]; b.copy_from_slice(payload);
        Some(i32::from_le_bytes(b))
    }
    /// Decode UTF-8 string/bytes
    pub fn string(payload: &[u8]) -> String {
        String::from_utf8_lossy(payload).to_string()
    }
}

/// TLV encode helpers for primitive Nyash values
pub mod encode {
    /// tag for I32
    const TAG_I32: u8 = 2;
    /// tag for UTF-8 string
    const TAG_STRING: u8 = 6;
    /// tag for Plugin Handle (type_id + instance_id)
    const TAG_HANDLE: u8 = 8;

    /// Append an i32 TLV entry (tag=2, size=4, little-endian)
    pub fn i32(buf: &mut Vec<u8>, v: i32) {
        buf.push(TAG_I32);
        buf.push(0u8); // reserved
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a string TLV entry (tag=6, size=u16 trunc, UTF-8)
    pub fn string(buf: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        let len = core::cmp::min(bytes.len(), u16::MAX as usize);
        buf.push(TAG_STRING);
        buf.push(0u8); // reserved
        buf.extend_from_slice(&((len as u16).to_le_bytes()));
        buf.extend_from_slice(&bytes[..len]);
    }

    /// Append a plugin handle TLV entry (tag=8, size=8, type_id:u32 + instance_id:u32)
    pub fn plugin_handle(buf: &mut Vec<u8>, type_id: u32, instance_id: u32) {
        buf.push(TAG_HANDLE);
        buf.push(0u8); // reserved
        buf.extend_from_slice(&(8u16).to_le_bytes());
        buf.extend_from_slice(&type_id.to_le_bytes());
        buf.extend_from_slice(&instance_id.to_le_bytes());
    }
}
