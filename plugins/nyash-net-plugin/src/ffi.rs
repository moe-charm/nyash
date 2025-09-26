use std::ffi::CStr;
use std::os::raw::c_char;

// Safe wrapper: convert C string pointer to owned String.
// Safety details are contained within; caller gets a safe String.
pub fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

// Re-export a safe view over a raw byte slice pointer.
// This function is unsafe since it trusts the pointer/length.
pub unsafe fn slice<'a>(p: *const u8, len: usize) -> &'a [u8] {
    std::slice::from_raw_parts(p, len)
}
