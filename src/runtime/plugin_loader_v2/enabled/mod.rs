mod errors;
mod globals;
mod host_bridge;
mod loader;
mod types;

pub use globals::{get_global_loader_v2, init_global_loader_v2, shutdown_plugins_v2};
pub use loader::PluginLoaderV2;
pub use types::{
    construct_plugin_box, make_plugin_box_v2, NyashTypeBoxFfi, PluginBoxMetadata, PluginBoxV2,
    PluginHandleInner,
};

pub fn metadata_for_type_id(type_id: u32) -> Option<PluginBoxMetadata> {
    let loader = get_global_loader_v2();
    let guard = loader.read().ok()?;
    guard.metadata_for_type_id(type_id)
}

pub fn backend_kind() -> &'static str {
    "enabled"
}
