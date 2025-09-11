use super::types::{PluginBoxV2, PluginHandleInner, NyashTypeBoxFfi, LoadedPluginV2};
use crate::bid::{BidResult, BidError};
use crate::box_trait::{NyashBox, BoxCore, StringBox, IntegerBox};
use crate::config::nyash_toml_v2::{NyashConfigV2, LibraryDefinition};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

fn dbg_on() -> bool { std::env::var("NYASH_DEBUG_PLUGIN").unwrap_or_default() == "1" }

#[derive(Debug, Clone, Default)]
struct LoadedBoxSpec {
    type_id: Option<u32>,
    methods: HashMap<String, MethodSpec>,
    fini_method_id: Option<u32>,
}
#[derive(Debug, Clone, Copy)]
struct MethodSpec { method_id: u32, returns_result: bool }

pub struct PluginLoaderV2 {
    pub(super) plugins: RwLock<HashMap<String, Arc<LoadedPluginV2>>>,
    pub config: Option<NyashConfigV2>,
    pub(super) config_path: Option<String>,
    pub(super) singletons: RwLock<HashMap<(String,String), Arc<PluginHandleInner>>>,
    pub(super) box_specs: RwLock<HashMap<(String,String), LoadedBoxSpec>>,
}

impl PluginLoaderV2 {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            config: None,
            config_path: None,
            singletons: RwLock::new(HashMap::new()),
            box_specs: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_config(&mut self, config_path: &str) -> BidResult<()> {
        let canonical = std::fs::canonicalize(config_path).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| config_path.to_string());
        self.config_path = Some(canonical.clone());
        self.config = Some(NyashConfigV2::from_file(&canonical).map_err(|_| BidError::PluginError)?);
        if let Some(cfg) = self.config.as_ref() {
            let mut labels: Vec<String> = Vec::new();
            for (_lib, def) in &cfg.libraries { for bt in &def.boxes { labels.push(format!("BoxRef:{}", bt)); } }
            crate::runtime::cache_versions::bump_many(&labels);
        }
        Ok(())
    }

    pub fn load_all_plugins(&self) -> BidResult<()> {
        let config = self.config.as_ref().ok_or(BidError::PluginError)?;
        for (lib_name, lib_def) in &config.libraries { let _ = self.load_plugin(lib_name, lib_def); }
        for (plugin_name, root) in &config.plugins { let _ = self.load_plugin_from_root(plugin_name, root); }
        self.prebirth_singletons()?;
        Ok(())
    }

    fn load_plugin(&self, lib_name: &str, lib_def: &LibraryDefinition) -> BidResult<()> {
        // Resolve platform-specific filename from configured base path
        let base = Path::new(&lib_def.path);
        let mut candidates: Vec<PathBuf> = Vec::new();
        if cfg!(target_os = "windows") {
            // Try exact + .dll, and without leading 'lib'
            candidates.push(base.with_extension("dll"));
            if let Some(file) = base.file_name().and_then(|s| s.to_str()) {
                if file.starts_with("lib") {
                    let mut alt = base.to_path_buf();
                    let alt_file = file.trim_start_matches("lib");
                    alt.set_file_name(alt_file);
                    candidates.push(alt.with_extension("dll"));
                }
            }
        } else if cfg!(target_os = "macos") {
            candidates.push(base.with_extension("dylib"));
        } else {
            candidates.push(base.with_extension("so"));
        }

        // Prefer existing path; otherwise try to resolve via plugin_paths.search_paths
        let mut lib_path = candidates.iter().find(|p| p.exists()).cloned();
        if lib_path.is_none() {
            if let Some(cfg) = &self.config {
                // Try each candidate filename against search paths
                for c in &candidates {
                    if let Some(fname) = c.file_name().and_then(|s| s.to_str()) {
                        if let Some(resolved) = cfg.resolve_plugin_path(fname) {
                            let pb = PathBuf::from(resolved);
                            if pb.exists() { lib_path = Some(pb); break; }
                        }
                    }
                }
            }
        }
        let lib_path = lib_path.unwrap_or_else(|| base.to_path_buf());
        if dbg_on() { eprintln!("[PluginLoaderV2] load_plugin: lib='{}' path='{}'", lib_name, lib_path.display()); }
        let lib = unsafe { Library::new(&lib_path) }.map_err(|_| BidError::PluginError)?;
        let lib_arc = Arc::new(lib);

        // Resolve required invoke symbol (TypeBox v2: nyash_plugin_invoke)
        unsafe {
            let invoke_sym: Symbol<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize) -> i32> =
                lib_arc.get(b"nyash_plugin_invoke\0").map_err(|_| BidError::PluginError)?;

            // Optional init (best-effort)
            if let Ok(init_sym) = lib_arc.get::<Symbol<unsafe extern "C" fn() -> i32>>(b"nyash_plugin_init\0") {
                let _ = init_sym();
            }

            let loaded = LoadedPluginV2 {
                _lib: lib_arc.clone(),
                box_types: lib_def.boxes.clone(),
                typeboxes: HashMap::new(),
                init_fn: None,
                invoke_fn: *invoke_sym,
            };
            self.plugins.write().map_err(|_| BidError::PluginError)?.insert(lib_name.to_string(), Arc::new(loaded));
        }

        Ok(())
    }

    fn load_plugin_from_root(&self, _plugin_name: &str, _root: &str) -> BidResult<()> { Ok(()) }

    fn prebirth_singletons(&self) -> BidResult<()> {
        let config = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_content = super::errors::from_fs(std::fs::read_to_string(cfg_path))?;
        let toml_value: toml::Value = super::errors::from_toml(toml::from_str(&toml_content))?;
        for (lib_name, lib_def) in &config.libraries {
            for box_name in &lib_def.boxes {
                if let Some(bc) = config.get_box_config(lib_name, box_name, &toml_value) { if bc.singleton { let _ = self.ensure_singleton_handle(lib_name, box_name); } }
            }
        }
        Ok(())
    }

    fn find_box_by_type_id<'a>(&'a self, config: &'a NyashConfigV2, toml_value: &'a toml::Value, type_id: u32) -> Option<(&'a str, &'a str)> {
        for (lib_name, lib_def) in &config.libraries {
            for box_name in &lib_def.boxes {
                if let Some(box_conf) = config.get_box_config(lib_name, box_name, toml_value) { if box_conf.type_id == type_id { return Some((lib_name.as_str(), box_name.as_str())); } }
            }
        }
        None
    }

    pub fn construct_existing_instance(&self, type_id: u32, instance_id: u32) -> Option<Box<dyn NyashBox>> {
        let config = self.config.as_ref()?;
        let cfg_path = self.config_path.as_ref()?;
        let toml_str = std::fs::read_to_string(cfg_path).ok()?;
        let toml_value: toml::Value = toml::from_str(&toml_str).ok()?;
        let (lib_name, box_type) = self.find_box_by_type_id(config, &toml_value, type_id)?;
        let plugins = self.plugins.read().ok()?;
        let plugin = plugins.get(lib_name)?.clone();
        let fini_method_id = if let Some(spec) = self.box_specs.read().ok()?.get(&(lib_name.to_string(), box_type.to_string())) { spec.fini_method_id } else { let box_conf = config.get_box_config(lib_name, box_type, &toml_value)?; box_conf.methods.get("fini").map(|m| m.method_id) };
        let bx = super::types::construct_plugin_box(box_type.to_string(), type_id, plugin.invoke_fn, instance_id, fini_method_id);
        Some(Box::new(bx))
    }

    fn find_lib_name_for_box(&self, box_type: &str) -> Option<String> {
        if let Some(cfg) = &self.config { if let Some((name, _)) = cfg.find_library_for_box(box_type) { return Some(name.to_string()); } }
        for ((lib, b), _) in self.box_specs.read().unwrap().iter() { if b == box_type { return Some(lib.clone()); } }
        None
    }

    fn ensure_singleton_handle(&self, lib_name: &str, box_type: &str) -> BidResult<()> {
        if self.singletons.read().unwrap().contains_key(&(lib_name.to_string(), box_type.to_string())) { return Ok(()); }
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value = toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?).map_err(|_| BidError::PluginError)?;
        let config = self.config.as_ref().ok_or(BidError::PluginError)?;
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
        let type_id = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) { spec.type_id.unwrap_or_else(|| config.box_types.get(box_type).copied().unwrap_or(0)) } else { let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?; box_conf.type_id };
        let out = vec![0u8; 1024];
        let out_len = out.len();
        let tlv_args = crate::runtime::plugin_ffi_common::encode_empty_args();
        let (birth_result, _len, out_vec) = super::host_bridge::invoke_alloc(plugin.invoke_fn, type_id, 0, 0, &tlv_args);
        let out = out_vec;
        if birth_result != 0 || out_len < 4 { return Err(BidError::PluginError); }
        let instance_id = u32::from_le_bytes([out[0], out[1], out[2], out[3]]);
        let fini_id = if let Some(spec) = self.box_specs.read().unwrap().get(&(lib_name.to_string(), box_type.to_string())) { spec.fini_method_id } else { let box_conf = config.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?; box_conf.methods.get("fini").map(|m| m.method_id) };
        let handle = Arc::new(PluginHandleInner { type_id, invoke_fn: plugin.invoke_fn, instance_id, fini_method_id: fini_id, finalized: std::sync::atomic::AtomicBool::new(false) });
        self.singletons.write().unwrap().insert((lib_name.to_string(), box_type.to_string()), handle);
        crate::runtime::cache_versions::bump_version(&format!("BoxRef:{}", box_type));
        Ok(())
    }

    pub fn extern_call(&self, iface_name: &str, method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
        match (iface_name, method_name) {
            ("env.console", "log") => { for a in args { println!("{}", a.to_string_box().value); } Ok(None) }
            ("env.modules", "set") => {
                if args.len() >= 2 {
                    let key = args[0].to_string_box().value;
                    let val = args[1].clone_box();
                    crate::runtime::modules_registry::set(key, val);
                }
                Ok(None)
            }
            ("env.modules", "get") => {
                if let Some(k) = args.get(0) {
                    let key = k.to_string_box().value;
                    if let Some(v) = crate::runtime::modules_registry::get(&key) { return Ok(Some(v)); }
                }
                Ok(Some(Box::new(crate::box_trait::VoidBox::new())))
            }
            ("env.task", "cancelCurrent") => { let tok = crate::runtime::global_hooks::current_group_token(); tok.cancel(); Ok(None) }
            ("env.task", "currentToken") => { let tok = crate::runtime::global_hooks::current_group_token(); let tb = crate::boxes::token_box::TokenBox::from_token(tok); Ok(Some(Box::new(tb))) }
            ("env.debug", "trace") => { if std::env::var("NYASH_DEBUG_TRACE").ok().as_deref() == Some("1") { for a in args { eprintln!("[debug.trace] {}", a.to_string_box().value); } } Ok(None) }
            ("env.runtime", "checkpoint") => { if crate::config::env::runtime_checkpoint_trace() { eprintln!("[runtime.checkpoint] reached"); } crate::runtime::global_hooks::safepoint_and_poll(); Ok(None) }
            ("env.future", "new") | ("env.future", "birth") => { let fut = crate::boxes::future::FutureBox::new(); if let Some(v) = args.get(0) { fut.set_result(v.clone_box()); } Ok(Some(Box::new(fut))) }
            ("env.future", "set") => { if args.len() >= 2 { if let Some(fut) = args[0].as_any().downcast_ref::<crate::boxes::future::FutureBox>() { fut.set_result(args[1].clone_box()); } } Ok(None) }
            ("env.future", "await") => { use crate::boxes::result::NyashResultBox; if let Some(arg) = args.get(0) { if let Some(fut) = arg.as_any().downcast_ref::<crate::boxes::future::FutureBox>() { let max_ms: u64 = crate::config::env::await_max_ms(); let start = std::time::Instant::now(); let mut spins = 0usize; while !fut.ready() { crate::runtime::global_hooks::safepoint_and_poll(); std::thread::yield_now(); spins += 1; if spins % 1024 == 0 { std::thread::sleep(std::time::Duration::from_millis(1)); } if start.elapsed() >= std::time::Duration::from_millis(max_ms) { let err = crate::box_trait::StringBox::new("Timeout"); return Ok(Some(Box::new(NyashResultBox::new_err(Box::new(err))))); } } return match fut.wait_and_get() { Ok(v) => Ok(Some(Box::new(NyashResultBox::new_ok(v)))), Err(e) => { let err = crate::box_trait::StringBox::new(format!("Error: {}", e)); Ok(Some(Box::new(NyashResultBox::new_err(Box::new(err))))) } }; } else { return Ok(Some(Box::new(NyashResultBox::new_ok(arg.clone_box())))); } } Ok(Some(Box::new(crate::boxes::result::NyashResultBox::new_err(Box::new(crate::box_trait::StringBox::new("InvalidArgs")))))) }
            _ => Err(BidError::PluginError)
        }
    }

    fn resolve_method_id_from_file(&self, box_type: &str, method_name: &str) -> BidResult<u32> {
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value = super::errors::from_toml(toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?))?;
        if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
            if let Some(bc) = cfg.get_box_config(&lib_name, box_type, &toml_value) { if let Some(m) = bc.methods.get(method_name) { return Ok(m.method_id); } }
        }
        Err(BidError::InvalidMethod)
    }

    pub fn method_returns_result(&self, box_type: &str, method_name: &str) -> bool {
        if let Some(cfg) = &self.config {
            if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
                if let Some(cfg_path) = self.config_path.as_deref() {
                    if let Ok(toml_value) = toml::from_str::<toml::Value>(&std::fs::read_to_string(cfg_path).unwrap_or_default()) {
                        if let Some(bc) = cfg.get_box_config(&lib_name, box_type, &toml_value) { return bc.methods.get(method_name).map(|m| m.returns_result).unwrap_or(false); }
                    }
                }
            }
        }
        false
    }

    /// Resolve (type_id, method_id, returns_result) for a box_type.method
    pub fn resolve_method_handle(&self, box_type: &str, method_name: &str) -> BidResult<(u32, u32, bool)> {
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value = toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?).map_err(|_| BidError::PluginError)?;
        let (lib_name, _) = cfg.find_library_for_box(box_type).ok_or(BidError::InvalidType)?;
        let bc = cfg.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
        let m = bc.methods.get(method_name).ok_or(BidError::InvalidMethod)?;
        Ok((bc.type_id, m.method_id, m.returns_result))
    }

    pub fn invoke_instance_method(&self, box_type: &str, method_name: &str, instance_id: u32, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
        // Non-recursive direct bridge for minimal methods used by semantics and basic VM paths
        // Resolve library/type/method ids from cached config
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value = toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?).map_err(|_| BidError::PluginError)?;
        let (lib_name, _lib_def) = cfg.find_library_for_box(box_type).ok_or(BidError::InvalidType)?;
        let box_conf = cfg.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
        let type_id = box_conf.type_id;
        let method = box_conf.methods.get(method_name).ok_or(BidError::InvalidMethod)?;
        // Get plugin handle
        let plugins = self.plugins.read().map_err(|_| BidError::PluginError)?;
        let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;
        // Encode TLV args via shared helper (numeric→string→toString)
        let tlv = crate::runtime::plugin_ffi_common::encode_args(args);
        let (_code, out_len, out) = super::host_bridge::invoke_alloc(plugin.invoke_fn, type_id, method.method_id, instance_id, &tlv);
        // Decode TLV (first entry) generically
        if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
            let bx: Box<dyn NyashBox> = match tag {
                1 => Box::new(crate::box_trait::BoolBox::new(crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false))),
                2 => Box::new(crate::box_trait::IntegerBox::new(crate::runtime::plugin_ffi_common::decode::i32(payload).unwrap_or(0) as i64)),
                3 => {
                    // i64 payload
                    if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); Box::new(crate::box_trait::IntegerBox::new(i64::from_le_bytes(b))) }
                    else { Box::new(crate::box_trait::IntegerBox::new(0)) }
                }
                5 => {
                    let x = crate::runtime::plugin_ffi_common::decode::f64(payload).unwrap_or(0.0);
                    Box::new(crate::boxes::FloatBox::new(x))
                }
                6 | 7 => {
                    let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                    Box::new(crate::box_trait::StringBox::new(s))
                }
                8 => {
                    // Plugin handle (type_id, instance_id) → wrap into PluginBoxV2
                    if let Some((ret_type, inst)) = crate::runtime::plugin_ffi_common::decode::plugin_handle(payload) {
                        let handle = Arc::new(super::types::PluginHandleInner {
                            type_id: ret_type,
                            invoke_fn: plugin.invoke_fn,
                            instance_id: inst,
                            fini_method_id: None,
                            finalized: std::sync::atomic::AtomicBool::new(false),
                        });
                        Box::new(super::types::PluginBoxV2 { box_type: box_type.to_string(), inner: handle })
                    } else { Box::new(crate::box_trait::VoidBox::new()) }
                }
                9 => {
                    // Host handle (u64) → try to map back to BoxRef, else void
                    if let Some(u) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                        if let Some(arc) = crate::runtime::host_handles::get(u) { return Ok(Some(arc.share_box())); }
                    }
                    Box::new(crate::box_trait::VoidBox::new())
                }
                _ => Box::new(crate::box_trait::VoidBox::new()),
            };
            return Ok(Some(bx));
        }
        Ok(Some(Box::new(crate::box_trait::VoidBox::new())))
    }

    pub fn create_box(&self, box_type: &str, _args: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
        // Non-recursive: directly call plugin 'birth' and construct PluginBoxV2
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_value: toml::Value = toml::from_str(&std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?).map_err(|_| BidError::PluginError)?;
        let (lib_name, _) = cfg.find_library_for_box(box_type).ok_or(BidError::InvalidType)?;

        // Resolve type_id and method ids
        let box_conf = cfg.get_box_config(lib_name, box_type, &toml_value).ok_or(BidError::InvalidType)?;
        let type_id = box_conf.type_id;
        let birth_id = box_conf.methods.get("birth").map(|m| m.method_id).ok_or(BidError::InvalidMethod)?;
        let fini_id = box_conf.methods.get("fini").map(|m| m.method_id);

        // Get loaded plugin invoke
        let plugins = self.plugins.read().map_err(|_| BidError::PluginError)?;
        let plugin = plugins.get(lib_name).ok_or(BidError::PluginError)?;

        // Call birth (no args TLV) and read returned instance id (little-endian u32 in bytes 0..4)
        if dbg_on() {
            eprintln!("[PluginLoaderV2] invoking birth: box_type={} type_id={} birth_id={}", box_type, type_id, birth_id);
        }
        let tlv = crate::runtime::plugin_ffi_common::encode_empty_args();
        let (code, out_len, out_buf) = super::host_bridge::invoke_alloc(plugin.invoke_fn, type_id, birth_id, 0, &tlv);
        if dbg_on() {
            eprintln!("[PluginLoaderV2] create_box: box_type={} type_id={} birth_id={} code={} out_len={}", box_type, type_id, birth_id, code, out_len);
            if out_len > 0 { eprintln!("[PluginLoaderV2] create_box: out[0..min(8)]={:02x?}", &out_buf[..out_len.min(8)]); }
        }
        if code != 0 || out_len < 4 { return Err(BidError::PluginError); }
        let instance_id = u32::from_le_bytes([out_buf[0], out_buf[1], out_buf[2], out_buf[3]]);

        let bx = PluginBoxV2 {
            box_type: box_type.to_string(),
            inner: Arc::new(PluginHandleInner {
                type_id,
                invoke_fn: plugin.invoke_fn,
                instance_id,
                fini_method_id: fini_id,
                finalized: std::sync::atomic::AtomicBool::new(false),
            }),
        };
        Ok(Box::new(bx))
    }

    /// Shutdown singletons: finalize and clear all singleton handles
    pub fn shutdown_singletons(&self) {
        let mut map = self.singletons.write().unwrap();
        for (_, handle) in map.drain() { handle.finalize_now(); }
    }
}
