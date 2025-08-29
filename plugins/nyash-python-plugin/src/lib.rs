//! Nyash Python Plugin (Phase 10.5a scaffold)
//! - ABI v1 compatible entry points
//! - Defines two Box types: PyRuntimeBox (TYPE_ID=40) and PyObjectBox (TYPE_ID=41)
//! - Currently stubs; returns NYB_E_INVALID_METHOD for unimplemented routes
//!
//! This crate intentionally does not link to CPython yet. 10.5a focuses on
//! ABI alignment and loader wiring. Future subphases (10.5b–d) will implement
//! actual CPython embedding and conversion.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void, c_long};
use std::sync::{Mutex, atomic::{AtomicU32, Ordering}};
use libloading::Library;

// ===== Error Codes (aligned with other plugins) =====
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// ===== Type IDs (must match nyash.toml) =====
const TYPE_ID_PY_RUNTIME: u32 = 40;
const TYPE_ID_PY_OBJECT:  u32 = 41;

// ===== Method IDs (initial draft) =====
// PyRuntimeBox
const PY_METHOD_BIRTH: u32 = 0;        // returns instance_id (u32 LE, no TLV)
const PY_METHOD_EVAL:  u32 = 1;        // args: string code -> returns Handle(PyObject)
const PY_METHOD_IMPORT:u32 = 2;        // args: string name -> returns Handle(PyObject)
const PY_METHOD_FINI:  u32 = u32::MAX; // destructor

// PyObjectBox
const PYO_METHOD_BIRTH:   u32 = 0;     // reserved (should not be used directly)
const PYO_METHOD_GETATTR: u32 = 1;     // args: string name -> returns Handle(PyObject)
const PYO_METHOD_CALL:    u32 = 2;     // args: variadic TLV -> returns Handle(PyObject)
const PYO_METHOD_STR:     u32 = 3;     // returns String
const PYO_METHOD_CALL_KW: u32 = 5;     // args: key:string, val:TLV, ... -> returns Handle(PyObject)
const PYO_METHOD_FINI:    u32 = u32::MAX; // destructor

// ===== Minimal in-memory state for stubs =====
#[derive(Default)]
struct PyRuntimeInstance {}

#[derive(Default)]
struct PyObjectInstance {}

static RUNTIMES: Lazy<Mutex<HashMap<u32, PyRuntimeInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PYOBJS:   Lazy<Mutex<HashMap<u32, PyObjectInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static RT_COUNTER:  AtomicU32 = AtomicU32::new(1);
static OBJ_COUNTER: AtomicU32 = AtomicU32::new(1);

// ====== Minimal dynamic CPython loader ======
type PyObject = c_void;
type PyGILState_STATE = c_int;

struct CPython {
    _lib: Library,
    Py_Initialize: unsafe extern "C" fn(),
    Py_Finalize: unsafe extern "C" fn(),
    Py_IsInitialized: unsafe extern "C" fn() -> c_int,
    PyGILState_Ensure: unsafe extern "C" fn() -> PyGILState_STATE,
    PyGILState_Release: unsafe extern "C" fn(PyGILState_STATE),
    PyRun_StringFlags: unsafe extern "C" fn(*const c_char, c_int, *mut PyObject, *mut PyObject, *mut c_void) -> *mut PyObject,
    PyImport_AddModule: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    PyModule_GetDict: unsafe extern "C" fn(*mut PyObject) -> *mut PyObject,
    PyImport_ImportModule: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    PyObject_Str: unsafe extern "C" fn(*mut PyObject) -> *mut PyObject,
    PyUnicode_AsUTF8: unsafe extern "C" fn(*mut PyObject) -> *const c_char,
    Py_DecRef: unsafe extern "C" fn(*mut PyObject),
    Py_IncRef: unsafe extern "C" fn(*mut PyObject),
    // Added for getattr/call
    PyObject_GetAttrString: unsafe extern "C" fn(*mut PyObject, *const c_char) -> *mut PyObject,
    PyObject_CallObject: unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject,
    PyObject_Call: unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> *mut PyObject,
    PyTuple_New: unsafe extern "C" fn(isize) -> *mut PyObject,
    PyTuple_SetItem: unsafe extern "C" fn(*mut PyObject, isize, *mut PyObject) -> c_int,
    PyLong_FromLongLong: unsafe extern "C" fn(i64) -> *mut PyObject,
    PyUnicode_FromString: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    PyBool_FromLong: unsafe extern "C" fn(c_long: c_long) -> *mut PyObject,
    // Added for float/bytes and error handling
    PyFloat_FromDouble: unsafe extern "C" fn(f64) -> *mut PyObject,
    PyFloat_AsDouble: unsafe extern "C" fn(*mut PyObject) -> f64,
    PyLong_AsLongLong: unsafe extern "C" fn(*mut PyObject) -> i64,
    PyBytes_FromStringAndSize: unsafe extern "C" fn(*const c_char, isize) -> *mut PyObject,
    PyBytes_AsStringAndSize: unsafe extern "C" fn(*mut PyObject, *mut *mut c_char, *mut isize) -> c_int,
    PyDict_New: unsafe extern "C" fn() -> *mut PyObject,
    PyDict_SetItemString: unsafe extern "C" fn(*mut PyObject, *const c_char, *mut PyObject) -> c_int,
    PyErr_Occurred: unsafe extern "C" fn() -> *mut PyObject,
    PyErr_Fetch: unsafe extern "C" fn(*mut *mut PyObject, *mut *mut PyObject, *mut *mut PyObject),
    PyErr_Clear: unsafe extern "C" fn(),
}

static CPY: Lazy<Mutex<Option<CPython>>> = Lazy::new(|| Mutex::new(None));

fn try_load_cpython() -> Result<(), ()> {
    let candidates = [
        // Linux/WSL common
        "libpython3.12.so",
        "libpython3.12.so.1.0",
        "libpython3.11.so",
        "libpython3.11.so.1.0",
        "libpython3.10.so",
        "libpython3.10.so.1.0",
        "libpython3.9.so",
        "libpython3.9.so.1.0",
        // macOS
        "libpython3.12.dylib",
        "libpython3.11.dylib",
        "libpython3.10.dylib",
        "libpython3.9.dylib",
        // Windows (not targeted in 10.5b)
        // "python312.dll", "python311.dll",
    ];
    for name in candidates {
        if let Ok(lib) = unsafe { Library::new(name) } {
            unsafe {
                let Py_Initialize = *lib.get::<unsafe extern "C" fn()>(b"Py_Initialize\0").map_err(|_| ())?;
                let Py_Finalize = *lib.get::<unsafe extern "C" fn()>(b"Py_Finalize\0").map_err(|_| ())?;
                let Py_IsInitialized = *lib.get::<unsafe extern "C" fn() -> c_int>(b"Py_IsInitialized\0").map_err(|_| ())?;
                let PyGILState_Ensure = *lib.get::<unsafe extern "C" fn() -> PyGILState_STATE>(b"PyGILState_Ensure\0").map_err(|_| ())?;
                let PyGILState_Release = *lib.get::<unsafe extern "C" fn(PyGILState_STATE)>(b"PyGILState_Release\0").map_err(|_| ())?;
                let PyRun_StringFlags = *lib.get::<unsafe extern "C" fn(*const c_char, c_int, *mut PyObject, *mut PyObject, *mut c_void) -> *mut PyObject>(b"PyRun_StringFlags\0").map_err(|_| ())?;
                let PyImport_AddModule = *lib.get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(b"PyImport_AddModule\0").map_err(|_| ())?;
                let PyModule_GetDict = *lib.get::<unsafe extern "C" fn(*mut PyObject) -> *mut PyObject>(b"PyModule_GetDict\0").map_err(|_| ())?;
                let PyImport_ImportModule = *lib.get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(b"PyImport_ImportModule\0").map_err(|_| ())?;
                let PyObject_Str = *lib.get::<unsafe extern "C" fn(*mut PyObject) -> *mut PyObject>(b"PyObject_Str\0").map_err(|_| ())?;
                let PyUnicode_AsUTF8 = *lib.get::<unsafe extern "C" fn(*mut PyObject) -> *const c_char>(b"PyUnicode_AsUTF8\0").map_err(|_| ())?;
                let Py_DecRef = *lib.get::<unsafe extern "C" fn(*mut PyObject)>(b"Py_DecRef\0").map_err(|_| ())?;
                let Py_IncRef = *lib.get::<unsafe extern "C" fn(*mut PyObject)>(b"Py_IncRef\0").map_err(|_| ())?;
                let PyObject_GetAttrString = *lib.get::<unsafe extern "C" fn(*mut PyObject, *const c_char) -> *mut PyObject>(b"PyObject_GetAttrString\0").map_err(|_| ())?;
                let PyObject_CallObject = *lib.get::<unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject>(b"PyObject_CallObject\0").map_err(|_| ())?;
                let PyObject_Call = *lib.get::<unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> *mut PyObject>(b"PyObject_Call\0").map_err(|_| ())?;
                let PyTuple_New = *lib.get::<unsafe extern "C" fn(isize) -> *mut PyObject>(b"PyTuple_New\0").map_err(|_| ())?;
                let PyTuple_SetItem = *lib.get::<unsafe extern "C" fn(*mut PyObject, isize, *mut PyObject) -> c_int>(b"PyTuple_SetItem\0").map_err(|_| ())?;
                let PyLong_FromLongLong = *lib.get::<unsafe extern "C" fn(i64) -> *mut PyObject>(b"PyLong_FromLongLong\0").map_err(|_| ())?;
                let PyUnicode_FromString = *lib.get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(b"PyUnicode_FromString\0").map_err(|_| ())?;
                let PyBool_FromLong = *lib.get::<unsafe extern "C" fn(c_long: c_long) -> *mut PyObject>(b"PyBool_FromLong\0").map_err(|_| ())?;
                let PyFloat_FromDouble = *lib.get::<unsafe extern "C" fn(f64) -> *mut PyObject>(b"PyFloat_FromDouble\0").map_err(|_| ())?;
                let PyFloat_AsDouble = *lib.get::<unsafe extern "C" fn(*mut PyObject) -> f64>(b"PyFloat_AsDouble\0").map_err(|_| ())?;
                let PyLong_AsLongLong = *lib.get::<unsafe extern "C" fn(*mut PyObject) -> i64>(b"PyLong_AsLongLong\0").map_err(|_| ())?;
                let PyBytes_FromStringAndSize = *lib.get::<unsafe extern "C" fn(*const c_char, isize) -> *mut PyObject>(b"PyBytes_FromStringAndSize\0").map_err(|_| ())?;
                let PyBytes_AsStringAndSize = *lib.get::<unsafe extern "C" fn(*mut PyObject, *mut *mut c_char, *mut isize) -> c_int>(b"PyBytes_AsStringAndSize\0").map_err(|_| ())?;
                let PyErr_Occurred = *lib.get::<unsafe extern "C" fn() -> *mut PyObject>(b"PyErr_Occurred\0").map_err(|_| ())?;
                let PyDict_New = *lib.get::<unsafe extern "C" fn() -> *mut PyObject>(b"PyDict_New\0").map_err(|_| ())?;
                let PyDict_SetItemString = *lib.get::<unsafe extern "C" fn(*mut PyObject, *const c_char, *mut PyObject) -> c_int>(b"PyDict_SetItemString\0").map_err(|_| ())?;
                let PyErr_Fetch = *lib.get::<unsafe extern "C" fn(*mut *mut PyObject, *mut *mut PyObject, *mut *mut PyObject)>(b"PyErr_Fetch\0").map_err(|_| ())?;
                let PyErr_Clear = *lib.get::<unsafe extern "C" fn()>(b"PyErr_Clear\0").map_err(|_| ())?;

                let cpy = CPython {
                    _lib: lib,
                    Py_Initialize,
                    Py_Finalize,
                    Py_IsInitialized,
                    PyGILState_Ensure,
                    PyGILState_Release,
                    PyRun_StringFlags,
                    PyImport_AddModule,
                    PyModule_GetDict,
                    PyImport_ImportModule,
                    PyObject_Str,
                    PyUnicode_AsUTF8,
                    Py_DecRef,
                    Py_IncRef,
                    PyObject_GetAttrString,
                    PyObject_CallObject,
                    PyObject_Call,
                    PyTuple_New,
                    PyTuple_SetItem,
                    PyLong_FromLongLong,
                    PyUnicode_FromString,
                    PyBool_FromLong,
                    PyFloat_FromDouble,
                    PyFloat_AsDouble,
                    PyLong_AsLongLong,
                    PyBytes_FromStringAndSize,
                    PyBytes_AsStringAndSize,
                    PyErr_Occurred,
                    PyDict_New,
                    PyDict_SetItemString,
                    PyErr_Fetch,
                    PyErr_Clear,
                };
                *CPY.lock().unwrap() = Some(cpy);
                return Ok(());
            }
        }
    }
    Err(())
}

fn ensure_cpython() -> Result<(), ()> {
    if CPY.lock().unwrap().is_none() {
        try_load_cpython()?;
        // Initialize on first load
        unsafe {
            if let Some(cpy) = &*CPY.lock().unwrap() {
                if (cpy.Py_IsInitialized)() == 0 { (cpy.Py_Initialize)(); }
            }
        }
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 { 1 }

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 { NYB_SUCCESS }

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
        TYPE_ID_PY_RUNTIME => handle_py_runtime(method_id, instance_id, args, args_len, result, result_len),
        TYPE_ID_PY_OBJECT  => handle_py_object(method_id, instance_id, args, args_len, result, result_len),
        _ => NYB_E_INVALID_TYPE,
    }
}

fn handle_py_runtime(method_id: u32, _instance_id: u32, _args: *const u8, _args_len: usize, result: *mut u8, result_len: *mut usize) -> i32 {
    unsafe {
        match method_id {
            PY_METHOD_BIRTH => {
                if result_len.is_null() { return NYB_E_INVALID_ARGS; }
                if preflight(result, result_len, 4) { return NYB_E_SHORT_BUFFER; }
                if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
                let id = RT_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = RUNTIMES.lock() { map.insert(id, PyRuntimeInstance::default()); } else { return NYB_E_PLUGIN_ERROR; }
                let bytes = id.to_le_bytes();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            PY_METHOD_FINI => {
                // Only drop runtime slot; avoid calling Py_Finalize to prevent shutdown crashes.
                if let Ok(mut map) = RUNTIMES.lock() { map.remove(&_instance_id); }
                NYB_SUCCESS
            }
            PY_METHOD_EVAL => {
                if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
                let Some(code) = read_arg_string(_args, _args_len, 0) else { return NYB_E_INVALID_ARGS; };
                let c_code = match CString::new(code) { Ok(s) => s, Err(_) => return NYB_E_INVALID_ARGS };
                let c_main = CString::new("__main__").unwrap();
                if let Some(cpy) = &*CPY.lock().unwrap() {
                    let state = (cpy.PyGILState_Ensure)();
                    let globals = (cpy.PyImport_AddModule)(c_main.as_ptr());
                    if globals.is_null() { (cpy.PyGILState_Release)(state); return NYB_E_PLUGIN_ERROR; }
                    let dict = (cpy.PyModule_GetDict)(globals);
                    // 258 == Py_eval_input
                    let obj = (cpy.PyRun_StringFlags)(c_code.as_ptr(), 258, dict, dict, std::ptr::null_mut());
                    if obj.is_null() {
                        let msg = take_py_error_string(cpy);
                        (cpy.PyGILState_Release)(state);
                        if let Some(m) = msg { return write_tlv_string(&m, result, result_len); }
                        return NYB_E_PLUGIN_ERROR;
                    }
                    // Store as PyObjectBox handle
                    let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut map) = PYOBJS.lock() {
                        map.insert(id, PyObjectInstance::default());
                    } else {
                        (cpy.Py_DecRef)(obj);
                        (cpy.PyGILState_Release)(state);
                        return NYB_E_PLUGIN_ERROR;
                    }
                    // Keep reference (obj is new ref). We model store via separate map and hold pointer via raw address table.
                    // To actually manage pointer per id, we extend PyObjectInstance in 10.5b with a field. For now, attach through side-table.
                    OBJ_PTRS.lock().unwrap().insert(id, PyPtr(obj));
                    (cpy.PyGILState_Release)(state);
                    return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
                }
                NYB_E_PLUGIN_ERROR
            }
            PY_METHOD_IMPORT => {
                if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
                let Some(name) = read_arg_string(_args, _args_len, 0) else { return NYB_E_INVALID_ARGS; };
                let c_name = match CString::new(name) { Ok(s) => s, Err(_) => return NYB_E_INVALID_ARGS };
                if let Some(cpy) = &*CPY.lock().unwrap() {
                    let state = (cpy.PyGILState_Ensure)();
                    let obj = (cpy.PyImport_ImportModule)(c_name.as_ptr());
                    if obj.is_null() {
                        let msg = take_py_error_string(cpy);
                        (cpy.PyGILState_Release)(state);
                        if let Some(m) = msg { return write_tlv_string(&m, result, result_len); }
                        return NYB_E_PLUGIN_ERROR;
                    }
                    let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if let Ok(mut map) = PYOBJS.lock() { map.insert(id, PyObjectInstance::default()); } else { (cpy.Py_DecRef)(obj); (cpy.PyGILState_Release)(state); return NYB_E_PLUGIN_ERROR; }
                    OBJ_PTRS.lock().unwrap().insert(id, PyPtr(obj));
                    (cpy.PyGILState_Release)(state);
                    return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
                }
                NYB_E_PLUGIN_ERROR
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}

fn handle_py_object(method_id: u32, instance_id: u32, _args: *const u8, _args_len: usize, result: *mut u8, result_len: *mut usize) -> i32 {
    match method_id {
        PYO_METHOD_BIRTH => NYB_E_INVALID_METHOD, // should be created via runtime
        PYO_METHOD_FINI => {
            if let Some(cpy) = &*CPY.lock().unwrap() {
                if let Some(ptr) = OBJ_PTRS.lock().unwrap().remove(&instance_id) {
                    unsafe { (cpy.Py_DecRef)(ptr.0); }
                }
            }
            if let Ok(mut map) = PYOBJS.lock() { map.remove(&instance_id); NYB_SUCCESS } else { NYB_E_PLUGIN_ERROR }
        }
        PYO_METHOD_GETATTR => {
            if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
            let Some(name) = read_arg_string(_args, _args_len, 0) else { return NYB_E_INVALID_ARGS; };
            if let Some(cpy) = &*CPY.lock().unwrap() {
                let Some(obj) = OBJ_PTRS.lock().unwrap().get(&instance_id).copied() else { return NYB_E_INVALID_HANDLE; };
                let c_name = match CString::new(name) { Ok(s) => s, Err(_) => return NYB_E_INVALID_ARGS };
                let state = unsafe { (cpy.PyGILState_Ensure)() };
                let attr = unsafe { (cpy.PyObject_GetAttrString)(obj.0, c_name.as_ptr()) };
                if attr.is_null() {
                    let msg = take_py_error_string(cpy);
                    unsafe { (cpy.PyGILState_Release)(state); }
                    if let Some(m) = msg { return write_tlv_string(&m, result, result_len); }
                    return NYB_E_PLUGIN_ERROR;
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                OBJ_PTRS.lock().unwrap().insert(id, PyPtr(attr));
                unsafe { (cpy.PyGILState_Release)(state); }
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_CALL => {
            if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
            if let Some(cpy) = &*CPY.lock().unwrap() {
                let Some(func) = OBJ_PTRS.lock().unwrap().get(&instance_id).copied() else { return NYB_E_INVALID_HANDLE; };
                let state = unsafe { (cpy.PyGILState_Ensure)() };
                // Build tuple from TLV args
                let argc = tlv_count_args(_args, _args_len);
                let tuple = unsafe { (cpy.PyTuple_New)(argc as isize) };
                if tuple.is_null() { unsafe { (cpy.PyGILState_Release)(state); } return NYB_E_PLUGIN_ERROR; }
                if !fill_tuple_from_tlv(cpy, tuple, _args, _args_len) {
                    unsafe { (cpy.Py_DecRef)(tuple); (cpy.PyGILState_Release)(state); }
                    return NYB_E_INVALID_ARGS;
                }
                let ret = unsafe { (cpy.PyObject_CallObject)(func.0, tuple) };
                unsafe { (cpy.Py_DecRef)(tuple); }
                if ret.is_null() {
                    let msg = take_py_error_string(cpy);
                    unsafe { (cpy.PyGILState_Release)(state); }
                    if let Some(m) = msg { return write_tlv_string(&m, result, result_len); }
                    return NYB_E_PLUGIN_ERROR;
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                OBJ_PTRS.lock().unwrap().insert(id, PyPtr(ret));
                unsafe { (cpy.PyGILState_Release)(state); }
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_CALL_KW => {
            if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
            if let Some(cpy) = &*CPY.lock().unwrap() {
                let Some(func) = OBJ_PTRS.lock().unwrap().get(&instance_id).copied() else { return NYB_E_INVALID_HANDLE; };
                let state = unsafe { (cpy.PyGILState_Ensure)() };
                // Empty args tuple for kwargs-only call
                let args_tup = unsafe { (cpy.PyTuple_New)(0) };
                if args_tup.is_null() { unsafe { (cpy.PyGILState_Release)(state); } return NYB_E_PLUGIN_ERROR; }
                // Build kwargs dict from TLV pairs
                let kwargs = unsafe { (cpy.PyDict_New)() };
                if kwargs.is_null() { unsafe { (cpy.Py_DecRef)(args_tup); (cpy.PyGILState_Release)(state); } return NYB_E_PLUGIN_ERROR; }
                if !fill_kwargs_from_tlv(cpy, kwargs, _args, _args_len) {
                    unsafe { (cpy.Py_DecRef)(kwargs); (cpy.Py_DecRef)(args_tup); (cpy.PyGILState_Release)(state); }
                    return NYB_E_INVALID_ARGS;
                }
                let ret = unsafe { (cpy.PyObject_Call)(func.0, args_tup, kwargs) };
                unsafe { (cpy.Py_DecRef)(kwargs); (cpy.Py_DecRef)(args_tup); }
                if ret.is_null() {
                    let msg = take_py_error_string(cpy);
                    unsafe { (cpy.PyGILState_Release)(state); }
                    if let Some(m) = msg { return write_tlv_string(&m, result, result_len); }
                    return NYB_E_PLUGIN_ERROR;
                }
                let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
                OBJ_PTRS.lock().unwrap().insert(id, PyPtr(ret));
                unsafe { (cpy.PyGILState_Release)(state); }
                return write_tlv_handle(TYPE_ID_PY_OBJECT, id, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        PYO_METHOD_STR => {
            if ensure_cpython().is_err() { return NYB_E_PLUGIN_ERROR; }
            if let Some(cpy) = &*CPY.lock().unwrap() {
                let Some(obj) = OBJ_PTRS.lock().unwrap().get(&instance_id).copied() else { return NYB_E_INVALID_HANDLE; };
                let state = unsafe { (cpy.PyGILState_Ensure)() };
                let s_obj = unsafe { (cpy.PyObject_Str)(obj.0) };
                if s_obj.is_null() { unsafe { (cpy.PyGILState_Release)(state); } return NYB_E_PLUGIN_ERROR; }
                let cstr = unsafe { (cpy.PyUnicode_AsUTF8)(s_obj) };
                if cstr.is_null() { unsafe { (cpy.Py_DecRef)(s_obj); (cpy.PyGILState_Release)(state); } return NYB_E_PLUGIN_ERROR; }
                let rust_str = unsafe { CStr::from_ptr(cstr) }.to_string_lossy().to_string();
                unsafe { (cpy.Py_DecRef)(s_obj); (cpy.PyGILState_Release)(state); }
                return write_tlv_string(&rust_str, result, result_len);
            }
            NYB_E_PLUGIN_ERROR
        }
        // keep others unimplemented in 10.5b-min
        PYO_METHOD_GETATTR | PYO_METHOD_CALL => NYB_E_INVALID_METHOD,
        _ => NYB_E_INVALID_METHOD,
    }
}

// ===== Minimal TLV helpers (copy from other plugins for consistency) =====
fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() { return false; }
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

fn write_tlv_handle(type_id: u32, instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    let mut payload = [0u8; 8];
    payload[..4].copy_from_slice(&type_id.to_le_bytes());
    payload[4..].copy_from_slice(&instance_id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], result, result_len)
}

/// Read nth TLV argument as String (tag 6)
fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // skip header
    for i in 0..=n {
        if buf.len() < off + 4 { return None; }
        let tag = buf[off];
        let _rsv = buf[off+1];
        let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if buf.len() < off + 4 + size { return None; }
        if i == n {
            if tag != 6 { return None; }
            let slice = &buf[off+4 .. off+4+size];
            return std::str::from_utf8(slice).ok().map(|s| s.to_string());
        }
        off += 4 + size;
    }
    None
}

// Side-table for PyObject* storage (instance_id -> pointer)
#[derive(Copy, Clone)]
struct PyPtr(*mut PyObject);
unsafe impl Send for PyPtr {}
unsafe impl Sync for PyPtr {}
static OBJ_PTRS: Lazy<Mutex<HashMap<u32, PyPtr>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Base TLV writer used by helpers
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() { return NYB_E_INVALID_ARGS; }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + payloads.iter().map(|(_,p)| 4 + p.len()).sum::<usize>());
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

fn tlv_count_args(args: *const u8, args_len: usize) -> usize {
    if args.is_null() || args_len < 4 { return 0; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    if buf.len() < 4 { return 0; }
    let argc = u16::from_le_bytes([buf[2], buf[3]]) as usize;
    argc
}

fn fill_tuple_from_tlv(cpy: &CPython, tuple: *mut PyObject, args: *const u8, args_len: usize) -> bool {
    if args.is_null() || args_len < 4 { return true; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    let mut idx: isize = 0;
    while off + 4 <= buf.len() {
        let tag = buf[off];
        let _rsv = buf[off+1];
        let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if off + 4 + size > buf.len() { return false; }
        let payload = &buf[off+4 .. off+4+size];
        let mut obj: *mut PyObject = std::ptr::null_mut();
        unsafe {
            match tag {
                1 => { // Bool (1 byte)
                    let v = if size >= 1 && payload[0] != 0 { 1 } else { 0 };
                    obj = (cpy.PyBool_FromLong)(v);
                }
                2 => { // I32
                    if size != 4 { return false; }
                    let mut b = [0u8;4]; b.copy_from_slice(payload);
                    let v = i32::from_le_bytes(b) as i64;
                    obj = (cpy.PyLong_FromLongLong)(v);
                }
                3 => { // I64
                    if size != 8 { return false; }
                    let mut b = [0u8;8]; b.copy_from_slice(payload);
                    let v = i64::from_le_bytes(b);
                    obj = (cpy.PyLong_FromLongLong)(v);
                }
                5 => { // F64
                    if size != 8 { return false; }
                    let mut b = [0u8;8]; b.copy_from_slice(payload);
                    let v = f64::from_le_bytes(b);
                    obj = (cpy.PyFloat_FromDouble)(v);
                }
                6 => { // String
                    let c = match CString::new(payload) { Ok(c) => c, Err(_) => return false };
                    obj = (cpy.PyUnicode_FromString)(c.as_ptr());
                }
                7 => { // Bytes
                    if size > 0 {
                        let ptr = payload.as_ptr() as *const c_char;
                        obj = (cpy.PyBytes_FromStringAndSize)(ptr, size as isize);
                    } else {
                        obj = (cpy.PyBytes_FromStringAndSize)(std::ptr::null(), 0);
                    }
                }
                8 => { // Handle
                    if size != 8 { return false; }
                    let mut t = [0u8;4]; t.copy_from_slice(&payload[0..4]);
                    let mut i = [0u8;4]; i.copy_from_slice(&payload[4..8]);
                    let _type_id = u32::from_le_bytes(t);
                    let inst_id = u32::from_le_bytes(i);
                    if let Some(ptr) = OBJ_PTRS.lock().unwrap().get(&inst_id).copied() {
                        // PyTuple_SetItem steals a reference; INCREF to keep original alive
                        (cpy.Py_IncRef)(ptr.0);
                        obj = ptr.0;
                    } else { return false; }
                }
                _ => { return false; }
            }
            if obj.is_null() { return false; }
            let rc = (cpy.PyTuple_SetItem)(tuple, idx, obj);
            if rc != 0 { return false; }
            idx += 1;
        }
        off += 4 + size;
    }
    true
}

fn take_py_error_string(cpy: &CPython) -> Option<String> {
    unsafe {
        if (cpy.PyErr_Occurred)().is_null() { return None; }
        let mut ptype: *mut PyObject = std::ptr::null_mut();
        let mut pvalue: *mut PyObject = std::ptr::null_mut();
        let mut ptrace: *mut PyObject = std::ptr::null_mut();
        (cpy.PyErr_Fetch)(&mut ptype, &mut pvalue, &mut ptrace);
        let s = if !pvalue.is_null() {
            let sobj = (cpy.PyObject_Str)(pvalue);
            if sobj.is_null() { (cpy.PyErr_Clear)(); return Some("Python error".to_string()); }
            let cstr = (cpy.PyUnicode_AsUTF8)(sobj);
            let msg = if cstr.is_null() { "Python error".to_string() } else { CStr::from_ptr(cstr).to_string_lossy().to_string() };
            (cpy.Py_DecRef)(sobj);
            msg
        } else {
            "Python error".to_string()
        };
        (cpy.PyErr_Clear)();
        Some(s)
    }
}

fn fill_kwargs_from_tlv(cpy: &CPython, dict: *mut PyObject, args: *const u8, args_len: usize) -> bool {
    if args.is_null() || args_len < 4 { return true; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    loop {
        if off + 4 > buf.len() { break; }
        let tag_k = buf[off];
        let _rsv = buf[off+1];
        let size_k = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if tag_k != 6 || off + 4 + size_k > buf.len() { return false; }
        let key_bytes = &buf[off+4 .. off+4+size_k];
        off += 4 + size_k;
        if off + 4 > buf.len() { return false; }
        let tag_v = buf[off];
        let _rsv2 = buf[off+1];
        let size_v = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if off + 4 + size_v > buf.len() { return false; }
        let val_payload = &buf[off+4 .. off+4+size_v];
        off += 4 + size_v;
        unsafe {
            let key_c = match CString::new(key_bytes) { Ok(c) => c, Err(_) => return false };
            let mut obj: *mut PyObject = std::ptr::null_mut();
            match tag_v {
                1 => { let v = if size_v >= 1 && val_payload[0] != 0 { 1 } else { 0 }; obj = (cpy.PyBool_FromLong)(v); }
                2 => { if size_v != 4 { return false; } let mut b = [0u8;4]; b.copy_from_slice(val_payload); obj = (cpy.PyLong_FromLongLong)(i32::from_le_bytes(b) as i64); }
                3 => { if size_v != 8 { return false; } let mut b=[0u8;8]; b.copy_from_slice(val_payload); obj = (cpy.PyLong_FromLongLong)(i64::from_le_bytes(b)); }
                5 => { if size_v != 8 { return false; } let mut b=[0u8;8]; b.copy_from_slice(val_payload); obj = (cpy.PyFloat_FromDouble)(f64::from_le_bytes(b)); }
                6 => { let c = match CString::new(val_payload) { Ok(c) => c, Err(_) => return false }; obj = (cpy.PyUnicode_FromString)(c.as_ptr()); }
                7 => { let ptr = if size_v>0 { val_payload.as_ptr() as *const c_char } else { std::ptr::null() }; obj = (cpy.PyBytes_FromStringAndSize)(ptr, size_v as isize); }
                8 => {
                    if size_v != 8 { return false; }
                    let mut i = [0u8;4]; i.copy_from_slice(&val_payload[4..8]);
                    let inst_id = u32::from_le_bytes(i);
                    if let Some(p) = OBJ_PTRS.lock().unwrap().get(&inst_id).copied() { (cpy.Py_IncRef)(p.0); obj = p.0; } else { return false; }
                }
                _ => return false,
            }
            if obj.is_null() { return false; }
            let rc = (cpy.PyDict_SetItemString)(dict, key_c.as_ptr(), obj);
            (cpy.Py_DecRef)(obj);
            if rc != 0 { return false; }
        }
    }
    true
}
fn try_write_auto_numeric(cpy: &CPython, obj: *mut PyObject, result: *mut u8, result_len: *mut usize) -> bool {
    unsafe {
        // Try float
        let mut had_err = false;
        let f = (cpy.PyFloat_AsDouble)(obj);
        if !(cpy.PyErr_Occurred)().is_null() {
            had_err = true; (cpy.PyErr_Clear)();
        }
        if !had_err {
            // Consider NaN as successful too
            let mut payload = [0u8;8];
            payload.copy_from_slice(&f.to_le_bytes());
            let rc = write_tlv_result(&[(5u8, &payload)], result, result_len);
            return rc == NYB_SUCCESS || rc == NYB_E_SHORT_BUFFER;
        }
        // Try integer
        let i = (cpy.PyLong_AsLongLong)(obj);
        if !(cpy.PyErr_Occurred)().is_null() { (cpy.PyErr_Clear)(); } else {
            let mut payload = [0u8;8];
            payload.copy_from_slice(&i.to_le_bytes());
            let rc = write_tlv_result(&[(3u8, &payload)], result, result_len);
            return rc == NYB_SUCCESS || rc == NYB_E_SHORT_BUFFER;
        }
        false
    }
}
