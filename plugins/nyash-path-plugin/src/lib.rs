//! Nyash PathBox Plugin - minimal path ops (join, dirname, basename, extname, isAbs, normalize)

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{
    read_arg_string,
    write_tlv_bool, write_tlv_string
};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Component, Path};
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

const M_BIRTH: u32 = 0; // constructor -> instance
const M_JOIN: u32 = 1; // join(base, rest) -> string
const M_DIRNAME: u32 = 2; // dirname(path) -> string
const M_BASENAME: u32 = 3; // basename(path) -> string
const M_EXTNAME: u32 = 4; // extname(path) -> string
const M_IS_ABS: u32 = 5; // isAbs(path) -> bool
const M_NORMALIZE: u32 = 6; // normalize(path) -> string
const M_FINI: u32 = u32::MAX; // fini

const TYPE_ID_PATH: u32 = 55;

struct PathInstance; // stateless

static INST: Lazy<Mutex<HashMap<u32, PathInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// legacy v1 entry points removed
/*
// legacy v1 abi/init removed
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
    if type_id != TYPE_ID_PATH {
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
                    m.insert(id, PathInstance);
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
            M_JOIN => {
                let base = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let rest = match read_arg_string(args, args_len, 1) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let joined = if base.ends_with('/') || base.ends_with('\\') {
                    format!("{}{}", base, rest)
                } else {
                    format!("{}/{}", base, rest)
                };
                write_tlv_string(&joined, result, result_len)
            }
            M_DIRNAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let d = Path::new(&p)
                    .parent()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&d, result, result_len)
            }
            M_BASENAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let b = Path::new(&p)
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&b, result, result_len)
            }
            M_EXTNAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let ext = Path::new(&p)
                    .extension()
                    .map(|x| format!(".{}", x.to_string_lossy()))
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&ext, result, result_len)
            }
            M_IS_ABS => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let abs = Path::new(&p).is_absolute() || p.contains(":\\");
                write_tlv_bool(abs, result, result_len)
            }
            M_NORMALIZE => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let norm = path_clean::PathClean::clean(Path::new(&p));
                write_tlv_string(norm.to_string_lossy().as_ref(), result, result_len)
            }
            _ => E_METHOD,
        }
    }
}
*/

// ===== TypeBox ABI (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,                      // 'TYBX'
    pub version: u16,                      // 1
    pub struct_size: u16,                  // sizeof(NyashTypeBoxFfi)
    pub name: *const std::os::raw::c_char, // C string
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn pathbox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "join" => M_JOIN,
        "dirname" => M_DIRNAME,
        "basename" => M_BASENAME,
        "extname" => M_EXTNAME,
        "isAbs" | "is_absolute" => M_IS_ABS,
        "normalize" => M_NORMALIZE,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn pathbox_invoke_id(
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
                    m.insert(id, PathInstance);
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
            M_JOIN => {
                let base = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let rest = match read_arg_string(args, args_len, 1) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let joined = if base.ends_with('/') || base.ends_with('\\') {
                    format!("{}{}", base, rest)
                } else {
                    format!("{}/{}", base, rest)
                };
                write_tlv_string(&joined, result, result_len)
            }
            M_DIRNAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let d = Path::new(&p)
                    .parent()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&d, result, result_len)
            }
            M_BASENAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let b = Path::new(&p)
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&b, result, result_len)
            }
            M_EXTNAME => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let ext = Path::new(&p)
                    .extension()
                    .map(|x| format!(".{}", x.to_string_lossy()))
                    .unwrap_or_else(|| "".to_string());
                write_tlv_string(&ext, result, result_len)
            }
            M_IS_ABS => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let abs = Path::new(&p).is_absolute() || p.contains(":\\");
                write_tlv_bool(abs, result, result_len)
            }
            M_NORMALIZE => {
                let p = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let norm = path_clean::PathClean::clean(Path::new(&p));
                write_tlv_string(norm.to_string_lossy().as_ref(), result, result_len)
            }
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_PathBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"PathBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(pathbox_resolve),
    invoke_id: Some(pathbox_invoke_id),
    capabilities: 0,
};

// TLV functions (write_tlv_result, write_tlv_bool, write_tlv_string, read_arg_string)
// now imported from hako_abi_impl

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
