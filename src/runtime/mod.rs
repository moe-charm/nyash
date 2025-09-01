//! Nyashランタイムモジュール
//! 
//! プラグインシステムとBox管理の中核

pub mod plugin_config;
pub mod box_registry;
pub mod plugin_loader_v2;
pub mod plugin_loader_unified;
pub mod plugin_ffi_common;
pub mod leak_tracker;
pub mod unified_registry;
pub mod nyash_runtime;
pub mod gc;
pub mod scheduler;
pub mod global_hooks;
pub mod semantics;
// pub mod plugin_box;  // legacy - 古いPluginBox
// pub mod plugin_loader;  // legacy - Host VTable使用
pub mod type_meta;

#[cfg(test)]
mod tests;

pub use plugin_config::PluginConfig;
pub use box_registry::{BoxFactoryRegistry, BoxProvider, get_global_registry};
pub use plugin_loader_v2::{PluginLoaderV2, get_global_loader_v2, init_global_loader_v2};
pub use plugin_loader_unified::{PluginHost, get_global_plugin_host, init_global_plugin_host, PluginLibraryHandle, PluginBoxType, MethodHandle};
pub mod cache_versions;
pub use unified_registry::{get_global_unified_registry, init_global_unified_registry, register_user_defined_factory};
pub use nyash_runtime::{NyashRuntime, NyashRuntimeBuilder};
pub use gc::{GcHooks, BarrierKind};
pub use scheduler::{Scheduler, SingleThreadScheduler};
// pub use plugin_box::PluginBox;  // legacy
// Use unified plugin loader (formerly v2)
// pub use plugin_loader::{PluginLoaderV2 as PluginLoader, get_global_loader_v2 as get_global_loader};  // legacy
