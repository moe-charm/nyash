//! Nyash IntegerBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), get(1), set(2), fini(u32::MAX)
//!
//! ## Phase 2-1: Instance Manager Macros Applied
//! - ✅ 5 lines (INST + NEXT_ID) → 1 line (define_instance_storage!)
//! - ✅ 8 lock() blocks (5-7 lines each) → with_instance!/with_instance_mut! (1-2 lines each)
//! - Pure削減: ~35行

use std::ffi::CStr;
use std::os::raw::c_char;

// Import shared TLV codec + instance manager macros from hako_abi_impl
use hako_abi_impl::tlv::{read_arg_i64, write_tlv_handle, write_tlv_i64};
use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};

// Error codes (standard plugin error codes)
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

// Instance storage (replaces 5 lines of boilerplate)
define_instance_storage!(IntInstance);

// legacy v1 abi/init removed


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
                let id = allocate_instance_id();
                let init = read_arg_i64(args, args_len, 0).unwrap_or(0);
                eprintln!("[IntegerBox] M_BIRTH called: id={}, init={}", id, init);

                if let Err(e) = store_instance(id, IntInstance { value: init }) {
                    return e;
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
                remove_instance(instance_id);
                OK
            }
            M_GET => {
                match with_instance!(instance_id, |inst: &IntInstance| {
                    write_tlv_i64(inst.value, result, result_len)
                }) {
                    Ok(r) => r,
                    Err(e) => e,
                }
            }
            M_SET => {
                let v = match read_arg_i64(args, args_len, 0) {
                    Some(v) => v,
                    None => return E_ARGS,
                };

                match with_instance_mut!(instance_id, |inst: &mut IntInstance| {
                    inst.value = v;
                    write_tlv_i64(inst.value, result, result_len)
                }) {
                    Ok(r) => r,
                    Err(e) => e,
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
