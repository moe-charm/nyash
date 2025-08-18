//! Multi-box plugin loader (v2)
//! 
//! Supports loading plugins that provide multiple Box types

use crate::bid::{BidHandle, TlvEncoder, TlvDecoder};
use crate::box_trait::{NyashBox, StringBox};
use crate::config::nyash_toml_v2::{NyashConfigV2, LibraryDefinition};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[cfg(feature = "dynamic-file")]
use libloading::{Library, Symbol};

/// FFI type definitions (must match plugin definitions)
#[repr(C)]
pub struct NyashHostVtable {
    pub alloc: unsafe extern "C" fn(size: usize) -> *mut u8,
    pub free: unsafe extern "C" fn(ptr: *mut u8),
    pub wake: unsafe extern "C" fn(handle: u64),
    pub log: unsafe extern "C" fn(level: i32, msg: *const c_char),
}

#[repr(C)]
pub struct NyashMethodInfo {
    pub method_id: u32,
    pub name: *const c_char,
    pub signature: u32,
}

#[repr(C)]
pub struct NyashPluginInfo {
    pub type_id: u32,
    pub type_name: *const c_char,
    pub method_count: usize,
    pub methods: *const NyashMethodInfo,
}

/// Multi-box plugin library handle
pub struct MultiBoxPluginLibrary {
    #[cfg(feature = "dynamic-file")]
    library: Library,
    
    /// Box type name -> type_id mapping
    box_types: HashMap<String, u32>,
    
    /// Type ID -> Box type name mapping
    type_names: HashMap<u32, String>,
}

/// Multi-box plugin loader
pub struct PluginLoaderV2 {
    /// Library name -> library handle
    libraries: RwLock<HashMap<String, Arc<MultiBoxPluginLibrary>>>,
    
    /// Box type name -> library name mapping
    box_to_library: RwLock<HashMap<String, String>>,
    
    /// Host vtable for plugins
    host_vtable: NyashHostVtable,
}

// Host function implementations
unsafe extern "C" fn host_alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
    std::alloc::alloc(layout)
}

unsafe extern "C" fn host_free(ptr: *mut u8) {
    // Simplified implementation - real implementation needs size tracking
}

unsafe extern "C" fn host_wake(_handle: u64) {
    // Async wake - not implemented yet
}

unsafe extern "C" fn host_log(level: i32, msg: *const c_char) {
    if !msg.is_null() {
        let c_str = CStr::from_ptr(msg);
        let message = c_str.to_string_lossy();
        match level {
            0 => log::debug!("{}", message),
            1 => log::info!("{}", message),
            2 => log::warn!("{}", message),
            3 => log::error!("{}", message),
            _ => log::info!("{}", message),
        }
    }
}

impl PluginLoaderV2 {
    /// Create new multi-box plugin loader
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
            box_to_library: RwLock::new(HashMap::new()),
            host_vtable: NyashHostVtable {
                alloc: host_alloc,
                free: host_free,
                wake: host_wake,
                log: host_log,
            },
        }
    }
    
    /// Load plugins from nyash.toml configuration
    pub fn load_from_config(&self, config: &NyashConfigV2) -> Result<(), String> {
        // Load v2 multi-box plugins
        if let Some(libs) = &config.plugins.libraries {
            for (lib_name, lib_def) in libs {
                self.load_multi_box_plugin(lib_name, lib_def)?;
            }
        }
        
        // Load legacy single-box plugins
        for (box_name, plugin_name) in &config.plugins.legacy_plugins {
            // For now, skip legacy plugins - focus on v2
            log::info!("Legacy plugin {} for {} - skipping", plugin_name, box_name);
        }
        
        Ok(())
    }
    
    /// Load a multi-box plugin library
    fn load_multi_box_plugin(&self, lib_name: &str, lib_def: &LibraryDefinition) -> Result<(), String> {
        #[cfg(feature = "dynamic-file")]
        {
            let library = unsafe {
                Library::new(&lib_def.plugin_path)
                    .map_err(|e| format!("Failed to load plugin {}: {}", lib_name, e))?
            };
            
            // Check ABI version
            let abi_fn: Symbol<unsafe extern "C" fn() -> u32> = unsafe {
                library.get(b"nyash_plugin_abi")
                    .map_err(|e| format!("nyash_plugin_abi not found: {}", e))?
            };
            
            let abi_version = unsafe { abi_fn() };
            if abi_version != 1 {
                return Err(format!("Unsupported ABI version: {}", abi_version));
            }
            
            // Initialize plugin
            let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut std::ffi::c_void) -> i32> = unsafe {
                library.get(b"nyash_plugin_init")
                    .map_err(|e| format!("nyash_plugin_init not found: {}", e))?
            };
            
            let result = unsafe { init_fn(&self.host_vtable, std::ptr::null_mut()) };
            if result != 0 {
                return Err(format!("Plugin initialization failed: {}", result));
            }
            
            // Check if this is a v2 multi-box plugin
            let get_box_count: Result<Symbol<unsafe extern "C" fn() -> u32>, _> = unsafe {
                library.get(b"nyash_plugin_get_box_count")
            };
            
            let mut box_types = HashMap::new();
            let mut type_names = HashMap::new();
            
            if let Ok(get_count_fn) = get_box_count {
                // V2 plugin - get box information
                let box_count = unsafe { get_count_fn() };
                
                let get_info_fn: Symbol<unsafe extern "C" fn(u32) -> *const NyashPluginInfo> = unsafe {
                    library.get(b"nyash_plugin_get_box_info")
                        .map_err(|e| format!("nyash_plugin_get_box_info not found: {}", e))?
                };
                
                for i in 0..box_count {
                    let info_ptr = unsafe { get_info_fn(i) };
                    if info_ptr.is_null() {
                        continue;
                    }
                    
                    let info = unsafe { &*info_ptr };
                    let box_name = if info.type_name.is_null() {
                        continue;
                    } else {
                        unsafe { CStr::from_ptr(info.type_name).to_string_lossy().to_string() }
                    };
                    
                    box_types.insert(box_name.clone(), info.type_id);
                    type_names.insert(info.type_id, box_name.clone());
                    
                    // Register box type to library mapping
                    self.box_to_library.write().unwrap().insert(box_name.clone(), lib_name.to_string());
                    
                    log::info!("Loaded {} (type_id: {}) from {}", box_name, info.type_id, lib_name);
                }
            } else {
                // V1 plugin - single box type
                // TODO: Handle legacy plugins
                return Err(format!("Legacy single-box plugins not yet supported"));
            }
            
            let plugin_lib = Arc::new(MultiBoxPluginLibrary {
                library,
                box_types,
                type_names,
            });
            
            self.libraries.write().unwrap().insert(lib_name.to_string(), plugin_lib);
            
            Ok(())
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(format!("Dynamic library loading disabled. Cannot load plugin: {}", lib_name))
        }
    }
    
    /// Get library name for a box type
    pub fn get_library_for_box(&self, box_type: &str) -> Option<String> {
        self.box_to_library.read().unwrap().get(box_type).cloned()
    }
    
    /// Invoke plugin method
    pub fn invoke_plugin_method(
        &self,
        box_type: &str,
        handle: BidHandle,
        method_name: &str,
        args: &[Box<dyn NyashBox>]
    ) -> Result<Box<dyn NyashBox>, String> {
        #[cfg(feature = "dynamic-file")]
        {
            // Find library for this box type
            let lib_name = self.get_library_for_box(box_type)
                .ok_or_else(|| format!("No plugin loaded for box type: {}", box_type))?;
            
            let libraries = self.libraries.read().unwrap();
            let library = libraries.get(&lib_name)
                .ok_or_else(|| format!("Library not loaded: {}", lib_name))?;
            
            // Get type_id for this box type
            let type_id = library.box_types.get(box_type)
                .ok_or_else(|| format!("Box type {} not found in library {}", box_type, lib_name))?;
            
            // Call plugin method
            self.call_plugin_method(&library.library, *type_id, handle.instance_id, method_name, args)
        }
        
        #[cfg(not(feature = "dynamic-file"))]
        {
            Err(format!("Dynamic library loading disabled"))
        }
    }
    
    #[cfg(feature = "dynamic-file")]
    fn call_plugin_method(
        &self,
        library: &Library,
        type_id: u32,
        instance_id: u32,
        method_name: &str,
        args: &[Box<dyn NyashBox>]
    ) -> Result<Box<dyn NyashBox>, String> {
        // Get invoke function
        let invoke_fn: Symbol<unsafe extern "C" fn(
            u32, u32, u32,          // type_id, method_id, instance_id
            *const u8, usize,       // args, args_len
            *mut u8, *mut usize     // result, result_len
        ) -> i32> = unsafe {
            library.get(b"nyash_plugin_invoke")
                .map_err(|e| format!("nyash_plugin_invoke not found: {}", e))?
        };
        
        // Encode arguments
        let mut encoder = TlvEncoder::new();
        for arg in args {
            encoder.encode_string(&arg.to_string_box().value)
                .map_err(|e| format!("Failed to encode argument: {:?}", e))?;
        }
        let args_data = encoder.finish();
        
        // Map method name to ID (simplified for now)
        let method_id = match method_name {
            "birth" => 0,
            "open" => 1,
            "read" => 2,
            "write" => 3,
            "close" => 4,
            "fini" => u32::MAX,
            _ => return Err(format!("Unknown method: {}", method_name)),
        };
        
        // Call plugin
        let mut result_buffer = vec![0u8; 4096];
        let mut result_len = result_buffer.len();
        
        let status = unsafe {
            invoke_fn(
                type_id,
                method_id,
                instance_id,
                args_data.as_ptr(),
                args_data.len(),
                result_buffer.as_mut_ptr(),
                &mut result_len
            )
        };
        
        if status != 0 {
            return Err(format!("Plugin method failed: status {}", status));
        }
        
        // Decode result (simplified)
        Ok(Box::new(StringBox::new("Plugin result")))
    }
}

/// Global multi-box plugin loader
use once_cell::sync::Lazy;

static GLOBAL_LOADER_V2: Lazy<Arc<PluginLoaderV2>> = 
    Lazy::new(|| Arc::new(PluginLoaderV2::new()));

/// Get global multi-box plugin loader
pub fn get_global_loader_v2() -> Arc<PluginLoaderV2> {
    GLOBAL_LOADER_V2.clone()
}