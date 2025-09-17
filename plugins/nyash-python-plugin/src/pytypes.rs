use crate::ffi::{self, CPython, PyObject};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::NonNull;

pub enum DecodedValue {
    Float(f64),
    Int(i64),
    Str(String),
    Bytes(Vec<u8>),
}

pub struct PyOwned {
    ptr: NonNull<PyObject>,
}

#[allow(dead_code)]
pub struct PyBorrowed<'a> {
    ptr: NonNull<PyObject>,
    _marker: PhantomData<&'a PyObject>,
}

impl PyOwned {
    pub unsafe fn from_new(ptr: *mut PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| PyOwned { ptr })
    }

    pub unsafe fn from_raw(ptr: *mut PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| PyOwned { ptr })
    }

    #[allow(dead_code)]
    pub unsafe fn from_borrowed(cpy: &CPython, borrowed: PyBorrowed<'_>) -> Self {
        (cpy.Py_IncRef)(borrowed.ptr.as_ptr());
        PyOwned { ptr: borrowed.ptr }
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        self.ptr.as_ptr()
    }

    #[allow(dead_code)]
    pub fn borrow(&self) -> PyBorrowed<'_> {
        PyBorrowed {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn clone_ref(&self, cpy: &CPython) -> Self {
        unsafe {
            (cpy.Py_IncRef)(self.ptr.as_ptr());
        }
        PyOwned { ptr: self.ptr }
    }

    pub fn into_raw(self) -> *mut PyObject {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        ptr
    }
}

impl Drop for PyOwned {
    fn drop(&mut self) {
        if let Ok(guard) = ffi::CPY.lock() {
            if let Some(cpy) = guard.as_ref() {
                unsafe {
                    (cpy.Py_DecRef)(self.ptr.as_ptr());
                }
            }
        }
    }
}

impl Clone for PyOwned {
    fn clone(&self) -> Self {
        let guard = ffi::CPY.lock().expect("CPython state poisoned");
        let cpy = guard.as_ref().expect("CPython not initialized");
        unsafe {
            (cpy.Py_IncRef)(self.ptr.as_ptr());
        }
        PyOwned { ptr: self.ptr }
    }
}

unsafe impl Send for PyOwned {}
unsafe impl Sync for PyOwned {}

#[allow(dead_code)]
impl<'a> PyBorrowed<'a> {
    pub unsafe fn new(ptr: *mut PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| PyBorrowed {
            ptr,
            _marker: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        self.ptr.as_ptr()
    }
}

impl<'a> Copy for PyBorrowed<'a> {}

impl<'a> Clone for PyBorrowed<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn autodecode(cpy: &CPython, obj: *mut PyObject) -> Option<DecodedValue> {
    unsafe {
        let f = (cpy.PyFloat_AsDouble)(obj);
        if (cpy.PyErr_Occurred)().is_null() {
            return Some(DecodedValue::Float(f));
        }
        (cpy.PyErr_Clear)();

        let i = (cpy.PyLong_AsLongLong)(obj);
        if (cpy.PyErr_Occurred)().is_null() {
            return Some(DecodedValue::Int(i));
        }
        (cpy.PyErr_Clear)();

        let u = (cpy.PyUnicode_AsUTF8)(obj);
        if (cpy.PyErr_Occurred)().is_null() && !u.is_null() {
            let s = CStr::from_ptr(u).to_string_lossy().to_string();
            return Some(DecodedValue::Str(s));
        }
        (cpy.PyErr_Clear)();

        let mut ptr: *mut c_char = std::ptr::null_mut();
        let mut sz: isize = 0;
        if (cpy.PyBytes_AsStringAndSize)(obj, &mut ptr, &mut sz) == 0 {
            let slice = std::slice::from_raw_parts(ptr as *const u8, sz as usize);
            return Some(DecodedValue::Bytes(slice.to_vec()));
        }
        if !(cpy.PyErr_Occurred)().is_null() {
            (cpy.PyErr_Clear)();
        }
    }
    None
}

pub fn cstring_from_str(s: &str) -> Result<CString, ()> {
    CString::new(s).map_err(|_| ())
}

pub unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().to_string())
}

#[allow(dead_code)]
pub unsafe fn incref(cpy: &CPython, obj: *mut PyObject) {
    (cpy.Py_IncRef)(obj);
}

#[allow(dead_code)]
pub unsafe fn decref(cpy: &CPython, obj: *mut PyObject) {
    (cpy.Py_DecRef)(obj);
}

pub fn take_py_error_string(cpy: &CPython) -> Option<String> {
    unsafe {
        if (cpy.PyErr_Occurred)().is_null() {
            return None;
        }
        let mut ptype: *mut PyObject = std::ptr::null_mut();
        let mut pvalue: *mut PyObject = std::ptr::null_mut();
        let mut ptrace: *mut PyObject = std::ptr::null_mut();
        (cpy.PyErr_Fetch)(&mut ptype, &mut pvalue, &mut ptrace);
        let s = if !pvalue.is_null() {
            let sobj = (cpy.PyObject_Str)(pvalue);
            if sobj.is_null() {
                (cpy.PyErr_Clear)();
                return Some("Python error".to_string());
            }
            let cstr = (cpy.PyUnicode_AsUTF8)(sobj);
            let msg = if cstr.is_null() {
                "Python error".to_string()
            } else {
                CStr::from_ptr(cstr).to_string_lossy().to_string()
            };
            (cpy.Py_DecRef)(sobj);
            msg
        } else {
            "Python error".to_string()
        };
        (cpy.PyErr_Clear)();
        Some(s)
    }
}

pub fn count_tlv_args(args: *const u8, args_len: usize) -> usize {
    if args.is_null() || args_len < 4 {
        return 0;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    if buf.len() < 4 {
        return 0;
    }
    u16::from_le_bytes([buf[2], buf[3]]) as usize
}

pub fn tuple_from_tlv(
    cpy: &CPython,
    args: *const u8,
    args_len: usize,
) -> Result<*mut PyObject, ()> {
    let argc = count_tlv_args(args, args_len) as isize;
    let tuple = unsafe { (cpy.PyTuple_New)(argc) };
    if tuple.is_null() {
        return Err(());
    }
    if argc == 0 {
        return Ok(tuple);
    }
    if !fill_tuple_from_tlv(cpy, tuple, args, args_len) {
        unsafe {
            (cpy.Py_DecRef)(tuple);
        }
        return Err(());
    }
    Ok(tuple)
}

pub fn kwargs_from_tlv(
    cpy: &CPython,
    args: *const u8,
    args_len: usize,
) -> Result<*mut PyObject, ()> {
    let dict = unsafe { (cpy.PyDict_New)() };
    if dict.is_null() {
        return Err(());
    }
    if !fill_kwargs_from_tlv(cpy, dict, args, args_len) {
        unsafe {
            (cpy.Py_DecRef)(dict);
        }
        return Err(());
    }
    Ok(dict)
}

pub fn fill_tuple_from_tlv(
    cpy: &CPython,
    tuple: *mut PyObject,
    args: *const u8,
    args_len: usize,
) -> bool {
    if args.is_null() || args_len < 4 {
        return true;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    let mut idx: isize = 0;
    while off + 4 <= buf.len() {
        let tag = buf[off];
        let _rsv = buf[off + 1];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if off + 4 + size > buf.len() {
            return false;
        }
        let payload = &buf[off + 4..off + 4 + size];
        let mut obj: *mut PyObject = std::ptr::null_mut();
        unsafe {
            let mut owned_transfer: Option<PyOwned> = None;
            match tag {
                1 => {
                    let v = if size >= 1 && payload[0] != 0 { 1 } else { 0 };
                    obj = (cpy.PyBool_FromLong)(v);
                }
                2 => {
                    if size != 4 {
                        return false;
                    }
                    let mut b = [0u8; 4];
                    b.copy_from_slice(payload);
                    obj = (cpy.PyLong_FromLongLong)(i32::from_le_bytes(b) as i64);
                }
                3 => {
                    if size != 8 {
                        return false;
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    obj = (cpy.PyLong_FromLongLong)(i64::from_le_bytes(b));
                }
                5 => {
                    if size != 8 {
                        return false;
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    obj = (cpy.PyFloat_FromDouble)(f64::from_le_bytes(b));
                }
                6 => {
                    let c = match CString::new(payload) {
                        Ok(c) => c,
                        Err(_) => return false,
                    };
                    obj = (cpy.PyUnicode_FromString)(c.as_ptr());
                }
                7 => {
                    let ptr = if size > 0 {
                        payload.as_ptr() as *const c_char
                    } else {
                        std::ptr::null()
                    };
                    obj = (cpy.PyBytes_FromStringAndSize)(ptr, size as isize);
                }
                8 => {
                    if size != 8 {
                        return false;
                    }
                    let mut i = [0u8; 4];
                    i.copy_from_slice(&payload[4..8]);
                    let inst_id = u32::from_le_bytes(i);
                    let Some(handle) = super::PY_HANDLES.lock().unwrap().get(&inst_id).cloned()
                    else {
                        return false;
                    };
                    owned_transfer = Some(handle);
                }
                _ => return false,
            }
            let raw = if let Some(handle) = owned_transfer.take() {
                handle.into_raw()
            } else {
                obj
            };
            if (cpy.PyTuple_SetItem)(tuple, idx, raw) != 0 {
                if let Some(rewrap) = PyOwned::from_raw(raw) {
                    drop(rewrap);
                }
                return false;
            }
            idx += 1;
        }
        off += 4 + size;
    }
    true
}

pub fn fill_kwargs_from_tlv(
    cpy: &CPython,
    dict: *mut PyObject,
    args: *const u8,
    args_len: usize,
) -> bool {
    if args.is_null() || args_len < 4 {
        return true;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    while off + 4 <= buf.len() {
        // key (string)
        if buf[off] != 6 {
            return false;
        }
        let key_size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if off + 4 + key_size > buf.len() {
            return false;
        }
        let key_slice = &buf[off + 4..off + 4 + key_size];
        let key_c = match CString::new(key_slice) {
            Ok(c) => c,
            Err(_) => return false,
        };
        off += 4 + key_size;
        if off + 4 > buf.len() {
            return false;
        }
        let tag_v = buf[off];
        let _rsv = buf[off + 1];
        let size_v = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if off + 4 + size_v > buf.len() {
            return false;
        }
        let val_payload = &buf[off + 4..off + 4 + size_v];
        let obj: *mut PyObject;
        unsafe {
            match tag_v {
                1 => {
                    let v = if size_v >= 1 && val_payload[0] != 0 {
                        1
                    } else {
                        0
                    };
                    obj = (cpy.PyBool_FromLong)(v);
                }
                2 => {
                    if size_v != 4 {
                        return false;
                    }
                    let mut b = [0u8; 4];
                    b.copy_from_slice(val_payload);
                    obj = (cpy.PyLong_FromLongLong)(i32::from_le_bytes(b) as i64);
                }
                3 => {
                    if size_v != 8 {
                        return false;
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(val_payload);
                    obj = (cpy.PyLong_FromLongLong)(i64::from_le_bytes(b));
                }
                5 => {
                    if size_v != 8 {
                        return false;
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(val_payload);
                    obj = (cpy.PyFloat_FromDouble)(f64::from_le_bytes(b));
                }
                6 => {
                    let c = match CString::new(val_payload) {
                        Ok(c) => c,
                        Err(_) => return false,
                    };
                    obj = (cpy.PyUnicode_FromString)(c.as_ptr());
                }
                7 => {
                    let ptr = if size_v > 0 {
                        val_payload.as_ptr() as *const c_char
                    } else {
                        std::ptr::null()
                    };
                    obj = (cpy.PyBytes_FromStringAndSize)(ptr, size_v as isize);
                }
                8 => {
                    if size_v != 8 {
                        return false;
                    }
                    let mut i = [0u8; 4];
                    i.copy_from_slice(&val_payload[4..8]);
                    let inst_id = u32::from_le_bytes(i);
                    let Some(handle) = super::PY_HANDLES.lock().unwrap().get(&inst_id).cloned()
                    else {
                        return false;
                    };
                    obj = handle.into_raw();
                }
                _ => return false,
            }
            let rc = (cpy.PyDict_SetItemString)(dict, key_c.as_ptr(), obj);
            (cpy.Py_DecRef)(obj);
            if rc != 0 {
                return false;
            }
        }
        off += 4 + size_v;
    }
    true
}
