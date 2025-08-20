//! Nyash v2 Plugin Loader
//!
//! cfg/features で2パスを提供:
//! - enabled: plugins feature 有効 かつ 非wasm32 ターゲット
//! - stub   : それ以外（WASMやplugins無効）

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
mod enabled {
    use crate::bid::{BidResult, BidError};
    use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, IntegerBox, BoolBox};
    use crate::config::nyash_toml_v2::{NyashConfigV2, LibraryDefinition};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    // use std::ffi::c_void; // unused
    use std::any::Any;
    use once_cell::sync::Lazy;

/// Loaded plugin information
    pub struct LoadedPluginV2 {
    /// Library handle
    _lib: Arc<libloading::Library>,
    
    /// Box types provided by this plugin
    box_types: Vec<String>,
    
    /// Optional init function
    init_fn: Option<unsafe extern "C" fn() -> i32>,
    
    /// Required invoke function  
    invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
}

/// v2 Plugin Box wrapper - temporary implementation
#[derive(Debug)]
    pub struct PluginBoxV2 {
        pub box_type: String,
        pub type_id: u32,
        pub invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
        pub instance_id: u32,
        /// Optional fini method_id from nyash.toml (None if not provided)
        pub fini_method_id: Option<u32>,
    }

    impl BoxCore for PluginBoxV2 {
    fn box_id(&self) -> u64 {
        self.instance_id as u64
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.box_type, self.instance_id)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

    impl NyashBox for PluginBoxV2 {
    fn type_name(&self) -> &'static str {
        // Return the actual box type name for proper method dispatch
        match self.box_type.as_str() {
            "FileBox" => "FileBox",
            _ => "PluginBoxV2",
        }
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        eprintln!("🔍 DEBUG: PluginBoxV2::clone_box called for {} (id={})", self.box_type, self.instance_id);
        
        // Clone means creating a new instance by calling birth()
        let mut output_buffer = vec![0u8; 1024];
        let mut output_len = output_buffer.len();
        let tlv_args = vec![1u8, 0, 0, 0]; // version=1, argc=0
        
        let result = unsafe {
            (self.invoke_fn)(
                self.type_id,
                0,                 // method_id=0 (birth)
                0,                 // instance_id=0 (static call)
                tlv_args.as_ptr(),
                tlv_args.len(),
                output_buffer.as_mut_ptr(),
                &mut output_len,
            )
        };
        
        if result == 0 && output_len >= 4 {
            // Extract new instance_id from output
            let new_instance_id = u32::from_le_bytes([
                output_buffer[0], output_buffer[1], 
                output_buffer[2], output_buffer[3]
            ]);
            
            eprintln!("🎉 clone_box success: created new {} instance_id={}", self.box_type, new_instance_id);
            
            // Return new PluginBoxV2 with new instance_id
            Box::new(PluginBoxV2 {
                box_type: self.box_type.clone(),
                type_id: self.type_id,
                invoke_fn: self.invoke_fn,
                instance_id: new_instance_id,
                fini_method_id: self.fini_method_id,
            })
        } else {
            eprintln!("❌ clone_box failed: birth() returned error code {}", result);
            // Fallback: return error message as StringBox
            Box::new(StringBox::new(format!("Clone failed for {}", self.box_type)))
        }
    }
    
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        StringBox::new(format!("{}({})", self.box_type, self.instance_id))
    }
    
    fn equals(&self, _other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        crate::box_trait::BoolBox::new(false)
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        eprintln!("🔍 DEBUG: PluginBoxV2::share_box called for {} (id={})", self.box_type, self.instance_id);
        
        // Share means returning a new Box with the same instance_id
        Box::new(PluginBoxV2 {
            box_type: self.box_type.clone(),
            type_id: self.type_id,
            invoke_fn: self.invoke_fn,
            instance_id: self.instance_id,  // Same instance_id - this is sharing!
            fini_method_id: self.fini_method_id,
        })
    }
}

impl PluginBoxV2 {
    /// Call fini() on this plugin instance if configured
    pub fn call_fini(&self) {
        if let Some(fini_id) = self.fini_method_id {
            // Empty TLV args
            let tlv_args: [u8; 4] = [1, 0, 0, 0];
            let mut out: [u8; 4] = [0; 4];
            let mut out_len: usize = out.len();
            let rc = unsafe {
                (self.invoke_fn)(
                    self.type_id,
                    fini_id,
                    self.instance_id,
                    tlv_args.as_ptr(),
                    tlv_args.len(),
                    out.as_mut_ptr(),
                    &mut out_len,
                )
            };
            if rc != 0 {
                eprintln!("⚠️ PluginBoxV2::fini failed for {} id={} rc={}", self.box_type, self.instance_id, rc);
            }
        }
    }
}

/// Plugin loader v2
    pub struct PluginLoaderV2 {
    /// Loaded plugins (library name -> plugin info)
    plugins: RwLock<HashMap<String, Arc<LoadedPluginV2>>>,
    
    /// Configuration
    pub config: Option<NyashConfigV2>,
}

    impl PluginLoaderV2 {
    /// Create new loader
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            config: None,
        }
    }
    
    /// Load configuration from nyash.toml
    pub fn load_config(&mut self, config_path: &str) -> BidResult<()> {
        self.config = Some(NyashConfigV2::from_file(config_path)
            .map_err(|e| {
                eprintln!("Failed to load config: {}", e);
                BidError::PluginError
            })?);
        Ok(())
    }
    
    /// Load all plugins from config
        pub fn load_all_plugins(&self) -> BidResult<()> {
        let config = self.config.as_ref()
            .ok_or(BidError::PluginError)?;
        
        for (lib_name, lib_def) in &config.libraries {
            if let Err(e) = self.load_plugin(lib_name, lib_def) {
                eprintln!("Warning: Failed to load plugin {}: {:?}", lib_name, e);
            }
        }
        
        Ok(())
    }

    /// Perform an external host call (env.* namespace) or return an error if unsupported
    /// Returns Some(Box) for a value result, or None for void-like calls
    pub fn extern_call(
            &self,
            iface_name: &str,
            method_name: &str,
            args: &[Box<dyn NyashBox>],
        ) -> BidResult<Option<Box<dyn NyashBox>>> {
        match (iface_name, method_name) {
            ("env.console", "log") => {
                for a in args {
                    println!("{}", a.to_string_box().value);
                }
                Ok(None)
            }
            ("env.canvas", _) => {
                eprintln!("[env.canvas] {} invoked (stub)", method_name);
                Ok(None)
            }
            _ => {
                // Future: route to plugin-defined extern interfaces via config
                Err(BidError::InvalidMethod)
            }
        }
    }

        fn resolve_method_id_from_file(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
            let config = self.config.as_ref().ok_or(BidError::PluginError)?;
            let (lib_name, _lib_def) = config.find_library_for_box(box_type)
                .ok_or(BidError::InvalidType)?;
            let toml_content = std::fs::read_to_string("nyash.toml").map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            let method = box_conf.methods.get(method_name).ok_or(BidError::InvalidMethod)?;
            Ok(method.method_id)
        }

        /// Invoke an instance method on a plugin box by name (minimal TLV encoding)
        pub fn invoke_instance_method(
            &self,
            box_type: &str,
            method_name: &str,
            instance_id: u32,
            args: &[Box<dyn NyashBox>],
        ) -> BidResult<Option<Box<dyn NyashBox>>> {
            // Only support zero-argument methods for now (minimal viable)
            if !args.is_empty() {
                return Err(BidError::InvalidMethod);
            }
            let method_id = self.resolve_method_id_from_file(box_type, method_name)?;
            // Find plugin and type_id
            let config = self.config.as_ref().ok_or(BidError::PluginError)?;
            let (lib_name, _lib_def) = config.find_library_for_box(box_type).ok_or(BidError::InvalidType)?;
            let plugins = self.plugins.read().unwrap();
            let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
            let toml_content = std::fs::read_to_string("nyash.toml").map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            let type_id = box_conf.type_id;
            // TLV args: encode provided arguments
            let tlv_args = {
                // minimal local encoder (duplicates of encode_tlv_args not accessible here)
                let mut buf = Vec::with_capacity(4 + args.len() * 12);
                buf.extend_from_slice(&[1u8, args.len() as u8, 0, 0]);
                for a in args {
                    if let Some(i) = a.as_any().downcast_ref::<IntegerBox>() {
                        buf.push(1);
                        buf.extend_from_slice(&(i.value as i64).to_le_bytes());
                    } else if let Some(b) = a.as_any().downcast_ref::<BoolBox>() {
                        buf.push(3);
                        buf.push(if b.value {1} else {0});
                    } else if let Some(s) = a.as_any().downcast_ref::<StringBox>() {
                        let bytes = s.value.as_bytes();
                        buf.push(2);
                        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        buf.extend_from_slice(bytes);
                    } else {
                        let s = a.to_string_box().value;
                        let bytes = s.as_bytes();
                        buf.push(2);
                        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        buf.extend_from_slice(bytes);
                    }
                }
                buf
            };
            let mut out = vec![0u8; 1024];
            let mut out_len: usize = out.len();
            let rc = unsafe {
                (plugin.invoke_fn)(
                    type_id,
                    method_id,
                    instance_id,
                    tlv_args.as_ptr(),
                    tlv_args.len(),
                    out.as_mut_ptr(),
                    &mut out_len,
                )
            };
            if rc != 0 {
                return Err(BidError::InvalidMethod);
            }
            // minimal decode: tag=1 int, tag=2 string, tag=3 bool, otherwise None
            let result = if out_len == 0 { None } else {
                let data = &out[..out_len];
                match data[0] {
                    1 if data.len() >= 9 => {
                        let mut arr = [0u8;8];
                        arr.copy_from_slice(&data[1..9]);
                        Some(Box::new(IntegerBox::new(i64::from_le_bytes(arr))) as Box<dyn NyashBox>)
                    }
                    2 if data.len() >= 5 => {
                        let mut larr = [0u8;4];
                        larr.copy_from_slice(&data[1..5]);
                        let len = u32::from_le_bytes(larr) as usize;
                        if data.len() >= 5+len {
                            let s = String::from_utf8_lossy(&data[5..5+len]).to_string();
                            Some(Box::new(StringBox::new(s)) as Box<dyn NyashBox>)
                        } else { None }
                    }
                    3 if data.len() >= 2 => {
                        Some(Box::new(BoolBox::new(data[1] != 0)) as Box<dyn NyashBox>)
                    }
                    _ => None,
                }
            };
            Ok(result)
        }
    
    /// Load single plugin
    pub fn load_plugin(&self, lib_name: &str, lib_def: &LibraryDefinition) -> BidResult<()> {
        // Check if already loaded
        {
            let plugins = self.plugins.read().unwrap();
            if plugins.contains_key(lib_name) {
                return Ok(());
            }
        }
        
        // Load library
        let lib = unsafe {
            libloading::Library::new(&lib_def.path)
                .map_err(|e| {
                    eprintln!("Failed to load library: {}", e);
                    BidError::PluginError
                })?
        };
        
        // Get required invoke function and dereference it
        let invoke_fn = unsafe {
            let symbol: libloading::Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32> = 
                lib.get(b"nyash_plugin_invoke")
                    .map_err(|e| {
                        eprintln!("Missing nyash_plugin_invoke: {}", e);
                        BidError::InvalidMethod
                    })?;
            *symbol // Dereference to get the actual function pointer
        };
        
        // Get optional init function and dereference it
        let init_fn = unsafe {
            lib.get::<unsafe extern "C" fn() -> i32>(b"nyash_plugin_init").ok()
                .map(|f| *f)
        };
        
        // Call init if available
        if let Some(init) = init_fn {
            let result = unsafe { init() };
            if result != 0 {
                eprintln!("Plugin init failed with code: {}", result);
                return Err(BidError::PluginError);
            }
        }
        
        // Store plugin with Arc-wrapped library
        let lib_arc = Arc::new(lib);
        let plugin = Arc::new(LoadedPluginV2 {
            _lib: lib_arc,
            box_types: lib_def.boxes.clone(),
            init_fn,
            invoke_fn,
        });
        
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(lib_name.to_string(), plugin);
        
        Ok(())
    }
    
    /// Create a Box instance
    pub fn create_box(&self, box_type: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
        eprintln!("🔍 create_box called for: {}", box_type);
        
        let config = self.config.as_ref()
            .ok_or(BidError::PluginError)?;
        
        eprintln!("🔍 Config loaded successfully");
        
        // Find library that provides this box type
        let (lib_name, _lib_def) = config.find_library_for_box(box_type)
            .ok_or_else(|| {
                eprintln!("No plugin provides box type: {}", box_type);
                BidError::InvalidType
            })?;
        
        eprintln!("🔍 Found library: {} for box type: {}", lib_name, box_type);
        
        // Get loaded plugin
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins.get(lib_name)
            .ok_or_else(|| {
                eprintln!("Plugin not loaded: {}", lib_name);
                BidError::PluginError
            })?;
        
        eprintln!("🔍 Plugin loaded successfully");
        
        // Get type_id from config - read actual nyash.toml content
        eprintln!("🔍 Reading nyash.toml for type configuration...");
        let (type_id, fini_method_id) = if let Ok(toml_content) = std::fs::read_to_string("nyash.toml") {
            eprintln!("🔍 nyash.toml read successfully");
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                eprintln!("🔍 nyash.toml parsed successfully");
                if let Some(box_config) = config.get_box_config(lib_name, box_type, &toml_value) {
                    eprintln!("🔍 Found box config for {} with type_id: {}", box_type, box_config.type_id);
                    let fini_id = box_config.methods.get("fini").map(|m| m.method_id);
                    (box_config.type_id, fini_id)
                } else {
                    eprintln!("No type configuration for {} in {}", box_type, lib_name);
                    return Err(BidError::InvalidType);
                }
            } else {
                eprintln!("Failed to parse nyash.toml");
                return Err(BidError::PluginError);
            }
        } else {
            eprintln!("Failed to read nyash.toml");
            return Err(BidError::PluginError);
        };
        
        // Call birth constructor (method_id = 0) via TLV encoding
        eprintln!("🔍 Preparing to call birth() with type_id: {}", type_id);
        let mut output_buffer = vec![0u8; 1024]; // 1KB buffer for output
        let mut output_len = output_buffer.len();
        
        // Create TLV-encoded empty arguments (version=1, argc=0)
        let tlv_args = vec![1u8, 0, 0, 0]; // version=1, argc=0
        eprintln!("🔍 Output buffer allocated, about to call plugin invoke_fn...");
        
        let birth_result = unsafe {
            eprintln!("🔍 Calling invoke_fn(type_id={}, method_id=0, instance_id=0, tlv_args={:?}, output_buf, output_size={})", type_id, tlv_args, output_buffer.len());
            (plugin.invoke_fn)(
                type_id,           // Box type ID
                0,                 // method_id for birth
                0,                 // instance_id = 0 for birth (static call)
                tlv_args.as_ptr(), // TLV-encoded input data
                tlv_args.len(),    // input size
                output_buffer.as_mut_ptr(), // output buffer
                &mut output_len,   // output buffer size (mutable)
            )
        };
        
        eprintln!("🔍 invoke_fn returned with result: {}", birth_result);
        
        if birth_result != 0 {
            eprintln!("birth() failed with code: {}", birth_result);
            return Err(BidError::PluginError);
        }
        
        // Parse instance_id from output (first 4 bytes as u32)
        let instance_id = if output_len >= 4 {
            u32::from_le_bytes([output_buffer[0], output_buffer[1], output_buffer[2], output_buffer[3]])
        } else {
            eprintln!("birth() returned insufficient data (got {} bytes, need 4)", output_len);
            return Err(BidError::PluginError);
        };
        
        eprintln!("🎉 birth() success: {} instance_id={}", box_type, instance_id);
        
        // Create v2 plugin box wrapper with actual instance_id
        let plugin_box = PluginBoxV2 {
            box_type: box_type.to_string(),
            type_id,
            invoke_fn: plugin.invoke_fn,
            instance_id,
            fini_method_id,
        };
        
        Ok(Box::new(plugin_box))
    }
}

// Global loader instance
    static GLOBAL_LOADER_V2: Lazy<Arc<RwLock<PluginLoaderV2>>> =
        Lazy::new(|| Arc::new(RwLock::new(PluginLoaderV2::new())));

    /// Get global v2 loader
    pub fn get_global_loader_v2() -> Arc<RwLock<PluginLoaderV2>> {
        GLOBAL_LOADER_V2.clone()
    }

    /// Initialize global loader with config
    pub fn init_global_loader_v2(config_path: &str) -> BidResult<()> {
        let loader = get_global_loader_v2();
        let mut loader = loader.write().unwrap();
        loader.load_config(config_path)?;
        drop(loader); // Release write lock

        // Load all plugins
        let loader = get_global_loader_v2();
        let loader = loader.read().unwrap();
        loader.load_all_plugins()
    }
}

#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
mod stub {
    use crate::bid::{BidResult, BidError};
    use crate::box_trait::NyashBox;
    use once_cell::sync::Lazy;
    use std::sync::{Arc, RwLock};

    pub struct PluginLoaderV2 {
        pub config: Option<()>,  // Dummy config for compatibility
    }
    impl PluginLoaderV2 { 
        pub fn new() -> Self { 
            Self { config: None } 
        } 
    } 
    impl PluginLoaderV2 {
        pub fn load_config(&mut self, _p: &str) -> BidResult<()> { Ok(()) }
        pub fn load_all_plugins(&self) -> BidResult<()> { Ok(()) }
        pub fn create_box(&self, _t: &str, _a: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
            Err(BidError::PluginError)
        }

        pub fn extern_call(
            &self,
            _iface_name: &str,
            _method_name: &str,
            _args: &[Box<dyn NyashBox>],
        ) -> BidResult<Option<Box<dyn NyashBox>>> {
            Err(BidError::PluginError)
        }

        pub fn invoke_instance_method(
            &self,
            _box_type: &str,
            _method_name: &str,
            _instance_id: u32,
            _args: &[Box<dyn NyashBox>],
        ) -> BidResult<Option<Box<dyn NyashBox>>> {
            Err(BidError::PluginError)
        }
    }

    static GLOBAL_LOADER_V2: Lazy<Arc<RwLock<PluginLoaderV2>>> =
        Lazy::new(|| Arc::new(RwLock::new(PluginLoaderV2::new())));

    pub fn get_global_loader_v2() -> Arc<RwLock<PluginLoaderV2>> { GLOBAL_LOADER_V2.clone() }
    pub fn init_global_loader_v2(_config_path: &str) -> BidResult<()> { Ok(()) }
}

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub use enabled::*;
#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
pub use stub::*;
