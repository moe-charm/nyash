//! FileBox implementation

use crate::constants::*;
use crate::state::{allocate_instance_id, remove_instance, store_instance, with_instance_mut, FileBoxInstance, INSTANCE_COUNTER, INSTANCES};
use crate::tlv_helpers::*;
use std::ffi::CStr;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::c_char;
use std::sync::atomic::Ordering;

// ===== File I/O Helpers =====

pub fn open_file(mode: &str, path: &str) -> Result<std::fs::File, std::io::Error> {
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

// ===== TLV Parsing Extensions =====

fn tlv_parse_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_string_at(data, &mut pos)
}

fn tlv_parse_optional_string_and_bytes(data: &[u8]) -> Result<(Option<String>, Vec<u8>), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }

    // Check first arg tag to determine if string or bytes
    if data.len() < pos + 4 {
        return Err(());
    }
    let first_tag = data[pos];

    if first_tag == TLV_TAG_STRING {
        // First arg is string (path)
        let s = tlv_parse_string_at(data, &mut pos)?;
        if argc >= 2 {
            let b = tlv_parse_bytes_at(data, &mut pos)?;
            Ok((Some(s), b))
        } else {
            Ok((Some(s), Vec::new()))
        }
    } else if first_tag == TLV_TAG_BYTES {
        // First arg is bytes (no path)
        let b = tlv_parse_bytes_at(data, &mut pos)?;
        Ok((None, b))
    } else {
        Err(())
    }
}

fn tlv_parse_handle(data: &[u8]) -> Result<(u32, u32), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    tlv_parse_handle_at(data, &mut pos)
}

// ===== TypeBox v2 Implementation =====

pub extern "C" fn filebox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        // lifecycle
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
            METHOD_CLONE_SELF => handle_clone_self(instance_id, result, result_len),
            _ => NYB_E_METHOD_NOT_FOUND,
        }
    }
}

// ===== Method Handlers =====

unsafe fn handle_birth(result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    if preflight(result, result_len, 4) {
        return NYB_E_SHORT_BUFFER;
    }
    let id = allocate_instance_id();
    if store_instance(id, FileBoxInstance::new()).is_err() {
        return NYB_E_PLUGIN_ERROR;
    }
    let b = id.to_le_bytes();
    std::ptr::copy_nonoverlapping(b.as_ptr(), result, 4);
    *result_len = 4;
    NYB_SUCCESS
}

unsafe fn handle_fini(instance_id: u32) -> i32 {
    remove_instance(instance_id);
    NYB_SUCCESS
}

unsafe fn handle_open(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    match tlv_parse_two_strings(slice) {
        Ok((path, mode)) => {
            if preflight(result, result_len, 8) {
                return NYB_E_SHORT_BUFFER;
            }
            match with_instance_mut(instance_id, |inst| {
                match open_file(&mode, &path) {
                    Ok(file) => {
                        inst.file = Some(file);
                        inst.path = path;
                        true
                    }
                    Err(_) => false,
                }
            }) {
                Ok(true) => write_tlv_void(result, result_len),
                Ok(false) => NYB_E_PLUGIN_ERROR,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        Err(_) => NYB_E_INVALID_ARGS,
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

    // Check if path argument provided
    if args_len > 0 {
        // Static file read (with path)
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
    } else {
        // Instance file read
        match with_instance_mut(instance_id, |inst| {
            if let Some(file) = inst.file.as_mut() {
                let _ = file.seek(SeekFrom::Start(0));
                let mut buf = Vec::new();
                match file.read_to_end(&mut buf) {
                    Ok(_) => Some(buf),
                    Err(_) => None,
                }
            } else {
                None
            }
        }) {
            Ok(Some(buf)) => {
                let need = 8usize.saturating_add(buf.len());
                if preflight(result, result_len, need) {
                    return NYB_E_SHORT_BUFFER;
                }
                write_tlv_bytes(&buf, result, result_len)
            }
            Ok(None) => NYB_E_INVALID_HANDLE,
            Err(_) => NYB_E_INVALID_HANDLE,
        }
    }
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
            // Static file write
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
            // Instance file write
            if preflight(result, result_len, 12) {
                return NYB_E_SHORT_BUFFER;
            }
            match with_instance_mut(instance_id, |inst| {
                if let Some(file) = inst.file.as_mut() {
                    match file.write(&data) {
                        Ok(n) => {
                            if file.flush().is_ok() {
                                inst.buffer = Some(data.clone());
                                Some(n)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }) {
                Ok(Some(n)) => write_tlv_i32(n as i32, result, result_len),
                Ok(None) => NYB_E_PLUGIN_ERROR,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        Err(_) => NYB_E_INVALID_ARGS,
    }
}

unsafe fn handle_close(
    instance_id: u32,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if preflight(result, result_len, 8) {
        return NYB_E_SHORT_BUFFER;
    }
    match with_instance_mut(instance_id, |inst| {
        inst.file = None;
    }) {
        Ok(_) => write_tlv_void(result, result_len),
        Err(_) => NYB_E_INVALID_HANDLE,
    }
}

unsafe fn handle_exists(
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    match tlv_parse_one_string(slice) {
        Ok(path) => {
            if preflight(result, result_len, 8) {
                return NYB_E_SHORT_BUFFER;
            }
            let exists = std::path::Path::new(&path).exists();
            write_tlv_bool(exists, result, result_len)
        }
        Err(_) => NYB_E_INVALID_ARGS,
    }
}

unsafe fn handle_copy_from(
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let slice = std::slice::from_raw_parts(args, args_len);
    match tlv_parse_handle(slice) {
        Ok((_type_id, other_id)) => {
            if preflight(result, result_len, 8) {
                return NYB_E_SHORT_BUFFER;
            }

            // Lock instances once and perform copy
            match INSTANCES.lock() {
                Ok(mut map) => {
                    // Extract data from source
                    let mut data = Vec::new();
                    let mut copy_ok = false;

                    if let Some(src) = map.get(&other_id) {
                        if let Some(file) = src.file.as_ref() {
                            if let Ok(mut f) = file.try_clone() {
                                let _ = f.seek(SeekFrom::Start(0));
                                if f.read_to_end(&mut data).is_ok() {
                                    copy_ok = true;
                                }
                            }
                        }
                        if !copy_ok {
                            if let Some(buf) = src.buffer.as_ref() {
                                data.extend_from_slice(buf);
                                copy_ok = true;
                            }
                        }
                    } else {
                        return NYB_E_INVALID_HANDLE;
                    }

                    if !copy_ok {
                        return NYB_E_PLUGIN_ERROR;
                    }

                    // Write to destination
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
                        write_tlv_void(result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                }
                Err(_) => NYB_E_PLUGIN_ERROR,
            }
        }
        Err(_) => NYB_E_INVALID_ARGS,
    }
}

unsafe fn handle_clone_self(
    instance_id: u32,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if preflight(result, result_len, 16) {
        return NYB_E_SHORT_BUFFER;
    }

    match INSTANCES.lock() {
        Ok(mut map) => {
            if let Some(src) = map.get(&instance_id) {
                let new_id = allocate_instance_id();
                let mut new_inst = FileBoxInstance::with_path(src.path.clone());

                // Clone buffer if present
                if let Some(buf) = src.buffer.as_ref() {
                    new_inst.buffer = Some(buf.clone());
                }

                // Try to clone file handle
                if let Some(file) = src.file.as_ref() {
                    if let Ok(cloned) = file.try_clone() {
                        new_inst.file = Some(cloned);
                    }
                }

                map.insert(new_id, new_inst);
                write_tlv_handle(FILEBOX_TYPE_ID, new_id, result, result_len)
            } else {
                NYB_E_INVALID_HANDLE
            }
        }
        Err(_) => NYB_E_PLUGIN_ERROR,
    }
}