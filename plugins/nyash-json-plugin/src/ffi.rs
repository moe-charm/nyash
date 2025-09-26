//! FFI definitions for yyjson C library integration

use std::os::raw::{c_char, c_void};

// External C functions for yyjson provider
extern "C" {
    pub fn nyash_json_shim_parse(text: *const c_char, len: usize) -> i32;
    pub fn nyjson_parse_doc(text: *const c_char, len: usize, out_err_code: *mut i32)
        -> *mut c_void;
    pub fn nyjson_doc_free(doc: *mut c_void);
    pub fn nyjson_doc_root(doc: *mut c_void) -> *mut c_void;
    pub fn nyjson_is_null(v: *mut c_void) -> i32;
    pub fn nyjson_is_bool(v: *mut c_void) -> i32;
    pub fn nyjson_is_int(v: *mut c_void) -> i32;
    pub fn nyjson_is_real(v: *mut c_void) -> i32;
    pub fn nyjson_is_str(v: *mut c_void) -> i32;
    pub fn nyjson_is_arr(v: *mut c_void) -> i32;
    pub fn nyjson_is_obj(v: *mut c_void) -> i32;
    pub fn nyjson_get_bool_val(v: *mut c_void) -> i32;
    pub fn nyjson_get_sint_val(v: *mut c_void) -> i64;
    pub fn nyjson_get_str_val(v: *mut c_void) -> *const c_char;
    pub fn nyjson_arr_size_val(v: *mut c_void) -> usize;
    pub fn nyjson_arr_get_val(v: *mut c_void, idx: usize) -> *mut c_void;
    pub fn nyjson_obj_size_val(v: *mut c_void) -> usize;
    pub fn nyjson_obj_get_key(v: *mut c_void, key: *const c_char) -> *mut c_void;
}
