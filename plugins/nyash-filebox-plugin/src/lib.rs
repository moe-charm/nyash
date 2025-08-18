//! Nyash FileBox Plugin - BID-FFI v1 Implementation
//! 
//! Provides file I/O operations as a Nyash plugin

use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

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

// ============ Error Codes ============
const NYB_SUCCESS: i32 = 0;
const NYB_E_INVALID_ARGS: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_HANDLE: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;

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
    NYB_SUCCESS
}

/// Plugin shutdown
#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() {
    unsafe {
        INSTANCES = None;
    }
}