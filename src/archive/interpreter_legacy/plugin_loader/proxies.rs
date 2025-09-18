//! Proxies for dynamic plugins (File/Math/Random/Time/DateTime)

use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Arc;

#[cfg(feature = "dynamic-file")]
use libloading::Symbol;

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase, IntegerBox};
use crate::boxes::FloatBox;
use crate::interpreter::RuntimeError;

use super::types::{PLUGIN_CACHE, FileBoxHandle, MathBoxHandle, RandomBoxHandle, TimeBoxHandle, DateTimeBoxHandle};
use super::PluginLoader;

// ================== FileBoxProxy ==================

#[derive(Debug)]
pub struct FileBoxProxy {
    pub(crate) handle: Arc<FileBoxHandle>,
    pub(crate) path: String,
    pub(crate) base: BoxBase,
}

unsafe impl Send for FileBoxProxy {}
unsafe impl Sync for FileBoxProxy {}

impl FileBoxProxy {
    pub fn new(handle: *mut c_void, path: String) -> Self {
        FileBoxProxy { handle: Arc::new(FileBoxHandle { ptr: handle }), path, base: BoxBase::new() }
    }

    pub fn read(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("file") {
                unsafe {
                    let read_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_char> =
                        plugin.library.get(b"nyash_file_read\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_file_read: {}", e) })?;
                    let result_ptr = read_fn(self.handle.ptr);
                    if result_ptr.is_null() { return Err(RuntimeError::InvalidOperation { message: "Failed to read file".to_string() }); }
                    let content = CStr::from_ptr(result_ptr).to_string_lossy().into_owned();
                    let free_fn: Symbol<unsafe extern "C" fn(*mut c_char)> =
                        plugin.library.get(b"nyash_string_free\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_string_free: {}", e) })?;
                    free_fn(result_ptr);
                    Ok(Box::new(StringBox::new(content)))
                }
            } else { Err(RuntimeError::InvalidOperation { message: "File plugin not loaded".to_string() }) }
        }
        #[cfg(not(feature = "dynamic-file"))]
        { Err(RuntimeError::InvalidOperation { message: "Dynamic file support not enabled".to_string() }) }
    }

    pub fn write(&self, content: Box<dyn NyashBox>) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("file") {
                let content_str = content.to_string_box().value;
                let c_content = CString::new(content_str).map_err(|_| RuntimeError::InvalidOperation { message: "Invalid content string".to_string() })?;
                unsafe {
                    let write_fn: Symbol<unsafe extern "C" fn(*mut c_void, *const c_char) -> i32> =
                        plugin.library.get(b"nyash_file_write\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_file_write: {}", e) })?;
                    let result = write_fn(self.handle.ptr, c_content.as_ptr());
                    if result == 0 { return Err(RuntimeError::InvalidOperation { message: "Failed to write file".to_string() }); }
                    Ok(Box::new(StringBox::new("ok")))
                }
            } else { Err(RuntimeError::InvalidOperation { message: "File plugin not loaded".to_string() }) }
        }
        #[cfg(not(feature = "dynamic-file"))]
        { Err(RuntimeError::InvalidOperation { message: "Dynamic file support not enabled".to_string() }) }
    }

    pub fn exists(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Ok(Box::new(BoolBox::new(std::path::Path::new(&self.path).exists())))
    }
}

impl BoxCore for FileBoxProxy {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "FileBox({})", self.path) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for FileBoxProxy {
    fn type_name(&self) -> &'static str { "FileBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { match PluginLoader::create_file_box(&self.path) { Ok(b) => b, Err(_) => Box::new(FileBoxProxy::new(self.handle.ptr, self.path.clone())) } }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn to_string_box(&self) -> StringBox { StringBox::new(format!("FileBox({})", self.path)) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { other.as_any().downcast_ref::<FileBoxProxy>().is_some().into() }
}

impl std::fmt::Display for FileBoxProxy { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) } }

// ================== MathBoxProxy ==================

#[derive(Debug)]
pub struct MathBoxProxy { pub(crate) handle: Arc<MathBoxHandle>, pub(crate) base: BoxBase }

unsafe impl Send for MathBoxProxy {}
unsafe impl Sync for MathBoxProxy {}

impl MathBoxProxy { pub fn new(handle: *mut c_void) -> Self { MathBoxProxy { handle: Arc::new(MathBoxHandle { ptr: handle }), base: BoxBase::new() } } }

impl BoxCore for MathBoxProxy {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "MathBox") }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for MathBoxProxy {
    fn type_name(&self) -> &'static str { "MathBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { match PluginLoader::create_math_box() { Ok(new_box) => new_box, Err(_) => Box::new(MathBoxProxy { handle: Arc::clone(&self.handle), base: BoxBase::new() }) } }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn to_string_box(&self) -> StringBox { StringBox::new("MathBox") }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { other.as_any().downcast_ref::<MathBoxProxy>().is_some().into() }
}

impl std::fmt::Display for MathBoxProxy { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) } }

// ================== RandomBoxProxy ==================

#[derive(Debug)]
pub struct RandomBoxProxy { pub(crate) handle: Arc<RandomBoxHandle>, pub(crate) base: BoxBase }

unsafe impl Send for RandomBoxProxy {}
unsafe impl Sync for RandomBoxProxy {}

impl RandomBoxProxy { pub fn new(handle: *mut c_void) -> Self { RandomBoxProxy { handle: Arc::new(RandomBoxHandle { ptr: handle }), base: BoxBase::new() } } }

impl RandomBoxProxy {
    pub fn next(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let next_fn: Symbol<unsafe extern "C" fn(*mut c_void) -> f64> = plugin.library.get(b"nyash_random_next\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_random_next: {}", e) })?;
                    let value = next_fn(self.handle.ptr);
                    Ok(Box::new(FloatBox::new(value)))
                }
            } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
        }
        #[cfg(not(feature = "dynamic-file"))]
        { Err(RuntimeError::InvalidOperation { message: "Dynamic loading not enabled".to_string() }) }
    }
    pub fn range(&self, min: f64, max: f64) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let range_fn: Symbol<unsafe extern "C" fn(*mut c_void, f64, f64) -> f64> = plugin.library.get(b"nyash_random_range\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_random_range: {}", e) })?;
                    let value = range_fn(self.handle.ptr, min, max);
                    Ok(Box::new(FloatBox::new(value)))
                }
            } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
        }
        #[cfg(not(feature = "dynamic-file"))]
        { Err(RuntimeError::InvalidOperation { message: "Dynamic loading not enabled".to_string() }) }
    }
    pub fn int(&self, min: i64, max: i64) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "dynamic-file")]
        {
            let cache = PLUGIN_CACHE.read().unwrap();
            if let Some(plugin) = cache.get("math") {
                unsafe {
                    let int_fn: Symbol<unsafe extern "C" fn(*mut c_void, i64, i64) -> i64> = plugin.library.get(b"nyash_random_int\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_random_int: {}", e) })?;
                    let value = int_fn(self.handle.ptr, min, max);
                    Ok(Box::new(IntegerBox::new(value)))
                }
            } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
        }
        #[cfg(not(feature = "dynamic-file"))]
        { Err(RuntimeError::InvalidOperation { message: "Dynamic loading not enabled".to_string() }) }
    }
}

impl BoxCore for RandomBoxProxy {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "RandomBox") }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for RandomBoxProxy {
    fn type_name(&self) -> &'static str { "RandomBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { match PluginLoader::create_random_box() { Ok(new_box) => new_box, Err(_) => Box::new(RandomBoxProxy { handle: Arc::clone(&self.handle), base: BoxBase::new() }) } }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn to_string_box(&self) -> StringBox { StringBox::new("RandomBox") }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { other.as_any().downcast_ref::<RandomBoxProxy>().is_some().into() }
}

impl std::fmt::Display for RandomBoxProxy { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) } }

// ================== TimeBoxProxy ==================

#[derive(Debug)]
pub struct TimeBoxProxy { pub(crate) handle: Arc<TimeBoxHandle>, pub(crate) base: BoxBase }

unsafe impl Send for TimeBoxProxy {}
unsafe impl Sync for TimeBoxProxy {}

impl TimeBoxProxy { pub fn new(handle: *mut c_void) -> Self { TimeBoxProxy { handle: Arc::new(TimeBoxHandle { ptr: handle }), base: BoxBase::new() } } }

impl BoxCore for TimeBoxProxy {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "TimeBox") }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for TimeBoxProxy {
    fn type_name(&self) -> &'static str { "TimeBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { match PluginLoader::create_time_box() { Ok(new_box) => new_box, Err(_) => Box::new(TimeBoxProxy { handle: Arc::clone(&self.handle), base: BoxBase::new() }) } }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn to_string_box(&self) -> StringBox { StringBox::new("TimeBox") }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { other.as_any().downcast_ref::<TimeBoxProxy>().is_some().into() }
}

impl std::fmt::Display for TimeBoxProxy { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) } }

// ================== DateTimeBoxProxy ==================

#[derive(Debug)]
pub struct DateTimeBoxProxy { pub(crate) handle: Arc<DateTimeBoxHandle>, pub(crate) base: BoxBase }

unsafe impl Send for DateTimeBoxProxy {}
unsafe impl Sync for DateTimeBoxProxy {}

impl DateTimeBoxProxy { pub fn new(handle: *mut c_void) -> Self { DateTimeBoxProxy { handle: Arc::new(DateTimeBoxHandle { ptr: handle }), base: BoxBase::new() } } }

impl BoxCore for DateTimeBoxProxy {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "DateTimeBox") }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for DateTimeBoxProxy {
    fn type_name(&self) -> &'static str { "DateTimeBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { match PluginLoader::create_datetime_now() { Ok(new_box) => new_box, Err(_) => Box::new(DateTimeBoxProxy { handle: Arc::clone(&self.handle), base: BoxBase::new() }) } }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
    fn to_string_box(&self) -> StringBox { StringBox::new("DateTimeBox") }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_datetime) = other.as_any().downcast_ref::<DateTimeBoxProxy>() {
            #[cfg(feature = "dynamic-file")]
            {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(timestamp_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void) -> i64>>(b"nyash_datetime_timestamp\0") {
                            let this_ts = timestamp_fn(self.handle.ptr);
                            let other_ts = timestamp_fn(other_datetime.handle.ptr);
                            return BoolBox::new(this_ts == other_ts);
                        }
                    }
                }
            }
            BoolBox::new(false)
        } else { BoolBox::new(false) }
    }
}

impl std::fmt::Display for DateTimeBoxProxy { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.fmt_box(f) } }

