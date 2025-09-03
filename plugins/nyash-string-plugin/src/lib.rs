//! Nyash StringBox Plugin - Minimal BID-FFI v1
//! Methods: birth(0), length(1), is_empty(2), charCodeAt(3), fini(u32::MAX)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};
use std::os::raw::c_char;
use std::ffi::CStr;

const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

const M_BIRTH: u32 = 0;
const M_LENGTH: u32 = 1;
const M_IS_EMPTY: u32 = 2;
const M_CHAR_CODE_AT: u32 = 3;
const M_CONCAT: u32 = 4;       // concat(other: String|Handle) -> Handle(new)
const M_FROM_UTF8: u32 = 5;    // fromUtf8(data: String|Bytes) -> Handle(new)
const M_TO_UTF8: u32   = 6;    // toUtf8() -> String
const M_FINI: u32 = u32::MAX;

const TYPE_ID_STRING: u32 = 13;

struct StrInstance { s: String }

static INST: Lazy<Mutex<HashMap<u32, StrInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 { 1 }

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 { OK }

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
    if type_id != TYPE_ID_STRING { return E_TYPE; }
    unsafe {
        match method_id {
            M_BIRTH => {
                if result_len.is_null() { return E_ARGS; }
                if preflight(result, result_len, 4) { return E_SHORT; }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                // Optional init from first arg (String/Bytes)
                let init = read_arg_string(args, args_len, 0).unwrap_or_else(|| String::new());
                if let Ok(mut m) = INST.lock() { m.insert(id, StrInstance { s: init }); }
                else { return E_PLUGIN; }
                let b = id.to_le_bytes(); std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4); *result_len = 4; OK
            }
            M_FINI => { if let Ok(mut m) = INST.lock() { m.remove(&instance_id); OK } else { E_PLUGIN } }
            M_LENGTH => {
                if let Ok(m) = INST.lock() { if let Some(inst) = m.get(&instance_id) { return write_tlv_i64(inst.s.len() as i64, result, result_len); } else { return E_HANDLE; } } else { return E_PLUGIN; }
            }
            M_IS_EMPTY => {
                if let Ok(m) = INST.lock() { if let Some(inst) = m.get(&instance_id) { return write_tlv_bool(inst.s.is_empty(), result, result_len); } else { return E_HANDLE; } } else { return E_PLUGIN; }
            }
            M_CHAR_CODE_AT => {
                let idx = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return E_ARGS };
                if idx < 0 { return E_ARGS; }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        // Interpret index as char-index into Unicode scalar values
                        let i = idx as usize;
                        let ch_opt = inst.s.chars().nth(i);
                        let code = ch_opt.map(|c| c as u32 as i64).unwrap_or(0);
                        return write_tlv_i64(code, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            M_CONCAT => {
                // Accept either Handle(tag=8) to another StringBox, or String/Bytes payload
                let (ok, rhs) = if let Some((t, inst)) = read_arg_handle(args, args_len, 0) {
                    if t != TYPE_ID_STRING { return E_TYPE; }
                    if let Ok(m) = INST.lock() { if let Some(s2) = m.get(&inst) { (true, s2.s.clone()) } else { (false, String::new()) } } else { return E_PLUGIN; }
                } else if let Some(s) = read_arg_string(args, args_len, 0) { (true, s) } else { (false, String::new()) };
                if !ok { return E_ARGS; }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let mut new_s = inst.s.clone();
                        new_s.push_str(&rhs);
                        drop(m);
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut mm) = INST.lock() { mm.insert(id, StrInstance { s: new_s }); }
                        return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            M_FROM_UTF8 => {
                // Create new instance from UTF-8 (accept String/Bytes)
                let s = if let Some(s) = read_arg_string(args, args_len, 0) { s } else { return E_ARGS; };
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() { m.insert(id, StrInstance { s }); } else { return E_PLUGIN; }
                return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
            }
            M_TO_UTF8 => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_string(&inst.s, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            _ => E_METHOD,
        }
    }
}

// ===== TypeBox FFI (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,        // 'TYBX'
    pub version: u16,        // 1
    pub struct_size: u16,    // sizeof(NyashTypeBoxFfi)
    pub name: *const c_char, // C string
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn string_resolve(name: *const c_char) -> u32 {
    if name.is_null() { return 0; }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "len" | "length" => M_LENGTH,
        "toUtf8" => M_TO_UTF8,
        "concat" => M_CONCAT,
        _ => 0,
    }
}

extern "C" fn string_invoke_id(instance_id: u32, method_id: u32, args: *const u8, args_len: usize, result: *mut u8, result_len: *mut usize) -> i32 {
    unsafe {
        match method_id {
            M_LENGTH => {
                if let Ok(m) = INST.lock() { if let Some(inst) = m.get(&instance_id) { return write_tlv_i64(inst.s.len() as i64, result, result_len); } else { return E_HANDLE; } } else { return E_PLUGIN; }
            }
            M_TO_UTF8 => {
                if let Ok(m) = INST.lock() { if let Some(inst) = m.get(&instance_id) { return write_tlv_string(&inst.s, result, result_len); } else { return E_HANDLE; } } else { return E_PLUGIN; }
            }
            M_CONCAT => {
                // support String/Bytes or StringBox handle
                let (ok, rhs) = if let Some((t, inst)) = read_arg_handle(args, args_len, 0) {
                    if t != TYPE_ID_STRING { return E_TYPE; }
                    if let Ok(m) = INST.lock() { if let Some(s2) = m.get(&inst) { (true, s2.s.clone()) } else { (false, String::new()) } } else { return E_PLUGIN; }
                } else if let Some(s) = read_arg_string(args, args_len, 0) { (true, s) } else { (false, String::new()) };
                if !ok { return E_ARGS; }
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        let mut new_s = inst.s.clone(); new_s.push_str(&rhs); drop(m);
                        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut mm) = INST.lock() { mm.insert(id, StrInstance { s: new_s }); }
                        return write_tlv_handle(TYPE_ID_STRING, id, result, result_len);
                    } else { return E_HANDLE; }
                } else { return E_PLUGIN; }
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_StringBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"StringBox\0".as_ptr() as *const c_char,
    resolve: Some(string_resolve),
    invoke_id: Some(string_invoke_id),
    capabilities: 0,
};

fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe { if result_len.is_null() { return false; } if result.is_null() || *result_len < needed { *result_len = needed; return true; } }
    false
}
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() { return E_ARGS; }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + payloads.iter().map(|(_,p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes()); buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, payload) in payloads { buf.push(*tag); buf.push(0); buf.extend_from_slice(&(payload.len() as u16).to_le_bytes()); buf.extend_from_slice(payload); }
    unsafe { let needed = buf.len(); if result.is_null() || *result_len < needed { *result_len = needed; return E_SHORT; } std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed); *result_len = needed; }
    OK
}
fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 { write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len) }
fn write_tlv_bool(v: bool, result: *mut u8, result_len: *mut usize) -> i32 { write_tlv_result(&[(1u8, &[if v {1u8} else {0u8}])], result, result_len) }
fn write_tlv_handle(type_id: u32, instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&type_id.to_le_bytes());
    payload.extend_from_slice(&instance_id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], result, result_len)
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}
fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; for i in 0..=n { if buf.len() < off + 4 { return None; } let tag = buf[off]; let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize; if buf.len() < off + 4 + size { return None; } if i == n { if tag != 3 || size != 8 { return None; } let mut b=[0u8;8]; b.copy_from_slice(&buf[off+4..off+12]); return Some(i64::from_le_bytes(b)); } off += 4 + size; }
    None
}
fn read_arg_handle(args: *const u8, args_len: usize, n: usize) -> Option<(u32,u32)> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; for i in 0..=n { if buf.len() < off + 4 { return None; } let tag = buf[off]; let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize; if buf.len() < off + 4 + size { return None; } if i == n { if tag != 8 || size != 8 { return None; } let mut t=[0u8;4]; t.copy_from_slice(&buf[off+4..off+8]); let mut id=[0u8;4]; id.copy_from_slice(&buf[off+8..off+12]); return Some((u32::from_le_bytes(t), u32::from_le_bytes(id))); } off += 4 + size; }
    None
}
fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; for i in 0..=n { if buf.len() < off + 4 { return None; } let tag = buf[off]; let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize; if buf.len() < off + 4 + size { return None; } if i == n {
        if tag == 6 || tag == 7 { let s = String::from_utf8_lossy(&buf[off+4..off+4+size]).to_string(); return Some(s); } else { return None; }
    } off += 4 + size; }
    None
}
