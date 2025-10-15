#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct HakoParseResult {
    pub abi_version: u32,
    pub struct_size: u32,
    pub success: u32,
    pub stmt_count: u32,
    pub kind: *const c_char,
    pub error_msg: *const c_char,
}

fn make_c_string_const(s: &str) -> *const c_char {
    CString::new(s)
        .map(|c| c.into_raw() as *const c_char)
        .unwrap_or(std::ptr::null())
}

/// Allocate a new C string (owned by Rust allocator). C side must free via hako_rust_free_cstr.
#[no_mangle]
pub extern "C" fn hako_rust_make_cstring(src_utf8: *const c_char) -> *const c_char {
    if src_utf8.is_null() {
        return std::ptr::null();
    }
    unsafe {
        match CStr::from_ptr(src_utf8).to_str() {
            Ok(s) => make_c_string_const(s),
            Err(_) => std::ptr::null(),
        }
    }
}

#[no_mangle]
pub extern "C" fn hako_rust_free_cstr(s: *const c_char) {
    if s.is_null() { return; }
    unsafe { let _ = CString::from_raw(s as *mut c_char); }
}

fn count_stmts_and_kind(ast: &crate::ast::ASTNode) -> (u32, &'static str) {
    match ast {
        crate::ast::ASTNode::Program { statements, .. } => (statements.len() as u32, "Program"),
        _ => (0, "Program"),
    }
}

#[no_mangle]
pub extern "C" fn hako_parse_source_rust_header(
    src_utf8: *const c_char,
    out_stmt_count: *mut u32,
    out_kind: *mut *const c_char,
    out_error: *mut *const c_char,
) -> i32 {
    if src_utf8.is_null() || out_stmt_count.is_null() || out_kind.is_null() || out_error.is_null() {
        return 0;
    }
    unsafe { *out_error = std::ptr::null(); }
    let code = unsafe { CStr::from_ptr(src_utf8) };
    let code = match code.to_str() { Ok(s) => s, Err(_) => {
        unsafe { *out_error = make_c_string("invalid utf-8"); }
        return 0;
    }};
    match crate::front::parser_layer::facade::parse_source_to_ast(code) {
        Ok(ast) => {
            let (n, kind) = count_stmts_and_kind(&ast);
            unsafe {
                *out_stmt_count = n;
                *out_kind = make_c_string_const(kind);
            }
            1
        }
        Err(e) => {
            unsafe { *out_error = make_c_string_const(&e.message); }
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn hako_parse_source_hako_header(
    src_utf8: *const c_char,
    out_stmt_count: *mut u32,
    out_kind: *mut *const c_char,
    out_error: *mut *const c_char,
) -> i32 {
    if src_utf8.is_null() || out_stmt_count.is_null() || out_kind.is_null() || out_error.is_null() {
        return 0;
    }
    unsafe { *out_error = std::ptr::null(); }
    let code = unsafe { CStr::from_ptr(src_utf8) };
    let code = match code.to_str() { Ok(s) => s, Err(_) => {
        unsafe { *out_error = make_c_string_const("invalid utf-8"); }
        return 0;
    }};
    // Phase‑4: Until Hakorune parser is active, reuse Rust parser for parity
    match crate::front::parser_layer::facade::parse_source_to_ast(code) {
        Ok(ast) => {
            let (n, kind) = count_stmts_and_kind(&ast);
            unsafe {
                *out_stmt_count = n;
                *out_kind = make_c_string_const(kind);
            }
            1
        }
        Err(e) => {
            unsafe { *out_error = make_c_string_const(&e.message); }
            0
        }
    }
}
