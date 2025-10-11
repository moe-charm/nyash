//! Nyash IntegerBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), get(1), set(2), fini(u32::MAX)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{
    read_arg_i64, write_tlv_i64, write_tlv_handle
};

// Error codes
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

// Methods
const M_BIRTH: u32 = 0;
const M_GET: u32 = 1;
const M_SET: u32 = 2;
const M_FINI: u32 = u32::MAX;

// Assigned type id (nyash.toml must match)
const TYPE_ID_INTEGER: u32 = 14;

struct IntInstance {
    value: i64,
}

static INST: Lazy<Mutex<HashMap<u32, IntInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
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
    if type_id != TYPE_ID_INTEGER {
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
                // Optional initial value from first arg (i64/i32)
                let init = read_arg_i64(args, args_len, 0).unwrap_or(0);
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, IntInstance { value: init });
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
            M_GET => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_i64(inst.value, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_SET => {
                let v = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.value = v;
                        return write_tlv_i64(inst.value, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            _ => E_METHOD,
        }
    }
}
*/

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

extern "C" fn integer_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "get" => M_GET,
        "set" => M_SET,
        _ => 0,
    }
}

extern "C" fn integer_invoke_id(
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
                // Create new IntegerBox instance, return raw 4-byte id
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let init = read_arg_i64(args, args_len, 0).unwrap_or(0);
                eprintln!("[IntegerBox] M_BIRTH called: id={}, init={}", id, init);
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, IntInstance { value: init });
                } else {
                    return E_PLUGIN;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let b = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
                *result_len = 4;
                OK
            }
            M_FINI => {
                // Destroy IntegerBox instance
                if let Ok(mut m) = INST.lock() {
                    m.remove(&instance_id);
                    return OK;
                } else {
                    return E_PLUGIN;
                }
            }
            M_GET => {
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        return write_tlv_i64(inst.value, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_SET => {
                let v = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.value = v;
                        return write_tlv_i64(inst.value, result, result_len);
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_IntegerBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"IntegerBox\0".as_ptr() as *const c_char,
    resolve: Some(integer_resolve),
    invoke_id: Some(integer_invoke_id),
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

// Removed duplicate TLV functions - now using shared codec from hako_abi_impl:
// - write_tlv_result() -> internal to hako_abi_impl
// - write_tlv_i64() -> use hako_abi_impl::tlv::write_tlv_i64
// - write_tlv_handle() -> use hako_abi_impl::tlv::write_tlv_handle
// - read_arg_i64() -> use hako_abi_impl::tlv::read_arg_i64
