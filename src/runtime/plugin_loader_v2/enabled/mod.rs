mod types;
mod loader;
mod globals;
mod errors;
mod host_bridge;

pub use types::{PluginBoxV2, PluginHandleInner, NyashTypeBoxFfi, make_plugin_box_v2, construct_plugin_box};
pub use loader::PluginLoaderV2;
pub use globals::{get_global_loader_v2, init_global_loader_v2, shutdown_plugins_v2};

pub fn backend_kind() -> &'static str { "enabled" }
