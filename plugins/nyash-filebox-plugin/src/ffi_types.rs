//! BID-FFI型定義
//! 
//! C FFI境界で使用される型定義

use std::os::raw::{c_char, c_void};

/// BID-1エラーコード
pub const BID_SUCCESS: i32 = 0;
pub const BID_INVALID_ARGUMENT: i32 = -1;
pub const BID_ENCODING_ERROR: i32 = -2;
pub const BID_OUT_OF_MEMORY: i32 = -3;
pub const BID_UNKNOWN_METHOD: i32 = -4;
pub const BID_INVALID_HANDLE: i32 = -5;
pub const BID_IO_ERROR: i32 = -6;

/// Box型ID（BID-1仕様）
pub const BOX_TYPE_STRING: u32 = 1;
pub const BOX_TYPE_INTEGER: u32 = 2;
pub const BOX_TYPE_BOOL: u32 = 3;
pub const BOX_TYPE_NULL: u32 = 4;
pub const BOX_TYPE_ARRAY: u32 = 5;
pub const BOX_TYPE_MAP: u32 = 6;
pub const BOX_TYPE_FUTURE: u32 = 7;
pub const BOX_TYPE_FILEBOX: u32 = 8;

/// プラグイン情報構造体
#[repr(C)]
pub struct NyashPluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub abi_version: *const c_char,
    pub provides_count: u32,
    pub provides: *const *const c_char,
}

/// ホスト機能テーブル
#[repr(C)]
pub struct HostVtable {
    pub alloc: extern "C" fn(size: usize) -> *mut c_void,
    pub free: extern "C" fn(ptr: *mut c_void),
    pub wake: extern "C" fn(future_handle: u64),
    pub log: extern "C" fn(level: u32, message: *const c_char),
}