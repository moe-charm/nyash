use crate::bid::{BidResult, BidError};
use crate::box_trait::NyashBox;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

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

pub struct PluginLoaderV2 { pub config: Option<()> }
impl PluginLoaderV2 { pub fn new() -> Self { Self { config: None } } }

impl PluginLoaderV2 {
    pub fn load_config(&mut self, _p: &str) -> BidResult<()> { Ok(()) }
    pub fn load_all_plugins(&self) -> BidResult<()> { Ok(()) }
    pub fn create_box(&self, _t: &str, _a: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> { Err(BidError::PluginError) }
    pub fn extern_call(&self, _iface_name: &str, _method_name: &str, _args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> { Err(BidError::PluginError) }
    pub fn invoke_instance_method(&self, _box_type: &str, _method_name: &str, _instance_id: u32, _args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> { Err(BidError::PluginError) }
    pub fn shutdown_singletons(&self) {}
}

static GLOBAL_LOADER_V2: Lazy<Arc<RwLock<PluginLoaderV2>>> = Lazy::new(|| Arc::new(RwLock::new(PluginLoaderV2::new())));
pub fn get_global_loader_v2() -> Arc<RwLock<PluginLoaderV2>> { GLOBAL_LOADER_V2.clone() }
pub fn init_global_loader_v2(_config_path: &str) -> BidResult<()> { Ok(()) }
pub fn shutdown_plugins_v2() -> BidResult<()> { Ok(()) }

