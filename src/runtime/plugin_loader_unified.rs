//! Unified Plugin Host facade
//!
//! Thin wrapper over v2 loader to provide a stable facade
//! with minimal, friendly API for runtime/runner and future transports.

use once_cell::sync::Lazy;
use std::cell::Cell;
use std::sync::{Arc, RwLock};

use crate::bid::{BidError, BidResult};
use crate::config::nyash_toml_v2::NyashConfigV2;
use crate::runtime::plugin_loader_v2::PluginLoaderV2;

/// Opaque library handle (by name for now)
#[derive(Clone, Debug)]
pub struct PluginLibraryHandle {
    pub name: String,
}

/// Box type descriptor
#[derive(Clone, Debug)]
pub struct PluginBoxType {
    pub lib: String,
    pub name: String,
    pub type_id: u32,
}

/// Resolved method handle
#[derive(Clone, Debug)]
pub struct MethodHandle {
    pub lib: String,
    pub box_type: String,
    pub type_id: u32,
    pub method_id: u32,
    pub returns_result: bool,
}

/// Unified facade
pub struct PluginHost {
    loader: Arc<RwLock<PluginLoaderV2>>, // delegate
    config: Option<NyashConfigV2>,       // cached config for resolution
    config_path: Option<String>,
}

impl PluginHost {
    pub fn new(loader: Arc<RwLock<PluginLoaderV2>>) -> Self {
        Self {
            loader,
            config: None,
            config_path: None,
        }
    }

    /// Load config and dynamic libraries, keeping a local config cache.
    pub fn load_libraries(&mut self, config_path: &str) -> BidResult<()> {
        {
            let mut l = self.loader.write().unwrap();
            l.load_config(config_path)?;
        }

        // Keep our own copy for quick lookups
        let canonical = std::fs::canonicalize(config_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| config_path.to_string());
        self.config =
            Some(NyashConfigV2::from_file(&canonical).map_err(|_| BidError::PluginError)?);
        self.config_path = Some(canonical);

        // Delegate actual library loads + pre-birth singletons to v2
        let l = self.loader.read().unwrap();
        l.load_all_plugins()
    }

    /// Register built-ins or user-defined boxes if needed (no-op for now).
    pub fn register_boxes(&self) -> BidResult<()> {
        Ok(())
    }

    /// Expose read-only view of loaded config for callers migrating from v2 paths.
    pub fn config_ref(&self) -> Option<&NyashConfigV2> {
        self.config.as_ref()
    }

    /// Resolve a method handle for a given plugin box type and method name.
    pub fn resolve_method(&self, box_type: &str, method_name: &str) -> BidResult<MethodHandle> {
        let cfg = self.config.as_ref().ok_or(BidError::PluginError)?;
        let (lib_name, _lib_def) = cfg
            .find_library_for_box(box_type)
            .ok_or(BidError::InvalidType)?;
        let cfg_path = self.config_path.as_deref().unwrap_or("nyash.toml");
        let toml_content = std::fs::read_to_string(cfg_path).map_err(|_| BidError::PluginError)?;
        let toml_value: toml::Value =
            toml::from_str(&toml_content).map_err(|_| BidError::PluginError)?;
        let box_conf = cfg
            .get_box_config(lib_name, box_type, &toml_value)
            .ok_or(BidError::InvalidType)?;
        // Prefer config mapping; fallback to loader's TypeBox resolve(name)
        let (method_id, returns_result) = if let Some(m) = box_conf.methods.get(method_name) {
            (m.method_id, m.returns_result)
        } else {
            let l = self.loader.read().unwrap();
            let mid = l
                .resolve_method_id(box_type, method_name)
                .map_err(|_| BidError::InvalidMethod)?;
            (mid, false)
        };
        Ok(MethodHandle {
            lib: lib_name.to_string(),
            box_type: box_type.to_string(),
            type_id: box_conf.type_id,
            method_id,
            returns_result,
        })
    }

    // --- v2 adapter layer: allow gradual migration of callers ---
    pub fn create_box(
        &self,
        box_type: &str,
        args: &[Box<dyn crate::box_trait::NyashBox>],
    ) -> BidResult<Box<dyn crate::box_trait::NyashBox>> {
        let l = self.loader.read().unwrap();
        l.create_box(box_type, args)
    }

    pub fn invoke_instance_method(
        &self,
        box_type: &str,
        method_name: &str,
        instance_id: u32,
        args: &[Box<dyn crate::box_trait::NyashBox>],
    ) -> BidResult<Option<Box<dyn crate::box_trait::NyashBox>>> {
        thread_local! { static HOST_REENTRANT: Cell<bool> = Cell::new(false); }
        let recursed = HOST_REENTRANT.with(|f| f.get());
        if recursed {
            // Break potential host<->loader recursion: return None (void) to keep VM running
            return Ok(None);
        }
        let out = HOST_REENTRANT.with(|f| {
            f.set(true);
            let res = {
                let l = self.loader.read().unwrap();
                l.invoke_instance_method(box_type, method_name, instance_id, args)
            };
            f.set(false);
            res
        });
        out
    }

    /// Check if a method returns Result (Ok/Err) per plugin spec or central config.
    pub fn method_returns_result(&self, box_type: &str, method_name: &str) -> bool {
        let l = self.loader.read().unwrap();
        l.method_returns_result(box_type, method_name)
    }

    pub fn extern_call(
        &self,
        iface_name: &str,
        method_name: &str,
        args: &[Box<dyn crate::box_trait::NyashBox>],
    ) -> BidResult<Option<Box<dyn crate::box_trait::NyashBox>>> {
        // Special-case env.future.await to avoid holding loader RwLock while polling scheduler
        if iface_name == "env.future" && method_name == "await" {
            use crate::boxes::result::NyashResultBox;
            if let Some(arg0) = args.get(0) {
                if let Some(fut) = arg0
                    .as_any()
                    .downcast_ref::<crate::boxes::future::FutureBox>()
                {
                    let max_ms: u64 = crate::config::env::await_max_ms();
                    let start = std::time::Instant::now();
                    let mut spins = 0usize;
                    while !fut.ready() {
                        crate::runtime::global_hooks::safepoint_and_poll();
                        std::thread::yield_now();
                        spins += 1;
                        if spins % 1024 == 0 {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                        if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                            let err = crate::box_trait::StringBox::new("Timeout");
                            return Ok(Some(Box::new(NyashResultBox::new_err(Box::new(err)))));
                        }
                    }
                    return Ok(fut.wait_and_get().ok().map(|v| {
                        Box::new(NyashResultBox::new_ok(v)) as Box<dyn crate::box_trait::NyashBox>
                    }));
                } else {
                    return Ok(Some(Box::new(NyashResultBox::new_ok(arg0.clone_box()))));
                }
            }
            return Ok(Some(Box::new(NyashResultBox::new_err(Box::new(
                crate::box_trait::StringBox::new("InvalidArgs"),
            )))));
        }
        let l = self.loader.read().unwrap();
        l.extern_call(iface_name, method_name, args)
    }
}

// Global singleton
static GLOBAL_HOST: Lazy<Arc<RwLock<PluginHost>>> = Lazy::new(|| {
    let loader = crate::runtime::plugin_loader_v2::get_global_loader_v2();
    Arc::new(RwLock::new(PluginHost::new(loader)))
});

pub fn get_global_plugin_host() -> Arc<RwLock<PluginHost>> {
    GLOBAL_HOST.clone()
}

pub fn init_global_plugin_host(config_path: &str) -> BidResult<()> {
    let host = get_global_plugin_host();
    host.write().unwrap().load_libraries(config_path)?;
    host.read().unwrap().register_boxes()?;
    Ok(())
}
