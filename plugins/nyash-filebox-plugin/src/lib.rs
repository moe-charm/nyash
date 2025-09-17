//! Nyash FileBox Plugin — TypeBox v2
//!
//! Provides file I/O operations as a Nyash plugin（TypeBox v2 エクスポート済み）

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
// std::ptr削除（未使用）
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// ============ FFI Types ============

// Host VTable廃止 - 不要

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
const METHOD_BIRTH: u32 = 0; // Constructor
const METHOD_OPEN: u32 = 1;
const METHOD_READ: u32 = 2;
const METHOD_WRITE: u32 = 3;
const METHOD_CLOSE: u32 = 4;
const METHOD_EXISTS: u32 = 5;
const METHOD_COPY_FROM: u32 = 7; // New: copyFrom(other: Handle)
const METHOD_CLONE_SELF: u32 = 8; // New: cloneSelf() -> Handle
const METHOD_FINI: u32 = u32::MAX; // Destructor

// ============ FileBox Instance ============
struct FileBoxInstance {
    file: Option<std::fs::File>,
    path: String,
    buffer: Option<Vec<u8>>, // プラグインが管理するバッファ
}

// グローバルインスタンス管理（実際の実装ではより安全な方法を使用）
use once_cell::sync::Lazy;
static INSTANCES: Lazy<Mutex<HashMap<u32, FileBoxInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ホスト関数テーブルは使用しない（Host VTable廃止）

// インスタンスIDカウンタ（1開始）
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

// ============ Plugin Entry Points ============

// legacy v1 abi/init removed

/// Method invocation - 仮実装
/* legacy v1 entry removed
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
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        instance_id,
                        FileBoxInstance {
                            file: None,
                            path: String::new(),
                            buffer: None,
                        },
                    );
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
                if let Ok(mut map) = INSTANCES.lock() {
                    map.remove(&_instance_id);
                    return NYB_SUCCESS;
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_OPEN => {
                // args: TLV { String path, String mode }
                let args = std::slice::from_raw_parts(_args, _args_len);
                match tlv_parse_two_strings(args) {
                    Ok((path, mode)) => {
                        // Preflight for Void TLV: header(4) + entry(4)
                        if preflight(_result, _result_len, 8) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        log_info(&format!("OPEN path='{}' mode='{}'", path, mode));
                        if let Ok(mut map) = INSTANCES.lock() {
                            if let Some(inst) = map.get_mut(&_instance_id) {
                                match open_file(&mode, &path) {
                                    Ok(file) => {
                                        inst.file = Some(file);
                                        // return TLV Void
                                        return write_tlv_void(_result, _result_len);
                                    }
                                    Err(_) => return NYB_E_PLUGIN_ERROR,
                                }
                            } else {
                                return NYB_E_PLUGIN_ERROR;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_READ => {
                // args: optional String path
                let args = std::slice::from_raw_parts(_args, _args_len);
                if _args_len > 0 {
                    match tlv_parse_string(args) {
                        Ok(path) => match open_file("r", &path) {
                            Ok(mut file) => {
                                let mut buf = Vec::new();
                                if let Err(_) = file.read_to_end(&mut buf) {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                let need = 8usize.saturating_add(buf.len());
                                if preflight(_result, _result_len, need) {
                                    return NYB_E_SHORT_BUFFER;
                                }
                                return write_tlv_bytes(&buf, _result, _result_len);
                            }
                            Err(_) => return NYB_E_PLUGIN_ERROR,
                        },
                        Err(_) => return NYB_E_INVALID_ARGS,
                    }
                } else {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&_instance_id) {
                            if let Some(file) = inst.file.as_mut() {
                                let _ = file.seek(SeekFrom::Start(0));
                                let mut buf = Vec::new();
                                match file.read_to_end(&mut buf) {
                                    Ok(n) => {
                                        log_info(&format!("READ {} bytes (entire file)", n));
                                        let need = 8usize.saturating_add(buf.len());
                                        if preflight(_result, _result_len, need) {
                                            return NYB_E_SHORT_BUFFER;
                                        }
                                        return write_tlv_bytes(&buf, _result, _result_len);
                                    }
                                    Err(_) => return NYB_E_PLUGIN_ERROR,
                                }
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                }
            }
            METHOD_WRITE => {
                // args: TLV { [String path]? Bytes data }
                let args = std::slice::from_raw_parts(_args, _args_len);
                match tlv_parse_optional_string_and_bytes(args) {
                    Ok((Some(path), data)) => {
                        if preflight(_result, _result_len, 12) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        match open_file("w", &path) {
                            Ok(mut file) => {
                                if let Err(_) = file.write_all(&data) {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                if let Err(_) = file.flush() {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                return write_tlv_i32(data.len() as i32, _result, _result_len);
                            }
                            Err(_) => return NYB_E_PLUGIN_ERROR,
                        }
                    }
                    Ok((None, data)) => {
                        if preflight(_result, _result_len, 12) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        if let Ok(mut map) = INSTANCES.lock() {
                            if let Some(inst) = map.get_mut(&_instance_id) {
                                if let Some(file) = inst.file.as_mut() {
                                    match file.write(&data) {
                                        Ok(n) => {
                                            if let Err(_) = file.flush() {
                                                return NYB_E_PLUGIN_ERROR;
                                            }
                                            log_info(&format!("WRITE {} bytes", n));
                                            inst.buffer = Some(data.clone());
                                            return write_tlv_i32(n as i32, _result, _result_len);
                                        }
                                        Err(_) => return NYB_E_PLUGIN_ERROR,
                                    }
                                } else {
                                    return NYB_E_INVALID_HANDLE;
                                }
                            } else {
                                return NYB_E_PLUGIN_ERROR;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_CLOSE => {
                // Preflight for Void TLV
                if preflight(_result, _result_len, 8) {
                    return NYB_E_SHORT_BUFFER;
                }
                log_info("CLOSE");
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&_instance_id) {
                        inst.file = None;
                        return write_tlv_void(_result, _result_len);
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_COPY_FROM => {
                // args: TLV { Handle (tag=8, size=8) }
                let args = std::slice::from_raw_parts(_args, _args_len);
                match tlv_parse_handle(args) {
                    Ok((type_id, other_id)) => {
                        if type_id != _type_id {
                            return NYB_E_INVALID_TYPE;
                        }
                        if preflight(_result, _result_len, 8) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        if let Ok(mut map) = INSTANCES.lock() {
                            // 1) まずsrcからデータを取り出す（不変参照のみ）
                            let mut data: Vec<u8> = Vec::new();
                            if let Some(src) = map.get(&other_id) {
                                let mut read_ok = false;
                                if let Some(file) = src.file.as_ref() {
                                    if let Ok(mut f) = file.try_clone() {
                                        let _ = f.seek(SeekFrom::Start(0));
                                        if f.read_to_end(&mut data).is_ok() {
                                            read_ok = true;
                                        }
                                    }
                                }
                                if !read_ok {
                                    if let Some(buf) = src.buffer.as_ref() {
                                        data.extend_from_slice(buf);
                                        read_ok = true;
                                    }
                                }
                                if !read_ok {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }

                            // 2) dstへ書き込み（可変参照）
                            if let Some(dst) = map.get_mut(&_instance_id) {
                                if let Some(fdst) = dst.file.as_mut() {
                                    let _ = fdst.seek(SeekFrom::Start(0));
                                    if fdst.write_all(&data).is_err() {
                                        return NYB_E_PLUGIN_ERROR;
                                    }
                                    let _ = fdst.set_len(data.len() as u64);
                                    let _ = fdst.flush();
                                }
                                dst.buffer = Some(data);
                                return write_tlv_void(_result, _result_len);
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_CLONE_SELF => {
                // Return a new instance (handle) as TLV Handle
                // Preflight for Handle TLV: header(4) + entry(4) + payload(8)
                if preflight(_result, _result_len, 16) {
                    return NYB_E_SHORT_BUFFER;
                }
                let new_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        new_id,
                        FileBoxInstance {
                            file: None,
                            path: String::new(),
                            buffer: None,
                        },
                    );
                }
                // Build TLV result
                let mut payload = [0u8; 8];
                payload[0..4].copy_from_slice(&_type_id.to_le_bytes());
                payload[4..8].copy_from_slice(&new_id.to_le_bytes());
                return write_tlv_result(&[(8u8, &payload)], _result, _result_len);
            }
            METHOD_EXISTS => {
                // args: TLV { String path }
                let args = std::slice::from_raw_parts(_args, _args_len);
                match tlv_parse_string(args) {
                    Ok(path) => {
                        let exists = std::path::Path::new(&path).exists();
                        if preflight(_result, _result_len, 9) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        return write_tlv_bool(exists, _result, _result_len);
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            _ => NYB_SUCCESS,
        }
    }
}
*/

// ===== Helpers =====

fn open_file(mode: &str, path: &str) -> Result<std::fs::File, std::io::Error> {
    use std::fs::OpenOptions;
    match mode {
        "r" => OpenOptions::new().read(true).open(path),
        "w" => OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path),
        "a" => OpenOptions::new().append(true).create(true).open(path),
        "rw" | "r+" => OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path),
        _ => OpenOptions::new().read(true).open(path),
    }
}

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
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

fn write_tlv_bool(v: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    let b = [if v { 1u8 } else { 0u8 }];
    write_tlv_result(&[(1u8, &b)], result, result_len)
}

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

fn tlv_parse_header(data: &[u8]) -> Result<(u16, u16, usize), ()> {
    if data.len() < 4 {
        return Err(());
    }
    let ver = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 {
        return Err(());
    }
    Ok((ver, argc, 4))
}

fn tlv_parse_two_strings(data: &[u8]) -> Result<(String, String), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 2 {
        return Err(());
    }
    let s1 = tlv_parse_string_at(data, &mut pos)?;
    let s2 = tlv_parse_string_at(data, &mut pos)?;
    Ok((s1, s2))
}

fn tlv_parse_string_at(data: &[u8], pos: &mut usize) -> Result<String, ()> {
    if *pos + 4 > data.len() {
        return Err(());
    }
    let tag = data[*pos];
    let _res = data[*pos + 1];
    let size = u16::from_le_bytes([data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if tag != 6 || *pos + size > data.len() {
        return Err(());
    }
    let slice = &data[*pos..*pos + size];
    *pos += size;
    std::str::from_utf8(slice)
        .map(|s| s.to_string())
        .map_err(|_| ())
}

fn tlv_parse_i32(data: &[u8]) -> Result<i32, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    if pos + 8 > data.len() {
        return Err(());
    }
    let tag = data[pos];
    let _res = data[pos + 1];
    let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;
    if tag != 2 || size != 4 || pos + size > data.len() {
        return Err(());
    }
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[pos..pos + 4]);
    Ok(i32::from_le_bytes(b))
}

fn tlv_parse_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    if pos + 4 > data.len() {
        return Err(());
    }
    let tag = data[pos];
    let _res = data[pos + 1];
    let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;
    // StringタグもBytesタグも受け付ける（互換性のため）
    if (tag != 6 && tag != 7) || pos + size > data.len() {
        return Err(());
    }
    Ok(data[pos..pos + size].to_vec())
}

fn tlv_parse_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_string_at(data, &mut pos)
}

fn tlv_parse_optional_string_and_bytes(data: &[u8]) -> Result<(Option<String>, Vec<u8>), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc == 1 {
        // only bytes
        if pos + 4 > data.len() {
            return Err(());
        }
        let tag = data[pos];
        let _res = data[pos + 1];
        let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if (tag != 6 && tag != 7) || pos + size > data.len() {
            return Err(());
        }
        return Ok((None, data[pos..pos + size].to_vec()));
    } else if argc >= 2 {
        let s = tlv_parse_string_at(data, &mut pos)?;
        if pos + 4 > data.len() {
            return Err(());
        }
        let tag = data[pos];
        let _res = data[pos + 1];
        let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if (tag != 6 && tag != 7) || pos + size > data.len() {
            return Err(());
        }
        Ok((Some(s), data[pos..pos + size].to_vec()))
    } else {
        Err(())
    }
}

fn tlv_parse_handle(data: &[u8]) -> Result<(u32, u32), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    if pos + 4 > data.len() {
        return Err(());
    }
    let tag = data[pos];
    let _res = data[pos + 1];
    let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;
    if tag != 8 || size != 8 || pos + size > data.len() {
        return Err(());
    }
    let mut t = [0u8; 4];
    t.copy_from_slice(&data[pos..pos + 4]);
    let mut i = [0u8; 4];
    i.copy_from_slice(&data[pos + 4..pos + 8]);
    Ok((u32::from_le_bytes(t), u32::from_le_bytes(i)))
}

fn log_info(message: &str) {
    eprintln!("[FileBox] {}", message);
}

/// Plugin shutdown
#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() {
    // インスタンスをクリア
    if let Ok(mut map) = INSTANCES.lock() {
        map.clear();
    }
    eprintln!("[FileBox] Plugin shutdown");
}

// ============ Unified Plugin API ============
// Note: Metadata (Box types, methods) now comes from nyash.toml
// This plugin provides only the actual processing functions

// ===== TypeBox ABI v2 (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,        // 'TYBX' (0x54594258)
    pub version: u16,        // 1
    pub struct_size: u16,    // sizeof(NyashTypeBoxFfi)
    pub name: *const c_char, // C string, e.g., "FileBox\0"
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn filebox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        // lifecycle (optional to expose via resolve)
        "birth" => METHOD_BIRTH,
        "fini" => METHOD_FINI,
        // methods
        "open" => METHOD_OPEN,
        "read" => METHOD_READ,
        "write" => METHOD_WRITE,
        "close" => METHOD_CLOSE,
        "exists" => METHOD_EXISTS,
        "copyFrom" => METHOD_COPY_FROM,
        "cloneSelf" => METHOD_CLONE_SELF,
        _ => 0,
    }
}

extern "C" fn filebox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            // Support birth through per-Box invoke for v2 shim
            METHOD_BIRTH => {
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return NYB_E_SHORT_BUFFER;
                }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        id,
                        FileBoxInstance {
                            file: None,
                            path: String::new(),
                            buffer: None,
                        },
                    );
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
                let b = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                if let Ok(mut map) = INSTANCES.lock() {
                    map.remove(&instance_id);
                    NYB_SUCCESS
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_OPEN => {
                // args: TLV { String path, String mode }
                let slice = std::slice::from_raw_parts(args, args_len);
                match tlv_parse_two_strings(slice) {
                    Ok((path, mode)) => {
                        if preflight(result, result_len, 8) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        if let Ok(mut map) = INSTANCES.lock() {
                            if let Some(inst) = map.get_mut(&instance_id) {
                                match open_file(&mode, &path) {
                                    Ok(file) => {
                                        inst.file = Some(file);
                                        return write_tlv_void(result, result_len);
                                    }
                                    Err(_) => return NYB_E_PLUGIN_ERROR,
                                }
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_READ => {
                let slice = std::slice::from_raw_parts(args, args_len);
                if args_len > 0 {
                    match tlv_parse_string(slice) {
                        Ok(path) => match open_file("r", &path) {
                            Ok(mut file) => {
                                let mut buf = Vec::new();
                                if let Err(_) = file.read_to_end(&mut buf) {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                let need = 8usize.saturating_add(buf.len());
                                if preflight(result, result_len, need) {
                                    return NYB_E_SHORT_BUFFER;
                                }
                                return write_tlv_bytes(&buf, result, result_len);
                            }
                            Err(_) => return NYB_E_PLUGIN_ERROR,
                        },
                        Err(_) => return NYB_E_INVALID_ARGS,
                    }
                } else {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            if let Some(file) = inst.file.as_mut() {
                                let _ = file.seek(SeekFrom::Start(0));
                                let mut buf = Vec::new();
                                match file.read_to_end(&mut buf) {
                                    Ok(_) => {
                                        let need = 8usize.saturating_add(buf.len());
                                        if preflight(result, result_len, need) {
                                            return NYB_E_SHORT_BUFFER;
                                        }
                                        return write_tlv_bytes(&buf, result, result_len);
                                    }
                                    Err(_) => return NYB_E_PLUGIN_ERROR,
                                }
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                }
            }
            METHOD_WRITE => {
                let slice = std::slice::from_raw_parts(args, args_len);
                match tlv_parse_optional_string_and_bytes(slice) {
                    Ok((Some(path), data)) => {
                        if preflight(result, result_len, 12) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        match open_file("w", &path) {
                            Ok(mut file) => {
                                if let Err(_) = file.write_all(&data) {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                if let Err(_) = file.flush() {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                                return write_tlv_i32(data.len() as i32, result, result_len);
                            }
                            Err(_) => return NYB_E_PLUGIN_ERROR,
                        }
                    }
                    Ok((None, data)) => {
                        if preflight(result, result_len, 12) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        if let Ok(mut map) = INSTANCES.lock() {
                            if let Some(inst) = map.get_mut(&instance_id) {
                                if let Some(file) = inst.file.as_mut() {
                                    match file.write(&data) {
                                        Ok(n) => {
                                            if let Err(_) = file.flush() {
                                                return NYB_E_PLUGIN_ERROR;
                                            }
                                            inst.buffer = Some(data.clone());
                                            return write_tlv_i32(n as i32, result, result_len);
                                        }
                                        Err(_) => return NYB_E_PLUGIN_ERROR,
                                    }
                                } else {
                                    return NYB_E_INVALID_HANDLE;
                                }
                            } else {
                                return NYB_E_PLUGIN_ERROR;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_CLOSE => {
                if preflight(result, result_len, 8) {
                    return NYB_E_SHORT_BUFFER;
                }
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.file = None;
                        return write_tlv_void(result, result_len);
                    } else {
                        return NYB_E_PLUGIN_ERROR;
                    }
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
            }
            METHOD_COPY_FROM => {
                let slice = std::slice::from_raw_parts(args, args_len);
                match tlv_parse_handle(slice) {
                    Ok((_type_id, other_id)) => {
                        if preflight(result, result_len, 8) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        if let Ok(mut map) = INSTANCES.lock() {
                            // Extract data from src
                            let mut data: Vec<u8> = Vec::new();
                            if let Some(src) = map.get(&other_id) {
                                let mut read_ok = false;
                                if let Some(file) = src.file.as_ref() {
                                    if let Ok(mut f) = file.try_clone() {
                                        let _ = f.seek(SeekFrom::Start(0));
                                        if f.read_to_end(&mut data).is_ok() {
                                            read_ok = true;
                                        }
                                    }
                                }
                                if !read_ok {
                                    if let Some(buf) = src.buffer.as_ref() {
                                        data.extend_from_slice(buf);
                                        read_ok = true;
                                    }
                                }
                                if !read_ok {
                                    return NYB_E_PLUGIN_ERROR;
                                }
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                            // Write into dst
                            if let Some(dst) = map.get_mut(&instance_id) {
                                if let Some(fdst) = dst.file.as_mut() {
                                    let _ = fdst.seek(SeekFrom::Start(0));
                                    if fdst.write_all(&data).is_err() {
                                        return NYB_E_PLUGIN_ERROR;
                                    }
                                    let _ = fdst.set_len(data.len() as u64);
                                    let _ = fdst.flush();
                                }
                                dst.buffer = Some(data);
                                return write_tlv_void(result, result_len);
                            } else {
                                return NYB_E_INVALID_HANDLE;
                            }
                        } else {
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            METHOD_CLONE_SELF => {
                if preflight(result, result_len, 16) {
                    return NYB_E_SHORT_BUFFER;
                }
                let new_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        new_id,
                        FileBoxInstance {
                            file: None,
                            path: String::new(),
                            buffer: None,
                        },
                    );
                }
                // Return Handle TLV (type_id from config resolves host-side; we encode (6,new_id) here if needed)
                let mut payload = [0u8; 8];
                // We don’t embed type_id here (host can ignore); keep zero for neutrality
                payload[0..4].copy_from_slice(&0u32.to_le_bytes());
                payload[4..8].copy_from_slice(&new_id.to_le_bytes());
                return write_tlv_result(&[(8u8, &payload)], result, result_len);
            }
            METHOD_EXISTS => {
                let slice = std::slice::from_raw_parts(args, args_len);
                match tlv_parse_string(slice) {
                    Ok(path) => {
                        let exists = std::path::Path::new(&path).exists();
                        if preflight(result, result_len, 9) {
                            return NYB_E_SHORT_BUFFER;
                        }
                        return write_tlv_bool(exists, result, result_len);
                    }
                    Err(_) => NYB_E_INVALID_ARGS,
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

#[no_mangle]
pub static nyash_typebox_FileBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"FileBox\0".as_ptr() as *const c_char,
    resolve: Some(filebox_resolve),
    invoke_id: Some(filebox_invoke_id),
    capabilities: 0,
};
