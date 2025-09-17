//! Nyash Math/Time Plugin — TypeBox v2
//! MathBox: sqrt/sin/cos/round
//! TimeBox: now() -> i64 (unix seconds)

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// Error codes
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_FAIL: i32 = -5;

// Type IDs (align with nyash.toml [box_types])
const TID_MATH: u32 = 50;
const TID_TIME: u32 = 51;

// Methods
const M_BIRTH: u32 = 0;
const M_FINI: u32 = u32::MAX;
// MathBox
const M_SQRT: u32 = 1;
const M_SIN: u32 = 2;
const M_COS: u32 = 3;
const M_ROUND: u32 = 4;
// TimeBox
const T_NOW: u32 = 1;

use once_cell::sync::Lazy;
#[derive(Default)]
struct Empty;
static MATH_INST: Lazy<Mutex<HashMap<u32, Empty>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static TIME_INST: Lazy<Mutex<HashMap<u32, Empty>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static ID: AtomicU32 = AtomicU32::new(1);

// TLV helpers
mod tlv {
    pub fn header(argc: u16) -> Vec<u8> {
        let mut b = Vec::with_capacity(4);
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&argc.to_le_bytes());
        b
    }
    pub fn encode_handle(buf: &mut Vec<u8>, t: u32, i: u32) {
        buf.push(8);
        buf.push(0);
        buf.push(8);
        buf.push(0);
        buf.extend_from_slice(&t.to_le_bytes());
        buf.extend_from_slice(&i.to_le_bytes());
    }
    pub fn encode_i64(buf: &mut Vec<u8>, v: i64) {
        buf.push(3);
        buf.push(0);
        buf.push(8);
        buf.push(0);
        buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn encode_void(buf: &mut Vec<u8>) {
        buf.push(9);
        buf.push(0);
        buf.push(0);
        buf.push(0);
    }
    pub fn decode_first(args: &[u8]) -> Option<(u16, u16, usize)> {
        if args.len() < 8 {
            return None;
        }
        let argc = u16::from_le_bytes([args[2], args[3]]);
        if argc == 0 {
            return None;
        }
        let tag = u16::from_le_bytes([args[4], args[5]]);
        let sz = u16::from_le_bytes([args[6], args[7]]);
        Some((tag, sz, 8))
    }
}

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
    unsafe {
        match (type_id, method_id) {
            (TID_MATH, M_BIRTH) => birth(TID_MATH, &MATH_INST, result, result_len),
            (TID_TIME, M_BIRTH) => birth(TID_TIME, &TIME_INST, result, result_len),
            (TID_MATH, M_FINI) => fini(&MATH_INST, instance_id),
            (TID_TIME, M_FINI) => fini(&TIME_INST, instance_id),
            (TID_MATH, M_SQRT) => sqrt_call(args, args_len, result, result_len),
            (TID_MATH, M_SIN) => trig_call(args, args_len, result, result_len, true),
            (TID_MATH, M_COS) => trig_call(args, args_len, result, result_len, false),
            (TID_MATH, M_ROUND) => round_call(args, args_len, result, result_len),
            (TID_TIME, T_NOW) => now_call(result, result_len),
            (TID_MATH, _) | (TID_TIME, _) => E_METHOD,
            _ => E_TYPE,
        }
    }
}
*/

// ===== TypeBox ABI v2 (resolve/invoke_id per box) =====
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
extern "C" fn mathbox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "sqrt" => M_SQRT,
        "sin" => M_SIN,
        "cos" => M_COS,
        "round" => M_ROUND,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}
extern "C" fn timebox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "now" => T_NOW,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn mathbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            M_BIRTH => birth(TID_MATH, &MATH_INST, result, result_len),
            M_FINI => fini(&MATH_INST, instance_id),
            M_SQRT => sqrt_call(args, args_len, result, result_len),
            M_SIN => trig_call(args, args_len, result, result_len, true),
            M_COS => trig_call(args, args_len, result, result_len, false),
            M_ROUND => round_call(args, args_len, result, result_len),
            _ => E_METHOD,
        }
    }
}

extern "C" fn timebox_invoke_id(
    instance_id: u32,
    method_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            M_BIRTH => birth(TID_TIME, &TIME_INST, result, result_len),
            M_FINI => fini(&TIME_INST, instance_id),
            T_NOW => now_call(result, result_len),
            _ => E_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_MathBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"MathBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(mathbox_resolve),
    invoke_id: Some(mathbox_invoke_id),
    capabilities: 0,
};

#[no_mangle]
pub static nyash_typebox_TimeBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"TimeBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(timebox_resolve),
    invoke_id: Some(timebox_invoke_id),
    capabilities: 0,
};

unsafe fn birth<T>(
    tid: u32,
    map: &Lazy<Mutex<HashMap<u32, T>>>,
    out: *mut u8,
    out_len: *mut usize,
) -> i32
where
    T: Default,
{
    let need = 4 + 4 + 8;
    if *out_len < need {
        *out_len = need;
        return E_SHORT;
    }
    let id = ID.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut m) = map.lock() {
        m.insert(id, T::default());
    } else {
        return E_FAIL;
    }
    let mut buf = tlv::header(1);
    tlv::encode_handle(&mut buf, tid, id);
    std::ptr::copy_nonoverlapping(buf.as_ptr(), out, buf.len());
    *out_len = buf.len();
    OK
}

unsafe fn fini<T>(map: &Lazy<Mutex<HashMap<u32, T>>>, instance_id: u32) -> i32 {
    if let Ok(mut m) = map.lock() {
        m.remove(&instance_id);
        OK
    } else {
        E_FAIL
    }
}

unsafe fn sqrt_call(args: *const u8, args_len: usize, out: *mut u8, out_len: *mut usize) -> i32 {
    if args_len < 8 {
        return E_ARGS;
    }
    let a = std::slice::from_raw_parts(args, args_len);
    if let Some((tag, sz, p)) = tlv::decode_first(a) {
        if tag == 3 && sz == 8 && a.len() >= p + 8 {
            let mut b = [0u8; 8];
            b.copy_from_slice(&a[p..p + 8]);
            let x = i64::from_le_bytes(b) as f64;
            let r = x.sqrt();
            let need = 4 + 4 + 8;
            if *out_len < need {
                *out_len = need;
                return E_SHORT;
            }
            let mut buf = tlv::header(1);
            // encode f64 (tag=5)
            buf.push(5);
            buf.push(0);
            buf.push(8);
            buf.push(0);
            buf.extend_from_slice(&r.to_le_bytes());
            std::ptr::copy_nonoverlapping(buf.as_ptr(), out, buf.len());
            *out_len = buf.len();
            return OK;
        }
    }
    E_ARGS
}

unsafe fn now_call(out: *mut u8, out_len: *mut usize) -> i32 {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let need = 4 + 4 + 8;
    if *out_len < need {
        *out_len = need;
        return E_SHORT;
    }
    let mut buf = tlv::header(1);
    tlv::encode_i64(&mut buf, ts);
    std::ptr::copy_nonoverlapping(buf.as_ptr(), out, buf.len());
    *out_len = buf.len();
    OK
}

unsafe fn trig_call(
    args: *const u8,
    args_len: usize,
    out: *mut u8,
    out_len: *mut usize,
    is_sin: bool,
) -> i32 {
    if args_len < 8 {
        return E_ARGS;
    }
    let a = std::slice::from_raw_parts(args, args_len);
    if let Some((tag, sz, p)) = tlv::decode_first(a) {
        if tag == 3 && sz == 8 && a.len() >= p + 8 {
            let mut b = [0u8; 8];
            b.copy_from_slice(&a[p..p + 8]);
            let x = i64::from_le_bytes(b) as f64;
            let r = if is_sin { x.sin() } else { x.cos() };
            let need = 4 + 4 + 8;
            if *out_len < need {
                *out_len = need;
                return E_SHORT;
            }
            let mut buf = tlv::header(1);
            // encode f64 (tag=5)
            buf.push(5);
            buf.push(0);
            buf.push(8);
            buf.push(0);
            buf.extend_from_slice(&r.to_le_bytes());
            std::ptr::copy_nonoverlapping(buf.as_ptr(), out, buf.len());
            *out_len = buf.len();
            return OK;
        }
    }
    E_ARGS
}

unsafe fn round_call(args: *const u8, args_len: usize, out: *mut u8, out_len: *mut usize) -> i32 {
    if args_len < 8 {
        return E_ARGS;
    }
    let a = std::slice::from_raw_parts(args, args_len);
    if let Some((tag, sz, p)) = tlv::decode_first(a) {
        if tag == 3 && sz == 8 && a.len() >= p + 8 {
            let mut b = [0u8; 8];
            b.copy_from_slice(&a[p..p + 8]);
            let x = i64::from_le_bytes(b) as f64;
            let r = x.round();
            let need = 4 + 4 + 8;
            if *out_len < need {
                *out_len = need;
                return E_SHORT;
            }
            let mut buf = tlv::header(1);
            // encode f64 (tag=5)
            buf.push(5);
            buf.push(0);
            buf.push(8);
            buf.push(0);
            buf.extend_from_slice(&r.to_le_bytes());
            std::ptr::copy_nonoverlapping(buf.as_ptr(), out, buf.len());
            *out_len = buf.len();
            return OK;
        }
    }
    E_ARGS
}
