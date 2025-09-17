//! Nyash Python Plugin (Phase 15):
//! - ABI v1 compatible entry points + ABI v2 TypeBox exports
//! - Two Box types: PyRuntimeBox (TYPE_ID=40) and PyObjectBox (TYPE_ID=41)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// ===== Error Codes (aligned with other plugins) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const _NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Type IDs (must match nyash.toml) =====
const _TYPE_ID_PY_RUNTIME: u32 = 40;
const TYPE_ID_PY_OBJECT: u32 = 41;

// ===== Method IDs (initial draft) =====
// PyRuntimeBox
const PY_METHOD_BIRTH: u32 = 0; // returns instance_id (u32 LE, no TLV)
const PY_METHOD_EVAL: u32 = 1; // args: string code -> returns Handle(PyObject)
const PY_METHOD_IMPORT: u32 = 2; // args: string name -> returns Handle(PyObject)
const PY_METHOD_FINI: u32 = u32::MAX; // destructor
                                      // Result-returning variants (R)
const PY_METHOD_EVAL_R: u32 = 11;
const PY_METHOD_IMPORT_R: u32 = 12;

// PyObjectBox
const PYO_METHOD_BIRTH: u32 = 0; // reserved (should not be used directly)
const PYO_METHOD_GETATTR: u32 = 1; // args: string name -> returns Handle(PyObject)
const PYO_METHOD_CALL: u32 = 2; // args: variadic TLV -> returns Handle(PyObject)
const PYO_METHOD_STR: u32 = 3; // returns String
const PYO_METHOD_CALL_KW: u32 = 5; // args: key:string, val:TLV, ... -> returns Handle(PyObject)
const PYO_METHOD_FINI: u32 = u32::MAX; // destructor
                                       // Result-returning variants (R)
const PYO_METHOD_GETATTR_R: u32 = 11;
const PYO_METHOD_CALL_R: u32 = 12;
const PYO_METHOD_CALL_KW_R: u32 = 15;

// ===== Minimal in-memory state for stubs =====
#[derive(Default)]
struct PyRuntimeInstance {
    globals: Option<*mut PyObject>,
}
// Safety: Access to CPython state is guarded by the GIL in all call sites
// and we only store raw pointers captured under the GIL. We never mutate
// from multiple threads without reacquiring the GIL. Therefore, mark as
// Send/Sync for storage inside global Lazy<Mutex<...>>.
unsafe impl Send for PyRuntimeInstance {}
unsafe impl Sync for PyRuntimeInstance {}

#[derive(Default)]
struct PyObjectInstance {}

static RUNTIMES: Lazy<Mutex<HashMap<u32, PyRuntimeInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PYOBJS: Lazy<Mutex<HashMap<u32, PyObjectInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static RT_COUNTER: AtomicU32 = AtomicU32::new(1);
static OBJ_COUNTER: AtomicU32 = AtomicU32::new(1);

// ====== CPython FFI and GIL guard ======
mod ffi;
mod gil;
mod pytypes;
use ffi::{ensure_cpython, PyObject};
use pytypes::{DecodedValue, PyOwned};

// loader moved to ffi.rs

// legacy v1 abi/init removed

/* legacy v1 entry removed
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match type_id {
        TYPE_ID_PY_RUNTIME => {
            handle_py_runtime(method_id, instance_id, args, args_len, result, result_len)
        }
        TYPE_ID_PY_OBJECT => {
            handle_py_object(method_id, instance_id, args, args_len, result, result_len)
        }
        _ => NYB_E_INVALID_TYPE,
    }
}
*/

fn handle_py_runtime(
    method_id: u32,
    _instance_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match method_id {
            PY_METHOD_BIRTH => {
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return NYB_E_SHORT_BUFFER;
                }
                if ensure_cpython().is_err() {
                    return NYB_E_PLUGIN_ERROR;
                }
                let id = RT_COUNTER.fetch_add(1, Ordering::Relaxed);
                let mut inst = PyRuntimeInstance::default();
                if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                    let c_main = pytypes::cstring_from_str("__main__").expect("literal __main__");
                    let _gil = gil::GILGuard::acquire(cpy);
                    let module = (cpy.PyImport_AddModule)(c_main.as_ptr());
                    if !module.is_null() {
                        let dict = (cpy.PyModule_GetDict)(module);
                        if !dict.is_null() {
                            inst.globals = Some(dict);
                        }
                    }
                }
                if let Ok(mut map) = RUNTIMES.lock() {
                    map.insert(id, inst);
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
                let bytes = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            PY_METHOD_FINI => {
                // Only drop runtime slot; avoid calling Py_Finalize to prevent shutdown crashes.
                if let Ok(mut map) = RUNTIMES.lock() {
                    map.remove(&_instance_id);
                }
                NYB_SUCCESS
            }
            PY_METHOD_EVAL | PY_METHOD_EVAL_R => {
                if ensure_cpython().is_err() {
                    return NYB_E_PLUGIN_ERROR;
                }
                // Allow zero-arg eval by reading from env var (NYASH_PY_EVAL_CODE) for bootstrap demos
                let argc = pytypes::count_tlv_args(_args, _args_len);
                let code = if argc == 0 {
                    std::env::var("NYASH_PY_EVAL_CODE").unwrap_or_else(|_| "".to_string())
                } else {
                    if let Some(s) = read_arg_string(_args, _args_len, 0) {
                        s
                    } else {
                        return NYB_E_INVALID_ARGS;
                    }
                };
                let c_code = match pytypes::cstring_from_str(&code) {
                    Ok(s) => s,
                    Err(_) => return NYB_E_INVALID_ARGS,
                };
                if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                    let _gil = gil::GILGuard::acquire(cpy);
                    // use per-runtime globals if available
                    let mut dict: *mut PyObject = std::ptr::null_mut();
                    if let Ok(map) = RUNTIMES.lock() {
                        if let Some(rt) = map.get(&_instance_id) {
                            if let Some(g) = rt.globals {
                                dict = g;
                            }
                        }
                    }
                    if dict.is_null() {
                        let c_main =
                            pytypes::cstring_from_str("__main__").expect("literal __main__");
                        let module = (cpy.PyImport_AddModule)(c_main.as_ptr());
                        if module.is_null() {
                            return NYB_E_PLUGIN_ERROR;
                        }
                        dict = (cpy.PyModule_GetDict)(module);
                    }
                    // 258 == Py_eval_input
                    let obj = (cpy.PyRun_StringFlags)(
                        c_code.as_ptr(),
                        258,
                        dict,
                        dict,
                        std::ptr::null_mut(),
                    );
                    if obj.is_null() {
                        let msg = pytypes::take_py_error_string(cpy);
                        if method_id == PY_METHOD_EVAL_R {
                            return NYB_E_PLUGIN_ERROR;
                        }
                        if let Some(m) = msg {
                            return write_tlv_string(&m, result, result_len);
                        }
                        return NYB_E_PLUGIN_ERROR;
                    }
                    if (method_id == PY_METHOD_EVAL || method_id == PY_METHOD_EVAL_R)
                        && should_autodecode()
                    {
                        if let Some(decoded) = pytypes::autodecode(cpy, obj) {
                            if write_autodecode_result(&decoded, result, result_len) {
                                (cpy.Py_DecRef)(obj);
                                return NYB_SUCCESS;
                            }
                        }
                    }
                    // Store as PyObjectBox handle
                    let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut map) = PYOBJS.lock() {
                        map.insert(id, PyObjectInstance::default());
                    } else {
                        (cpy.Py_DecRef)(obj);
                        return NYB_E_PLUGIN_ERROR;
                    }
                    // Keep reference (obj is new ref). We model store via separate map and hold pointer via raw address table.
                    // To actually manage pointer per id, we extend PyObjectInstance in 10.5b with a field. For now, attach through side-table.
                    let owned = PyOwned::from_new(obj).expect("non-null PyObject");
                    PY_HANDLES.lock().unwrap().insert(id, owned);
                    return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
                }
                NYB_E_PLUGIN_ERROR
            }
            PY_METHOD_IMPORT | PY_METHOD_IMPORT_R => {
                if ensure_cpython().is_err() {
                    return NYB_E_PLUGIN_ERROR;
                }
                let Some(name) = read_arg_string(_args, _args_len, 0) else {
                    return NYB_E_INVALID_ARGS;
                };
                let c_name = match pytypes::cstring_from_str(&name) {
                    Ok(s) => s,
                    Err(_) => return NYB_E_INVALID_ARGS,
                };
                if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                    let _gil = gil::GILGuard::acquire(cpy);
                    let obj = (cpy.PyImport_ImportModule)(c_name.as_ptr());
                    if obj.is_null() {
                        let msg = pytypes::take_py_error_string(cpy);
                        if method_id == PY_METHOD_IMPORT_R {
                            return NYB_E_PLUGIN_ERROR;
                        }
                        if let Some(m) = msg {
                            return write_tlv_string(&m, result, result_len);
                        }
                        return NYB_E_PLUGIN_ERROR;
                    }
                    // expose module into runtime globals
                    if let Ok(map) = RUNTIMES.lock() {
                        if let Some(rt) = map.get(&_instance_id) {
                            if let Some(gl) = rt.globals {
                                (cpy.PyDict_SetItemString)(gl, c_name.as_ptr(), obj);
                            }
                        }
                    }
                    let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut map) = PYOBJS.lock() {
                        map.insert(id, PyObjectInstance::default());
                    } else {
                        (cpy.Py_DecRef)(obj);
                        return NYB_E_PLUGIN_ERROR;
                    }
                    let owned = PyOwned::from_new(obj).expect("non-null PyObject");
                    PY_HANDLES.lock().unwrap().insert(id, owned);
                    return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
                }
                NYB_E_PLUGIN_ERROR
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

fn handle_py_object(
    method_id: u32,
    instance_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        PYO_METHOD_BIRTH => NYB_E_INVALID_METHOD, // should be created via runtime
        PYO_METHOD_FINI => {
            PY_HANDLES.lock().unwrap().remove(&instance_id);
            if let Ok(mut map) = PYOBJS.lock() {
                map.remove(&instance_id);
                NYB_SUCCESS
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        PYO_METHOD_GETATTR | PYO_METHOD_GETATTR_R => {
            if ensure_cpython().is_err() {
                return NYB_E_PLUGIN_ERROR;
            }
            let Some(name) = read_arg_string(_args, _args_len, 0) else {
                return NYB_E_INVALID_ARGS;
            };
            if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                let obj_ptr = {
                    let guard = PY_HANDLES.lock().unwrap();
                    let Some(handle) = guard.get(&instance_id) else {
                        return NYB_E_INVALID_HANDLE;
                    };
                    handle.as_ptr()
                };
                let c_name = match pytypes::cstring_from_str(&name) {
                    Ok(s) => s,
                    Err(_) => return NYB_E_INVALID_ARGS,
                };
                let _gil = gil::GILGuard::acquire(cpy);
                let attr = unsafe { (cpy.PyObject_GetAttrString)(obj_ptr, c_name.as_ptr()) };
                if attr.is_null() {
                    let msg = pytypes::take_py_error_string(cpy);
                    if method_id == PYO_METHOD_GETATTR_R {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    if let Some(m) = msg {
                        return write_tlv_string(&m, result, result_len);
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
                if should_autodecode() {
                    if let Some(decoded) = pytypes::autodecode(cpy, attr) {
                        if write_autodecode_result(&decoded, result, result_len) {
                            unsafe {
                                (cpy.Py_DecRef)(attr);
                            }
                            return NYB_SUCCESS;
                        }
                    }
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                let owned = unsafe { PyOwned::from_new(attr).expect("non-null PyObject") };
                PY_HANDLES.lock().unwrap().insert(id, owned);
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_CALL | PYO_METHOD_CALL_R => {
            if ensure_cpython().is_err() {
                return NYB_E_PLUGIN_ERROR;
            }
            if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                let func_ptr = {
                    let guard = PY_HANDLES.lock().unwrap();
                    let Some(handle) = guard.get(&instance_id) else {
                        return NYB_E_INVALID_HANDLE;
                    };
                    handle.as_ptr()
                };
                let _gil = gil::GILGuard::acquire(cpy);
                // Build tuple from TLV args via pytypes
                let tuple = match pytypes::tuple_from_tlv(cpy, _args, _args_len) {
                    Ok(t) => t,
                    Err(_) => return NYB_E_INVALID_ARGS,
                };
                let ret = unsafe { (cpy.PyObject_CallObject)(func_ptr, tuple) };
                unsafe {
                    (cpy.Py_DecRef)(tuple);
                }
                if ret.is_null() {
                    let msg = pytypes::take_py_error_string(cpy);
                    if method_id == PYO_METHOD_CALL_R {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    if let Some(m) = msg {
                        return write_tlv_string(&m, result, result_len);
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
                if should_autodecode() {
                    if let Some(decoded) = pytypes::autodecode(cpy, ret) {
                        if write_autodecode_result(&decoded, result, result_len) {
                            unsafe {
                                (cpy.Py_DecRef)(ret);
                            }
                            return NYB_SUCCESS;
                        }
                    }
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                let owned = unsafe { PyOwned::from_new(ret).expect("non-null PyObject") };
                PY_HANDLES.lock().unwrap().insert(id, owned);
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_CALL_KW | PYO_METHOD_CALL_KW_R => {
            if ensure_cpython().is_err() {
                return NYB_E_PLUGIN_ERROR;
            }
            if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                let func_ptr = {
                    let guard = PY_HANDLES.lock().unwrap();
                    let Some(handle) = guard.get(&instance_id) else {
                        return NYB_E_INVALID_HANDLE;
                    };
                    handle.as_ptr()
                };
                let _gil = gil::GILGuard::acquire(cpy);
                // Empty args tuple for kwargs-only call
                let args_tup = unsafe { (cpy.PyTuple_New)(0) };
                if args_tup.is_null() {
                    return NYB_E_PLUGIN_ERROR;
                }
                // Build kwargs dict from TLV pairs via pytypes
                let kwargs = match pytypes::kwargs_from_tlv(cpy, _args, _args_len) {
                    Ok(d) => d,
                    Err(_) => {
                        unsafe {
                            (cpy.Py_DecRef)(args_tup);
                        }
                        return NYB_E_INVALID_ARGS;
                    }
                };
                let ret = unsafe { (cpy.PyObject_Call)(func_ptr, args_tup, kwargs) };
                unsafe {
                    (cpy.Py_DecRef)(kwargs);
                    (cpy.Py_DecRef)(args_tup);
                }
                if ret.is_null() {
                    let msg = pytypes::take_py_error_string(cpy);
                    if method_id == PYO_METHOD_CALL_KW_R {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    if let Some(m) = msg {
                        return write_tlv_string(&m, result, result_len);
                    }
                    return NYB_E_PLUGIN_ERROR;
                }
                if (method_id == PYO_METHOD_CALL_KW || method_id == PYO_METHOD_CALL_KW_R)
                    && should_autodecode()
                {
                    if let Some(decoded) = pytypes::autodecode(cpy, ret) {
                        if write_autodecode_result(&decoded, result, result_len) {
                            unsafe {
                                (cpy.Py_DecRef)(ret);
                            }
                            return NYB_SUCCESS;
                        }
                    }
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                let owned = unsafe { PyOwned::from_new(ret).expect("non-null PyObject") };
                PY_HANDLES.lock().unwrap().insert(id, owned);
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_STR => {
            if ensure_cpython().is_err() {
                return NYB_E_PLUGIN_ERROR;
            }
            if let Some(cpy) = &*ffi::CPY.lock().unwrap() {
                let obj_ptr = {
                    let guard = PY_HANDLES.lock().unwrap();
                    let Some(handle) = guard.get(&instance_id) else {
                        return NYB_E_INVALID_HANDLE;
                    };
                    handle.as_ptr()
                };
                let _gil = gil::GILGuard::acquire(cpy);
                let s_obj = unsafe { (cpy.PyObject_Str)(obj_ptr) };
                if s_obj.is_null() {
                    return NYB_E_PLUGIN_ERROR;
                }
                let rust_str = unsafe {
                    let cstr = (cpy.PyUnicode_AsUTF8)(s_obj);
                    match pytypes::cstr_to_string(cstr) {
                        Some(s) => s,
                        None => {
                            (cpy.Py_DecRef)(s_obj);
                            return NYB_E_PLUGIN_ERROR;
                        }
                    }
                };
                unsafe {
                    (cpy.Py_DecRef)(s_obj);
                }
                return write_tlv_string(&rust_str, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        // keep others unimplemented in 10.5b-min
        _ => NYB_E_INVALID_METHOD,
    }
}

// ===== Minimal TLV helpers (copy from other plugins for consistency) =====
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

extern "C" fn pyruntime_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let Some(s) = (unsafe { pytypes::cstr_to_string(name) }) else {
        return 0;
    };
    match s.as_str() {
        "birth" => PY_METHOD_BIRTH,
        "eval" | "evalR" => {
            if s == "evalR" {
                PY_METHOD_EVAL_R
            } else {
                PY_METHOD_EVAL
            }
        }
        "import" | "importR" => {
            if s == "importR" {
                PY_METHOD_IMPORT_R
            } else {
                PY_METHOD_IMPORT
            }
        }
        "fini" => PY_METHOD_FINI,
        _ => 0,
    }
}

extern "C" fn pyruntime_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    handle_py_runtime(method_id, instance_id, args, args_len, result, result_len)
}

extern "C" fn pyobject_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let Some(s) = (unsafe { pytypes::cstr_to_string(name) }) else {
        return 0;
    };
    match s.as_str() {
        "getattr" | "getAttr" | "getattrR" | "getAttrR" => {
            if s.ends_with('R') {
                PYO_METHOD_GETATTR_R
            } else {
                PYO_METHOD_GETATTR
            }
        }
        "call" | "callR" => {
            if s.ends_with('R') {
                PYO_METHOD_CALL_R
            } else {
                PYO_METHOD_CALL
            }
        }
        "callKw" | "callKW" | "call_kw" | "callKwR" | "callKWR" => {
            if s.to_lowercase().ends_with('r') {
                PYO_METHOD_CALL_KW_R
            } else {
                PYO_METHOD_CALL_KW
            }
        }
        "str" | "toString" => PYO_METHOD_STR,
        "birth" => PYO_METHOD_BIRTH,
        "fini" => PYO_METHOD_FINI,
        _ => 0,
    }
}

extern "C" fn pyobject_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    handle_py_object(method_id, instance_id, args, args_len, result, result_len)
}

#[no_mangle]
pub static nyash_typebox_PyRuntimeBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"PyRuntimeBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(pyruntime_resolve),
    invoke_id: Some(pyruntime_invoke_id),
    capabilities: 0,
};

#[no_mangle]
pub static nyash_typebox_PyObjectBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"PyObjectBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(pyobject_resolve),
    invoke_id: Some(pyobject_invoke_id),
    capabilities: 0,
};
fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() {
            return false;
        }
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return true;
        }
    }
    false
}

fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    let payload = s.as_bytes();
    write_tlv_result(&[(6u8, payload)], result, result_len)
}

fn write_tlv_handle(
    type_id: u32,
    instance_id: u32,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    let mut payload = [0u8; 8];
    payload[..4].copy_from_slice(&type_id.to_le_bytes());
    payload[4..].copy_from_slice(&instance_id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], result, result_len)
}

/// Read nth TLV argument as String (tag 6)
fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // skip header
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let _rsv = buf[off + 1];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag != 6 {
                return None;
            }
            let slice = &buf[off + 4..off + 4 + size];
            return std::str::from_utf8(slice).ok().map(|s| s.to_string());
        }
        off += 4 + size;
    }
    None
}

// Side-table for PyObject* storage (instance_id -> pointer)
static PY_HANDLES: Lazy<Mutex<HashMap<u32, PyOwned>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Base TLV writer used by helpers
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes()); // argc
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

fn should_autodecode() -> bool {
    std::env::var("NYASH_PY_AUTODECODE")
        .map(|v| v != "0")
        .unwrap_or(false)
}

fn autodecode_logging_enabled() -> bool {
    std::env::var("NYASH_PY_LOG")
        .map(|v| v != "0")
        .unwrap_or(false)
}

fn write_autodecode_result(
    decoded: &DecodedValue,
    result: *mut u8,
    result_len: *mut usize,
) -> bool {
    let rc = match decoded {
        DecodedValue::Float(value) => {
            if autodecode_logging_enabled() {
                eprintln!("[PyPlugin] autodecode: Float {}", value);
            }
            let payload = value.to_le_bytes();
            write_tlv_result(&[(5u8, payload.as_slice())], result, result_len)
        }
        DecodedValue::Int(value) => {
            if autodecode_logging_enabled() {
                eprintln!("[PyPlugin] autodecode: I64 {}", value);
            }
            let payload = value.to_le_bytes();
            write_tlv_result(&[(3u8, payload.as_slice())], result, result_len)
        }
        DecodedValue::Str(text) => {
            if autodecode_logging_enabled() {
                eprintln!(
                    "[PyPlugin] autodecode: String '{}', len={} ",
                    text,
                    text.len()
                );
            }
            write_tlv_result(&[(6u8, text.as_bytes())], result, result_len)
        }
        DecodedValue::Bytes(data) => {
            if autodecode_logging_enabled() {
                eprintln!("[PyPlugin] autodecode: Bytes {} bytes", data.len());
            }
            write_tlv_result(&[(7u8, data.as_slice())], result, result_len)
        }
    };
    rc == NYB_SUCCESS || rc == NYB_E_SHORT_BUFFER
}
