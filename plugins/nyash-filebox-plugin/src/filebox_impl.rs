//! FileBox TypeBox v2 resolve/invoke implementation

use crate::constants::*;
use crate::state::{FileBoxInstance, INSTANCES, INSTANCE_COUNTER};
use crate::tlv_helpers::{
    preflight, tlv_parse_handle, tlv_parse_optional_string_and_bytes, tlv_parse_string,
    tlv_parse_two_strings, write_tlv_bool, write_tlv_bytes, write_tlv_i32, write_tlv_result,
    write_tlv_void,
};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::c_char;
use std::sync::atomic::Ordering;

/// Resolve method name to method ID
pub extern "C" fn filebox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { std::ffi::CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => METHOD_BIRTH,
        "fini" => METHOD_FINI,
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

/// Invoke method by ID
pub extern "C" fn filebox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            METHOD_BIRTH => handle_birth(result, result_len),
            METHOD_FINI => handle_fini(instance_id),
            METHOD_OPEN => handle_open(instance_id, args, args_len, result, result_len),
            METHOD_READ => handle_read(instance_id, args, args_len, result, result_len),
            METHOD_WRITE => handle_write(instance_id, args, args_len, result, result_len),
            METHOD_CLOSE => handle_close(instance_id, result, result_len),
            METHOD_EXISTS => handle_exists(args, args_len, result, result_len),
            METHOD_COPY_FROM => handle_copy_from(instance_id, args, args_len, result, result_len),
            METHOD_CLONE_SELF => handle_clone_self(result, result_len),
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

unsafe fn handle_birth(result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    if preflight(result, result_len, 4) {
        return NYB_E_SHORT_BUFFER;
    }
    let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut map) = INSTANCES.lock() {
        map.insert(id, FileBoxInstance::new());
    } else {
        return NYB_E_PLUGIN_ERROR;
    }
    let bytes = id.to_le_bytes();
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
    *result_len = 4;
    NYB_SUCCESS
}

unsafe fn handle_fini(instance_id: u32) -> i32 {
    INSTANCES
        .lock()
        .map(|mut map| {
            map.remove(&instance_id);
            NYB_SUCCESS
        })
        .unwrap_or(NYB_E_PLUGIN_ERROR)
}

unsafe fn handle_open(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    let (path, mode) = match tlv_parse_two_strings(slice) {
        Ok(pair) => pair,
        Err(_) => return NYB_E_INVALID_ARGS,
    };
    if preflight(result, result_len, 8) {
        return NYB_E_SHORT_BUFFER;
    }
    let mut guard = match INSTANCES.lock() {
        Ok(g) => g,
        Err(_) => return NYB_E_PLUGIN_ERROR,
    };
    let inst = match guard.get_mut(&instance_id) {
        Some(i) => i,
        None => return NYB_E_INVALID_HANDLE,
    };
    match open_file(&mode, &path) {
        Ok(file) => {
            inst.file = Some(file);
            inst.path = path;
            inst.buffer = None;
            write_tlv_void(result, result_len)
        }
        Err(_) => NYB_E_PLUGIN_ERROR,
    }
}

unsafe fn handle_read(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    if args_len > 0 {
        match tlv_parse_string(slice) {
            Ok(path) => match open_file("r", &path) {
                Ok(mut file) => {
                    let mut buf = Vec::new();
                    if file.read_to_end(&mut buf).is_err() {
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
    }
    let mut guard = match INSTANCES.lock() {
        Ok(g) => g,
        Err(_) => return NYB_E_PLUGIN_ERROR,
    };
    let inst = match guard.get_mut(&instance_id) {
        Some(i) => i,
        None => return NYB_E_INVALID_HANDLE,
    };
    let file = match inst.file.as_mut() {
        Some(f) => f,
        None => return NYB_E_INVALID_HANDLE,
    };
    if file.seek(SeekFrom::Start(0)).is_err() {
        return NYB_E_PLUGIN_ERROR;
    }
    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() {
        return NYB_E_PLUGIN_ERROR;
    }
    let need = 8usize.saturating_add(buf.len());
    if preflight(result, result_len, need) {
        return NYB_E_SHORT_BUFFER;
    }
    write_tlv_bytes(&buf, result, result_len)
}

unsafe fn handle_write(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    match tlv_parse_optional_string_and_bytes(slice) {
        Ok((Some(path), data)) => {
            if preflight(result, result_len, 12) {
                return NYB_E_SHORT_BUFFER;
            }
            match open_file("w", &path) {
                Ok(mut file) => {
                    if file.write_all(&data).is_err() || file.flush().is_err() {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    write_tlv_i32(data.len() as i32, result, result_len)
                }
                Err(_) => NYB_E_PLUGIN_ERROR,
            }
        }
        Ok((None, data)) => {
            if preflight(result, result_len, 12) {
                return NYB_E_SHORT_BUFFER;
            }
            let mut guard = match INSTANCES.lock() {
                Ok(g) => g,
                Err(_) => return NYB_E_PLUGIN_ERROR,
            };
            let inst = match guard.get_mut(&instance_id) {
                Some(i) => i,
                None => return NYB_E_INVALID_HANDLE,
            };
            let file = match inst.file.as_mut() {
                Some(f) => f,
                None => return NYB_E_INVALID_HANDLE,
            };
            match file.write(&data) {
                Ok(written) => {
                    if file.flush().is_err() {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    inst.buffer = Some(data.clone());
                    write_tlv_i32(written as i32, result, result_len)
                }
                Err(_) => NYB_E_PLUGIN_ERROR,
            }
        }
        Err(_) => NYB_E_INVALID_ARGS,
    }
}

unsafe fn handle_close(instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if preflight(result, result_len, 8) {
        return NYB_E_SHORT_BUFFER;
    }
    let mut guard = match INSTANCES.lock() {
        Ok(g) => g,
        Err(_) => return NYB_E_PLUGIN_ERROR,
    };
    let inst = match guard.get_mut(&instance_id) {
        Some(i) => i,
        None => return NYB_E_INVALID_HANDLE,
    };
    inst.file = None;
    inst.buffer = None;
    inst.path.clear();
    write_tlv_void(result, result_len)
}

unsafe fn handle_exists(
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    let path = match tlv_parse_string(slice) {
        Ok(p) => p,
        Err(_) => return NYB_E_INVALID_ARGS,
    };
    if preflight(result, result_len, 9) {
        return NYB_E_SHORT_BUFFER;
    }
    let exists = std::path::Path::new(&path).exists();
    write_tlv_bool(exists, result, result_len)
}

unsafe fn handle_copy_from(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    let (_src_type, other_id) = match tlv_parse_handle(slice) {
        Ok(pair) => pair,
        Err(_) => return NYB_E_INVALID_ARGS,
    };
    if preflight(result, result_len, 8) {
        return NYB_E_SHORT_BUFFER;
    }
    let mut guard = match INSTANCES.lock() {
        Ok(g) => g,
        Err(_) => return NYB_E_PLUGIN_ERROR,
    };
    // Extract data from source
    let mut data = Vec::new();
    if let Some(src) = guard.get(&other_id) {
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
    // Write into destination
    if let Some(dst) = guard.get_mut(&instance_id) {
        if let Some(file) = dst.file.as_mut() {
            let _ = file.seek(SeekFrom::Start(0));
            if file.write_all(&data).is_err() {
                return NYB_E_PLUGIN_ERROR;
            }
            let _ = file.set_len(data.len() as u64);
            let _ = file.flush();
        }
        dst.buffer = Some(data);
        write_tlv_void(result, result_len)
    } else {
        NYB_E_INVALID_HANDLE
    }
}

unsafe fn handle_clone_self(result: *mut u8, result_len: *mut usize) -> i32 {
    if preflight(result, result_len, 16) {
        return NYB_E_SHORT_BUFFER;
    }
    let new_id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut map) = INSTANCES.lock() {
        map.insert(new_id, FileBoxInstance::new());
    }
    let mut payload = [0u8; 8];
    payload[4..8].copy_from_slice(&new_id.to_le_bytes());
    write_tlv_result(&[(TLV_TAG_HANDLE, &payload)], result, result_len)
}

fn open_file(mode: &str, path: &str) -> std::io::Result<std::fs::File> {
    match mode {
        "r" => std::fs::File::open(path),
        "w" => std::fs::File::create(path),
        "a" => std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path),
        "rw" | "r+" => std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path),
        _ => std::fs::File::open(path),
    }
}
