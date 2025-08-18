//! Nyash v2 Plugin Loader
//! 
//! nyash.toml v2ベースの新しいプラグインローダー
//! Single FFI entry point (nyash_plugin_invoke) + optional init

use crate::bid::{BidResult, BidError};
use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox};
use crate::config::nyash_toml_v2::{NyashConfigV2, LibraryDefinition};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::ffi::c_void;
use std::any::Any;

/// Loaded plugin information
pub struct LoadedPluginV2 {
    /// Library handle
    _lib: Arc<libloading::Library>,
    
    /// Box types provided by this plugin
    box_types: Vec<String>,
    
    /// Optional init function
    init_fn: Option<unsafe extern "C" fn() -> i32>,
    
    /// Required invoke function  
    invoke_fn: unsafe extern "C" fn(u32, u32, *const u8, u32, *mut u8, u32) -> i32,
}

/// v2 Plugin Box wrapper - temporary implementation
#[derive(Debug)]
pub struct PluginBoxV2 {
    box_type: String,
    invoke_fn: unsafe extern "C" fn(u32, u32, *const u8, u32, *mut u8, u32) -> i32,
    instance_id: u32,
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
        "PluginBoxV2"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(format!("Cannot clone plugin box {}", self.box_type)))
    }
    
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        StringBox::new(format!("{}({})", self.box_type, self.instance_id))
    }
    
    fn equals(&self, _other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        crate::box_trait::BoolBox::new(false)
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(format!("Cannot share plugin box {}", self.box_type)))
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
            let symbol: libloading::Symbol<unsafe extern "C" fn(u32, u32, *const u8, u32, *mut u8, u32) -> i32> = 
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
        let config = self.config.as_ref()
            .ok_or(BidError::PluginError)?;
        
        // Find library that provides this box type
        let (lib_name, _lib_def) = config.find_library_for_box(box_type)
            .ok_or_else(|| {
                eprintln!("No plugin provides box type: {}", box_type);
                BidError::InvalidType
            })?;
        
        // Get loaded plugin
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins.get(lib_name)
            .ok_or_else(|| {
                eprintln!("Plugin not loaded: {}", lib_name);
                BidError::PluginError
            })?;
        
        // Create v2 plugin box wrapper
        let plugin_box = PluginBoxV2 {
            box_type: box_type.to_string(),
            invoke_fn: plugin.invoke_fn,
            instance_id: 0, // Will be set after birth call
        };
        
        // TODO: Call birth constructor with args via TLV encoding
        
        Ok(Box::new(plugin_box))
    }
}

// Global loader instance
use once_cell::sync::Lazy;

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