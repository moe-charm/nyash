//! Nyash CounterBox Plugin - BID-FFI v1 Implementation

use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};

use once_cell::sync::Lazy;

// ===== Error Codes (BID-1 alignment) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Method IDs =====
const METHOD_BIRTH: u32 = 0;  // constructor
const METHOD_INC: u32 = 1;    // increments and returns new count
const METHOD_GET: u32 = 2;    // returns current count
const METHOD_FINI: u32 = u32::MAX;  // destructor

// Assign a unique type_id for CounterBox (distinct from FileBox=6)
const TYPE_ID_COUNTER: u32 = 7;

// ===== Instance state =====
struct CounterInstance { count: i32 }

static INSTANCES: Lazy<Mutex<HashMap<u32, CounterInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
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
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if type_id != TYPE_ID_COUNTER { return NYB_E_INVALID_TYPE; }

    unsafe {
        match method_id {
            METHOD_BIRTH => {
                // Return new instance handle (u32 id)
                if result_len.is_null() { return NYB_E_INVALID_ARGS; }
                if preflight(result, result_len, 4) { return NYB_E_SHORT_BUFFER; }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(id, CounterInstance { count: 0 });
                } else { return NYB_E_PLUGIN_ERROR; }
                let bytes = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                if let Ok(mut map) = INSTANCES.lock() {
                    map.remove(&instance_id);
                    NYB_SUCCESS
                } else { NYB_E_PLUGIN_ERROR }
            }
            METHOD_INC => {
                // increments and returns new count as I32 TLV
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.count += 1;
                        let v = inst.count;
                        if preflight(result, result_len, 12) { return NYB_E_SHORT_BUFFER; }
                        return write_tlv_i32(v, result, result_len);
                    } else { return NYB_E_INVALID_HANDLE; }
                } else { return NYB_E_PLUGIN_ERROR; }
            }
            METHOD_GET => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        if preflight(result, result_len, 12) { return NYB_E_SHORT_BUFFER; }
                        return write_tlv_i32(inst.count, result, result_len);
                    } else { return NYB_E_INVALID_HANDLE; }
                } else { return NYB_E_PLUGIN_ERROR; }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

// ===== TLV helpers =====
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() { return NYB_E_INVALID_ARGS; }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + payloads.iter().map(|(_,p)| 4 + p.len()).sum::<usize>());
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

fn write_tlv_i32(v: i32, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(2u8, &v.to_le_bytes())], result, result_len)
}

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

