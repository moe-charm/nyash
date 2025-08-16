//! FileBox Dynamic Plugin
//! 
//! C ABIを使用した動的ライブラリとしてFileBox機能を提供

use std::ffi::{c_char, c_void, CStr, CString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek};
use std::os::raw::c_int;

/// プラグインのマジックナンバー（'NYAS'）
const PLUGIN_MAGIC: u32 = 0x4E594153;

/// プラグイン情報構造体
#[repr(C)]
pub struct FileBoxPlugin {
    magic: u32,
    version: u32,
    api_version: u32,
}

/// FileBoxのハンドル（不透明ポインタ）
pub struct FileBoxHandle {
    file: File,
    path: String,
}

/// プラグイン初期化
#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> *const FileBoxPlugin {
    let plugin = Box::new(FileBoxPlugin {
        magic: PLUGIN_MAGIC,
        version: 1,
        api_version: 1,
    });
    Box::into_raw(plugin)
}

/// FileBoxを開く
/// 
/// # Safety
/// - pathは有効なUTF-8のC文字列である必要がある
/// - 返されたポインタはnyash_file_freeで解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn nyash_file_open(path: *const c_char) -> *mut c_void {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path_str)
    {
        Ok(file) => {
            let handle = Box::new(FileBoxHandle {
                file,
                path: path_str.to_string(),
            });
            Box::into_raw(handle) as *mut c_void
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// ファイルの内容を読み取る
/// 
/// # Safety
/// - handleはnyash_file_openから返された有効なポインタである必要がある
/// - 返された文字列はnyash_string_freeで解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn nyash_file_read(handle: *mut c_void) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    
    let file_box = &mut *(handle as *mut FileBoxHandle);
    let mut content = String::new();
    
    // ファイルポインタを最初に戻す
    if let Err(_) = file_box.file.seek(std::io::SeekFrom::Start(0)) {
        return std::ptr::null_mut();
    }
    
    match file_box.file.read_to_string(&mut content) {
        Ok(_) => {
            match CString::new(content) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// ファイルに内容を書き込む
/// 
/// # Safety
/// - handleはnyash_file_openから返された有効なポインタである必要がある
/// - contentは有効なUTF-8のC文字列である必要がある
#[no_mangle]
pub unsafe extern "C" fn nyash_file_write(
    handle: *mut c_void,
    content: *const c_char
) -> c_int {
    if handle.is_null() || content.is_null() {
        return 0;
    }
    
    let file_box = &mut *(handle as *mut FileBoxHandle);
    let content_str = match CStr::from_ptr(content).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    // ファイルをクリアして最初から書き込む
    if let Err(_) = file_box.file.set_len(0) {
        return 0;
    }
    if let Err(_) = file_box.file.seek(std::io::SeekFrom::Start(0)) {
        return 0;
    }
    
    match file_box.file.write_all(content_str.as_bytes()) {
        Ok(_) => 1,  // 成功
        Err(_) => 0,  // 失敗
    }
}

/// ファイルが存在するかチェック
/// 
/// # Safety
/// - pathは有効なUTF-8のC文字列である必要がある
#[no_mangle]
pub unsafe extern "C" fn nyash_file_exists(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }
    
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if std::path::Path::new(path_str).exists() {
        1
    } else {
        0
    }
}

/// FileBoxハンドルを解放
/// 
/// # Safety
/// - handleはnyash_file_openから返された有効なポインタである必要がある
/// - 解放後はhandleを使用してはいけない
#[no_mangle]
pub unsafe extern "C" fn nyash_file_free(handle: *mut c_void) {
    if !handle.is_null() {
        drop(Box::from_raw(handle as *mut FileBoxHandle));
    }
}

/// 文字列を解放（nyash_file_readの戻り値用）
/// 
/// # Safety
/// - strはnyash_file_readから返された有効なポインタである必要がある
#[no_mangle]
pub unsafe extern "C" fn nyash_string_free(str: *mut c_char) {
    if !str.is_null() {
        drop(CString::from_raw(str));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_plugin_init() {
        unsafe {
            let plugin = nyash_plugin_init();
            assert!(!plugin.is_null());
            let plugin_info = &*plugin;
            assert_eq!(plugin_info.magic, PLUGIN_MAGIC);
            assert_eq!(plugin_info.version, 1);
            assert_eq!(plugin_info.api_version, 1);
        }
    }
}