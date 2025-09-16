use crate::bid::{BidError, BidResult};
use crate::box_trait::NyashBox;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

pub type InvokeFn =
    unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32;

#[derive(Debug, Clone)]
pub struct PluginBoxV2 {
    pub box_type: String,
    pub inner: std::sync::Arc<PluginHandleInner>,
}

#[derive(Debug, Clone)]
pub struct PluginBoxMetadata {
    pub lib_name: String,
    pub box_type: String,
    pub type_id: u32,
    pub invoke_fn: InvokeFn,
    pub fini_method_id: Option<u32>,
}

#[derive(Debug)]
pub struct PluginHandleInner {
    pub type_id: u32,
    pub invoke_fn: InvokeFn,
    pub instance_id: u32,
    pub fini_method_id: Option<u32>,
}

pub struct PluginLoaderV2 {
    pub config: Option<()>,
}
impl PluginLoaderV2 {
    pub fn new() -> Self {
        Self { config: None }
    }
}

impl PluginLoaderV2 {
    pub fn load_config(&mut self, _p: &str) -> BidResult<()> {
        Ok(())
    }
    pub fn load_all_plugins(&self) -> BidResult<()> {
        Ok(())
    }
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
    pub fn metadata_for_type_id(&self, _type_id: u32) -> Option<PluginBoxMetadata> {
        None
    }
    pub fn shutdown_singletons(&self) {}
}

static GLOBAL_LOADER_V2: Lazy<Arc<RwLock<PluginLoaderV2>>> =
    Lazy::new(|| Arc::new(RwLock::new(PluginLoaderV2::new())));
pub fn get_global_loader_v2() -> Arc<RwLock<PluginLoaderV2>> {
    GLOBAL_LOADER_V2.clone()
}
pub fn init_global_loader_v2(_config_path: &str) -> BidResult<()> {
    Ok(())
}
pub fn shutdown_plugins_v2() -> BidResult<()> {
    Ok(())
}

pub fn backend_kind() -> &'static str {
    "stub"
}

pub fn metadata_for_type_id(_type_id: u32) -> Option<PluginBoxMetadata> {
    None
}

pub fn make_plugin_box_v2(
    box_type: String,
    type_id: u32,
    instance_id: u32,
    invoke_fn: InvokeFn,
) -> PluginBoxV2 {
    PluginBoxV2 {
        box_type,
        inner: Arc::new(PluginHandleInner {
            type_id,
            invoke_fn,
            instance_id,
            fini_method_id: None,
        }),
    }
}

pub fn construct_plugin_box(
    box_type: String,
    type_id: u32,
    invoke_fn: InvokeFn,
    instance_id: u32,
    fini_method_id: Option<u32>,
) -> PluginBoxV2 {
    PluginBoxV2 {
        box_type,
        inner: Arc::new(PluginHandleInner {
            type_id,
            invoke_fn,
            instance_id,
            fini_method_id,
        }),
    }
}
