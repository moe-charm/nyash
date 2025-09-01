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
    use crate::runtime::leak_tracker;
    fn dbg_on() -> bool { std::env::var("NYASH_DEBUG_PLUGIN").unwrap_or_default() == "1" }

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

// (moved: public constructor wrapper is declared after the enabled module)

/// v2 Plugin Box wrapper - temporary implementation
#[derive(Debug)]
    pub struct PluginHandleInner {
        pub type_id: u32,
        pub invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
        pub instance_id: u32,
        pub fini_method_id: Option<u32>,
        finalized: std::sync::atomic::AtomicBool,
    }

    // Loaded box spec from plugins/<name>/nyash_box.toml
    #[derive(Debug, Clone, Default)]
    struct LoadedBoxSpec {
        type_id: Option<u32>,
        methods: HashMap<String, MethodSpec>,
        fini_method_id: Option<u32>,
    }
    #[derive(Debug, Clone, Copy)]
    struct MethodSpec { method_id: u32, returns_result: bool }

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
                    leak_tracker::finalize_plugin("PluginBox", self.instance_id);
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

    /// Helper to construct a PluginBoxV2 from raw ids and invoke pointer safely
    pub fn make_plugin_box_v2_inner(box_type: String, type_id: u32, instance_id: u32, invoke_fn: unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize) -> i32) -> PluginBoxV2 {
        PluginBoxV2 { box_type, inner: std::sync::Arc::new(PluginHandleInner { type_id, invoke_fn, instance_id, fini_method_id: None, finalized: std::sync::atomic::AtomicBool::new(false) }) }
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
    fn is_identity(&self) -> bool { true }
    fn type_name(&self) -> &'static str {
        // Return the actual box type name for proper method dispatch
        match self.box_type.as_str() {
            "FileBox" => "FileBox",
            _ => "PluginBoxV2",
        }
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        if dbg_on() { eprintln!("🔍 DEBUG: PluginBoxV2::clone_box called for {} (id={})", self.box_type, self.inner.instance_id); }
        // Clone means creating a new instance by calling birth() on the plugin
        let mut output_buffer = vec![0u8; 1024];
        let mut output_len = output_buffer.len();
        let tlv_args = [1u8, 0, 0, 0]; // version=1, argc=0

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
            let new_instance_id = u32::from_le_bytes([
                output_buffer[0], output_buffer[1], output_buffer[2], output_buffer[3]
            ]);
            if dbg_on() { eprintln!("🎉 clone_box success: created new {} instance_id={}", self.box_type, new_instance_id); }
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
            if dbg_on() { eprintln!("❌ clone_box failed: birth() returned error code {}", result); }
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
        if dbg_on() { eprintln!("🔍 DEBUG: PluginBoxV2::share_box called for {} (id={})", self.box_type, self.inner.instance_id); }
        
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
        pub fn is_finalized(&self) -> bool { self.inner.finalized.load(std::sync::atomic::Ordering::SeqCst) }
    }

    /// Public helper to construct a PluginBoxV2 from raw parts (for VM/JIT integration)
    pub fn construct_plugin_box(
        box_type: String,
        type_id: u32,
        invoke_fn: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
        instance_id: u32,
        fini_method_id: Option<u32>,
    ) -> PluginBoxV2 {
        PluginBoxV2 {
            box_type,
            inner: std::sync::Arc::new(PluginHandleInner {
                type_id,
                invoke_fn,
                instance_id,
                fini_method_id,
                finalized: std::sync::atomic::AtomicBool::new(false),
            }),
        }
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
    /// Loaded per-plugin box specs from nyash_box.toml: (lib_name, box_type) -> spec
    box_specs: RwLock<HashMap<(String,String), LoadedBoxSpec>>,
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
            box_specs: RwLock::new(HashMap::new()),
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
        // Bump cache versions for all box types (config reload may change method layout)
        if let Some(cfg) = self.config.as_ref() {
            let mut labels: Vec<String> = Vec::new();
            for (_lib, def) in &cfg.libraries {
                for bt in &def.boxes {
                    labels.push(format!("BoxRef:{}", bt));
                }
            }
            crate::runtime::cache_versions::bump_many(&labels);
        }
        Ok(())
    }
    
    /// Load all plugins from config
    pub fn load_all_plugins(&self) -> BidResult<()> {
        let config = self.config.as_ref()
            .ok_or(BidError::PluginError)?;
        
        // Load legacy libraries (backward compatible)
        for (lib_name, lib_def) in &config.libraries {
            if let Err(e) = self.load_plugin(lib_name, lib_def) {
                eprintln!("Warning: Failed to load plugin {}: {:?}", lib_name, e);
            }
        }
        // Load new-style plugins from [plugins] map (name -> root dir)
        for (plugin_name, root) in &config.plugins {
            // Synthesize a LibraryDefinition from plugin spec (nyash_box.toml) if present; otherwise minimal
            let mut boxes: Vec<String> = Vec::new();
            let spec_path = std::path::Path::new(root).join("nyash_box.toml");
            // Optional artifact path from spec
            let mut artifact_override: Option<String> = None;
            if let Ok(txt) = std::fs::read_to_string(&spec_path) {
                if let Ok(val) = txt.parse::<toml::Value>() {
                    if let Some(prov) = val.get("provides").and_then(|t| t.get("boxes")).and_then(|a| a.as_array()) {
                        for it in prov.iter() { if let Some(s) = it.as_str() { boxes.push(s.to_string()); } }
                    }
                    // Artifacts section: choose OS-specific path template if provided
                    if let Some(arts) = val.get("artifacts").and_then(|t| t.as_table()) {
                        let key = if cfg!(target_os = "windows") { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" };
                        if let Some(p) = arts.get(key).and_then(|v| v.as_str()) {
                            artifact_override = Some(p.to_string());
                        }
                    }
                    // Build per-box specs
                    for bname in &boxes {
                        let mut spec = LoadedBoxSpec::default();
                        if let Some(tb) = val.get(bname) {
                            if let Some(tid) = tb.get("type_id").and_then(|v| v.as_integer()) { spec.type_id = Some(tid as u32); }
                            if let Some(lc) = tb.get("lifecycle") {
                                if let Some(fini) = lc.get("fini").and_then(|m| m.get("id")).and_then(|v| v.as_integer()) { spec.fini_method_id = Some(fini as u32); }
                            }
                            if let Some(mtbl) = tb.get("methods").and_then(|t| t.as_table()) {
                                for (mname, md) in mtbl.iter() {
                                    if let Some(mid) = md.get("id").and_then(|v| v.as_integer()) {
                                        let rr = md.get("returns_result").and_then(|v| v.as_bool()).unwrap_or(false);
                                        spec.methods.insert(mname.clone(), MethodSpec { method_id: mid as u32, returns_result: rr });
                                    }
                                }
                            }
                        }
                        if spec.type_id.is_some() || !spec.methods.is_empty() {
                            self.box_specs.write().unwrap().insert((plugin_name.clone(), bname.clone()), spec);
                        }
                    }
                }
            }
            // Path heuristic: use artifact override if present, else "<root>/<plugin_name>"
            let synth_path = if let Some(p) = artifact_override {
                let jp = std::path::Path::new(root).join(p);
                jp.to_string_lossy().to_string()
            } else {
                std::path::Path::new(root).join(plugin_name).to_string_lossy().to_string()
            };
            let lib_def = LibraryDefinition { boxes: boxes.clone(), path: synth_path };
            if let Err(e) = self.load_plugin(plugin_name, &lib_def) {
                eprintln!("Warning: Failed to load plugin {} from [plugins]: {:?}", plugin_name, e);
            }
        }
        // Strict validation: central [box_types] vs plugin spec type_id
        let strict = std::env::var("NYASH_PLUGIN_STRICT").ok().as_deref() == Some("1");
        if !config.box_types.is_empty() {
            for ((lib, bname), spec) in self.box_specs.read().unwrap().iter() {
                if let Some(cid) = config.box_types.get(bname) {
                    if let Some(tid) = spec.type_id {
                        if tid != *cid {
                            eprintln!("[PluginLoaderV2] type_id mismatch for {} (plugin={}): central={}, spec={}", bname, lib, cid, tid);
                            if strict { return Err(BidError::PluginError); }
                        }
                    }
                }
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

    fn find_lib_name_for_box(&self, box_type: &str) -> Option<String> {
        if let Some(cfg) = &self.config {
            if let Some((name, _)) = cfg.find_library_for_box(box_type) { return Some(name.to_string()); }
        }
        for ((lib, b), _) in self.box_specs.read().unwrap().iter() {
            if b == box_type { return Some(lib.clone()); }
        }
        None
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
        // Prefer spec-loaded type_id
        let type_id = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) {
            spec.type_id.unwrap_or_else(|| config.box_types.get(box_type).copied().unwrap_or(0))
        } else {
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            box_conf.type_id
        };
        // Call birth
        let mut output_buffer = vec![0u8; 1024];
        let mut output_len = output_buffer.len();
        let tlv_args = crate::runtime::plugin_ffi_common::encode_empty_args();
        let birth_result = unsafe {
            (plugin.invoke_fn)(type_id, 0, 0, tlv_args.as_ptr(), tlv_args.len(), output_buffer.as_mut_ptr(), &mut output_len)
        };
        if birth_result != 0 || output_len < 4 { return Err(BidError::PluginError); }
        let instance_id = u32::from_le_bytes([output_buffer[0], output_buffer[1], output_buffer[2], output_buffer[3]]);
        let fini_id = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) {
            spec.fini_method_id
        } else {
            let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            box_conf.methods.get("fini").map(|m| m.method_id)
        };
        let handle = std::sync::Arc::new(PluginHandleInner {
            type_id,
            invoke_fn: plugin.invoke_fn,
            instance_id,
            fini_method_id: fini_id,
            finalized: std::sync::atomic::AtomicBool::new(false),
        });
        self.singletons.write().unwrap().insert((lib_name.to_string(), box_type.to_string()), handle);
        // bump version for this box type to invalidate caches
        crate::runtime::cache_versions::bump_version(&format!("BoxRef:{}", box_type));
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
            ("env.debug", "trace") => {
                // Minimal debug trace; prints to stderr when enabled
                if std::env::var("NYASH_DEBUG_TRACE").ok().as_deref() == Some("1") {
                    for a in args { eprintln!("[debug.trace] {}", a.to_string_box().value); }
                }
                Ok(None)
            }
            ("env.runtime", "checkpoint") => {
                // Safepoint + scheduler poll via global hooks
                if std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[runtime.checkpoint] reached");
                }
                crate::runtime::global_hooks::safepoint_and_poll();
                Ok(None)
            }
            // Future/Await bridge (scaffold): maps MIR Future* to Box operations
            ("env.future", "new") | ("env.future", "birth") => {
                // new(value) -> FutureBox(set to value)
                let fut = crate::boxes::future::FutureBox::new();
                if let Some(v) = args.get(0) { fut.set_result(v.clone_box()); }
                Ok(Some(Box::new(fut)))
            }
            ("env.future", "set") => {
                // set(future, value)
                if args.len() >= 2 {
                    if let Some(fut) = args[0].as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
                        fut.set_result(args[1].clone_box());
                    }
                }
                Ok(None)
            }
            ("env.future", "await") => {
                // await(future) -> value (pass-through if not a FutureBox)
                if let Some(arg) = args.get(0) {
                    if let Some(fut) = arg.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
                        match fut.wait_and_get() { Ok(v) => return Ok(Some(v)), Err(e) => {
                            eprintln!("[env.future.await] error: {}", e);
                            return Ok(None);
                        } }
                    } else {
                        return Ok(Some(arg.clone_box()));
                    }
                }
                Ok(None)
            }
            ("env.future", "spawn_instance") => {
                // spawn_instance(recv, method_name, args...) -> FutureBox
                // If a scheduler is available, schedule the call; else invoke synchronously.
                use crate::box_trait::{NyashBox, VoidBox, ErrorBox};
                let fut = crate::boxes::future::FutureBox::new();
                if args.len() >= 2 {
                    if let Some(pb) = args[0].as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        let method_name = if let Some(sb) = args[1].as_any().downcast_ref::<crate::box_trait::StringBox>() { sb.value.clone() } else { args[1].to_string_box().value };
                        // Clone boxes for scheduled call (same-thread scheduler; no Send bound)
                        let tail: Vec<Box<dyn NyashBox>> = args.iter().skip(2).map(|a| a.clone_box()).collect();
                        let recv_type = pb.box_type.clone();
                        let recv_id = pb.instance_id();
                        // Clones for fallback inline path
                        let recv_type_inline = recv_type.clone();
                        let method_name_inline = method_name.clone();
                        let tail_inline: Vec<Box<dyn NyashBox>> = tail.iter().map(|a| a.clone_box()).collect();
                        let fut_setter = fut.clone();
                        let scheduled = crate::runtime::global_hooks::spawn_task("spawn_instance", Box::new(move || {
                            let host = crate::runtime::get_global_plugin_host();
                            let read_res = host.read();
                            if let Ok(ro) = read_res {
                                match ro.invoke_instance_method(&recv_type, &method_name, recv_id, &tail) {
                                    Ok(ret) => { if let Some(v) = ret { fut_setter.set_result(v); } else { fut_setter.set_result(Box::new(VoidBox::new())); } }
                                    Err(e) => { fut_setter.set_result(Box::new(ErrorBox::new("InvokeError", &format!("{}.{}: {:?}", recv_type, method_name, e)))); }
                                }
                            }
                        }));
                        if !scheduled {
                            // Fallback: run inline synchronously
                            let host = crate::runtime::get_global_plugin_host();
                            let read_res = host.read();
                            if let Ok(ro) = read_res {
                                match ro.invoke_instance_method(&recv_type_inline, &method_name_inline, recv_id, &tail_inline) {
                                    Ok(ret) => { if let Some(v) = ret { fut.set_result(v); } else { fut.set_result(Box::new(VoidBox::new())); } }
                                    Err(e) => { fut.set_result(Box::new(ErrorBox::new("InvokeError", &format!("{}.{}: {:?}", recv_type_inline, method_name_inline, e)))); }
                                }
                            }
                        }
                        return Ok(Some(Box::new(fut)));
                    }
                }
                // Fallback: resolved future of first arg
                if let Some(v) = args.get(0) { fut.set_result(v.clone_box()); }
                Ok(Some(Box::new(fut)))
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
            let lib_name = self.find_lib_name_for_box(box_type).ok_or(BidError::InvalidType)?;
            // Prefer spec-loaded methods
            if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.clone(), box_type.to_string())) {
                if let Some(m) = spec.methods.get(method_name) { return Ok(m.method_id); }
            }
            // Fallback to central nyash.toml nested box config
            let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
            let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            let box_conf = config.get_box_config(&lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
            let method = box_conf.methods.get(method_name).ok_or_else(|| {
                eprintln!("[PluginLoaderV2] Method '{}' not found for box '{}' in {}", method_name, box_type, cfg_path);
                eprintln!("[PluginLoaderV2] Available methods: {:?}", box_conf.methods.keys().collect::<Vec<_>>());
                BidError::InvalidMethod
            })?;
            Ok(method.method_id)
        }

        /// Determine whether a method returns a Result (Ok/Err) wrapper.
        pub fn method_returns_result(&self, box_type: &str, method_name: &str) -> bool {
            let config = match &self.config { Some(c) => c, None => return false };
            let lib_name = match self.find_lib_name_for_box(box_type) { Some(n) => n, None => return false };
            // Prefer spec if present
            if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.clone(), box_type.to_string())) {
                if let Some(m) = spec.methods.get(method_name) { return m.returns_result; }
            }
            // Fallback to central nyash.toml
            let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
            if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(bc) = config.get_box_config(&lib_name, box_type, &toml_value) {
                        return bc.methods.get(method_name).map(|m| m.returns_result).unwrap_or(false);
                    }
                }
            }
            false
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
            let lib_name = self.find_lib_name_for_box(box_type).ok_or(BidError::InvalidType)?;
            let plugins = self.plugins.read().unwrap();
            let plugin = plugins.get(&lib_name).ok_or(BidError::PluginError)?;
            let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
            let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
            let toml_value: toml::Value = toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
            // Prefer spec values; but if method isn't listed in spec, fallback to central nyash.toml for returns_result
            let (type_id, returns_result) = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.clone(), box_type.to_string())) {
                let tid = spec
                    .type_id
                    .or_else(|| config.box_types.get(box_type).copied())
                    .ok_or(BidError::InvalidType)?;
                let rr = if let Some(m) = spec.methods.get(method_name) {
                    m.returns_result
                } else {
                    // Fallback to central config for method flags
                    if let Some(box_conf) = config.get_box_config(&lib_name, box_type, &toml_value) {
                        box_conf.methods.get(method_name).map(|m| m.returns_result).unwrap_or(false)
                    } else { false }
                };
                (tid, rr)
            } else {
                let box_conf = config.get_box_config(&lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
                let rr = box_conf.methods.get(method_name).map(|m| m.returns_result).unwrap_or(false);
                (box_conf.type_id, rr)
            };
            eprintln!("[PluginLoaderV2] Invoke {}.{}: resolving and encoding args (argc={})", box_type, method_name, args.len());
            // TLV args: encode using BID-1 style (u16 ver, u16 argc, then entries)
            let tlv_args = {
                let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(args.len() as u16);
                // Validate against nyash.toml method args schema if present
                let expected_args = if self.box_specs.read().unwrap().get(&(lib_name.clone(), box_type.to_string())).is_some() {
                    None
                } else {
                    config
                        .get_box_config(&lib_name, box_type, &toml_value)
                        .and_then(|bc| bc.methods.get(method_name).and_then(|m| m.args.clone()))
                };
                if let Some(exp) = expected_args.as_ref() {
                    if exp.len() != args.len() {
                        eprintln!(
                            "[PluginLoaderV2] InvalidArgs: {}.{} expects {} args, got {} (schema={:?})",
                            box_type,
                            method_name,
                            exp.len(),
                            args.len(),
                            exp.iter().map(|a| a.kind_str()).collect::<Vec<_>>()
                        );
                        return Err(BidError::InvalidArgs);
                    }
                }

                for (idx, a) in args.iter().enumerate() {
                    // Coerce commonly-used plugin boxed primitives to builtin primitives when schema is weak (Name)
                    // Example: Plugin IntegerBox passed to ArrayBox.push(value) should encode as I64
                    let mut enc_ref: &Box<dyn NyashBox> = a;
                    let mut enc_owned: Option<Box<dyn NyashBox>> = None;
                    if let Some(p) = a.as_any().downcast_ref::<PluginBoxV2>() {
                        if p.box_type == "IntegerBox" {
                            // Read integer value via get(); on success, encode that value instead of a handle
                            if let Ok(val_opt) = self.invoke_instance_method("IntegerBox", "get", p.inner.instance_id, &[]) {
                                if let Some(val_box) = val_opt { enc_owned = Some(val_box); enc_ref = enc_owned.as_ref().unwrap(); }
                            }
                        } else if p.box_type == "StringBox" {
                            // Prefer TLV string for expected string params
                            let need_string = if let Some(exp) = expected_args.as_ref() {
                                matches!(exp.get(idx), Some(crate::config::nyash_toml_v2::ArgDecl::Typed { kind, .. }) if kind == "string")
                            } else {
                                // Name fallback: allow coercion to string
                                true
                            };
                            if need_string {
                                if let Ok(val_opt) = self.invoke_instance_method("StringBox", "toUtf8", p.inner.instance_id, &[]) {
                                    if let Some(val_box) = val_opt { enc_owned = Some(val_box); enc_ref = enc_owned.as_ref().unwrap(); }
                                }
                            }
                        }
                        // Future: BoolBox/F64 coercions
                    }
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
                                        if enc_ref.as_any().downcast_ref::<PluginBoxV2>().is_none() {
                                            return Err(BidError::InvalidArgs);
                                        }
                                    }
                                    "string" => {
                                        if enc_ref.as_any().downcast_ref::<StringBox>().is_none() {
                                            // Attempt late coercion for plugin StringBox
                                            if let Some(p) = enc_ref.as_any().downcast_ref::<PluginBoxV2>() {
                                                if p.box_type == "StringBox" {
                                                    if let Ok(val_opt) = self.invoke_instance_method("StringBox", "toUtf8", p.inner.instance_id, &[]) {
                                                        if let Some(val_box) = val_opt { enc_owned = Some(val_box); enc_ref = enc_owned.as_ref().unwrap(); }
                                                    }
                                                }
                                            }
                                        }
                                        if enc_ref.as_any().downcast_ref::<StringBox>().is_none() { return Err(BidError::InvalidArgs); }
                                    }
                                    "int" | "i32" => {
                                        if enc_ref.as_any().downcast_ref::<IntegerBox>().is_none() {
                                            return Err(BidError::InvalidArgs);
                                        }
                                    }
                                    _ => {
                                        eprintln!("[PluginLoaderV2] InvalidArgs: unsupported kind '{}' for {}.{} arg[{}]",
                                            kind, box_type, method_name, idx);
                                        return Err(BidError::InvalidArgs);
                                    }
                                }
                            }
                            crate::config::nyash_toml_v2::ArgDecl::Name(_) => {
                                // Back-compat: allow common primitives (string or int)
                                let is_string = enc_ref.as_any().downcast_ref::<StringBox>().is_some();
                                let is_int = enc_ref.as_any().downcast_ref::<IntegerBox>().is_some();
                                if !(is_string || is_int) {
                                    eprintln!("[PluginLoaderV2] InvalidArgs: expected string/int for {}.{} arg[{}]",
                                        box_type, method_name, idx);
                                    return Err(BidError::InvalidArgs);
                                }
                            }
                        }
                    }

                    // Plugin Handle (BoxRef): tag=8, size=8
                    if let Some(p) = enc_ref.as_any().downcast_ref::<PluginBoxV2>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: PluginBoxV2({}, id={}) -> Handle(tag=8)", idx, p.box_type, p.inner.instance_id);
                        crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.inner.instance_id);
                        continue;
                    }
                    // Integer: use I64 (tag=3) for broad plugin compatibility
                    if let Some(i) = enc_ref.as_any().downcast_ref::<IntegerBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: Integer({}) -> I64(tag=3)", idx, i.value);
                        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value);
                        continue;
                    }
                    // Bool
                    if let Some(b) = enc_ref.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: Bool({}) -> Bool(tag=1)", idx, b.value);
                        crate::runtime::plugin_ffi_common::encode::bool(&mut buf, b.value);
                        continue;
                    }
                    // Float (F64)
                    if let Some(f) = enc_ref.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: Float({}) -> F64(tag=5)", idx, f.value);
                        crate::runtime::plugin_ffi_common::encode::f64(&mut buf, f.value);
                        continue;
                    }
                    // Bytes from Array<uint8>
                    if let Some(arr) = enc_ref.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        let items = arr.items.read().unwrap();
                        let mut tmp = Vec::with_capacity(items.len());
                        let mut ok = true;
                        for item in items.iter() {
                            if let Some(intb) = item.as_any().downcast_ref::<IntegerBox>() {
                                if intb.value >= 0 && intb.value <= 255 { tmp.push(intb.value as u8); } else { ok = false; break; }
                            } else { ok = false; break; }
                        }
                        if ok {
                            eprintln!("[PluginLoaderV2]  arg[{}]: Array<uint8>[{}] -> Bytes(tag=7)", idx, tmp.len());
                            crate::runtime::plugin_ffi_common::encode::bytes(&mut buf, &tmp);
                            continue;
                        }
                    }
                    // String: tag=6
                    if let Some(s) = enc_ref.as_any().downcast_ref::<StringBox>() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: String(len={}) -> String(tag=6)", idx, s.value.len());
                        crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value);
                        continue;
                    }
                    // No schema or unsupported type: only allow fallback when schema is None
                    if expected_args.is_none() {
                        eprintln!("[PluginLoaderV2]  arg[{}]: fallback stringify", idx);
                        let sv = enc_ref.to_string_box().value;
                        crate::runtime::plugin_ffi_common::encode::string(&mut buf, &sv);
                    } else {
                        return Err(BidError::InvalidArgs);
                    }
                }
                buf
            };
            eprintln!("[VM→Plugin] call {}.{} recv_id={} returns_result={}", box_type, method_name, instance_id, returns_result);
            if dbg_on() {
                // Dump compact TLV header and first few bytes for diagnostics
                let hdr_ver = u16::from_le_bytes([tlv_args[0], tlv_args[1]]);
                let hdr_argc = u16::from_le_bytes([tlv_args[2], tlv_args[3]]);
                let preview_len = std::cmp::min(tlv_args.len(), 64);
                let preview: Vec<String> = tlv_args[..preview_len].iter().map(|b| format!("{:02X}", b)).collect();
                eprintln!("[VM→Plugin] TLV ver={} argc={} bytes={} preview={}...",
                    hdr_ver, hdr_argc, tlv_args.len(), preview.join(" "));
            }
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
                if dbg_on() { eprintln!("[PluginLoaderV2] invoke rc={} ({}) for {}.{}", rc, be.message(), box_type, method_name); }
                if returns_result {
                    let err = crate::exception_box::ErrorBox::new(&format!("{} (code: {})", be.message(), rc));
                    return Ok(Some(Box::new(crate::boxes::result::NyashResultBox::new_err(Box::new(err)))));
                }
                return Err(be);
            }
            // Decode: BID-1 style header + first entry
            let result = if out_len == 0 {
                if returns_result {
                    Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(Box::new(crate::box_trait::VoidBox::new()))) as Box<dyn NyashBox>)
                } else { None }
            } else {
                let data = &out[..out_len];
                if data.len() < 4 { return Ok(None); }
                let _ver = u16::from_le_bytes([data[0], data[1]]);
                let argc = u16::from_le_bytes([data[2], data[3]]);
                if argc == 0 {
                    if returns_result {
                        return Ok(Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(Box::new(crate::box_trait::VoidBox::new())))));
                    } else {
                        return Ok(None);
                    }
                }
                if let Some((tag, size, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(data) {
                if dbg_on() {
                    let preview_len = std::cmp::min(payload.len(), 16);
                    let preview: Vec<String> = payload[..preview_len].iter().map(|b| format!("{:02X}", b)).collect();
                    eprintln!("[Plugin→VM] tag={} size={} preview={} returns_result={} for {}.{}",
                        tag, size, preview.join(" "), returns_result, box_type, method_name);
                }
                match tag {
                    1 if size == 1 => { // Bool
                        let b = crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false);
                        let val: Box<dyn NyashBox> = Box::new(crate::box_trait::BoolBox::new(b));
                        if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val)) as Box<dyn NyashBox>) } else { Some(val) }
                    }
                    5 if size == 8 => { // F64
                        if let Some(f) = crate::runtime::plugin_ffi_common::decode::f64(payload) {
                            let val: Box<dyn NyashBox> = Box::new(crate::boxes::math_box::FloatBox::new(f));
                            if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val)) as Box<dyn NyashBox>) } else { Some(val) }
                        } else { None }
                    }
                    8 if size == 8 => { // Handle -> PluginBoxV2
                        let mut t = [0u8;4]; t.copy_from_slice(&payload[0..4]);
                        let mut i = [0u8;4]; i.copy_from_slice(&payload[4..8]);
                        let r_type = u32::from_le_bytes(t);
                        let r_inst = u32::from_le_bytes(i);
                        if dbg_on() { eprintln!("[Plugin→VM] return handle type_id={} inst={} (returns_result={})", r_type, r_inst, returns_result); }
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
                                    let val: Box<dyn NyashBox> = Box::new(pbox);
                                    if returns_result {
                                        return Ok(Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val))));
                                    } else {
                                        return Ok(Some(val));
                                    }
                                }
                            }
                        }
                        None
                    }
                    2 if size == 4 => { // I32
                        let n = crate::runtime::plugin_ffi_common::decode::i32(payload).unwrap();
                        let val: Box<dyn NyashBox> = Box::new(IntegerBox::new(n as i64));
                        if dbg_on() { eprintln!("[Plugin→VM] return i32 value={} (returns_result={})", n, returns_result); }
                        if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val)) as Box<dyn NyashBox>) } else { Some(val) }
                    }
                    6 | 7 => { // String/Bytes
                        let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                        if dbg_on() { eprintln!("[Plugin→VM] return str/bytes len={} (returns_result={})", size, returns_result); }
                        // Treat as Ok payload; Err is indicated by non-zero rc earlier
                        let val: Box<dyn NyashBox> = Box::new(StringBox::new(s));
                        if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val)) as Box<dyn NyashBox>) } else { Some(val) }
                    }
                    3 if size == 8 => { // I64
                        // Try decoding as i64 directly; also support legacy i32 payload size 4 when mis-encoded
                        let n = if payload.len() == 8 {
                            let mut b = [0u8;8]; b.copy_from_slice(&payload[0..8]); i64::from_le_bytes(b)
                        } else { 0 }
                            ;
                        let val: Box<dyn NyashBox> = Box::new(IntegerBox::new(n));
                        if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(val)) as Box<dyn NyashBox>) } else { Some(val) }
                    }
                    9 => {
                        if dbg_on() { eprintln!("[Plugin→VM] return void (returns_result={})", returns_result); }
                        if returns_result { Some(Box::new(crate::boxes::result::NyashResultBox::new_ok(Box::new(crate::box_trait::VoidBox::new()))) as Box<dyn NyashBox>) } else { None }
                    },
                    _ => None,
                }} else { None }
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
        
        // Resolve platform-specific path (.so/.dll/.dylib) and optional search paths
        let resolved_path = self.resolve_library_path(&lib_def.path)
            .or_else(|| self.config.as_ref().and_then(|c| c.resolve_plugin_path(&lib_def.path)))
            .unwrap_or_else(|| lib_def.path.clone());
        // Load library
        let lib = unsafe {
            libloading::Library::new(&resolved_path)
                .map_err(|e| {
                    eprintln!(
                        "Failed to load library '{}': {}\n  raw='{}'",
                        resolved_path, e, lib_def.path
                    );
                    BidError::PluginError
                })?
        };
        
        // Optional ABI version check (v0: expect 1)
        if let Ok(sym) = unsafe { lib.get::<unsafe extern "C" fn() -> u32>(b"nyash_plugin_abi") } {
            let ver = unsafe { (*sym)() };
            if ver != 1 {
                eprintln!("[PluginLoaderV2] nyash_plugin_abi version mismatch: {} (expected 1) for {}", ver, lib_name);
                return Err(BidError::PluginError);
            }
        } else {
            eprintln!("[PluginLoaderV2] nyash_plugin_abi not found for {} (assuming v0=1)", lib_name);
        }

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
            eprintln!("[PluginLoaderV2] nyash_plugin_init rc={} for {}", result, lib_name);
            if result != 0 {
                eprintln!("Plugin init failed with code: {}", result);
                return Err(BidError::PluginError);
            }
        } else {
            eprintln!("[PluginLoaderV2] nyash_plugin_init not found for {} (optional)", lib_name);
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

    /// Resolve a plugin library path for the current OS by adapting extensions and common prefixes.
    /// - Maps .so/.dylib/.dll according to target OS
    /// - On Windows also tries removing leading 'lib' prefix
    /// - Searches configured plugin_paths if only filename is provided
    fn resolve_library_path(&self, raw: &str) -> Option<String> {
        use std::path::{Path, PathBuf};
        let p = Path::new(raw);
        if p.exists() { return Some(raw.to_string()); }
        let dir = p.parent().map(|d| d.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
        let file = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| raw.to_string());
        let stem = Path::new(&file).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or(file.clone());
        let cur_ext: &str = if cfg!(target_os = "windows") { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };

        // Candidate A: replace extension with OS-specific
        let mut cand_a = dir.join(Path::new(&stem).with_extension(cur_ext));
        if cand_a.exists() { return Some(cand_a.to_string_lossy().to_string()); }

        // Candidate B (Windows): drop 'lib' prefix if present
        if cfg!(target_os = "windows") {
            if let Some(stripped) = stem.strip_prefix("lib") {
                let cand_b = dir.join(Path::new(stripped).with_extension(cur_ext));
                if cand_b.exists() { return Some(cand_b.to_string_lossy().to_string()); }
            }
        }

        // Candidate C: search in configured plugin_paths with adapted filename
        if let Some(cfg) = &self.config {
            // try original filename
            if let Some(path) = cfg.resolve_plugin_path(&file) { return Some(path); }
            // try adapted A
            if let Some(path) = cfg.resolve_plugin_path(&cand_a.file_name().unwrap_or_default().to_string_lossy()) { return Some(path); }
            // try adapted B on windows
            if cfg!(target_os = "windows") {
                if let Some(stripped) = stem.strip_prefix("lib") {
                    let name = format!("{}.{}", stripped, cur_ext);
                    if let Some(path) = cfg.resolve_plugin_path(&name) { return Some(path); }
                }
            }
        }
        None
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
        
        // Resolve type_id/fini: prefer per-plugin spec (nyash_box.toml), fallback to central nyash.toml
        let (type_id, fini_method_id) = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) {
            // Prefer explicit spec values; if missing, fallback to central [box_types] and no fini
            let tid = spec
                .type_id
                .or_else(|| config.box_types.get(box_type).copied())
                .ok_or_else(|| {
                    eprintln!(
                        "No type_id found for {} (plugin spec missing and central [box_types] not set)",
                        box_type
                    );
                    BidError::InvalidType
                })?;
            (tid, spec.fini_method_id)
        } else {
            eprintln!("🔍 Reading nyash.toml for type configuration...");
            if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
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
            }
        };
        
        // Call birth constructor (method_id = 0) via TLV encoding
        eprintln!("🔍 Preparing to call birth() with type_id: {}", type_id);
        let mut output_buffer = vec![0u8; 1024]; // 1KB buffer for output
        let mut output_len = output_buffer.len();
        
        // Encode constructor arguments (best-effort). Birth schemas are optional; we support common primitives.
        let tlv_args = {
            let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(_args.len() as u16);
            for (idx, a) in _args.iter().enumerate() {
                // Coerce plugin integer/string primitive boxes to builtins first
                let mut enc_ref: &Box<dyn NyashBox> = a;
                let mut enc_owned: Option<Box<dyn NyashBox>> = None;
                if let Some(p) = a.as_any().downcast_ref::<PluginBoxV2>() {
                    if p.box_type == "IntegerBox" {
                        if let Ok(val_opt) = self.invoke_instance_method("IntegerBox", "get", p.inner.instance_id, &[]) {
                            if let Some(val_box) = val_opt { enc_owned = Some(val_box); enc_ref = enc_owned.as_ref().unwrap(); }
                        }
                    } else if p.box_type == "StringBox" {
                        if let Ok(val_opt) = self.invoke_instance_method("StringBox", "toUtf8", p.inner.instance_id, &[]) {
                            if let Some(val_box) = val_opt { enc_owned = Some(val_box); enc_ref = enc_owned.as_ref().unwrap(); }
                        }
                    }
                }
                if let Some(i) = enc_ref.as_any().downcast_ref::<IntegerBox>() {
                    crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value);
                    continue;
                }
                if let Some(s) = enc_ref.as_any().downcast_ref::<StringBox>() {
                    crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value);
                    continue;
                }
                if let Some(b) = enc_ref.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                    crate::runtime::plugin_ffi_common::encode::bool(&mut buf, b.value);
                    continue;
                }
                if let Some(f) = enc_ref.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
                    crate::runtime::plugin_ffi_common::encode::f64(&mut buf, f.value);
                    continue;
                }
                // Fallback: stringify
                let sv = enc_ref.to_string_box().value;
                crate::runtime::plugin_ffi_common::encode::string(&mut buf, &sv);
            }
            buf
        };
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
        // Post-birth default initialization for known types
        // StringBox: ensure empty content to avoid plugin-side uninitialized access when length() is called immediately
        if box_type == "StringBox" {
            let cfg_path = self.config_path.as_ref().map(|s| s.as_str()).unwrap_or("nyash.toml");
            if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                    if let Some(box_conf) = self.config.as_ref().and_then(|c| c.get_box_config(lib_name, box_type, &toml_value)) {
                        if let Some(from_utf8) = box_conf.methods.get("fromUtf8") {
                            // TLV: header argc=1, then Bytes(tag=7, len=0)
                            let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(1);
                            crate::runtime::plugin_ffi_common::encode::bytes(&mut tlv, &[]);
                            let mut out = [0u8; 8];
                            let mut out_len: usize = out.len();
                            let _ = unsafe { (plugin.invoke_fn)(type_id, from_utf8.method_id, instance_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len) };
                        }
                    }
                }
            }
        }
        leak_tracker::init();
        leak_tracker::register_plugin(&plugin_box.box_type, instance_id);
        
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

// Public constructor wrapper for PluginBoxV2 (enabled build)
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub use self::enabled::make_plugin_box_v2_inner as make_plugin_box_v2;

#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
mod stub {
    use crate::bid::{BidResult, BidError};
    use crate::box_trait::NyashBox;
    use once_cell::sync::Lazy;
    use std::sync::{Arc, RwLock};

    // Stub implementation of PluginBoxV2 for WASM/non-plugin builds
    #[derive(Debug, Clone)]
    pub struct PluginBoxV2 {
        pub box_type: String,
        pub inner: std::sync::Arc<PluginHandleInner>,
    }

    #[derive(Debug)]
    pub struct PluginHandleInner {
        pub type_id: u32,
        pub instance_id: u32,
        pub fini_method_id: Option<u32>,
    }

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
