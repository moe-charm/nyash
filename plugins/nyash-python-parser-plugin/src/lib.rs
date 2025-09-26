use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ParseResult {
    success: bool,
    dump: Option<String>,
    counts: ParseCounts,
    unsupported: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParseCounts {
    total_nodes: usize,
    functions: usize,
    classes: usize,
    supported: usize,
    unsupported: usize,
}

/* legacy v1 entries removed: nyash_plugin_abi_version/nyash_plugin_init/nyash_plugin_invoke */

/// FFI: Pythonコードをパース
#[no_mangle]
pub extern "C" fn nyash_python_parse(
    code: *const std::os::raw::c_char,
) -> *mut std::os::raw::c_char {
    let code = unsafe {
        if code.is_null() {
            return std::ptr::null_mut();
        }
        match std::ffi::CStr::from_ptr(code).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let result = Python::with_gil(|py| parse_python_code(py, code));

    match serde_json::to_string(&result) {
        Ok(json) => {
            let c_str = std::ffi::CString::new(json).unwrap();
            c_str.into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// FFI: 文字列解放
#[no_mangle]
pub extern "C" fn nyash_python_free_string(ptr: *mut std::os::raw::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}

fn parse_python_code(py: Python, code: &str) -> ParseResult {
    let mut result = ParseResult {
        success: false,
        dump: None,
        counts: ParseCounts {
            total_nodes: 0,
            functions: 0,
            classes: 0,
            supported: 0,
            unsupported: 0,
        },
        unsupported: Vec::new(),
    };

    // Pythonのastモジュールをインポート
    let ast_module = match py.import_bound("ast") {
        Ok(m) => m,
        Err(e) => {
            result.dump = Some(format!("Failed to import ast module: {}", e));
            return result;
        }
    };

    // コードをパース
    let tree = match ast_module.call_method1("parse", (code,)) {
        Ok(t) => t,
        Err(e) => {
            result.dump = Some(format!("Parse error: {}", e));
            return result;
        }
    };

    // ASTをダンプ（文字列表現）
    if let Ok(dump_str) = ast_module.call_method1("dump", (&tree,)) {
        if let Ok(s) = dump_str.extract::<String>() {
            result.dump = Some(s);
        }
    }

    // ASTを解析してカウント
    analyze_ast(py, &tree, &mut result);

    result.success = true;
    result
}

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

const TYPE_ID_PARSER: u32 = 60;
const METHOD_BIRTH: u32 = 0;
const METHOD_PARSE: u32 = 1;
const METHOD_FINI: u32 = u32::MAX;

use std::ffi::CStr;
extern "C" fn pyparser_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => METHOD_BIRTH,
        "parse" => METHOD_PARSE,
        "fini" => METHOD_FINI,
        _ => 0,
    }
}

extern "C" fn pyparser_invoke_id(
    _instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => unsafe {
            let instance_id = 1u32; // simple singleton
            if result_len.is_null() {
                return -1;
            }
            if *result_len < 4 {
                *result_len = 4;
                return -1;
            }
            let out = std::slice::from_raw_parts_mut(result, *result_len);
            out[0..4].copy_from_slice(&instance_id.to_le_bytes());
            *result_len = 4;
            0
        },
        METHOD_PARSE => {
            // Decode TLV string from args if present, else env fallback
            let code = unsafe {
                if args.is_null() || args_len < 4 {
                    std::env::var("NYASH_PY_CODE")
                        .unwrap_or_else(|_| "def main():\n    return 0".to_string())
                } else {
                    let buf = std::slice::from_raw_parts(args, args_len);
                    if args_len >= 8 {
                        let tag = u16::from_le_bytes([buf[0], buf[1]]);
                        let len = u16::from_le_bytes([buf[2], buf[3]]) as usize;
                        if tag == 6 && 4 + len <= args_len {
                            match std::str::from_utf8(&buf[4..4 + len]) {
                                Ok(s) => s.to_string(),
                                Err(_) => std::env::var("NYASH_PY_CODE")
                                    .unwrap_or_else(|_| "def main():\n    return 0".to_string()),
                            }
                        } else {
                            std::env::var("NYASH_PY_CODE")
                                .unwrap_or_else(|_| "def main():\n    return 0".to_string())
                        }
                    } else {
                        std::env::var("NYASH_PY_CODE")
                            .unwrap_or_else(|_| "def main():\n    return 0".to_string())
                    }
                }
            };
            let parse_result = Python::with_gil(|py| parse_python_code(py, &code));
            match serde_json::to_string(&parse_result) {
                Ok(json) => unsafe {
                    if result_len.is_null() {
                        return -1;
                    }
                    let bytes = json.as_bytes();
                    let need = 4 + bytes.len();
                    if *result_len < need {
                        *result_len = need;
                        return -1;
                    }
                    let out = std::slice::from_raw_parts_mut(result, *result_len);
                    out[0..2].copy_from_slice(&6u16.to_le_bytes());
                    out[2..4].copy_from_slice(&(bytes.len() as u16).to_le_bytes());
                    out[4..4 + bytes.len()].copy_from_slice(bytes);
                    *result_len = need;
                    0
                },
                Err(_) => -4,
            }
        }
        METHOD_FINI => 0,
        _ => -3,
    }
}

#[no_mangle]
pub static nyash_typebox_PythonParserBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"PythonParserBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(pyparser_resolve),
    invoke_id: Some(pyparser_invoke_id),
    capabilities: 0,
};

fn analyze_ast(_py: Python, node: &Bound<'_, PyAny>, result: &mut ParseResult) {
    result.counts.total_nodes += 1;

    // ノードタイプを取得 - __class__.__name__ を使用
    if let Ok(class_obj) = node.getattr("__class__") {
        if let Ok(name_obj) = class_obj.getattr("__name__") {
            if let Ok(type_name) = name_obj.extract::<String>() {
                match type_name.as_str() {
                    "FunctionDef" => {
                        result.counts.functions += 1;
                        result.counts.supported += 1;
                    }
                    "AsyncFunctionDef" => {
                        result.counts.functions += 1;
                        result.counts.unsupported += 1;
                        result.unsupported.push("async function".to_string());
                    }
                    "ClassDef" => {
                        result.counts.classes += 1;
                        result.counts.unsupported += 1;
                        result.unsupported.push("class definition".to_string());
                    }
                    "For" | "While" | "If" => {
                        result.counts.supported += 1;
                    }
                    "Yield" | "YieldFrom" => {
                        result.counts.unsupported += 1;
                        result.unsupported.push("generator".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    // 子ノードを再帰的に解析
    // ast.walk() を使って全ノードを取得
    if let Ok(ast_module) = node.py().import_bound("ast") {
        if let Ok(walk_iter) = ast_module.call_method1("walk", (node,)) {
            if let Ok(nodes) = walk_iter.iter() {
                for child_result in nodes {
                    if let Ok(child) = child_result {
                        // 自分自身はスキップ（すでにカウント済み）
                        if !child.is(node) {
                            // 再帰的に解析（ただし walk は全ノードを返すので、
                            // 実際には再帰なしでフラットに処理される）
                            result.counts.total_nodes += 1;

                            if let Ok(class_obj) = child.getattr("__class__") {
                                if let Ok(name_obj) = class_obj.getattr("__name__") {
                                    if let Ok(type_name) = name_obj.extract::<String>() {
                                        match type_name.as_str() {
                                            "FunctionDef" => {
                                                result.counts.functions += 1;
                                                result.counts.supported += 1;
                                            }
                                            "AsyncFunctionDef" => {
                                                result.counts.functions += 1;
                                                result.counts.unsupported += 1;
                                                if !result
                                                    .unsupported
                                                    .contains(&"async function".to_string())
                                                {
                                                    result
                                                        .unsupported
                                                        .push("async function".to_string());
                                                }
                                            }
                                            "ClassDef" => {
                                                result.counts.classes += 1;
                                                result.counts.unsupported += 1;
                                                if !result
                                                    .unsupported
                                                    .contains(&"class definition".to_string())
                                                {
                                                    result
                                                        .unsupported
                                                        .push("class definition".to_string());
                                                }
                                            }
                                            "For" | "While" | "If" => {
                                                result.counts.supported += 1;
                                            }
                                            "Yield" | "YieldFrom" => {
                                                result.counts.unsupported += 1;
                                                if !result
                                                    .unsupported
                                                    .contains(&"generator".to_string())
                                                {
                                                    result
                                                        .unsupported
                                                        .push("generator".to_string());
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let code = "def main():\n    return 0";
            let result = parse_python_code(py, code);

            assert!(result.success);
            assert_eq!(result.counts.functions, 1);
            assert_eq!(result.counts.supported, 1);
        });
    }
}
