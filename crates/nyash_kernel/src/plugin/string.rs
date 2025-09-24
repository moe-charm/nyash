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

// Exported as: nyash.string.substring_sii(i8* s, i64 start, i64 end) -> i8*
#[export_name = "nyash.string.substring_sii"]
pub extern "C" fn nyash_string_substring_sii(s: *const i8, start: i64, end: i64) -> *mut i8 {
    use std::ffi::CStr;
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let src = unsafe { CStr::from_ptr(s) };
    let src = match src.to_str() {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };
    let n = src.len() as i64;
    let mut st = if start < 0 { 0 } else { start };
    let mut en = if end < 0 { 0 } else { end };
    if st > n {
        st = n;
    }
    if en > n {
        en = n;
    }
    if en < st {
        std::mem::swap(&mut st, &mut en);
    }
    let (st_u, en_u) = (st as usize, en as usize);
    let sub = &src[st_u.min(src.len())..en_u.min(src.len())];
    let mut bytes = sub.as_bytes().to_vec();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Exported as: nyash.string.lastIndexOf_ss(i8* s, i8* needle) -> i64
#[export_name = "nyash.string.lastIndexOf_ss"]
pub extern "C" fn nyash_string_lastindexof_ss(s: *const i8, needle: *const i8) -> i64 {
    use std::ffi::CStr;
    if s.is_null() || needle.is_null() {
        return -1;
    }
    let hs = unsafe { CStr::from_ptr(s) };
    let ns = unsafe { CStr::from_ptr(needle) };
    let h = match hs.to_str() {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let n = match ns.to_str() {
        Ok(v) => v,
        Err(_) => return -1,
    };
    if n.is_empty() {
        return h.len() as i64;
    }
    if let Some(pos) = h.rfind(n) {
        pos as i64
    } else {
        -1
    }
}

// Exported as: nyash.string.to_i8p_h(i64 handle) -> i8*
#[export_name = "nyash.string.to_i8p_h"]
pub extern "C" fn nyash_string_to_i8p_h(handle: i64) -> *mut i8 {
    use nyash_rust::jit::rt::handles;
    if handle <= 0 {
        // return "0" for consistency with existing fallback behavior
        let s = handle.to_string();
        let mut bytes = s.into_bytes();
        bytes.push(0);
        let boxed = bytes.into_boxed_slice();
        let raw = Box::into_raw(boxed) as *mut u8;
        return raw as *mut i8;
    }
    if let Some(obj) = handles::get(handle) {
        let s = obj.to_string_box().value;
        let mut bytes = s.into_bytes();
        bytes.push(0);
        let boxed = bytes.into_boxed_slice();
        let raw = Box::into_raw(boxed) as *mut u8;
        raw as *mut i8
    } else {
        // not found -> print numeric handle string
        let s = handle.to_string();
        let mut bytes = s.into_bytes();
        bytes.push(0);
        let boxed = bytes.into_boxed_slice();
        let raw = Box::into_raw(boxed) as *mut u8;
        raw as *mut i8
    }
}
