//! Nyash FileBox Plugin - BID-FFI v1 Implementation
//! 
//! Provides file I/O operations as a Nyash plugin

use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};
use std::io::{Read, Write, Seek, SeekFrom};

// ============ FFI Types ============

#[repr(C)]
pub struct NyashHostVtable {
    pub alloc: unsafe extern "C" fn(size: usize) -> *mut u8,
    pub free: unsafe extern "C" fn(ptr: *mut u8),
    pub wake: unsafe extern "C" fn(handle: u64),
    pub log: unsafe extern "C" fn(level: i32, msg: *const c_char),
}

#[repr(C)]
pub struct NyashMethodInfo {
    pub method_id: u32,
    pub name: *const c_char,
    pub signature: u32,
}

#[repr(C)]
pub struct NyashPluginInfo {
    pub type_id: u32,
    pub type_name: *const c_char,
    pub method_count: usize,
    pub methods: *const NyashMethodInfo,
}

// ============ Error Codes (BID-1 alignment) ============
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ============ Method IDs ============
const METHOD_BIRTH: u32 = 0;  // Constructor
const METHOD_OPEN: u32 = 1;
const METHOD_READ: u32 = 2;
const METHOD_WRITE: u32 = 3;
const METHOD_CLOSE: u32 = 4;
const METHOD_FINI: u32 = u32::MAX;  // Destructor

// ============ FileBox Instance ============
struct FileBoxInstance {
    file: Option<std::fs::File>,
    path: String,
    buffer: Option<Vec<u8>>,  // プラグインが管理するバッファ
}

// グローバルインスタンス管理（実際の実装ではより安全な方法を使用）
static mut INSTANCES: Option<Mutex<HashMap<u32, FileBoxInstance>>> = None;

// ホスト関数テーブル（初期化時に設定）
static mut HOST_VTABLE: Option<&'static NyashHostVtable> = None;

// インスタンスIDカウンタ（1開始）
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

// ============ Plugin Entry Points ============

/// ABI version
#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 {
    1  // BID-1 support
}

/// Plugin initialization
#[no_mangle]
pub extern "C" fn nyash_plugin_init(
    host: *const NyashHostVtable,
    info: *mut NyashPluginInfo,
) -> i32 {
    if host.is_null() || info.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    
    unsafe {
        HOST_VTABLE = Some(&*host);
        
        // インスタンス管理初期化
        INSTANCES = Some(Mutex::new(HashMap::new()));
        
        // Method table
        static TYPE_NAME: &[u8] = b"FileBox\0";
        
        (*info).type_id = 6;  // FileBox type ID
        (*info).type_name = TYPE_NAME.as_ptr() as *const c_char;
        
        // メソッドテーブルは動的に作成（Syncトレイト問題回避）
        static METHOD_STORAGE: &'static [[u8; 7]] = &[
            *b"birth\0\0",
            *b"open\0\0\0",
            *b"read\0\0\0",
            *b"write\0\0",
            *b"close\0\0",
            *b"fini\0\0\0",
        ];
        
        static mut METHODS: [NyashMethodInfo; 6] = [
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
            NyashMethodInfo { method_id: 0, name: ptr::null(), signature: 0 },
        ];
        
        // 初回のみメソッドテーブルを初期化
        if METHODS[0].name.is_null() {
            METHODS[0] = NyashMethodInfo {
                method_id: METHOD_BIRTH,
                name: METHOD_STORAGE[0].as_ptr() as *const c_char,
                signature: 0xBEEFCAFE,
            };
            METHODS[1] = NyashMethodInfo {
                method_id: METHOD_OPEN,
                name: METHOD_STORAGE[1].as_ptr() as *const c_char,
                signature: 0x12345678,
            };
            METHODS[2] = NyashMethodInfo {
                method_id: METHOD_READ,
                name: METHOD_STORAGE[2].as_ptr() as *const c_char,
                signature: 0x87654321,
            };
            METHODS[3] = NyashMethodInfo {
                method_id: METHOD_WRITE,
                name: METHOD_STORAGE[3].as_ptr() as *const c_char,
                signature: 0x11223344,
            };
            METHODS[4] = NyashMethodInfo {
                method_id: METHOD_CLOSE,
                name: METHOD_STORAGE[4].as_ptr() as *const c_char,
                signature: 0xABCDEF00,
            };
            METHODS[5] = NyashMethodInfo {
                method_id: METHOD_FINI,
                name: METHOD_STORAGE[5].as_ptr() as *const c_char,
                signature: 0xDEADBEEF,
            };
        }
        
        (*info).method_count = METHODS.len();
        (*info).methods = METHODS.as_ptr();
    }
    
    NYB_SUCCESS
}

/// Method invocation - 仮実装
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    _type_id: u32,
    _method_id: u32,
    _instance_id: u32,
    _args: *const u8,
    _args_len: usize,
    _result: *mut u8,
    _result_len: *mut usize,
) -> i32 {
    // 簡易実装：type_id検証、省略可能
    if _type_id != 6 {
        return NYB_E_INVALID_TYPE;
    }

    unsafe {
        match _method_id {
            METHOD_BIRTH => {
                // 引数は未使用
                let needed: usize = 4; // u32 instance_id
                if _result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                // Two-phase protocol: report required size if buffer missing/small
                if _result.is_null() {
                    *_result_len = needed;
                    return NYB_E_SHORT_BUFFER;
                }
                let buf_len = *_result_len;
                if buf_len < needed {
                    *_result_len = needed;
                    return NYB_E_SHORT_BUFFER;
                }

                // 新しいインスタンスを作成
                let instance_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Some(ref mutex) = INSTANCES {
                    if let Ok(mut map) = mutex.lock() {
                        map.insert(instance_id, FileBoxInstance {
                            file: None,
                            path: String::new(),
                            buffer: None,
                        });
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }

                // 結果バッファにinstance_idを書き込む（LE）
                let bytes = instance_id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), _result, 4);
                *_result_len = needed;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                // 指定インスタンスを解放
                if let Some(ref mutex) = INSTANCES {
                    if let Ok(mut map) = mutex.lock() {
                        map.remove(&_instance_id);
                        return NYB_SUCCESS;
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                }
                NYB_E_PLUGIN_ERROR
            }
            METHOD_OPEN => {
                // args: TLV { String path, String mode }
                let args = unsafe { std::slice::from_raw_parts(_args, _args_len) };
                match tlv_parse_two_strings(args) {
                    Ok((path, mode)) => {
                        // Preflight for Void TLV: header(4) + entry(4)
                        if preflight(_result, _result_len, 8) { return NYB_E_SHORT_BUFFER; }
                        log_info(&format!("OPEN path='{}' mode='{}'", path, mode));
                        if let Some(ref mutex) = INSTANCES {
                            if let Ok(mut map) = mutex.lock() {
                                if let Some(inst) = map.get_mut(&_instance_id) {
                                    match open_file(&mode, &path) {
                                        Ok(file) => {
                                            inst.file = Some(file);
                                            // return TLV Void
                                            return write_tlv_void(_result, _result_len);
                                        }
                                        Err(_) => return NYB_E_PLUGIN_ERROR,
                                    }
                                } else { return NYB_E_PLUGIN_ERROR; }
                            } else { return NYB_E_PLUGIN_ERROR; }
                        }
                        NYB_E_PLUGIN_ERROR
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_READ => {
                // args: TLV { I32 size }
                let args = unsafe { std::slice::from_raw_parts(_args, _args_len) };
                match tlv_parse_i32(args) {
                    Ok(sz) => {
                        // Preflight for Bytes TLV: header(4) + entry(4) + sz
                        let need = 8usize.saturating_add(sz as usize);
                        if preflight(_result, _result_len, need) { return NYB_E_SHORT_BUFFER; }
                        if let Some(ref mutex) = INSTANCES {
                            if let Ok(mut map) = mutex.lock() {
                                if let Some(inst) = map.get_mut(&_instance_id) {
                                    if let Some(file) = inst.file.as_mut() {
                                        let mut buf = vec![0u8; sz as usize];
                                        // Read from beginning for simple semantics
                                        let _ = file.seek(SeekFrom::Start(0));
                                        match file.read(&mut buf) {
                                             Ok(n) => {
                                                 buf.truncate(n);
                                                 log_info(&format!("READ {} bytes", n));
                                                 return write_tlv_bytes(&buf, _result, _result_len);
                                             }
                                            Err(_) => return NYB_E_PLUGIN_ERROR,
                                        }
                                    } else { return NYB_E_INVALID_HANDLE; }
                                } else { return NYB_E_PLUGIN_ERROR; }
                            } else { return NYB_E_PLUGIN_ERROR; }
                        }
                        NYB_E_PLUGIN_ERROR
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_WRITE => {
                // args: TLV { Bytes data }
                let args = unsafe { std::slice::from_raw_parts(_args, _args_len) };
                match tlv_parse_bytes(args) {
                    Ok(data) => {
                        // Preflight for I32 TLV: header(4) + entry(4) + 4
                        if preflight(_result, _result_len, 12) { return NYB_E_SHORT_BUFFER; }
                        if let Some(ref mutex) = INSTANCES {
                            if let Ok(mut map) = mutex.lock() {
                                if let Some(inst) = map.get_mut(&_instance_id) {
                                    if let Some(file) = inst.file.as_mut() {
                                        match file.write(&data) {
                                            Ok(n) => {
                                                // ファイルバッファをフラッシュ（重要！）
                                                if let Err(_) = file.flush() {
                                                    return NYB_E_PLUGIN_ERROR;
                                                }
                                                log_info(&format!("WRITE {} bytes", n));
                                                return write_tlv_i32(n as i32, _result, _result_len);
                                            }
                                            Err(_) => return NYB_E_PLUGIN_ERROR,
                                        }
                                    } else { return NYB_E_INVALID_HANDLE; }
                                } else { return NYB_E_PLUGIN_ERROR; }
                            } else { return NYB_E_PLUGIN_ERROR; }
                        }
                        NYB_E_PLUGIN_ERROR
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_CLOSE => {
                // Preflight for Void TLV
                if preflight(_result, _result_len, 8) { return NYB_E_SHORT_BUFFER; }
                log_info("CLOSE");
                if let Some(ref mutex) = INSTANCES {
                    if let Ok(mut map) = mutex.lock() {
                        if let Some(inst) = map.get_mut(&_instance_id) {
                            inst.file = None;
                            return write_tlv_void(_result, _result_len);
                        } else { return NYB_E_PLUGIN_ERROR; }
                    } else { return NYB_E_PLUGIN_ERROR; }
                }
                NYB_E_PLUGIN_ERROR
            }
            _ => NYB_SUCCESS
        }
    }
}

// ===== Helpers =====

fn open_file(mode: &str, path: &str) -> Result<std::fs::File, std::io::Error> {
    use std::fs::OpenOptions;
    match mode {
        "r" => OpenOptions::new().read(true).open(path),
        "w" => OpenOptions::new().write(true).create(true).truncate(true).open(path),
        "a" => OpenOptions::new().append(true).create(true).open(path),
        "rw" | "r+" => OpenOptions::new().read(true).write(true).create(true).open(path),
        _ => OpenOptions::new().read(true).open(path),
    }
}

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

fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(9u8, &[])], result, result_len)
}

fn write_tlv_bytes(data: &[u8], result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(7u8, data)], result, result_len)
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

fn tlv_parse_header(data: &[u8]) -> Result<(u16,u16,usize), ()> {
    if data.len() < 4 { return Err(()); }
    let ver = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 { return Err(()); }
    Ok((ver, argc, 4))
}

fn tlv_parse_two_strings(data: &[u8]) -> Result<(String, String), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 2 { return Err(()); }
    let s1 = tlv_parse_string_at(data, &mut pos)?;
    let s2 = tlv_parse_string_at(data, &mut pos)?;
    Ok((s1, s2))
}

fn tlv_parse_string_at(data: &[u8], pos: &mut usize) -> Result<String, ()> {
    if *pos + 4 > data.len() { return Err(()); }
    let tag = data[*pos]; let _res = data[*pos+1];
    let size = u16::from_le_bytes([data[*pos+2], data[*pos+3]]) as usize;
    *pos += 4;
    if tag != 6 || *pos + size > data.len() { return Err(()); }
    let slice = &data[*pos..*pos+size];
    *pos += size;
    std::str::from_utf8(slice).map(|s| s.to_string()).map_err(|_| ())
}

fn tlv_parse_i32(data: &[u8]) -> Result<i32, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 { return Err(()); }
    if pos + 8 > data.len() { return Err(()); }
    let tag = data[pos]; let _res = data[pos+1];
    let size = u16::from_le_bytes([data[pos+2], data[pos+3]]) as usize; pos += 4;
    if tag != 2 || size != 4 || pos + size > data.len() { return Err(()); }
    let mut b = [0u8;4]; b.copy_from_slice(&data[pos..pos+4]);
    Ok(i32::from_le_bytes(b))
}

fn tlv_parse_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 { return Err(()); }
    if pos + 4 > data.len() { return Err(()); }
    let tag = data[pos]; let _res = data[pos+1];
    let size = u16::from_le_bytes([data[pos+2], data[pos+3]]) as usize; pos += 4;
    // StringタグもBytesタグも受け付ける（互換性のため）
    if (tag != 6 && tag != 7) || pos + size > data.len() { return Err(()); }
    Ok(data[pos..pos+size].to_vec())
}

fn log_info(message: &str) {
    unsafe {
        if let Some(vt) = HOST_VTABLE {
            let log_fn = vt.log;
            if let Ok(c) = std::ffi::CString::new(message) {
                log_fn(1, c.as_ptr());
            }
        }
    }
}

/// Plugin shutdown
#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() {
    unsafe {
        INSTANCES = None;
    }
}
