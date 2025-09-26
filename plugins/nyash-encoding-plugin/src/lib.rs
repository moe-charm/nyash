//! Nyash EncodingBox Plugin - UTF-8/Base64/Hex helpers

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

const M_BIRTH: u32 = 0; // constructor (stateless)
const M_TO_UTF8_BYTES: u32 = 1; // toUtf8Bytes(s) -> bytes
const M_FROM_UTF8_BYTES: u32 = 2; // fromUtf8Bytes(bytes) -> string
const M_BASE64_ENC: u32 = 3; // base64Encode(s|bytes) -> string
const M_BASE64_DEC: u32 = 4; // base64Decode(str) -> bytes
const M_HEX_ENC: u32 = 5; // hexEncode(s|bytes) -> string
const M_HEX_DEC: u32 = 6; // hexDecode(str) -> bytes
const M_FINI: u32 = u32::MAX;

// Assign an unused type id
const TYPE_ID_ENCODING: u32 = 53;

struct EncInstance; // stateless

static INST: Lazy<Mutex<HashMap<u32, EncInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// legacy v1 abi/init removed

/* legacy v1 entry removed
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if type_id != TYPE_ID_ENCODING {
        return E_TYPE;
    }
    unsafe {
        match method_id {
            M_BIRTH => {
                if result_len.is_null() {
                    return E_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, EncInstance);
                } else {
                    return E_PLUGIN;
                }
                let b = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
                *result_len = 4;
                OK
            }
            M_FINI => {
                if let Ok(mut m) = INST.lock() {
                    m.remove(&instance_id);
                    OK
                } else {
                    E_PLUGIN
                }
            }
            M_TO_UTF8_BYTES => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                write_tlv_bytes(s.as_bytes(), result, result_len)
            }
            M_FROM_UTF8_BYTES => {
                let bytes = match read_arg_bytes(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match String::from_utf8(bytes) {
                    Ok(s) => write_tlv_string(&s, result, result_len),
                    Err(_) => write_tlv_string("", result, result_len),
                }
            }
            M_BASE64_ENC => {
                if let Some(b) = read_arg_bytes(args, args_len, 0) {
                    let s = base64::encode(b);
                    return write_tlv_string(&s, result, result_len);
                }
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                let enc = base64::encode(s.as_bytes());
                write_tlv_string(&enc, result, result_len)
            }
            M_BASE64_DEC => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match base64::decode(s.as_bytes()) {
                    Ok(b) => write_tlv_bytes(&b, result, result_len),
                    Err(_) => write_tlv_bytes(&[], result, result_len),
                }
            }
            M_HEX_ENC => {
                if let Some(b) = read_arg_bytes(args, args_len, 0) {
                    let s = hex::encode(b);
                    return write_tlv_string(&s, result, result_len);
                }
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                let enc = hex::encode(s.as_bytes());
                write_tlv_string(&enc, result, result_len)
            }
            M_HEX_DEC => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match hex::decode(s.as_bytes()) {
                    Ok(b) => write_tlv_bytes(&b, result, result_len),
                    Err(_) => write_tlv_bytes(&[], result, result_len),
                }
            }
            _ => E_METHOD,
        }
    }
}
*/

// ===== TypeBox ABI v2 (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,     // 'TYBX'
    pub version: u16,     // 1
    pub struct_size: u16, // sizeof(NyashTypeBoxFfi)
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

use std::ffi::CStr;
extern "C" fn encoding_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "toUtf8Bytes" => M_TO_UTF8_BYTES,
        "fromUtf8Bytes" => M_FROM_UTF8_BYTES,
        "base64Encode" => M_BASE64_ENC,
        "base64Decode" => M_BASE64_DEC,
        "hexEncode" => M_HEX_ENC,
        "hexDecode" => M_HEX_DEC,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn encoding_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            M_BIRTH => {
                if result_len.is_null() {
                    return E_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, EncInstance);
                } else {
                    return E_PLUGIN;
                }
                let b = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
                *result_len = 4;
                OK
            }
            M_FINI => {
                if let Ok(mut m) = INST.lock() {
                    m.remove(&instance_id);
                    OK
                } else {
                    E_PLUGIN
                }
            }
            M_TO_UTF8_BYTES => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                write_tlv_bytes(s.as_bytes(), result, result_len)
            }
            M_FROM_UTF8_BYTES => {
                let bytes = match read_arg_bytes(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match String::from_utf8(bytes) {
                    Ok(s) => write_tlv_string(&s, result, result_len),
                    Err(_) => write_tlv_string("", result, result_len),
                }
            }
            M_BASE64_ENC => {
                if let Some(b) = read_arg_bytes(args, args_len, 0) {
                    let s = base64::encode(b);
                    return write_tlv_string(&s, result, result_len);
                }
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                let enc = base64::encode(s.as_bytes());
                write_tlv_string(&enc, result, result_len)
            }
            M_BASE64_DEC => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match base64::decode(s.as_bytes()) {
                    Ok(b) => write_tlv_bytes(&b, result, result_len),
                    Err(_) => write_tlv_bytes(&[], result, result_len),
                }
            }
            M_HEX_ENC => {
                if let Some(b) = read_arg_bytes(args, args_len, 0) {
                    let s = hex::encode(b);
                    return write_tlv_string(&s, result, result_len);
                }
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                let enc = hex::encode(s.as_bytes());
                write_tlv_string(&enc, result, result_len)
            }
            M_HEX_DEC => {
                let s = match read_arg_string(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                match hex::decode(s.as_bytes()) {
                    Ok(b) => write_tlv_bytes(&b, result, result_len),
                    Err(_) => write_tlv_bytes(&[], result, result_len),
                }
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_EncodingBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"EncodingBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(encoding_resolve),
    invoke_id: Some(encoding_invoke_id),
    capabilities: 0,
};

fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
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
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
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
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}
fn write_tlv_bytes(b: &[u8], result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(7u8, b)], result, result_len)
}

fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
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
            if tag == 6 || tag == 7 {
                return Some(String::from_utf8_lossy(&buf[off + 4..off + 4 + size]).to_string());
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}
fn read_arg_bytes(args: *const u8, args_len: usize, n: usize) -> Option<Vec<u8>> {
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
            if tag == 7 || tag == 6 {
                return Some(buf[off + 4..off + 4 + size].to_vec());
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}
