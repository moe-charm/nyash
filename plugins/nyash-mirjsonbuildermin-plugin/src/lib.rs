//! MirJsonBuilderMin plugin (minimal) — emits MIR(JSON v0) strings

use std::ffi::CStr;
use std::os::raw::c_char;
use hako_abi_impl::tlv::{read_arg_i64, read_arg_string, write_tlv_string};
use hako_abi_impl::{define_instance_storage, with_instance_mut};

// Error codes (aligned)
const OK: i32 = 0;
const E_INVALID_ARGS: i32 = -4;
const E_INVALID_METHOD: i32 = -3;

// Method ids
const M_BIRTH: u32 = 0;
const M_START_MODULE: u32 = 1;
const M_START_FUNCTION: u32 = 2;
const M_START_BLOCK: u32 = 3;
const M_ADD_CONST: u32 = 4;
const M_ADD_COMPARE: u32 = 5;
const M_ADD_RET: u32 = 6;
const M_END_ALL: u32 = 7;
const M_TO_STRING: u32 = 8;
const M_FINI: u32 = u32::MAX;

// Type id (unique; keep in sync with hako.toml if used)
const TYPE_ID: u32 = 72;

struct Builder {
    buf: String,
    first_inst: bool,
}

define_instance_storage!(Builder);

fn esc(s: &str) -> String {
    // minimal JSON quoting for test strings (names are ascii)
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out
}

extern "C" fn resolve(name: *const c_char) -> u32 {
    if name.is_null() { return 0; }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => M_BIRTH,
        "start_module" => M_START_MODULE,
        "start_function" => M_START_FUNCTION,
        "start_block" => M_START_BLOCK,
        "add_const" => M_ADD_CONST,
        "add_compare" => M_ADD_COMPARE,
        "add_ret" => M_ADD_RET,
        "end_all" => M_END_ALL,
        "to_string" | "emit_to_string" => M_TO_STRING,
        "fini" => M_FINI,
        _ => 0,
    }
}

extern "C" fn invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        M_BIRTH => {
            // create new empty builder
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                inst.buf.clear();
                inst.first_inst = true;
            })
            .unwrap_or_else(|_| ());
            OK
        }
        M_START_MODULE => with_instance_mut!(instance_id, |inst: &mut Builder| {
            inst.buf.push_str("{\"functions\":[");
            OK
        })
        .unwrap_or(E_INVALID_METHOD),
        M_START_FUNCTION => {
            let name = match read_arg_string(args, args_len, 0) { Some(s) => s, None => return E_INVALID_ARGS };
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                inst.buf.push_str("{\"name\":\"");
                inst.buf.push_str(&esc(&name));
                inst.buf.push_str("\",\"params\":[],\"blocks\":[");
                OK
            })
            .unwrap_or(E_INVALID_METHOD)
        }
        M_START_BLOCK => {
            let id = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return E_INVALID_ARGS };
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                inst.first_inst = true;
                inst.buf.push_str(&format!("{{\"id\":{},\"instructions\":[", id));
                OK
            })
            .unwrap_or(E_INVALID_METHOD)
        }
        M_ADD_CONST => {
            let dst = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return E_INVALID_ARGS };
            let val = match read_arg_i64(args, args_len, 1) { Some(v) => v, None => return E_INVALID_ARGS };
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                if !inst.first_inst { inst.buf.push(','); } else { inst.first_inst = false; }
                inst.buf.push_str(&format!(
                    "{{\"op\":\"const\",\"dst\":{},\"value\":{{\"type\":\"i64\",\"value\":{}}}}}",
                    dst, val
                ));
                OK
            })
            .unwrap_or(E_INVALID_METHOD)
        }
        M_ADD_COMPARE => {
            let kind = match read_arg_string(args, args_len, 0) { Some(s) => s, None => return E_INVALID_ARGS };
            let lhs = match read_arg_i64(args, args_len, 1) { Some(v) => v, None => return E_INVALID_ARGS };
            let rhs = match read_arg_i64(args, args_len, 2) { Some(v) => v, None => return E_INVALID_ARGS };
            let dst = match read_arg_i64(args, args_len, 3) { Some(v) => v, None => return E_INVALID_ARGS };
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                if !inst.first_inst { inst.buf.push(','); } else { inst.first_inst = false; }
                inst.buf.push_str(&format!(
                    "{{\"op\":\"compare\",\"kind\":\"{}\",\"dst\":{},\"lhs\":{},\"rhs\":{}}}",
                    esc(&kind), dst, lhs, rhs
                ));
                OK
            })
            .unwrap_or(E_INVALID_METHOD)
        }
        M_ADD_RET => {
            let val = match read_arg_i64(args, args_len, 0) { Some(v) => v, None => return E_INVALID_ARGS };
            with_instance_mut!(instance_id, |inst: &mut Builder| {
                if !inst.first_inst { inst.buf.push(','); } else { inst.first_inst = false; }
                inst.buf.push_str(&format!("{{\"op\":\"ret\",\"value\":{}}}", val));
                OK
            })
            .unwrap_or(E_INVALID_METHOD)
        }
        M_END_ALL => with_instance_mut!(instance_id, |inst: &mut Builder| {
            inst.buf.push_str("]}]}]}");
            OK
        })
        .unwrap_or(E_INVALID_METHOD),
        M_TO_STRING => {
            // Return TLV string of current buffer
            let s = with_instance_mut!(instance_id, |inst: &mut Builder| inst.buf.clone()).unwrap_or_default();
            unsafe { write_tlv_string(&s, result, result_len) }
        }
        M_FINI => OK,
        _ => E_INVALID_METHOD,
    }
}

#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const c_char,
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

#[no_mangle]
pub static nyash_typebox_MirJsonBuilderMin: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"MirJsonBuilderMin\0".as_ptr() as *const c_char,
    resolve: Some(resolve),
    invoke_id: Some(invoke_id),
    capabilities: 0,
};

#[no_mangle]
pub static nyash_plugin_name: &[u8] = b"nyash-mirjsonbuildermin\0";
#[no_mangle]
pub static nyash_plugin_version: &[u8] = b"0.1.0\0";

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 { OK }
#[no_mangle]
pub extern "C" fn nyash_plugin_fini() -> i32 { OK }
