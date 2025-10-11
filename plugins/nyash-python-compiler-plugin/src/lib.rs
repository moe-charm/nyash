use once_cell::sync::Lazy;
use serde_json::Value as Json;
use std::sync::Mutex;

const NYB_SUCCESS: i32 = 0;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_SHORT_BUFFER: i32 = -1;

const TYPE_ID_COMPILER: u32 = 61;
const METHOD_BIRTH: u32 = 0;
const METHOD_COMPILE: u32 = 1;
const METHOD_FINI: u32 = u32::MAX;

static NEXT_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1));

// legacy v1 abi/init removed


// ===== TypeBox ABI v2 (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,     // 'TYBX'
    pub version: u16,     // 1
    pub struct_size: u16, // sizeof(NyashTypeBoxFfi)
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

use std::ffi::CStr;
extern "C" fn pycompiler_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => METHOD_BIRTH,
        "compile" => METHOD_COMPILE,
        "fini" => METHOD_FINI,
        _ => 0,
    }
}

extern "C" fn pycompiler_invoke_id(
    _instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => unsafe {
            let mut id_g = NEXT_ID.lock().unwrap();
            let id = *id_g;
            *id_g += 1;
            if result_len.is_null() {
                return NYB_E_SHORT_BUFFER;
            }
            let need = 4usize;
            if *result_len < need {
                *result_len = need;
                return NYB_E_SHORT_BUFFER;
            }
            let out = std::slice::from_raw_parts_mut(result, *result_len);
            out[0..4].copy_from_slice(&(id as u32).to_le_bytes());
            *result_len = need;
            NYB_SUCCESS
        },
        METHOD_COMPILE => unsafe {
            let ir = if args.is_null() || args_len < 8 {
                None
            } else {
                let buf = std::slice::from_raw_parts(args, args_len);
                let tag = u16::from_le_bytes([buf[4], buf[5]]);
                let len = u16::from_le_bytes([buf[6], buf[7]]) as usize;
                if tag == 6 && 8 + len <= buf.len() {
                    std::str::from_utf8(&buf[8..8 + len])
                        .ok()
                        .map(|s| s.to_string())
                } else {
                    None
                }
            };
            let nyash_source = if let Some(s) = ir.or_else(|| std::env::var("NYASH_PY_IR").ok()) {
                match serde_json::from_str::<Json>(&s).ok() {
                    Some(Json::Object(map)) => {
                        if let Some(Json::String(src)) = map.get("nyash_source") {
                            src.clone()
                        } else if let Some(module) = map.get("module") {
                            let mut ret_expr = "0".to_string();
                            if let Some(funcs) = module.get("functions").and_then(|v| v.as_array())
                            {
                                if let Some(fun0) = funcs.get(0) {
                                    if let Some(retv) = fun0.get("return_value") {
                                        if retv.is_number() {
                                            ret_expr = retv.to_string();
                                        } else if let Some(s) = retv.as_str() {
                                            ret_expr = s.to_string();
                                        }
                                    }
                                }
                            }
                            format!(
                                "static box Generated {\n  main() {\n    return {}\n  }\n}}",
                                ret_expr
                            )
                        } else {
                            "static box Generated { main() { return 0 } }".to_string()
                        }
                    }
                    _ => "static box Generated { main() { return 0 } }".to_string(),
                }
            } else {
                "static box Generated { main() { return 0 } }".to_string()
            };
            let bytes = nyash_source.as_bytes();
            if result_len.is_null() {
                return NYB_E_SHORT_BUFFER;
            }
            let need = 4 + bytes.len();
            if *result_len < need {
                *result_len = need;
                return NYB_E_SHORT_BUFFER;
            }
            let out = std::slice::from_raw_parts_mut(result, *result_len);
            out[0..2].copy_from_slice(&6u16.to_le_bytes());
            out[2..4].copy_from_slice(&(bytes.len() as u16).to_le_bytes());
            out[4..4 + bytes.len()].copy_from_slice(bytes);
            *result_len = need;
            NYB_SUCCESS
        },
        METHOD_FINI => NYB_SUCCESS,
        _ => NYB_E_INVALID_METHOD,
    }
}

#[no_mangle]
pub static nyash_typebox_PythonCompilerBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"PythonCompilerBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(pycompiler_resolve),
    invoke_id: Some(pycompiler_invoke_id),
    capabilities: 0,
};
