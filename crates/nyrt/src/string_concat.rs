// String concat helpers for LLVM lowering

#[no_mangle]
#[export_name = "nyash.string.concat_ss"]
pub extern "C" fn nyash_string_concat_ss(a: *const i8, b: *const i8) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() { s.push_str(sa); }
        }
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() { s.push_str(sb); }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

#[no_mangle]
#[export_name = "nyash.string.concat_si"]
pub extern "C" fn nyash_string_concat_si(a: *const i8, b: i64) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() { s.push_str(sa); }
        }
    }
    s.push_str(&b.to_string());
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

#[no_mangle]
#[export_name = "nyash.string.concat_is"]
pub extern "C" fn nyash_string_concat_is(a: i64, b: *const i8) -> *mut i8 {
    let mut s = a.to_string();
    unsafe {
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() { s.push_str(sb); }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Needed imports
use std::os::raw::c_char as i8;
use std::ffi::CStr;
use std::boxed::Box;
use std::string::String;

