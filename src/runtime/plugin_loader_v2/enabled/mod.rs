mod errors;
mod extern_functions;
mod ffi_bridge;
mod globals;
mod host_bridge;
mod instance_manager;
mod loader;
mod method_resolver;
mod types;

pub use globals::{get_global_loader_v2, init_global_loader_v2, shutdown_plugins_v2};
pub use loader::PluginLoaderV2;
pub use types::{
    cache, construct_plugin_box, make_plugin_box_v2, NyashTypeBoxFfi, PluginBoxMetadata,
    PluginBoxV2, PluginHandleInner,
};
// Re-export BoxInvokeFn alias for external callers
pub use host_bridge::BoxInvokeFn;

pub fn metadata_for_type_id(type_id: u32) -> Option<PluginBoxMetadata> {
    let loader = get_global_loader_v2();
    let guard = loader.read().ok()?;
    guard.metadata_for_type_id(type_id)
}

/// Resolve per-Box invoke function for a type_id (v2 only)
pub fn box_invoke_for_type_id(type_id: u32) -> Option<super::enabled::host_bridge::BoxInvokeFn> {
    let loader = get_global_loader_v2();
    let guard = loader.read().ok()?;
    guard.box_invoke_fn_for_type_id(type_id)
}

/// Library-level shim to dispatch a v2 per-Box invoke function using type_id
pub extern "C" fn nyash_plugin_invoke_v2_shim(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if let Some(f) = box_invoke_for_type_id(type_id) {
        return f(instance_id, method_id, args, args_len, result, result_len);
    }
    if let Some(inv) = lib_invoke_for_type_id(type_id) {
        return unsafe { inv(type_id, method_id, instance_id, args, args_len, result, result_len) };
    }
    -5
}

pub fn backend_kind() -> &'static str {
    "enabled"
}

fn lib_invoke_for_type_id(type_id: u32) -> Option<super::enabled::host_bridge::InvokeFn> {
    let loader = get_global_loader_v2();
    let guard = loader.read().ok()?;
    if let Some(meta) = guard.metadata_for_type_id(type_id) {
        if let Ok(plugins) = guard.plugins.read() {
            if let Some(loaded) = plugins.get(&meta.lib_name) {
                unsafe {
                    if let Ok(sym) = loaded._lib.get::<libloading::Symbol<super::enabled::host_bridge::InvokeFn>>(b"nyash_plugin_invoke\0") {
                        return Some(**sym);
                    }
                }
            }
        }
    }
    None
}
