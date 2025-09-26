//! Nyash RegexBox Plugin — TypeBox v2（compile / isMatch / find / replaceAll / split）

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// Error/status codes aligned with other plugins
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_PLUGIN: i32 = -5;
const E_HANDLE: i32 = -8;

// Methods
const M_BIRTH: u32 = 0; // birth(pattern?) -> instance
const M_COMPILE: u32 = 1; // compile(pattern) -> self (new compiled)
const M_IS_MATCH: u32 = 2; // isMatch(text) -> bool
const M_FIND: u32 = 3; // find(text) -> String (first match or empty)
const M_REPLACE_ALL: u32 = 4; // replaceAll(text, repl) -> String
const M_SPLIT: u32 = 5; // split(text, limit) -> String (joined by '\n') minimal
const M_FINI: u32 = u32::MAX; // fini()

// Assign an unused type id (see nyash.toml [box_types])
const TYPE_ID_REGEX: u32 = 52;

struct RegexInstance {
    re: Option<Regex>,
}

static INST: Lazy<Mutex<HashMap<u32, RegexInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
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
    if type_id != TYPE_ID_REGEX {
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
                // Optional pattern in arg0
                let inst = if let Some(pat) = read_arg_string(args, args_len, 0) {
                    match Regex::new(&pat) {
                        Ok(re) => RegexInstance { re: Some(re) },
                        Err(_) => RegexInstance { re: None },
                    }
                } else {
                    RegexInstance { re: None }
                };
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, inst);
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
            M_COMPILE => {
                let pat = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.re = Regex::new(&pat).ok();
                        OK
                    } else {
                        E_HANDLE
                    }
                } else {
                    E_PLUGIN
                }
            }
            M_IS_MATCH => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            return write_tlv_bool(re.is_match(&text), result, result_len);
                        } else {
                            return write_tlv_bool(false, result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_FIND => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let s = re
                                .find(&text)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_else(|| "".to_string());
                            return write_tlv_string(&s, result, result_len);
                        } else {
                            return write_tlv_string("", result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_REPLACE_ALL => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let repl = match read_arg_string(args, args_len, 1) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let out = re.replace_all(&text, repl.as_str()).to_string();
                            return write_tlv_string(&out, result, result_len);
                        } else {
                            return write_tlv_string(&text, result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_SPLIT => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let limit = read_arg_i64(args, args_len, 1).unwrap_or(0);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let mut parts: Vec<String> = if limit > 0 {
                                re.splitn(&text, limit as usize)
                                    .map(|s| s.to_string())
                                    .collect()
                            } else {
                                re.split(&text).map(|s| s.to_string()).collect()
                            };
                            let out = parts.join("\n");
                            return write_tlv_string(&out, result, result_len);
                        } else {
                            return write_tlv_string(&text, result, result_len);
                        }
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

// ===== TypeBox ABI v2 (resolve/invoke_id) =====
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
extern "C" fn regex_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "compile" => M_COMPILE,
        "isMatch" | "is_match" => M_IS_MATCH,
        "find" => M_FIND,
        "replaceAll" | "replace_all" => M_REPLACE_ALL,
        "split" => M_SPLIT,
        "birth" => M_BIRTH,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn regex_invoke_id(
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
                // mirror v1: birth may take optional pattern
                if result_len.is_null() {
                    return E_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return E_SHORT;
                }
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let inst = if let Some(pat) = read_arg_string(args, args_len, 0) {
                    match Regex::new(&pat) {
                        Ok(re) => RegexInstance { re: Some(re) },
                        Err(_) => RegexInstance { re: None },
                    }
                } else {
                    RegexInstance { re: None }
                };
                if let Ok(mut m) = INST.lock() {
                    m.insert(id, inst);
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
            M_COMPILE => {
                let pat = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(mut m) = INST.lock() {
                    if let Some(inst) = m.get_mut(&instance_id) {
                        inst.re = Regex::new(&pat).ok();
                        OK
                    } else {
                        E_HANDLE
                    }
                } else {
                    E_PLUGIN
                }
            }
            M_IS_MATCH => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            return write_tlv_bool(re.is_match(&text), result, result_len);
                        } else {
                            return write_tlv_bool(false, result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_FIND => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let s = re
                                .find(&text)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_else(|| "".to_string());
                            return write_tlv_string(&s, result, result_len);
                        } else {
                            return write_tlv_string("", result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_REPLACE_ALL => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let repl = match read_arg_string(args, args_len, 1) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let out = re.replace_all(&text, repl.as_str()).to_string();
                            return write_tlv_string(&out, result, result_len);
                        } else {
                            return write_tlv_string(&text, result, result_len);
                        }
                    } else {
                        return E_HANDLE;
                    }
                } else {
                    return E_PLUGIN;
                }
            }
            M_SPLIT => {
                let text = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return E_ARGS,
                };
                let limit = read_arg_i64(args, args_len, 1).unwrap_or(0);
                if let Ok(m) = INST.lock() {
                    if let Some(inst) = m.get(&instance_id) {
                        if let Some(re) = &inst.re {
                            let parts: Vec<String> = if limit > 0 {
                                re.splitn(&text, limit as usize)
                                    .map(|s| s.to_string())
                                    .collect()
                            } else {
                                re.split(&text).map(|s| s.to_string()).collect()
                            };
                            let out = parts.join("\n");
                            return write_tlv_string(&out, result, result_len);
                        } else {
                            return write_tlv_string(&text, result, result_len);
                        }
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
pub static nyash_typebox_RegexBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"RegexBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(regex_resolve),
    invoke_id: Some(regex_invoke_id),
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
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
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
            return E_SHORT;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    OK
}
fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len)
}
fn write_tlv_bool(v: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(1u8, &[if v { 1u8 } else { 0u8 }])], result, result_len)
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag != 3 || size != 8 {
                return None;
            }
            let mut b = [0u8; 8];
            b.copy_from_slice(&buf[off + 4..off + 12]);
            return Some(i64::from_le_bytes(b));
        }
        off += 4 + size;
    }
    None
}
fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == 6 || tag == 7 {
                let s = String::from_utf8_lossy(&buf[off + 4..off + 4 + size]).to_string();
                return Some(s);
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}
