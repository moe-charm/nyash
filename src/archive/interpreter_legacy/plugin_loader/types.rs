//! Types and globals for interpreter plugin loader

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::ffi::c_void;

#[cfg(feature = "dynamic-file")]
use libloading::Library;

lazy_static::lazy_static! {
    /// Global cache for loaded plugins (keyed by simple name like "file" or "math")
    pub(crate) static ref PLUGIN_CACHE: RwLock<HashMap<String, LoadedPlugin>> = RwLock::new(HashMap::new());
}

/// Loaded plugin handle + basic info
#[cfg(feature = "dynamic-file")]
pub(crate) struct LoadedPlugin {
    pub(crate) library: Library,
    pub(crate) info: PluginInfo,
}

/// Minimal plugin info (simplified)
#[derive(Clone)]
pub(crate) struct PluginInfo {
    pub(crate) name: String,
    pub(crate) version: u32,
    pub(crate) api_version: u32,
}

/// FileBox native handle wrapper
#[derive(Debug)]
pub(crate) struct FileBoxHandle { pub(crate) ptr: *mut c_void }

impl Drop for FileBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("file") {
                    unsafe {
                        use libloading::Symbol;
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_file_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for FileBoxHandle {}
unsafe impl Sync for FileBoxHandle {}

/// MathBox native handle wrapper
#[derive(Debug)]
pub(crate) struct MathBoxHandle { pub(crate) ptr: *mut c_void }

impl Drop for MathBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        use libloading::Symbol;
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_math_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for MathBoxHandle {}
unsafe impl Sync for MathBoxHandle {}

/// RandomBox native handle wrapper
#[derive(Debug)]
pub(crate) struct RandomBoxHandle { pub(crate) ptr: *mut c_void }

impl Drop for RandomBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        use libloading::Symbol;
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_random_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for RandomBoxHandle {}
unsafe impl Sync for RandomBoxHandle {}

/// TimeBox native handle wrapper
#[derive(Debug)]
pub(crate) struct TimeBoxHandle { pub(crate) ptr: *mut c_void }

impl Drop for TimeBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        use libloading::Symbol;
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_time_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for TimeBoxHandle {}
unsafe impl Sync for TimeBoxHandle {}

/// DateTimeBox native handle wrapper
#[derive(Debug)]
pub(crate) struct DateTimeBoxHandle { pub(crate) ptr: *mut c_void }

impl Drop for DateTimeBoxHandle {
    fn drop(&mut self) {
        #[cfg(feature = "dynamic-file")]
        {
            if !self.ptr.is_null() {
                let cache = PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        use libloading::Symbol;
                        if let Ok(free_fn) = plugin.library.get::<Symbol<unsafe extern "C" fn(*mut c_void)>>(b"nyash_datetime_free\0") {
                            free_fn(self.ptr);
                        }
                    }
                }
            }
        }
    }
}

unsafe impl Send for DateTimeBoxHandle {}
unsafe impl Sync for DateTimeBoxHandle {}

