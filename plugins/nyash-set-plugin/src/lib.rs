//! Nyash SetBox Plugin — TypeBox v2 (minimal)
//! Methods: birth, add, remove, has, size, clear, toArray, fini

use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;

use hako_abi_impl::tlv::{
    read_arg_i64, read_arg_string, write_tlv_bool, write_tlv_handle, write_tlv_i64,
    write_tlv_result, write_tlv_void,
};
use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};

// ==== Error Codes (aligned with other plugins) ====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;

// ==== Method IDs ====
const METHOD_BIRTH: u32 = 0;
const METHOD_SIZE: u32 = 1; // returns TLV i64
const METHOD_HAS: u32 = 3; // args: any -> TLV bool
const METHOD_ADD: u32 = 4; // args: any -> TLV void
const METHOD_REMOVE: u32 = 6; // args: any -> TLV void
const METHOD_CLEAR: u32 = 7; // args: () -> TLV void
const METHOD_TO_ARRAY: u32 = 15; // args: () -> HostHandle(ArrayBox)
const METHOD_FINI: u32 = u32::MAX; // destructor

// ==== Type ID (configured in nyash.toml/hako.toml) ====
const TYPE_ID_SET: u32 = 15;
const TYPE_ID_ARRAY: u32 = 12;

#[derive(Clone, Debug)]
enum SetVal {
    I64(i64),
    Str(String),
}

struct SetInstance {
    s_i64: HashSet<i64>,
    s_str: HashSet<String>,
}

define_instance_storage!(SetInstance);

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

extern "C" fn setbox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => METHOD_BIRTH,
        "size" | "len" | "length" => METHOD_SIZE,
        "has" => METHOD_HAS,
        "add" => METHOD_ADD,
        "remove" => METHOD_REMOVE,
        "clear" => METHOD_CLEAR,
        "toArray" => METHOD_TO_ARRAY,
        "fini" => METHOD_FINI,
        _ => 0,
    }
}

extern "C" fn setbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            METHOD_BIRTH => {
                // Create new instance and return TLV handle (tag=8)
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                let id = allocate_instance_id();
                if let Err(e) = store_instance(
                    id,
                    SetInstance {
                        s_i64: HashSet::new(),
                        s_str: HashSet::new(),
                    },
                ) {
                    return e;
                }
                return write_tlv_handle(TYPE_ID_SET, id, result, result_len);
            }
            METHOD_SIZE => {
                return match with_instance!(instance_id, |inst: &SetInstance| {
                    let n = (inst.s_i64.len() + inst.s_str.len()) as i64;
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[set-plugin] SIZE instance_id={} n={}", instance_id, n);
                    }
                    write_tlv_i64(n, result, result_len)
                }) {
                    Ok(rc) => rc,
                    Err(e) => e,
                };
            }
            METHOD_HAS => {
                // Try i64 first, then string
                if let Some(v) = read_arg_i64(args, args_len, 0) {
                    return match with_instance!(instance_id, |inst: &SetInstance| {
                        write_tlv_bool(inst.s_i64.contains(&v), result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                if let Some(s) = read_arg_string(args, args_len, 0) {
                    return match with_instance!(instance_id, |inst: &SetInstance| {
                        write_tlv_bool(inst.s_str.contains(&s), result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                return NYB_E_INVALID_ARGS;
            }
            METHOD_ADD => {
                if let Some(v) = read_arg_i64(args, args_len, 0) {
                    return match with_instance_mut!(instance_id, |inst: &mut SetInstance| {
                        inst.s_i64.insert(v);
                        write_tlv_void(result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                if let Some(s) = read_arg_string(args, args_len, 0) {
                    return match with_instance_mut!(instance_id, |inst: &mut SetInstance| {
                        inst.s_str.insert(s);
                        write_tlv_void(result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                return NYB_E_INVALID_ARGS;
            }
            METHOD_REMOVE => {
                if let Some(v) = read_arg_i64(args, args_len, 0) {
                    return match with_instance_mut!(instance_id, |inst: &mut SetInstance| {
                        let _ = inst.s_i64.remove(&v);
                        write_tlv_void(result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                if let Some(s) = read_arg_string(args, args_len, 0) {
                    return match with_instance_mut!(instance_id, |inst: &mut SetInstance| {
                        let _ = inst.s_str.remove(&s);
                        write_tlv_void(result, result_len)
                    }) {
                        Ok(rc) => rc,
                        Err(e) => e,
                    };
                }
                return NYB_E_INVALID_ARGS;
            }
            METHOD_CLEAR => {
                return match with_instance_mut!(instance_id, |inst: &mut SetInstance| {
                    inst.s_i64.clear();
                    inst.s_str.clear();
                    write_tlv_void(result, result_len)
                }) {
                    Ok(rc) => rc,
                    Err(e) => e,
                };
            }
            METHOD_TO_ARRAY => {
                // Stage-2: create Array via host, then populate by slot 101 (set index)
                extern "C" {
                    fn nyash_array_new_h() -> i64;
                }
                extern "C" {
                    fn nyrt_host_call_slot(
                        handle: u64,
                        selector_id: u64,
                        args_ptr: *const u8,
                        args_len: usize,
                        out_ptr: *mut u8,
                        out_len: *mut usize,
                    ) -> i32;
                }
                // Collect values under lock, sorted deterministically (string order on stringified form)
                let values: Vec<SetVal> = match with_instance!(instance_id, |inst: &SetInstance| {
                    let mut buf: Vec<SetVal> =
                        Vec::with_capacity(inst.s_i64.len() + inst.s_str.len());
                    for v in inst.s_i64.iter() {
                        buf.push(SetVal::I64(*v));
                    }
                    for s in inst.s_str.iter() {
                        buf.push(SetVal::Str(s.clone()));
                    }
                    // sort by stringified representation
                    buf.sort_by(|a, b| match (a, b) {
                        (SetVal::I64(x), SetVal::I64(y)) => x.cmp(y),
                        (SetVal::Str(x), SetVal::Str(y)) => x.cmp(y),
                        (SetVal::I64(x), SetVal::Str(y)) => x.to_string().cmp(y),
                        (SetVal::Str(x), SetVal::I64(y)) => x.cmp(&y.to_string()),
                    });
                    buf
                }) {
                    Ok(v) => v,
                    Err(e) => return e,
                };

                let arr_h = unsafe { nyash_array_new_h() };
                if arr_h <= 0 {
                    return NYB_E_INVALID_ARGS;
                }
                for (i, v) in values.into_iter().enumerate() {
                    // Build TLV args: (i64 index, value)
                    let tlv = match v {
                        SetVal::I64(n) => build_tlv_i64_i64(i as i64, n),
                        SetVal::Str(s) => build_tlv_i64_string(i as i64, &s),
                    };
                    let mut out_len: usize = 32;
                    let mut out_buf: Vec<u8> = vec![0u8; out_len];
                    let _ = unsafe {
                        nyrt_host_call_slot(
                            arr_h as u64,
                            101u64,
                            tlv.as_ptr(),
                            tlv.len(),
                            out_buf.as_mut_ptr(),
                            &mut out_len,
                        )
                    };
                }
                // Return PluginHandle(ArrayBox)
                let payload_type = TYPE_ID_ARRAY.to_le_bytes();
                let payload_inst = (arr_h as u32).to_le_bytes();
                let mut payload = [0u8; 8];
                payload[..4].copy_from_slice(&payload_type);
                payload[4..].copy_from_slice(&payload_inst);
                return write_tlv_result(&[(8u8, &payload)], result, result_len);
            }
            METHOD_FINI => {
                // No resources outside instance storage
                return NYB_SUCCESS;
            }
            _ => return NYB_E_INVALID_METHOD,
        }
    }
}

// ---- TLV builders (local, for host slot calls) ----
fn build_tlv_i64_string(idx: i64, s: &str) -> Vec<u8> {
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
fn build_tlv_i64_i64(idx: i64, value: i64) -> Vec<u8> {
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

// Optional generic entrypoint (not strictly required but kept for parity)
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
    if type_id != TYPE_ID_SET {
        return NYB_E_INVALID_TYPE;
    }
    setbox_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}

#[no_mangle]
#[used]
pub static nyash_typebox_SetBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"SetBox\0".as_ptr() as *const c_char,
    resolve: Some(setbox_resolve),
    invoke_id: Some(setbox_invoke_id),
    capabilities: 0,
};

#[no_mangle]
pub extern "C" fn nyash_set_plugin_invoke_static(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    setbox_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}
