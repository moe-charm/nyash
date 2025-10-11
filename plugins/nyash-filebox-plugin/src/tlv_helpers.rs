//! TLV (Type-Length-Value) serialization helpers for FileBox plugin

use crate::constants::*;

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{
    write_tlv_bool, write_tlv_string, write_tlv_handle
};

pub fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes()); // argc
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
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

pub fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(TLV_TAG_VOID, &[])], result, result_len)
}

pub fn write_tlv_bytes(data: &[u8], result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(TLV_TAG_BYTES, data)], result, result_len)
}

pub fn write_tlv_i32(v: i32, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(TLV_TAG_I32, &v.to_le_bytes())], result, result_len)
}

// Deleted: write_tlv_bool - use hako_abi_impl::tlv::write_tlv_bool
// Deleted: write_tlv_string - use hako_abi_impl::tlv::write_tlv_string
// Deleted: write_tlv_handle - use hako_abi_impl::tlv::write_tlv_handle

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

pub fn tlv_parse_header(data: &[u8]) -> Result<(u16, u16, usize), ()> {
    if data.len() < 4 {
        return Err(());
    }
    let ver = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 {
        return Err(());
    }
    Ok((ver, argc, 4))
}

pub fn tlv_parse_two_strings(data: &[u8]) -> Result<(String, String), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 2 {
        return Err(());
    }
    let s1 = tlv_parse_string_at(data, &mut pos)?;
    let s2 = tlv_parse_string_at(data, &mut pos)?;
    Ok((s1, s2))
}

pub fn tlv_parse_string_at(data: &[u8], pos: &mut usize) -> Result<String, ()> {
    if data.len() < *pos + 4 {
        return Err(());
    }
    let tag = data[*pos];
    if tag != TLV_TAG_STRING {
        return Err(());
    }
    let len = u16::from_le_bytes([data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if data.len() < *pos + len {
        return Err(());
    }
    let s = String::from_utf8_lossy(&data[*pos..*pos + len]).to_string();
    *pos += len;
    Ok(s)
}

pub fn tlv_parse_handle_at(data: &[u8], pos: &mut usize) -> Result<(u32, u32), ()> {
    if data.len() < *pos + 4 {
        return Err(());
    }
    let tag = data[*pos];
    if tag != TLV_TAG_HANDLE {
        return Err(());
    }
    let len = u16::from_le_bytes([data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if len != 8 || data.len() < *pos + 8 {
        return Err(());
    }
    let type_id = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    let instance_id = u32::from_le_bytes([
        data[*pos + 4],
        data[*pos + 5],
        data[*pos + 6],
        data[*pos + 7],
    ]);
    *pos += 8;
    Ok((type_id, instance_id))
}

pub fn tlv_parse_bytes_at(data: &[u8], pos: &mut usize) -> Result<Vec<u8>, ()> {
    if data.len() < *pos + 4 {
        return Err(());
    }
    let tag = data[*pos];
    if tag != TLV_TAG_BYTES && tag != TLV_TAG_STRING {
        return Err(());
    }
    let len = u16::from_le_bytes([data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if data.len() < *pos + len {
        return Err(());
    }
    let bytes = data[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(bytes)
}

pub fn tlv_parse_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_string_at(data, &mut pos)
}

#[allow(dead_code)]
pub fn tlv_parse_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_bytes_at(data, &mut pos)
}

pub fn tlv_parse_optional_string_and_bytes(data: &[u8]) -> Result<(Option<String>, Vec<u8>), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    match argc {
        0 => Err(()),
        1 => {
            let bytes = tlv_parse_bytes_at(data, &mut pos)?;
            Ok((None, bytes))
        }
        _ => {
            let s = tlv_parse_string_at(data, &mut pos)?;
            let bytes = tlv_parse_bytes_at(data, &mut pos)?;
            Ok((Some(s), bytes))
        }
    }
}

pub fn tlv_parse_optional_string_payload(data: &[u8]) -> Result<(Option<String>, Vec<u8>), ()> {
    if let Ok(res) = tlv_parse_optional_string_and_bytes(data) {
        return Ok(res);
    }
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    match argc {
        0 => Err(()),
        1 => {
            let s = tlv_parse_string_at(data, &mut pos)?;
            Ok((None, s.into_bytes()))
        }
        _ => {
            let path = tlv_parse_string_at(data, &mut pos)?;
            let data_str = tlv_parse_string_at(data, &mut pos)?;
            Ok((Some(path), data_str.into_bytes()))
        }
    }
}

pub fn tlv_parse_handle(data: &[u8]) -> Result<(u32, u32), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    let (type_id, instance_id) = tlv_parse_handle_at(data, &mut pos)?;
    Ok((type_id, instance_id))
}

#[allow(dead_code)]
pub fn tlv_parse_one_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_string_at(data, &mut pos)
}

#[allow(dead_code)]
pub fn tlv_parse_string_and_bytes(data: &[u8]) -> Result<(String, Vec<u8>), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 2 {
        return Err(());
    }
    let s = tlv_parse_string_at(data, &mut pos)?;
    let b = tlv_parse_bytes_at(data, &mut pos)?;
    Ok((s, b))
}
