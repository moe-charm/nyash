//! Nyash MapBox Plugin - Minimal BID-FFI v1 (RO path only)
//! Methods: birth(0), size(1), get(2), has(3), fini(u32::MAX)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};

// Error codes
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// Methods
const METHOD_BIRTH: u32 = 0;
const METHOD_SIZE:  u32 = 1;
const METHOD_GET:   u32 = 2; // args: i64 key -> TLV i64
const METHOD_HAS:   u32 = 3; // args: i64 key -> TLV bool
const METHOD_SET:   u32 = 4; // args: i64 key, i64 value -> TLV i64 (size)
const METHOD_FINI:  u32 = u32::MAX;

// Type id (nyash.toml に合わせる)
const TYPE_ID_MAP: u32 = 11;

struct MapInstance { data: HashMap<i64,i64> }
static INSTANCES: Lazy<Mutex<HashMap<u32, MapInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 { 1 }

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 { NYB_SUCCESS }

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
    if type_id != TYPE_ID_MAP { return NYB_E_INVALID_TYPE; }
    unsafe {
        match method_id {
            METHOD_BIRTH => {
                if result_len.is_null() { return NYB_E_INVALID_ARGS; }
                if preflight(result, result_len, 4) { return NYB_E_SHORT_BUFFER; }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() { map.insert(id, MapInstance { data: HashMap::new() }); } else { return NYB_E_PLUGIN_ERROR; }
                std::ptr::copy_nonoverlapping(id.to_le_bytes().as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                if let Ok(mut map) = INSTANCES.lock() { map.remove(&instance_id); NYB_SUCCESS } else { NYB_E_PLUGIN_ERROR }
            }
            METHOD_SIZE => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) { write_tlv_i64(inst.data.len() as i64, result, result_len) } else { NYB_E_INVALID_HANDLE }
                } else { NYB_E_PLUGIN_ERROR }
            }
            METHOD_GET => {
                let key = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return NYB_E_INVALID_ARGS };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data.get(&key).copied() { Some(v) => write_tlv_i64(v, result, result_len), None => NYB_E_INVALID_ARGS }
                    } else { NYB_E_INVALID_HANDLE }
                } else { NYB_E_PLUGIN_ERROR }
            }
            METHOD_HAS => {
                let key = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return NYB_E_INVALID_ARGS };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) { write_tlv_bool(inst.data.contains_key(&key), result, result_len) } else { NYB_E_INVALID_HANDLE }
                } else { NYB_E_PLUGIN_ERROR }
            }
            METHOD_SET => {
                let key = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return NYB_E_INVALID_ARGS };
                let val = match read_arg_i64(args, args_len, 1) { Some(v) => v, None => return NYB_E_INVALID_ARGS };
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data.insert(key, val);
                        return write_tlv_i64(inst.data.len() as i64, result, result_len);
                    } else { NYB_E_INVALID_HANDLE }
                } else { NYB_E_PLUGIN_ERROR }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

// TLV helpers
fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() { return false; }
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return true;
        }
    }
    false
}

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() { return NYB_E_INVALID_ARGS; }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + payloads.iter().map(|(_,p)| 4 + p.len()).sum::<usize>());
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
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 { write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len) }
fn write_tlv_bool(bv: bool, result: *mut u8, result_len: *mut usize) -> i32 { let b = [if bv {1u8} else {0u8}]; write_tlv_result(&[(1u8, &b)], result, result_len) }

fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 { return None; }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if buf.len() < off + 4 + size { return None; }
        if i == n {
            if tag != 3 || size != 8 { return None; }
            let mut b = [0u8;8]; b.copy_from_slice(&buf[off+4..off+12]);
            return Some(i64::from_le_bytes(b));
        }
        off += 4 + size;
    }
    None
}
