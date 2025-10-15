#![cfg(feature = "parser-c-abi")]
use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;

#[repr(C)]
struct HakoParseResult {
    abi_version: u32,
    struct_size: u32,
    success: u32,
    stmt_count: u32,
    kind: *const c_char,
    error_msg: *const c_char,
}

#[allow(non_camel_case_types)]
#[repr(i32)]
enum HakoParseMode {
    RUST = 0,
    HAKO = 1,
    BOTH = 2,
}

extern "C" {
    fn parse_source_dual(src: *const c_char, mode: HakoParseMode) -> *mut HakoParseResult;
    fn free_parse_result(r: *mut HakoParseResult);
}

fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    let mode = match env::var("SMOKES_PARSER_MODE").ok().as_deref() {
        Some("both") => HakoParseMode::BOTH,
        Some("hako") => HakoParseMode::HAKO,
        _ => HakoParseMode::RUST,
    };
    let args: Vec<String> = env::args().collect();
    let mut code = String::new();
    if args.len() >= 3 && args[1] == "-c" {
        code = args[2].clone();
    } else if args.len() >= 2 {
        code = fs::read_to_string(&args[1]).expect("read source file");
    } else {
        eprintln!("usage: parser_facade_both_min (-c CODE | FILE)");
        std::process::exit(2);
    }
    let ccode = CString::new(code).unwrap();
    let res = unsafe { parse_source_dual(ccode.as_ptr(), mode) };
    if res.is_null() {
        eprintln!("ERR: null result");
        std::process::exit(1);
    }
    let r = unsafe { &*res };
    if r.success == 1 {
        println!("OK kind={} stmts={}", cstr_to_string(r.kind), r.stmt_count);
        unsafe {
            free_parse_result(res);
        }
        return;
    } else {
        eprintln!("ERR: {}", cstr_to_string(r.error_msg));
        unsafe {
            free_parse_result(res);
        }
        std::process::exit(1);
    }
}
