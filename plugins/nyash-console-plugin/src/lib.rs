//! Nyash ConsoleBox Plugin — TypeBox v2
//! Provides simple stdout printing via ConsoleBox

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// ===== Error Codes (BID-1) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;

// ===== Method IDs =====
const METHOD_BIRTH: u32 = 0;
const METHOD_LOG: u32 = 1; // log(text)
const METHOD_PRINTLN: u32 = 2; // println(text)
const METHOD_FINI: u32 = u32::MAX;

// ===== Type ID =====
const TYPE_ID_CONSOLE_BOX: u32 = 5; // keep in sync with nyash.toml [box_types]

// ===== Instance management =====
struct ConsoleInstance {/* no state for now */}

use once_cell::sync::Lazy;
static INSTANCES: Lazy<Mutex<HashMap<u32, ConsoleInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

// ===== TLV helpers (minimal) =====
// TLV layout: [u16 ver=1][u16 argc][entries...]
// Entry: [u16 tag][u16 size][payload...]
fn parse_first_string(args: &[u8]) -> Result<String, ()> {
    if args.len() < 4 {
        return Err(());
    }
    let argc = u16::from_le_bytes([args[2], args[3]]) as usize;
    if argc == 0 {
        return Err(());
    }
    let mut p = 4usize;
    // first entry
    if args.len() < p + 4 {
        return Err(());
    }
    let tag = u16::from_le_bytes([args[p], args[p + 1]]);
    p += 2;
    let sz = u16::from_le_bytes([args[p], args[p + 1]]) as usize;
    p += 2;
    if tag != 6 && tag != 7 {
        // String or Bytes
        return Err(());
    }
    if args.len() < p + sz {
        return Err(());
    }
    let s = String::from_utf8_lossy(&args[p..p + sz]).to_string();
    Ok(s)
}

fn format_first_any(args: &[u8]) -> Option<String> {
    if args.len() < 4 {
        return None;
    }
    let mut p = 4usize;
    if args.len() < p + 4 {
        return None;
    }
    let tag = u16::from_le_bytes([args[p], args[p + 1]]);
    p += 2;
    let sz = u16::from_le_bytes([args[p], args[p + 1]]) as usize;
    p += 2;
    if args.len() < p + sz {
        return None;
    }
    let payload = &args[p..p + sz];
    match tag {
        1 => Some(if sz > 0 && payload[0] != 0 {
            "true".into()
        } else {
            "false".into()
        }),
        2 => {
            if sz != 4 {
                None
            } else {
                let mut b = [0u8; 4];
                b.copy_from_slice(payload);
                Some((i32::from_le_bytes(b)).to_string())
            }
        }
        3 => {
            if sz != 8 {
                None
            } else {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                Some((i64::from_le_bytes(b)).to_string())
            }
        }
        5 => {
            if sz != 8 {
                None
            } else {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                Some(f64::from_le_bytes(b).to_string())
            }
        }
        6 => std::str::from_utf8(payload).ok().map(|s| s.to_string()),
        7 => Some(format!("<bytes:{}>", sz)),
        8 => {
            if sz == 8 {
                let mut t = [0u8; 4];
                t.copy_from_slice(&payload[0..4]);
                let mut i = [0u8; 4];
                i.copy_from_slice(&payload[4..8]);
                Some(format!(
                    "<handle {}:{}>",
                    u32::from_le_bytes(t),
                    u32::from_le_bytes(i)
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

// Write TLV birth result: Handle(tag=8,size=8) with (type_id, instance_id)
unsafe fn write_tlv_birth(
    type_id: u32,
    instance_id: u32,
    out: *mut u8,
    out_len: *mut usize,
) -> i32 {
    let need = 4 + 4 + 8; // header + entry + payload
    if *out_len < need {
        *out_len = need;
        return NYB_E_SHORT_BUFFER;
    }
    let mut buf = Vec::with_capacity(need);
    // header
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    // entry: Handle
    buf.extend_from_slice(&8u16.to_le_bytes());
    buf.extend_from_slice(&8u16.to_le_bytes());
    buf.extend_from_slice(&type_id.to_le_bytes());
    buf.extend_from_slice(&instance_id.to_le_bytes());
    std::ptr::copy_nonoverlapping(buf.as_ptr(), out, need);
    *out_len = need;
    NYB_SUCCESS
}

unsafe fn write_tlv_void(out: *mut u8, out_len: *mut usize) -> i32 {
    let need = 4 + 4; // header + entry
    if *out_len < need {
        *out_len = need;
        return NYB_E_SHORT_BUFFER;
    }
    let mut buf = Vec::with_capacity(need);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&9u16.to_le_bytes()); // Void
    buf.extend_from_slice(&0u16.to_le_bytes());
    std::ptr::copy_nonoverlapping(buf.as_ptr(), out, need);
    *out_len = need;
    NYB_SUCCESS
}

// ===== Entry points =====
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
    if type_id != TYPE_ID_CONSOLE_BOX {
        return NYB_E_INVALID_TYPE;
    }
    unsafe {
        match method_id {
            METHOD_BIRTH => {
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut m) = INSTANCES.lock() {
                    m.insert(id, ConsoleInstance {});
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
                return write_tlv_birth(TYPE_ID_CONSOLE_BOX, id, result, result_len);
            }
            METHOD_FINI => {
                if let Ok(mut m) = INSTANCES.lock() {
                    m.remove(&instance_id);
                }
                return NYB_SUCCESS;
            }
            METHOD_LOG | METHOD_PRINTLN => {
                let slice = std::slice::from_raw_parts(args, args_len);
                let s = match parse_first_string(slice) {
                    Ok(s) => s,
                    Err(_) => format_first_any(slice).unwrap_or_else(|| "".to_string()),
                };
                if method_id == METHOD_LOG {
                    print!("{}", s);
                } else {
                    println!("{}", s);
                }
                return write_tlv_void(result, result_len);
            }
            _ => NYB_E_INVALID_METHOD,
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

extern "C" fn console_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "log" => METHOD_LOG,
        "println" => METHOD_PRINTLN,
        _ => 0,
    }
}

extern "C" fn console_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            METHOD_LOG | METHOD_PRINTLN => {
                let slice = std::slice::from_raw_parts(args, args_len);
                let s = match parse_first_string(slice) {
                    Ok(s) => s,
                    Err(_) => format_first_any(slice).unwrap_or_else(|| "".to_string()),
                };
                if method_id == METHOD_LOG {
                    print!("{}", s);
                } else {
                    println!("{}", s);
                }
                return write_tlv_void(result, result_len);
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_ConsoleBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ConsoleBox\0".as_ptr() as *const c_char,
    resolve: Some(console_resolve),
    invoke_id: Some(console_invoke_id),
    capabilities: 0,
};
