//! Nyashランタイムモジュール
//!
//! プラグインシステムとBox管理の中核

pub mod box_registry;
pub mod gc;
pub mod gc_controller;
pub mod gc_mode;
pub mod gc_trace;
pub mod global_hooks;
pub mod leak_tracker;
pub mod nyash_runtime;
pub mod plugin_config;
pub mod plugin_ffi_common;
pub mod plugin_loader_unified;
pub mod plugin_loader_v2;
pub mod plugin_boot_box; // unified plugin init (idempotent)
pub mod static_plugins; // static metadata registration (features-driven)
pub mod method_router_box; // single entry for method dispatch (façade)
pub mod provider_box;
pub mod plugin_host_box;
pub mod codec;
pub mod method_registry_box;
pub mod scheduler;
pub mod semantics;
pub mod unified_registry;
pub mod provider_lock;
pub mod provider_verify;
pub mod adapters;
pub mod host_handle_router;
pub mod extern_registry; // ExternCall (env.*) 登録・診断用レジストリ
pub mod host_api; // C ABI: plugins -> host 逆呼び出しAPI（TLSでVMに橋渡し）
pub mod host_api_box; // Thin facade (slots + grow wrappers)
pub mod host_api_anchors; // Force-link host API symbols for plugin dlsym() support
pub mod host_handles; // C ABI(TLV) 向け HostHandle レジストリ（ユーザー/内蔵Box受け渡し）
pub mod host_handle_box; // Box wrapper to carry HostHandle(u64) across Router→FFI
pub mod console_adapter; // Print normalization (stdout)
pub mod modules_registry;
pub mod type_box_abi; // Phase 12: Nyash ABI (vtable) 雛形
pub mod type_meta;
pub mod type_registry; // Phase 12: TypeId→TypeBox 解決（雛形） // env.modules minimal registry
pub mod nykernel_stub; // Dev-only nykernel.* stub (shared)
pub mod types; // small newtypes (e.g., VerifiedPath)
pub mod diagnostics;
pub mod spec_ingest_box;
pub mod env_gate_box;
pub mod method_ids_box;

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