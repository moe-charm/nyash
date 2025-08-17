//! Nyash FileBox Plugin - BID-FFI Reference Implementation
//! 
//! ファイル操作を提供するプラグインの実装例
//! Everything is Box哲学に基づく透過的置き換え

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use once_cell::sync::Lazy;

mod ffi_types;
use ffi_types::*;

mod filebox;
use filebox::*;

/// グローバルファイルハンドルレジストリ
static FILE_REGISTRY: Lazy<Arc<Mutex<FileBoxRegistry>>> = 
    Lazy::new(|| Arc::new(Mutex::new(FileBoxRegistry::new())));

/// プラグインハンドルカウンター
static mut HANDLE_COUNTER: u32 = 1;

/// ユニークハンドル生成
fn next_handle() -> u32 {
    unsafe {
        let handle = HANDLE_COUNTER;
        HANDLE_COUNTER += 1;
        handle
    }
}

/// BID-FFIエントリーポイント
/// 
/// すべてのプラグイン操作はこの関数を通して実行される
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    method: *const c_char,
    handle: u64,
    input_data: *const u8,
    input_len: usize,
    output_data: *mut *mut u8,
    output_len: *mut usize,
) -> i32 {
    // 基本的な引数チェック
    if method.is_null() || output_data.is_null() || output_len.is_null() {
        return -1; // INVALID_ARGUMENT
    }

    // メソッド名解析
    let method_name = unsafe {
        match CStr::from_ptr(method).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // ENCODING_ERROR
        }
    };

    // ハンドル分解 (BID-1仕様)
    let type_id = (handle >> 32) as u32;
    let instance_id = (handle & 0xFFFFFFFF) as u32;

    // メソッド呼び出し
    match invoke_method(method_name, type_id, instance_id, input_data, input_len) {
        Ok(result) => {
            // 結果をC側に返す
            if let Ok(c_result) = CString::new(result) {
                let result_bytes = c_result.into_bytes_with_nul();
                let len = result_bytes.len();
                
                // メモリ確保（C側でfreeする必要がある）
                let ptr = unsafe { libc::malloc(len) as *mut u8 };
                if ptr.is_null() {
                    return -3; // OUT_OF_MEMORY
                }
                
                unsafe {
                    std::ptr::copy_nonoverlapping(result_bytes.as_ptr(), ptr, len);
                    *output_data = ptr;
                    *output_len = len;
                }
                0 // SUCCESS
            } else {
                -2 // ENCODING_ERROR
            }
        }
        Err(code) => code,
    }
}

/// メソッド呼び出し実装
fn invoke_method(
    method: &str,
    type_id: u32,
    instance_id: u32,
    input_data: *const u8,
    input_len: usize,
) -> Result<String, i32> {
    match method {
        "new" => handle_new(input_data, input_len),
        "open" => handle_open(instance_id, input_data, input_len),
        "read" => handle_read(instance_id, input_data, input_len),
        "write" => handle_write(instance_id, input_data, input_len),
        "close" => handle_close(instance_id),
        "toString" => handle_to_string(instance_id),
        _ => Err(-4), // UNKNOWN_METHOD
    }
}

/// FileBox::new() - 新しいFileBoxインスタンス作成
fn handle_new(input_data: *const u8, input_len: usize) -> Result<String, i32> {
    let handle = next_handle();
    let filebox = FileBoxInstance::new();
    
    {
        let mut registry = FILE_REGISTRY.lock().unwrap();
        registry.register(handle, filebox);
    }
    
    // BID-1ハンドル返却 (FileBox type_id = 8)
    let bid_handle = ((8u64) << 32) | (handle as u64);
    Ok(bid_handle.to_string())
}

/// FileBox::open(path) - ファイルオープン
fn handle_open(instance_id: u32, input_data: *const u8, input_len: usize) -> Result<String, i32> {
    // TLV解析（簡易版）
    let path = parse_string_from_tlv(input_data, input_len)?;
    
    let mut registry = FILE_REGISTRY.lock().unwrap();
    match registry.get_mut(instance_id) {
        Some(filebox) => {
            match filebox.open(&path) {
                Ok(()) => Ok("true".to_string()),
                Err(_) => Ok("false".to_string()),
            }
        }
        None => Err(-5), // INVALID_HANDLE
    }
}

/// FileBox::read() - ファイル読み取り
fn handle_read(instance_id: u32, _input_data: *const u8, _input_len: usize) -> Result<String, i32> {
    let mut registry = FILE_REGISTRY.lock().unwrap();
    match registry.get_mut(instance_id) {
        Some(filebox) => {
            match filebox.read() {
                Ok(content) => Ok(content),
                Err(_) => Ok("".to_string()),
            }
        }
        None => Err(-5), // INVALID_HANDLE
    }
}

/// FileBox::write(data) - ファイル書き込み
fn handle_write(instance_id: u32, input_data: *const u8, input_len: usize) -> Result<String, i32> {
    let data = parse_string_from_tlv(input_data, input_len)?;
    
    let mut registry = FILE_REGISTRY.lock().unwrap();
    match registry.get_mut(instance_id) {
        Some(filebox) => {
            match filebox.write(&data) {
                Ok(()) => Ok("true".to_string()),
                Err(_) => Ok("false".to_string()),
            }
        }
        None => Err(-5), // INVALID_HANDLE
    }
}

/// FileBox::close() - ファイルクローズ
fn handle_close(instance_id: u32) -> Result<String, i32> {
    let mut registry = FILE_REGISTRY.lock().unwrap();
    match registry.remove(instance_id) {
        Some(_) => Ok("true".to_string()),
        None => Err(-5), // INVALID_HANDLE
    }
}

/// FileBox::toString() - 文字列表現
fn handle_to_string(instance_id: u32) -> Result<String, i32> {
    let registry = FILE_REGISTRY.lock().unwrap();
    match registry.get(instance_id) {
        Some(filebox) => Ok(format!("FileBox({})", filebox.path().unwrap_or("none"))),
        None => Err(-5), // INVALID_HANDLE
    }
}

/// 簡易TLV解析（文字列のみ）
fn parse_string_from_tlv(data: *const u8, len: usize) -> Result<String, i32> {
    if data.is_null() || len == 0 {
        return Ok(String::new());
    }
    
    // 簡易実装：UTF-8文字列として直接解析
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    match std::str::from_utf8(slice) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(-2), // ENCODING_ERROR
    }
}

/// プラグイン情報（メタデータAPI）
#[no_mangle]
pub extern "C" fn nyash_plugin_info() -> *const c_char {
    // プラグイン情報のJSON
    let info = r#"{
        "name": "nyash-filebox-plugin",
        "version": "0.1.0",
        "provides": ["FileBox"],
        "abi_version": "bid-1.0"
    }"#;
    
    // リークさせて永続化（プラグインライフタイム中有効）
    Box::leak(CString::new(info).unwrap().into_boxed_c_str()).as_ptr()
}

/// プラグイン初期化
#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 {
    // 初期化処理（必要に応じて）
    0 // SUCCESS
}

/// プラグイン終了処理
#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() -> i32 {
    // クリーンアップ処理
    0 // SUCCESS
}