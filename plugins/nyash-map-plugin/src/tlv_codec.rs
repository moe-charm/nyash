//! TLV (Tag-Length-Value) Codec - Plugin-Specific Helpers for MapBox
//!
//! Standard TLV functions (read_arg_*, write_tlv_*) are now imported from hako_abi_impl.
//! This module contains only MapBox-specific helper functions.

use crate::{MapVal, NYB_E_INVALID_ARGS};

// Import shared TLV functions for use by local helpers
use hako_abi_impl::tlv::{
    write_tlv_handle, write_tlv_host_handle, write_tlv_i64, write_tlv_string,
};

/// JSON escape helper
pub fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ---- Plugin-Specific Helpers ----

/// Preflight check for buffer size
pub fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() {
            return false;
        }
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return true;
        }
    }
    false
}

/// Write MapVal as TLV
pub fn write_mapval_tlv(value: &MapVal, result: *mut u8, result_len: *mut usize) -> i32 {
    match value {
        MapVal::I64(n) => write_tlv_i64(*n, result, result_len),
        MapVal::Str(s) => write_tlv_string(s, result, result_len),
        MapVal::Handle(t, i) => write_tlv_handle(*t, *i, result, result_len),
        MapVal::Host(h) => write_tlv_host_handle(*h, result, result_len),
    }
}

// Standard TLV functions (write_tlv_handle, write_tlv_i64, write_tlv_bool, write_tlv_string, write_tlv_host_handle)
// and all read_arg_* functions are now imported from hako_abi_impl::tlv

// ---- TLV Builders (for Stage-2) ----

/// Build TLV with i64 + string arguments (for Array.set via host)
pub fn build_tlv_i64_string(idx: i64, s: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + s.len());
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3,size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: string (tag=6)
    buf.push(6u8);
    buf.push(0u8);
    let len = core::cmp::min(s.as_bytes().len(), u16::MAX as usize) as u16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&s.as_bytes()[..len as usize]);
    buf
}

/// Build TLV with two i64 arguments (for Array.set via host)
pub fn build_tlv_i64_i64(idx: i64, value: i64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + 8);
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: i64 value (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&value.to_le_bytes());
    buf
}

/// Build TLV with i64 + handle arguments (for Array.set via host)
pub fn build_tlv_i64_handle(idx: i64, type_id: u32, instance_id: u32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + 8);
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: handle (tag=8, size=8: type_id u32 + instance_id u32)
    buf.push(8u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&type_id.to_le_bytes());
    buf.extend_from_slice(&instance_id.to_le_bytes());
    buf
}

/// Build TLV with i64 + host_handle arguments (for Array.set via host)
pub fn build_tlv_i64_host_handle(idx: i64, handle: u64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + 8);
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3, size=8)
    buf.push(3u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: host_handle (tag=9, size=8)
    buf.push(9u8);
    buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&handle.to_le_bytes());
    buf
}

/// Convert MapVal to string for debugging
pub fn v_to_string(v: &MapVal) -> String {
    match v {
        MapVal::I64(n) => n.to_string(),
        MapVal::Str(s) => s.clone(),
        MapVal::Handle(t, i) => format!("handle({},{})", t, i),
        MapVal::Host(_) => "host-handle".to_string(),
    }
}
