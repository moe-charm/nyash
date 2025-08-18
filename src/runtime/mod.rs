//! Nyashランタイムモジュール
//! 
//! プラグインシステムとBox管理の中核

pub mod plugin_config;
pub mod box_registry;
// pub mod plugin_box;  // legacy - 古いPluginBox
// pub mod plugin_loader;  // legacy - Host VTable使用

#[cfg(test)]
mod tests;

pub use plugin_config::PluginConfig;
pub use box_registry::{BoxFactoryRegistry, BoxProvider, get_global_registry};
// pub use plugin_box::PluginBox;  // legacy
// Use unified plugin loader (formerly v2)
// pub use plugin_loader::{PluginLoaderV2 as PluginLoader, get_global_loader_v2 as get_global_loader};  // legacy