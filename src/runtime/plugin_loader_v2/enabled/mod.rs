mod types;
mod loader;
mod globals;
mod errors;
mod host_bridge;

pub use types::{PluginBoxMetadata, PluginBoxV2, PluginHandleInner, NyashTypeBoxFfi, make_plugin_box_v2, construct_plugin_box};
pub use loader::PluginLoaderV2;
pub use globals::{get_global_loader_v2, init_global_loader_v2, shutdown_plugins_v2};

pub fn metadata_for_type_id(type_id: u32) -> Option<PluginBoxMetadata> {
    let loader = get_global_loader_v2();
    let guard = loader.read().ok()?;
    guard.metadata_for_type_id(type_id)
}

pub fn backend_kind() -> &'static str { "enabled" }
