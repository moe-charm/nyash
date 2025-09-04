//! Common FFI helpers for Plugin system
//! Minimal TLV utilities extracted for unified facade usage.

use crate::box_trait::NyashBox;

/// Encode empty TLV arguments: version=1, argc=0
pub fn encode_empty_args() -> Vec<u8> { vec![1u8, 0, 0, 0] }

/// Encode TLV header with argc (no payload entries encoded here)
pub fn encode_tlv_header(argc: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&argc.to_le_bytes());
    buf
}

/// Encode a slice of NyashBox arguments into TLV buffer (v1)
/// Policy: prefer i64 numeric when coercible; otherwise UTF-8 string; otherwise to_string_box()
pub fn encode_args(args: &[Box<dyn NyashBox>]) -> Vec<u8> {
    let mut buf = encode_tlv_header(args.len() as u16);
    for a in args {
        if let Some(i) = crate::runtime::semantics::coerce_to_i64(a.as_ref()) {
            encode::i64(&mut buf, i);
        } else if let Some(s) = crate::runtime::semantics::coerce_to_string(a.as_ref()) {
            encode::string(&mut buf, &s);
        } else {
            encode::string(&mut buf, &a.to_string_box().value);
        }
    }
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
    /// Decode u64 payload (size must be 8)
    pub fn u64(payload: &[u8]) -> Option<u64> {
        if payload.len() != 8 { return None; }
        let mut b = [0u8;8]; b.copy_from_slice(payload);
        Some(u64::from_le_bytes(b))
    }
    /// Decode bool payload (size must be 1; nonzero => true)
    pub fn bool(payload: &[u8]) -> Option<bool> {
        if payload.len() != 1 { return None; }
        Some(payload[0] != 0)
    }
    /// Decode i32 payload (size must be 4)
    pub fn i32(payload: &[u8]) -> Option<i32> {
        if payload.len() != 4 { return None; }
        let mut b = [0u8;4]; b.copy_from_slice(payload);
        Some(i32::from_le_bytes(b))
    }
    /// Decode f64 payload (size must be 8)
    pub fn f64(payload: &[u8]) -> Option<f64> {
        if payload.len() != 8 { return None; }
        let mut b = [0u8;8]; b.copy_from_slice(payload);
        Some(f64::from_le_bytes(b))
    }
    /// Decode UTF-8 string/bytes
    pub fn string(payload: &[u8]) -> String {
        String::from_utf8_lossy(payload).to_string()
    }

    /// Decode plugin handle payload (type_id:u32 + instance_id:u32)
    pub fn plugin_handle(payload: &[u8]) -> Option<(u32, u32)> {
        if payload.len() != 8 { return None; }
        let mut a = [0u8;4];
        let mut b = [0u8;4];
        a.copy_from_slice(&payload[0..4]);
        b.copy_from_slice(&payload[4..8]);
        Some((u32::from_le_bytes(a), u32::from_le_bytes(b)))
    }

    /// Get nth TLV entry from a buffer with header
    pub fn tlv_nth(buf: &[u8], n: usize) -> Option<(u8, usize, &[u8])> {
        if buf.len() < 4 { return None; }
        let mut off = 4usize;
        for i in 0..=n {
            if buf.len() < off + 4 { return None; }
            let tag = buf[off];
            let _rsv = buf[off+1];
            let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
            if buf.len() < off + 4 + size { return None; }
            if i == n {
                return Some((tag, size, &buf[off+4..off+4+size]));
            }
            off += 4 + size;
        }
        None
    }
}

/// TLV encode helpers for primitive Nyash values
pub mod encode {
    /// tag for Bool
    const TAG_BOOL: u8 = 1;
    /// tag for I32
    const TAG_I32: u8 = 2;
    /// tag for I64
    const TAG_I64: u8 = 3;
    /// tag for F32
    const TAG_F32: u8 = 4;
    /// tag for F64
    const TAG_F64: u8 = 5;
    /// tag for UTF-8 string
    const TAG_STRING: u8 = 6;
    /// tag for raw bytes
    const TAG_BYTES: u8 = 7;
    /// tag for Plugin Handle (type_id + instance_id)
    const TAG_HANDLE: u8 = 8;
    /// tag for Host Handle (host-managed handle id u64)
    const TAG_HOST_HANDLE: u8 = 9;

    /// Append a bool TLV entry (tag=1, size=1)
    pub fn bool(buf: &mut Vec<u8>, v: bool) {
        buf.push(TAG_BOOL);
        buf.push(0u8);
        buf.extend_from_slice(&(1u16).to_le_bytes());
        buf.push(if v { 1u8 } else { 0u8 });
    }
    /// Append an i32 TLV entry (tag=2, size=4, little-endian)
    pub fn i32(buf: &mut Vec<u8>, v: i32) {
        buf.push(TAG_I32);
        buf.push(0u8); // reserved
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
    }
    /// Append an i64 TLV entry (tag=3, size=8)
    pub fn i64(buf: &mut Vec<u8>, v: i64) {
        buf.push(TAG_I64);
        buf.push(0u8);
        buf.extend_from_slice(&(8u16).to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
    }
    /// Append an f32 TLV entry (tag=4, size=4)
    pub fn f32(buf: &mut Vec<u8>, v: f32) {
        buf.push(TAG_F32);
        buf.push(0u8);
        buf.extend_from_slice(&(4u16).to_le_bytes());
        buf.extend_from_slice(&v.to_le_bytes());
    }
    /// Append an f64 TLV entry (tag=5, size=8)
    pub fn f64(buf: &mut Vec<u8>, v: f64) {
        buf.push(TAG_F64);
        buf.push(0u8);
        buf.extend_from_slice(&(8u16).to_le_bytes());
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
    /// Append bytes TLV (tag=7)
    pub fn bytes(buf: &mut Vec<u8>, data: &[u8]) {
        let len = core::cmp::min(data.len(), u16::MAX as usize);
        buf.push(TAG_BYTES);
        buf.push(0u8);
        buf.extend_from_slice(&(len as u16).to_le_bytes());
        buf.extend_from_slice(&data[..len]);
    }

    /// Append a plugin handle TLV entry (tag=8, size=8, type_id:u32 + instance_id:u32)
    pub fn plugin_handle(buf: &mut Vec<u8>, type_id: u32, instance_id: u32) {
        buf.push(TAG_HANDLE);
        buf.push(0u8); // reserved
        buf.extend_from_slice(&(8u16).to_le_bytes());
        buf.extend_from_slice(&type_id.to_le_bytes());
        buf.extend_from_slice(&instance_id.to_le_bytes());
    }
    /// Append a host handle TLV entry (tag=9, size=8, handle_id:u64)
    pub fn host_handle(buf: &mut Vec<u8>, handle_id: u64) {
        buf.push(TAG_HOST_HANDLE);
        buf.push(0u8);
        buf.extend_from_slice(&(8u16).to_le_bytes());
        buf.extend_from_slice(&handle_id.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::{encode, decode};

    #[test]
    fn test_encode_decode_bool() {
        let mut buf = super::encode_tlv_header(1);
        encode::bool(&mut buf, true);
        let (tag, sz, payload) = decode::tlv_first(&buf).unwrap();
        assert_eq!(tag, 1);
        assert_eq!(sz, 1);
        assert_eq!(decode::bool(payload), Some(true));
    }

    #[test]
    fn test_encode_decode_i32() {
        let mut buf = super::encode_tlv_header(1);
        encode::i32(&mut buf, 123456);
        let (tag, sz, payload) = decode::tlv_first(&buf).unwrap();
        assert_eq!(tag, 2);
        assert_eq!(sz, 4);
        assert_eq!(decode::i32(payload), Some(123456));
    }

    #[test]
    fn test_encode_decode_f64() {
        let mut buf = super::encode_tlv_header(1);
        encode::f64(&mut buf, 3.14159);
        let (tag, sz, payload) = decode::tlv_first(&buf).unwrap();
        assert_eq!(tag, 5);
        assert_eq!(sz, 8);
        let v = decode::f64(payload).unwrap();
        assert!((v - 3.14159).abs() < 1e-9);
    }

    #[test]
    fn test_encode_decode_string_and_bytes() {
        let mut buf = super::encode_tlv_header(2);
        encode::string(&mut buf, "hello");
        encode::bytes(&mut buf, &[1,2,3,4]);
        // First entry string
        let (tag, _sz, payload) = decode::tlv_nth(&buf, 0).unwrap();
        assert_eq!(tag, 6);
        assert_eq!(decode::string(payload), "hello");
        // Second entry bytes
        let (tag2, sz2, payload2) = decode::tlv_nth(&buf, 1).unwrap();
        assert_eq!(tag2, 7);
        assert_eq!(sz2, 4);
        assert_eq!(payload2, &[1,2,3,4]);
    }
}
