// ---- String helpers for LLVM lowering ----
// Exported as: nyash_string_new(i8* ptr, i32 len) -> i8*
#[no_mangle]
pub extern "C" fn nyash_string_new(ptr: *const u8, len: i32) -> *mut i8 {
    use std::ptr;
    if ptr.is_null() || len < 0 {
        return std::ptr::null_mut();
    }
    let n = len as usize;
    // Allocate n+1 and null-terminate for C interop (puts, etc.)
    let mut buf = Vec::<u8>::with_capacity(n + 1);
    unsafe {
        ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr(), n);
        buf.set_len(n);
    }
    buf.push(0);
    let boxed = buf.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// ---- String concat helpers for LLVM lowering ----
// Exported as: nyash.string.concat_ss(i8* a, i8* b) -> i8*
#[export_name = "nyash.string.concat_ss"]
pub extern "C" fn nyash_string_concat_ss(a: *const i8, b: *const i8) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() {
                s.push_str(sa);
            }
        }
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() {
                s.push_str(sb);
            }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Exported as: nyash.string.concat_si(i8* a, i64 b) -> i8*
#[export_name = "nyash.string.concat_si"]
pub extern "C" fn nyash_string_concat_si(a: *const i8, b: i64) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() {
                s.push_str(sa);
            }
        }
    }
    s.push_str(&b.to_string());
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Exported as: nyash.string.concat_is(i64 a, i8* b) -> i8*
#[export_name = "nyash.string.concat_is"]
pub extern "C" fn nyash_string_concat_is(a: i64, b: *const i8) -> *mut i8 {
    let mut s = a.to_string();
    unsafe {
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() {
                s.push_str(sb);
            }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}
