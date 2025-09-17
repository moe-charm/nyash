#![allow(non_snake_case, non_camel_case_types, dead_code)]
use libloading::Library;
use once_cell::sync::Lazy;
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::sync::Mutex;

pub type PyObject = c_void;
pub type PyGILState_STATE = c_int;

pub struct CPython {
    pub(crate) _lib: Library,
    pub(crate) Py_Initialize: unsafe extern "C" fn(),
    pub(crate) Py_Finalize: unsafe extern "C" fn(),
    pub(crate) Py_IsInitialized: unsafe extern "C" fn() -> c_int,
    pub(crate) PyGILState_Ensure: unsafe extern "C" fn() -> PyGILState_STATE,
    pub(crate) PyGILState_Release: unsafe extern "C" fn(PyGILState_STATE),
    pub(crate) PyRun_StringFlags: unsafe extern "C" fn(
        *const c_char,
        c_int,
        *mut PyObject,
        *mut PyObject,
        *mut c_void,
    ) -> *mut PyObject,
    pub(crate) PyImport_AddModule: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    pub(crate) PyModule_GetDict: unsafe extern "C" fn(*mut PyObject) -> *mut PyObject,
    pub(crate) PyImport_ImportModule: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    pub(crate) PyObject_Str: unsafe extern "C" fn(*mut PyObject) -> *mut PyObject,
    pub(crate) PyUnicode_AsUTF8: unsafe extern "C" fn(*mut PyObject) -> *const c_char,
    pub(crate) Py_DecRef: unsafe extern "C" fn(*mut PyObject),
    pub(crate) Py_IncRef: unsafe extern "C" fn(*mut PyObject),
    pub(crate) PyObject_GetAttrString:
        unsafe extern "C" fn(*mut PyObject, *const c_char) -> *mut PyObject,
    pub(crate) PyObject_CallObject:
        unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject,
    pub(crate) PyObject_Call:
        unsafe extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject) -> *mut PyObject,
    pub(crate) PyTuple_New: unsafe extern "C" fn(isize) -> *mut PyObject,
    pub(crate) PyTuple_SetItem: unsafe extern "C" fn(*mut PyObject, isize, *mut PyObject) -> c_int,
    pub(crate) PyLong_FromLongLong: unsafe extern "C" fn(i64) -> *mut PyObject,
    pub(crate) PyUnicode_FromString: unsafe extern "C" fn(*const c_char) -> *mut PyObject,
    pub(crate) PyBool_FromLong: unsafe extern "C" fn(c_long: c_long) -> *mut PyObject,
    pub(crate) PyFloat_FromDouble: unsafe extern "C" fn(f64) -> *mut PyObject,
    pub(crate) PyFloat_AsDouble: unsafe extern "C" fn(*mut PyObject) -> f64,
    pub(crate) PyLong_AsLongLong: unsafe extern "C" fn(*mut PyObject) -> i64,
    pub(crate) PyBytes_FromStringAndSize:
        unsafe extern "C" fn(*const c_char, isize) -> *mut PyObject,
    pub(crate) PyBytes_AsStringAndSize:
        unsafe extern "C" fn(*mut PyObject, *mut *mut c_char, *mut isize) -> c_int,
    pub(crate) PyDict_New: unsafe extern "C" fn() -> *mut PyObject,
    pub(crate) PyDict_SetItemString:
        unsafe extern "C" fn(*mut PyObject, *const c_char, *mut PyObject) -> c_int,
    pub(crate) PyErr_Occurred: unsafe extern "C" fn() -> *mut PyObject,
    pub(crate) PyErr_Fetch:
        unsafe extern "C" fn(*mut *mut PyObject, *mut *mut PyObject, *mut *mut PyObject),
    pub(crate) PyErr_Clear: unsafe extern "C" fn(),
}

pub static CPY: Lazy<Mutex<Option<CPython>>> = Lazy::new(|| Mutex::new(None));

pub fn try_load_cpython() -> Result<(), ()> {
    let mut candidates: Vec<String> = vec![
        // Linux
        "libpython3.12.so".into(),
        "libpython3.12.so.1.0".into(),
        "libpython3.11.so".into(),
        "libpython3.11.so.1.0".into(),
        "libpython3.10.so".into(),
        "libpython3.10.so.1.0".into(),
        "libpython3.9.so".into(),
        "libpython3.9.so.1.0".into(),
        // macOS
        "libpython3.12.dylib".into(),
        "libpython3.11.dylib".into(),
        "libpython3.10.dylib".into(),
        "libpython3.9.dylib".into(),
    ];
    if cfg!(target_os = "windows") {
        let dlls = [
            "python312.dll",
            "python311.dll",
            "python310.dll",
            "python39.dll",
        ];
        for d in dlls {
            candidates.push(d.into());
        }
        if let Ok(pyhome) = std::env::var("PYTHONHOME") {
            for d in [
                "python312.dll",
                "python311.dll",
                "python310.dll",
                "python39.dll",
            ]
            .iter()
            {
                let p = std::path::Path::new(&pyhome).join(d);
                if p.exists() {
                    candidates.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    for name in candidates.into_iter() {
        if let Ok(lib) = unsafe { Library::new(&name) } {
            unsafe {
                let Py_Initialize = *lib
                    .get::<unsafe extern "C" fn()>(b"Py_Initialize\0")
                    .map_err(|_| ())?;
                let Py_Finalize = *lib
                    .get::<unsafe extern "C" fn()>(b"Py_Finalize\0")
                    .map_err(|_| ())?;
                let Py_IsInitialized = *lib
                    .get::<unsafe extern "C" fn() -> c_int>(b"Py_IsInitialized\0")
                    .map_err(|_| ())?;
                let PyGILState_Ensure = *lib
                    .get::<unsafe extern "C" fn() -> PyGILState_STATE>(b"PyGILState_Ensure\0")
                    .map_err(|_| ())?;
                let PyGILState_Release = *lib
                    .get::<unsafe extern "C" fn(PyGILState_STATE)>(b"PyGILState_Release\0")
                    .map_err(|_| ())?;
                let PyRun_StringFlags = *lib
                    .get::<unsafe extern "C" fn(
                        *const c_char,
                        c_int,
                        *mut PyObject,
                        *mut PyObject,
                        *mut c_void,
                    ) -> *mut PyObject>(b"PyRun_StringFlags\0")
                    .map_err(|_| ())?;
                let PyImport_AddModule = *lib
                    .get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(
                        b"PyImport_AddModule\0",
                    )
                    .map_err(|_| ())?;
                let PyModule_GetDict = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject) -> *mut PyObject>(
                        b"PyModule_GetDict\0",
                    )
                    .map_err(|_| ())?;
                let PyImport_ImportModule = *lib
                    .get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(
                        b"PyImport_ImportModule\0",
                    )
                    .map_err(|_| ())?;
                let PyObject_Str = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject) -> *mut PyObject>(b"PyObject_Str\0")
                    .map_err(|_| ())?;
                let PyUnicode_AsUTF8 = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject) -> *const c_char>(
                        b"PyUnicode_AsUTF8\0",
                    )
                    .map_err(|_| ())?;
                let Py_DecRef = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject)>(b"Py_DecRef\0")
                    .map_err(|_| ())?;
                let Py_IncRef = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject)>(b"Py_IncRef\0")
                    .map_err(|_| ())?;
                let PyObject_GetAttrString = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject, *const c_char) -> *mut PyObject>(
                        b"PyObject_GetAttrString\0",
                    )
                    .map_err(|_| ())?;
                let PyObject_CallObject = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject, *mut PyObject) -> *mut PyObject>(
                        b"PyObject_CallObject\0",
                    )
                    .map_err(|_| ())?;
                let PyObject_Call = *lib
                    .get::<unsafe extern "C" fn(
                        *mut PyObject,
                        *mut PyObject,
                        *mut PyObject,
                    ) -> *mut PyObject>(b"PyObject_Call\0")
                    .map_err(|_| ())?;
                let PyTuple_New = *lib
                    .get::<unsafe extern "C" fn(isize) -> *mut PyObject>(b"PyTuple_New\0")
                    .map_err(|_| ())?;
                let PyTuple_SetItem = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject, isize, *mut PyObject) -> c_int>(
                        b"PyTuple_SetItem\0",
                    )
                    .map_err(|_| ())?;
                let PyLong_FromLongLong = *lib
                    .get::<unsafe extern "C" fn(i64) -> *mut PyObject>(b"PyLong_FromLongLong\0")
                    .map_err(|_| ())?;
                let PyUnicode_FromString = *lib
                    .get::<unsafe extern "C" fn(*const c_char) -> *mut PyObject>(
                        b"PyUnicode_FromString\0",
                    )
                    .map_err(|_| ())?;
                let PyBool_FromLong = *lib
                    .get::<unsafe extern "C" fn(c_long: c_long) -> *mut PyObject>(
                        b"PyBool_FromLong\0",
                    )
                    .map_err(|_| ())?;
                let PyFloat_FromDouble = *lib
                    .get::<unsafe extern "C" fn(f64) -> *mut PyObject>(b"PyFloat_FromDouble\0")
                    .map_err(|_| ())?;
                let PyFloat_AsDouble = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject) -> f64>(b"PyFloat_AsDouble\0")
                    .map_err(|_| ())?;
                let PyLong_AsLongLong = *lib
                    .get::<unsafe extern "C" fn(*mut PyObject) -> i64>(b"PyLong_AsLongLong\0")
                    .map_err(|_| ())?;
                let PyBytes_FromStringAndSize = *lib
                    .get::<unsafe extern "C" fn(*const c_char, isize) -> *mut PyObject>(
                        b"PyBytes_FromStringAndSize\0",
                    )
                    .map_err(|_| ())?;
                let PyBytes_AsStringAndSize = *lib.get::<unsafe extern "C" fn(*mut PyObject, *mut *mut c_char, *mut isize) -> c_int>(b"PyBytes_AsStringAndSize\0").map_err(|_| ())?;
                let PyDict_New = *lib
                    .get::<unsafe extern "C" fn() -> *mut PyObject>(b"PyDict_New\0")
                    .map_err(|_| ())?;
                let PyDict_SetItemString = *lib.get::<unsafe extern "C" fn(*mut PyObject, *const c_char, *mut PyObject) -> c_int>(b"PyDict_SetItemString\0").map_err(|_| ())?;
                let PyErr_Occurred = *lib
                    .get::<unsafe extern "C" fn() -> *mut PyObject>(b"PyErr_Occurred\0")
                    .map_err(|_| ())?;
                let PyErr_Fetch =
                    *lib.get::<unsafe extern "C" fn(
                        *mut *mut PyObject,
                        *mut *mut PyObject,
                        *mut *mut PyObject,
                    )>(b"PyErr_Fetch\0")
                        .map_err(|_| ())?;
                let PyErr_Clear = *lib
                    .get::<unsafe extern "C" fn()>(b"PyErr_Clear\0")
                    .map_err(|_| ())?;

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
                    PyDict_New,
                    PyDict_SetItemString,
                    PyErr_Occurred,
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

pub fn ensure_cpython() -> Result<(), ()> {
    if CPY.lock().unwrap().is_none() {
        try_load_cpython()?;
        unsafe {
            if let Some(cpy) = &*CPY.lock().unwrap() {
                if (cpy.Py_IsInitialized)() == 0 {
                    (cpy.Py_Initialize)();
                }
            }
        }
    }
    Ok(())
}
