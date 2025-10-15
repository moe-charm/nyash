//! Nyash CounterBox Plugin - BID-FFI v1 Implementation
//!
//! ## Phase 2-1: Instance Manager Macros Applied
//! - ✅ 3 lines (INSTANCES + INSTANCE_COUNTER) → 1 line (define_instance_storage!)
//! - ✅ 6 lock() blocks → with_instance!/with_instance_mut! macros (both v1 and v2 entries)

// Import shared TLV codec + instance manager macros from hako_abi_impl
use hako_abi_impl::tlv::write_tlv_i64;
use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};

// ===== Error Codes (BID-1 alignment) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Method IDs =====
const METHOD_BIRTH: u32 = 0; // constructor
const METHOD_INC: u32 = 1; // increments and returns new count
const METHOD_GET: u32 = 2; // returns current count
const METHOD_FINI: u32 = u32::MAX; // destructor

// Assign a unique type_id for CounterBox (distinct from FileBox=6)
const TYPE_ID_COUNTER: u32 = 7;

// ===== Instance state =====
struct CounterInstance {
    count: i32,
}

// Instance storage (replaces 3 lines of boilerplate)
define_instance_storage!(CounterInstance);

// legacy v1 abi entry (kept for compatibility with host shim)
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
    if type_id != TYPE_ID_COUNTER {
        return NYB_E_INVALID_TYPE;
    }

    unsafe {
        match method_id {
            METHOD_BIRTH => {
                // Return new instance handle (u32 id)
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                if result.is_null() || *result_len < 4 {
                    *result_len = 4;
                    return NYB_E_SHORT_BUFFER;
                }

                let id = allocate_instance_id();
                if let Err(e) = store_instance(id, CounterInstance { count: 0 }) {
                    return e;
                }

                let bytes = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                remove_instance(instance_id);
                NYB_SUCCESS
            }
            METHOD_INC => {
                // increments and returns new count as I64 TLV
                match with_instance_mut!(instance_id, |inst: &mut CounterInstance| {
                    inst.count += 1;
                    write_tlv_i64(inst.count as i64, result, result_len)
                }) {
                    Ok(r) => r,
                    Err(e) => e,
                }
            }
            METHOD_GET => {
                match with_instance!(instance_id, |inst: &CounterInstance| {
                    write_tlv_i64(inst.count as i64, result, result_len)
                }) {
                    Ok(r) => r,
                    Err(e) => e,
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

// ===== Nyash ABI v2 TypeBox FFI =====
#[allow(non_camel_case_types)]
type InvokeFn = extern "C" fn(
    u32, /* instance_id */
    u32, /* method_id */
    *const u8,
    usize,
    *mut u8,
    *mut usize,
) -> i32;

#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<InvokeFn>,
    pub capabilities: u64,
}

// The FFI descriptor is immutable and contains only function pointers and a const c-string pointer.
// Mark it Sync to allow use as a shared static.
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn counter_resolve(name: *const std::os::raw::c_char) -> u32 {
    unsafe {
        if name.is_null() {
            return 0;
        }
        let s = std::ffi::CStr::from_ptr(name).to_string_lossy();
        match s.as_ref() {
            "birth" => METHOD_BIRTH,
            "inc" => METHOD_INC,
            "get" => METHOD_GET,
            "fini" => METHOD_FINI,
            _ => 0,
        }
    }
}

extern "C" fn counter_invoke(
    instance_id: u32,
    method_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => {
            // Return new instance handle (u32 id) as raw 4 bytes (not TLV)
            if result_len.is_null() {
                return NYB_E_INVALID_ARGS;
            }
            unsafe {
                if result.is_null() || *result_len < 4 {
                    *result_len = 4;
                    return NYB_E_SHORT_BUFFER;
                }
            }

            let id = allocate_instance_id();
            if let Err(e) = store_instance(id, CounterInstance { count: 0 }) {
                return e;
            }

            unsafe {
                let bytes = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
            }
            NYB_SUCCESS
        }
        METHOD_FINI => {
            remove_instance(instance_id);
            NYB_SUCCESS
        }
        METHOD_INC => {
            match with_instance_mut!(instance_id, |inst: &mut CounterInstance| {
                inst.count += 1;
                write_tlv_i64(inst.count as i64, result, result_len)
            }) {
                Ok(r) => r,
                Err(e) => e,
            }
        }
        METHOD_GET => {
            match with_instance!(instance_id, |inst: &CounterInstance| {
                write_tlv_i64(inst.count as i64, result, result_len)
            }) {
                Ok(r) => r,
                Err(e) => e,
            }
        }
        _ => NYB_E_INVALID_METHOD,
    }
}

#[no_mangle]
pub static nyash_typebox_CounterBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x5459_4258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"CounterBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(counter_resolve),
    invoke_id: Some(counter_invoke),
    capabilities: 0,
};

// ===== TLV helpers removed - now imported from hako_abi_impl::tlv =====
// Note: write_tlv_i32 is replaced with write_tlv_i64 (converting i32 to i64)
// preflight helper is no longer needed with the shared codec
