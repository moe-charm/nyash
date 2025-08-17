//! Nyashランタイムモジュール
//! 
//! プラグインシステムとBox管理の中核

pub mod plugin_config;
pub mod box_registry;
pub mod plugin_box;

pub use plugin_config::PluginConfig;
pub use box_registry::{BoxFactoryRegistry, BoxProvider, get_global_registry};
pub use plugin_box::PluginBox;