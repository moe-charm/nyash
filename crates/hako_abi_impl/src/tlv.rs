//! TLV encoding/decoding utilities (shared by all)

use hako_abi::{
    TLV_TAG_BOOL, TLV_TAG_I64, TLV_TAG_STRING, TLV_TAG_PLUGIN_HANDLE, TLV_TAG_HOST_HANDLE,
    HAKO_SUCCESS, HAKO_E_SHORT_BUFFER, HAKO_E_INVALID_ARGS
};

/// Read i64 from TLV-encoded args at position n
pub fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // Skip header (version + argc)

    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;

        if buf.len() < off + 4 + size {
            return None;
        }

        if i == n {
            if tag != TLV_TAG_I64 || size != 8 {
                return None;
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&buf[off + 4..off + 12]);
            return Some(i64::from_le_bytes(bytes));
        }

        off += 4 + size;
    }

    None
}

/// Read bool from TLV-encoded args at position n
pub fn read_arg_bool(args: *const u8, args_len: usize, n: usize) -> Option<bool> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // Skip header (version + argc)

    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;

        if buf.len() < off + 4 + size {
            return None;
        }

        if i == n {
            if tag != TLV_TAG_BOOL || size != 1 {
                return None;
            }
            return Some(buf[off + 4] != 0);
        }

        off += 4 + size;
    }

    None
}

/// Write i64 to TLV-encoded result buffer
pub fn write_tlv_i64(val: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_I64);
    buf.push(0); // reserved
    buf.extend_from_slice(&8u16.to_le_bytes()); // size
    buf.extend_from_slice(&val.to_le_bytes());

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

/// Write bool to TLV-encoded result buffer
pub fn write_tlv_bool(val: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(9);
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_BOOL);
    buf.push(0); // reserved
    buf.extend_from_slice(&1u16.to_le_bytes()); // size = 1 byte
    buf.push(if val { 1u8 } else { 0u8 });

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

/// Read string from TLV-encoded args at position n
pub fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == TLV_TAG_STRING || tag == 7 {  // 6 or 7 for strings
                let s = &buf[off + 4..off + 4 + size];
                return Some(String::from_utf8_lossy(s).to_string());
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

/// Read plugin handle from TLV-encoded args at position n (returns (type_id, instance_id))
pub fn read_arg_handle(args: *const u8, args_len: usize, n: usize) -> Option<(u32, u32)> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == TLV_TAG_PLUGIN_HANDLE && size == 8 {
                let mut t = [0u8; 4];
                t.copy_from_slice(&buf[off + 4..off + 8]);
                let mut id = [0u8; 4];
                id.copy_from_slice(&buf[off + 8..off + 12]);
                return Some((u32::from_le_bytes(t), u32::from_le_bytes(id)));
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

/// Read host handle from TLV-encoded args at position n
pub fn read_arg_host_handle(args: *const u8, args_len: usize, n: usize) -> Option<u64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == TLV_TAG_HOST_HANDLE && size == 8 {
                let mut h = [0u8; 8];
                h.copy_from_slice(&buf[off + 4..off + 12]);
                return Some(u64::from_le_bytes(h));
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

/// Write string to TLV-encoded result buffer
pub fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(16 + s.len());
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_STRING);
    buf.push(0); // reserved
    buf.extend_from_slice(&(s.len() as u16).to_le_bytes()); // size
    buf.extend_from_slice(s.as_bytes());

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

/// Write plugin handle to TLV-encoded result buffer
pub fn write_tlv_handle(type_id: u32, instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_PLUGIN_HANDLE);
    buf.push(0); // reserved
    buf.extend_from_slice(&8u16.to_le_bytes()); // size
    buf.extend_from_slice(&type_id.to_le_bytes());
    buf.extend_from_slice(&instance_id.to_le_bytes());

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

/// Write host handle to TLV-encoded result buffer
pub fn write_tlv_host_handle(handle_id: u64, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_HOST_HANDLE);
    buf.push(0); // reserved
    buf.extend_from_slice(&8u16.to_le_bytes()); // size
    buf.extend_from_slice(&handle_id.to_le_bytes());

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i64_roundtrip() {
        // Encode
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_i64(42, buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        // Decode
        let val = read_arg_i64(buf.as_ptr(), len, 0);
        assert_eq!(val, Some(42));
    }

    #[test]
    fn test_string_roundtrip() {
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_string("hello", buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        let val = read_arg_string(buf.as_ptr(), len, 0);
        assert_eq!(val, Some("hello".to_string()));
    }

    #[test]
    fn test_handle_roundtrip() {
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_handle(12, 34, buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        let val = read_arg_handle(buf.as_ptr(), len, 0);
        assert_eq!(val, Some((12, 34)));
    }

    #[test]
    fn test_host_handle_roundtrip() {
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_host_handle(0x1234567890ABCDEF, buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        let val = read_arg_host_handle(buf.as_ptr(), len, 0);
        assert_eq!(val, Some(0x1234567890ABCDEF));
    }

    #[test]
    fn test_bool_roundtrip() {
        // Test true
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_bool(true, buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        let val = read_arg_bool(buf.as_ptr(), len, 0);
        assert_eq!(val, Some(true));

        // Test false
        let mut buf2 = [0u8; 256];
        let mut len2 = buf2.len();
        let result2 = write_tlv_bool(false, buf2.as_mut_ptr(), &mut len2);
        assert_eq!(result2, HAKO_SUCCESS);

        let val2 = read_arg_bool(buf2.as_ptr(), len2, 0);
        assert_eq!(val2, Some(false));
    }
}
