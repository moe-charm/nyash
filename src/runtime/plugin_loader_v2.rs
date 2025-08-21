//! Nyash v2 Plugin Loader
//!
//! cfg/features で2パスを提供:
//! - enabled: plugins feature 有効 かつ 非wasm32 ターゲット
//! - stub   : それ以外（WASMやplugins無効）

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
mod enabled {
    use crate::bid::{BidResult, BidError};
    use crate::box_trait::{NyashBox, BoxCore, StringBox, IntegerBox};
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
    #[allow(dead_code)]
    box_types: Vec<String>,
    
    /// Optional init function
    #[allow(dead_code)]
    init_fn: Option<unsafe extern "C" fn() -> i32>,
    
    /// Required invoke function  
    invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
}

/// v2 Plugin Box wrapper - temporary implementation
#[derive(Debug)]
    pub struct PluginHandleInner {
        pub type_id: u32,
        pub invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
        pub instance_id: u32,
        pub fini_method_id: Option<u32>,
        finalized: std::sync::atomic::AtomicBool,
    }

    impl Drop for PluginHandleInner {
        fn drop(&mut self) {
            // Finalize exactly once when the last shared handle is dropped
            if let Some(fini_id) = self.fini_method_id {
                if !self.finalized.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    let tlv_args: [u8; 4] = [1, 0, 0, 0];
                    let mut out: [u8; 4] = [0; 4];
                    let mut out_len: usize = out.len();
                    unsafe {
                        (self.invoke_fn)(
                            self.type_id,
                            fini_id,
                            self.instance_id,
                            tlv_args.as_ptr(),
                            tlv_args.len(),
                            out.as_mut_ptr(),
                            &mut out_len,
                        );
                    }
                }
            }
        }
    }

    impl PluginHandleInner {
        /// Explicitly finalize this handle now (idempotent)
        pub fn finalize_now(&self) {
            if let Some(fini_id) = self.fini_method_id {
                if !self.finalized.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    let tlv_args: [u8; 4] = [1, 0, 0, 0];
                    let mut out: [u8; 4] = [0; 4];
                    let mut out_len: usize = out.len();
                    unsafe {
                        (self.invoke_fn)(
                            self.type_id,
                            fini_id,
                            self.instance_id,
                            tlv_args.as_ptr(),
                            tlv_args.len(),
                            out.as_mut_ptr(),
                            &mut out_len,
                        );
                    }
                }
            }
        }
    }

#[derive(Debug, Clone)]
    pub struct PluginBoxV2 {
        pub box_type: String,
        pub inner: std::sync::Arc<PluginHandleInner>,
    }

    impl BoxCore for PluginBoxV2 {
    fn box_id(&self) -> u64 {
        self.inner.instance_id as u64
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.box_type, self.inner.instance_id)
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
        eprintln!("🔍 DEBUG: PluginBoxV2::clone_box called for {} (id={})", self.box_type, self.inner.instance_id);
        
        // Clone means creating a new instance by calling birth()
        let mut output_buffer = vec![0u8; 1024];
        let mut output_len = output_buffer.len();
        let tlv_args = vec![1u8, 0, 0, 0]; // version=1, argc=0
        
        let result = unsafe {
            (self.inner.invoke_fn)(
                self.inner.type_id,
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
            
            // Return new PluginBoxV2 with new instance_id (separate inner handle)
            Box::new(PluginBoxV2 {
                box_type: self.box_type.clone(),
                inner: std::sync::Arc::new(PluginHandleInner {
                    type_id: self.inner.type_id,
                    invoke_fn: self.inner.invoke_fn,
                    instance_id: new_instance_id,
                    fini_method_id: self.inner.fini_method_id,
                    finalized: std::sync::atomic::AtomicBool::new(false),
                }),
            })
        } else {
            eprintln!("❌ clone_box failed: birth() returned error code {}", result);
            // Fallback: return error message as StringBox
            Box::new(StringBox::new(format!("Clone failed for {}", self.box_type)))
        }
    }
    
    fn to_string_box(&self) -> crate::box_trait::StringBox {
        StringBox::new(format!("{}({})", self.box_type, self.inner.instance_id))
    }
    
    fn equals(&self, _other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        crate::box_trait::BoolBox::new(false)
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        eprintln!("🔍 DEBUG: PluginBoxV2::share_box called for {} (id={})", self.box_type, self.inner.instance_id);
        
        // Share means returning a new Box with the same instance_id
        Box::new(PluginBoxV2 {
            box_type: self.box_type.clone(),
            inner: self.inner.clone(),
        })
    }
}

impl PluginBoxV2 {
    pub fn instance_id(&self) -> u32 { self.inner.instance_id }
    pub fn finalize_now(&self) { self.inner.finalize_now() }
}

/// Plugin loader v2
    pub struct PluginLoaderV2 {
    /// Loaded plugins (library name -> plugin info)
    plugins: RwLock<HashMap<String, Arc<LoadedPluginV2>>>,
    
    /// Configuration
    pub config: Option<NyashConfigV2>,
    /// Path to the loaded nyash.toml (absolute), used for consistent re-reads
    config_path: Option<String>,

    /// Singleton instances: (lib_name, box_type) -> shared handle
    singletons: RwLock<HashMap<(String,String), std::sync::Arc<PluginHandleInner>>>,
}

    impl PluginLoaderV2 {
    fn find_box_by_type_id<'a>(&'a self, config: &'a NyashConfigV2, toml_value: &'a toml::Value, type_id: u32) -> Option<(&'a str, &'a str)> {
        for (lib_name, lib_def) in &config.libraries {
            for box_name in &lib_def.boxes {
                if let Some(box_conf) = config.get_box_config(lib_name, box_name, toml_value) {
                    if box_conf.type_id == type_id {
                        return Some((lib_name.as_str(), box_name.as_str()));
                    }
                }
            }
        }
        None
    }
    /// Create new loader
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            config: None,
            config_path: None,
            singletons: RwLock::new(HashMap::new()),
        }
    }
    
    /// Load configuration from nyash.toml
    pub fn load_config(&mut self, config_path: &str) -> BidResult<()> {
        // Canonicalize path for later re-reads
        let canonical = std::fs::canonicalize(config_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| config_path.to_string());
        self.config_path = Some(canonical.clone());

        self.config = Some(NyashConfigV2::from_file(&canonical)
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
        // Pre-birth singletons configured in nyash.toml
        let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
        let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
        let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
        for (lib_name, lib_def) in &config.libraries {
            for box_name in &lib_def.boxes {
                if let Some(bc) = config.get_box_config(lib_name, box_name, &toml_value) {
                    if bc.singleton {
                        let _ = self.ensure_singleton_handle(lib_name, box_name);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Ensure a singleton handle is created and stored
    fn ensure_singleton_handle(&self, lib_name: &str, box_type: &str) -> BidResult<()> {
        // Fast path: already present
        if self.singletons.read().unwrap().contains_key(&(lib_name.to_string(), box_type.to_string())) {
            return Ok(());
        }
        // Create via birth
        let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
        let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
        let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
        let config = self.config.as_ref().ok_or(BidError::PluginError)?;
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
        let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
        let type_id = box_conf.type_id;
        // Call birth
        let mut output_buffer = vec![0u8; 1024];
        let mut output_len = output_buffer.len();
        let tlv_args = vec![1u8, 0, 0, 0];
        let birth_result = unsafe {
            (plugin.invoke_fn)(type_id, 0, 0, tlv_args.as_ptr(), tlv_args.len(), output_buffer.as_mut_ptr(), &mut output_len)
        };
        if birth_result != 0 || output_len < 4 { return Err(BidError::PluginError); }
        let instance_id = u32::from_le_bytes([output_buffer[0], output_buffer[1], output_buffer[2], output_buffer[3]]);
        let fini_id = box_conf.methods.get("fini").map(|m| m.method_id);
        let handle = std::sync::Arc::new(PluginHandleInner {
            type_id,
            invoke_fn: plugin.invoke_fn,
            instance_id,
            fini_method_id: fini_id,
            finalized: std::sync::atomic::AtomicBool::new(false),
        });
        self.singletons.write().unwrap().insert((lib_name.to_string(), box_type.to_string()), handle);
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
            let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
            let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            let method = box_conf.methods.get(method_name).ok_or_else(|| {
                eprintln!("[PluginLoaderV2] Method '{}' not found for box '{}' in {}", method_name, box_type, cfg_path);
                eprintln!("[PluginLoaderV2] Available methods: {:?}", box_conf.methods.keys().collect::<Vec<_>>());
                BidError::InvalidMethod
            })?;
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
            // v2.1: 引数ありのメソッドを許可（BoxRef/基本型/文字列化フォールバック）
            let method_id = self.resolve_method_id_from_file(box_type, method_name)?;
            // Find plugin and type_id
            let config = self.config.as_ref().ok_or(BidError::PluginError)?;
            let (lib_name, _lib_def) = config.find_library_for_box(box_type).ok_or(BidError::InvalidType)?;
            let plugins = self.plugins.read().unwrap();
            let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
            let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
            let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            let type_id = box_conf.type_id;
            eprintln!("[PluginLoaderV2] Invoke {}.{}: resolving and encoding args (argc={})", box_type, method_name, args.len());
            // TLV args: encode using BID-1 style (u16 ver, u16 argc, then entries)
            let tlv_args = {
                let mut buf = Vec::with_capacity(4 + args.len() * 16);
                // Header: ver=1, argc=args.len()
                buf.extend_from_slice(&1u16.to_le_bytes());
                buf.extend_from_slice(&(args.len() as u16).to_le_bytes());
                // Validate against nyash.toml method args schema if present
                let expected_args = box_conf.methods.get(method_name).and_then(|m| m.args.clone());
                if let Some(exp) = expected_args.as_ref() {
                    if exp.len() != args.len() {
                        return Err(BidError::InvalidArgs);
                    }
                }

                for (idx, a) in args.iter().enumerate() {
                    // If schema exists, validate per expected kind
                    if let Some(exp) = expected_args.as_ref() {
                        let decl = &exp[idx];
                        match decl {
                            crate::config::nyash_toml_v2::ArgDecl::Typed { kind, category } => {
                                match kind.as_str() {
                                    "box" => {
                                        // Only plugin box supported for now
                                        if category.as_deref() != Some("plugin") {
                                            return Err(BidError::InvalidArgs);
                                        }
                                        if a.as_any().downcast_ref::<PluginBoxV2>().is_none() {
                                            return Err(BidError::InvalidArgs);
                                        }
                                    }
                                    "string" => {
                                        if a.as_any().downcast_ref::<StringBox>().is_none() {
                                            return Err(BidError::InvalidArgs);
                                        }
                                    }
                                    "int" | "i32" => {
                                        if a.as_any().downcast_ref::<IntegerBox>().is_none() {
                                            return Err(BidError::InvalidArgs);
                                        }
                                    }
                                    _ => {
                                        // Unsupported kind in this minimal implementation
                                        return Err(BidError::InvalidArgs);
                                    }
                                }
                            }
                            crate::config::nyash_toml_v2::ArgDecl::Name(_) => {
                                // Back-compat: allow common primitives (string or int)
                                let is_string = a.as_any().downcast_ref::<StringBox>().is_some();
                                let is_int = a.as_any().downcast_ref::<IntegerBox>().is_some();
                                if !(is_string || is_int) {
                                    return Err(BidError::InvalidArgs);
                                }
                            }
                        }
                    }

                    // Plugin Handle (BoxRef): tag=8, size=8
                    if let Some(p) = a.as_any().downcast_ref::<PluginBoxV2>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: PluginBoxV2({}, id={}) -> Handle(tag=8)", idx, p.box_type, p.inner.instance_id);
                        buf.push(8u8); // tag
                        buf.push(0u8); // reserved
                        buf.extend_from_slice(&(8u16).to_le_bytes());
                        buf.extend_from_slice(&p.inner.type_id.to_le_bytes());
                        buf.extend_from_slice(&p.inner.instance_id.to_le_bytes());
                        continue;
                    }
                    // Integer: prefer i32
                    if let Some(i) = a.as_any().downcast_ref::<IntegerBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: Integer({}) -> I32(tag=2)", idx, i.value);
                        buf.push(2u8); // tag=I32
                        buf.push(0u8);
                        buf.extend_from_slice(&(4u16).to_le_bytes());
                        let v = i.value as i32;
                        buf.extend_from_slice(&v.to_le_bytes());
                        continue;
                    }
                    // String: tag=6
                    if let Some(s) = a.as_any().downcast_ref::<StringBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: String(len={}) -> String(tag=6)", idx, s.value.len());
                        let bytes = s.value.as_bytes();
                        let len = std::cmp::min(bytes.len(), u16::MAX as usize);
                        buf.push(6u8);
                        buf.push(0u8);
                        buf.extend_from_slice(&((len as u16).to_le_bytes()));
                        buf.extend_from_slice(&bytes[..len]);
                        continue;
                    }
                    // No schema or unsupported type: only allow fallback when schema is None
                    if expected_args.is_none() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: fallback stringify", idx);
                        let sv = a.to_string_box().value;
                        let bytes = sv.as_bytes();
                        let len = std::cmp::min(bytes.len(), u16::MAX as usize);
                        buf.push(6u8);
                        buf.push(0u8);
                        buf.extend_from_slice(&((len as u16).to_le_bytes()));
                        buf.extend_from_slice(&bytes[..len]);
                    } else {
                        return Err(BidError::InvalidArgs);
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
                let be = BidError::from_raw(rc);
                eprintln!("[PluginLoaderV2] invoke rc={} ({}) for {}.{}", rc, be.message(), box_type, method_name);
                return Err(be);
            }
            // Decode: BID-1 style header + first entry
            let result = if out_len == 0 { None } else {
                let data = &out[..out_len];
                if data.len() < 4 { return Ok(None); }
                let _ver = u16::from_le_bytes([data[0], data[1]]);
                let argc = u16::from_le_bytes([data[2], data[3]]);
                if argc == 0 { return Ok(None); }
                if data.len() < 8 { return Ok(None); }
                let tag = data[4];
                let _rsv = data[5];
                let size = u16::from_le_bytes([data[6], data[7]]) as usize;
                if data.len() < 8 + size { return Ok(None); }
                let payload = &data[8..8+size];
                match tag {
                    8 if size == 8 => { // Handle -> PluginBoxV2
                        let mut t = [0u8;4]; t.copy_from_slice(&payload[0..4]);
                        let mut i = [0u8;4]; i.copy_from_slice(&payload[4..8]);
                        let r_type = u32::from_le_bytes(t);
                        let r_inst = u32::from_le_bytes(i);
                        // Map type_id -> (lib_name, box_name)
                        if let Some((ret_lib, ret_box)) = self.find_box_by_type_id(config, &toml_value, r_type) {
                            // Get plugin for ret_lib
                            let plugins = self.plugins.read().unwrap();
                            if let Some(ret_plugin) = plugins.get(ret_lib) {
                                // Need fini_method_id from config
                                if let Some(ret_conf) = config.get_box_config(ret_lib, ret_box, &toml_value) {
                                    let fini_id = ret_conf.methods.get("fini").map(|m| m.method_id);
                                    let pbox = PluginBoxV2 {
                                        box_type: ret_box.to_string(),
                                        inner: std::sync::Arc::new(PluginHandleInner {
                                            type_id: r_type,
                                            invoke_fn: ret_plugin.invoke_fn,
                                            instance_id: r_inst,
                                            fini_method_id: fini_id,
                                            finalized: std::sync::atomic::AtomicBool::new(false),
                                        }),
                                    };
                                    return Ok(Some(Box::new(pbox) as Box<dyn NyashBox>));
                                }
                            }
                        }
                        None
                    }
                    2 if size == 4 => { // I32
                        let mut b = [0u8;4]; b.copy_from_slice(payload);
                        Some(Box::new(IntegerBox::new(i32::from_le_bytes(b) as i64)) as Box<dyn NyashBox>)
                    }
                    6 | 7 => { // String/Bytes
                        let s = String::from_utf8_lossy(payload).to_string();
                        Some(Box::new(StringBox::new(s)) as Box<dyn NyashBox>)
                    }
                    9 => None, // Void
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
    pub fn create_box(&self, box_type: &str, _args: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
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
        
        // If singleton, return the pre-birthed shared handle
        let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
        if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                if let Some(bc) = config.get_box_config(lib_name, box_type, &toml_value) {
                    if bc.singleton {
                        // ensure created
                        let _ = self.ensure_singleton_handle(lib_name, box_type);
                        if let Some(inner) = self.singletons.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) {
                            let plugin_box = PluginBoxV2 { box_type: box_type.to_string(), inner: inner.clone() };
                            return Ok(Box::new(plugin_box));
                        }
                    }
                }
            }
        }
        
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
        let (type_id, fini_method_id) = if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
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
            inner: std::sync::Arc::new(PluginHandleInner {
                type_id,
                invoke_fn: plugin.invoke_fn,
                instance_id,
                fini_method_id,
                finalized: std::sync::atomic::AtomicBool::new(false),
            }),
        };
        
        Ok(Box::new(plugin_box))
    }

    /// Shutdown singletons: finalize and clear all singleton handles
    pub fn shutdown_singletons(&self) {
        let mut map = self.singletons.write().unwrap();
        for (_, handle) in map.drain() {
            handle.finalize_now();
        }
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

    /// Gracefully shutdown plugins (finalize singletons)
    pub fn shutdown_plugins_v2() -> BidResult<()> {
        let loader = get_global_loader_v2();
        let loader = loader.read().unwrap();
        loader.shutdown_singletons();
        Ok(())
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
    pub fn shutdown_plugins_v2() -> BidResult<()> { Ok(()) }
}

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub use enabled::*;
#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
pub use stub::*;
