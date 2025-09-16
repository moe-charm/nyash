//! Nyashランタイムモジュール
//!
//! プラグインシステムとBox管理の中核

pub mod box_registry;
pub mod gc;
pub mod global_hooks;
pub mod leak_tracker;
pub mod nyash_runtime;
pub mod plugin_config;
pub mod plugin_ffi_common;
pub mod plugin_loader_unified;
pub mod plugin_loader_v2;
pub mod scheduler;
pub mod semantics;
pub mod unified_registry;
// pub mod plugin_box;  // legacy - 古いPluginBox
// pub mod plugin_loader;  // legacy - Host VTable使用
pub mod extern_registry; // ExternCall (env.*) 登録・診断用レジストリ
pub mod host_api; // C ABI: plugins -> host 逆呼び出しAPI（TLSでVMに橋渡し）
pub mod host_handles; // C ABI(TLV) 向け HostHandle レジストリ（ユーザー/内蔵Box受け渡し）
pub mod modules_registry;
pub mod type_box_abi; // Phase 12: Nyash ABI (vtable) 雛形
pub mod type_meta;
pub mod type_registry; // Phase 12: TypeId→TypeBox 解決（雛形） // env.modules minimal registry

#[cfg(test)]
mod tests;

pub use box_registry::{get_global_registry, BoxFactoryRegistry, BoxProvider};
pub use plugin_config::PluginConfig;
pub use plugin_loader_unified::{
    get_global_plugin_host, init_global_plugin_host, MethodHandle, PluginBoxType, PluginHost,
    PluginLibraryHandle,
};
pub use plugin_loader_v2::{get_global_loader_v2, init_global_loader_v2, PluginLoaderV2};
pub mod cache_versions;
pub use gc::{BarrierKind, GcHooks};
pub use nyash_runtime::{NyashRuntime, NyashRuntimeBuilder};
pub use scheduler::{Scheduler, SingleThreadScheduler};
pub use unified_registry::{
    get_global_unified_registry, init_global_unified_registry, register_user_defined_factory,
};
// pub use plugin_box::PluginBox;  // legacy
// Use unified plugin loader (formerly v2)
// pub use plugin_loader::{PluginLoaderV2 as PluginLoader, get_global_loader_v2 as get_global_loader};  // legacy
