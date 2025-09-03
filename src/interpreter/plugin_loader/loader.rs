//! Loader entrypoints for dynamic plugins

use std::ffi::{CString, c_char, c_void};

#[cfg(feature = "dynamic-file")]
use libloading::{Library, Symbol};

use crate::box_trait::NyashBox;
use crate::interpreter::RuntimeError;

use super::proxies::{FileBoxProxy, MathBoxProxy, RandomBoxProxy, TimeBoxProxy, DateTimeBoxProxy};
use super::types::{PLUGIN_CACHE, LoadedPlugin, PluginInfo};

/// Public plugin loader API
pub struct PluginLoader;

impl PluginLoader {
    /// Load File plugin
    #[cfg(feature = "dynamic-file")]
    pub fn load_file_plugin() -> Result<(), RuntimeError> {
        let mut cache = PLUGIN_CACHE.write().unwrap();
        if cache.contains_key("file") { return Ok(()); }
        let lib_name = if cfg!(target_os = "windows") { "nyash_file.dll" } else if cfg!(target_os = "macos") { "libnyash_file.dylib" } else { "libnyash_file.so" };
        let possible_paths = vec![
            format!("./target/release/{}", lib_name),
            format!("./target/debug/{}", lib_name),
            format!("./plugins/{}", lib_name),
            format!("./{}", lib_name),
        ];
        let lib_path = possible_paths.iter().find(|p| std::path::Path::new(p.as_str()).exists()).cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation { message: format!("Failed to find file plugin library. Searched paths: {:?}", possible_paths) })?;
        unsafe {
            let library = Library::new(&lib_path).map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to load file plugin: {}", e) })?;
            let init_fn: Symbol<unsafe extern "C" fn() -> *const c_void> = library.get(b"nyash_plugin_init\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get plugin init: {}", e) })?;
            let plugin_info_ptr = init_fn();
            if plugin_info_ptr.is_null() { return Err(RuntimeError::InvalidOperation { message: "Plugin initialization failed".to_string() }); }
            let info = PluginInfo { name: "file".to_string(), version: 1, api_version: 1 };
            cache.insert("file".to_string(), LoadedPlugin { library, info });
        }
        Ok(())
    }

    /// Create FileBox
    #[cfg(feature = "dynamic-file")]
    pub fn create_file_box(path: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_file_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("file") {
            let c_path = CString::new(path).map_err(|_| RuntimeError::InvalidOperation { message: "Invalid path string".to_string() })?;
            unsafe {
                let open_fn: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = plugin.library.get(b"nyash_file_open\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_file_open: {}", e) })?;
                let handle = open_fn(c_path.as_ptr());
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: format!("Failed to open file: {}", path) }); }
                Ok(Box::new(FileBoxProxy::new(handle, path.to_string())))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "File plugin not loaded".to_string() }) }
    }

    /// Check FileBox existence
    #[cfg(feature = "dynamic-file")]
    pub fn file_exists(path: &str) -> Result<bool, RuntimeError> {
        Self::load_file_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("file") {
            let c_path = CString::new(path).map_err(|_| RuntimeError::InvalidOperation { message: "Invalid path string".to_string() })?;
            unsafe {
                let exists_fn: Symbol<unsafe extern "C" fn(*const c_char) -> i32> = plugin.library.get(b"nyash_file_exists\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_file_exists: {}", e) })?;
                Ok(exists_fn(c_path.as_ptr()) != 0)
            }
        } else { Err(RuntimeError::InvalidOperation { message: "File plugin not loaded".to_string() }) }
    }

    /// Load Math plugin
    #[cfg(feature = "dynamic-file")]
    pub fn load_math_plugin() -> Result<(), RuntimeError> {
        let mut cache = PLUGIN_CACHE.write().unwrap();
        if cache.contains_key("math") { return Ok(()); }
        let lib_name = if cfg!(target_os = "windows") { "nyash_math.dll" } else if cfg!(target_os = "macos") { "libnyash_math.dylib" } else { "libnyash_math.so" };
        let possible_paths = vec![
            format!("./target/release/{}", lib_name),
            format!("./target/debug/{}", lib_name),
            format!("./plugins/{}", lib_name),
            format!("./{}", lib_name),
        ];
        let lib_path = possible_paths.iter().find(|p| std::path::Path::new(p.as_str()).exists()).cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation { message: format!("Failed to find math plugin library. Searched paths: {:?}", possible_paths) })?;
        unsafe {
            let library = Library::new(&lib_path).map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to load math plugin: {}", e) })?;
            let info = PluginInfo { name: "math".to_string(), version: 1, api_version: 1 };
            cache.insert("math".to_string(), LoadedPlugin { library, info });
        }
        Ok(())
    }

    /// Create MathBox
    #[cfg(feature = "dynamic-file")]
    pub fn create_math_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = plugin.library.get(b"nyash_math_create\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_math_create: {}", e) })?;
                let handle = create_fn();
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: "Failed to create MathBox".to_string() }); }
                Ok(Box::new(MathBoxProxy::new(handle)))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
    }

    /// Create RandomBox
    #[cfg(feature = "dynamic-file")]
    pub fn create_random_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = plugin.library.get(b"nyash_random_create\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_random_create: {}", e) })?;
                let handle = create_fn();
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: "Failed to create RandomBox".to_string() }); }
                Ok(Box::new(RandomBoxProxy::new(handle)))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
    }

    /// Create TimeBox
    #[cfg(feature = "dynamic-file")]
    pub fn create_time_box() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let create_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = plugin.library.get(b"nyash_time_create\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_time_create: {}", e) })?;
                let handle = create_fn();
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: "Failed to create TimeBox".to_string() }); }
                Ok(Box::new(TimeBoxProxy::new(handle)))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
    }

    /// Create DateTimeBox (now)
    #[cfg(feature = "dynamic-file")]
    pub fn create_datetime_now() -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let now_fn: Symbol<unsafe extern "C" fn() -> *mut c_void> = plugin.library.get(b"nyash_time_now\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_time_now: {}", e) })?;
                let handle = now_fn();
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: "Failed to create DateTimeBox".to_string() }); }
                Ok(Box::new(DateTimeBoxProxy::new(handle)))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
    }

    /// Create DateTimeBox from string
    #[cfg(feature = "dynamic-file")]
    pub fn create_datetime_from_string(time_str: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Self::load_math_plugin()?;
        let cache = PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            let c_str = CString::new(time_str).map_err(|_| RuntimeError::InvalidOperation { message: "Invalid time string".to_string() })?;
            unsafe {
                let parse_fn: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = plugin.library.get(b"nyash_time_parse\0").map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to get nyash_time_parse: {}", e) })?;
                let handle = parse_fn(c_str.as_ptr());
                if handle.is_null() { return Err(RuntimeError::InvalidOperation { message: format!("Failed to parse time string: {}", time_str) }); }
                Ok(Box::new(DateTimeBoxProxy::new(handle)))
            }
        } else { Err(RuntimeError::InvalidOperation { message: "Math plugin not loaded".to_string() }) }
    }
}

