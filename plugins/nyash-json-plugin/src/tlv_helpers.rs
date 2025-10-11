//! TLV (Type-Length-Value) serialization helpers

use crate::constants::*;

// Re-export shared TLV codec functions from hako_abi_impl
pub use hako_abi_impl::tlv::{
    read_arg_i64, read_arg_string, write_tlv_bool, write_tlv_handle, write_tlv_i64,
    write_tlv_string,
};

pub fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return E_SHORT;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    OK
}

pub fn write_u32(v: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    unsafe {
        if result.is_null() || *result_len < 4 {
            *result_len = 4;
            return E_SHORT;
        }
        let b = v.to_le_bytes();
        std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
        *result_len = 4;
    }
    OK
}

pub fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    // Align with common helpers: use tag=9 for void/host-handle-like empty
    write_tlv_result(&[(9u8, &[])], result, result_len)
}

// Removed duplicate TLV functions - now using shared codec from hako_abi_impl:
// - write_tlv_i64() -> re-exported from hako_abi_impl::tlv
// - write_tlv_bool() -> re-exported from hako_abi_impl::tlv
// - write_tlv_handle() -> re-exported from hako_abi_impl::tlv
// - write_tlv_string() -> re-exported from hako_abi_impl::tlv
// - read_arg_string() -> re-exported from hako_abi_impl::tlv
// - read_arg_i64() -> re-exported from hako_abi_impl::tlv
