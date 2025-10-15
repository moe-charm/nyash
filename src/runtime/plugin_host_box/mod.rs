//! PluginHostBox — facade for plugin create/invoke/resolve
//!
//! Responsibility
//! - Provide a single stable entrypoint for VM and other layers to interact with
//!   the plugin host (v2/unified), without importing loader internals.
//! - Encapsulate handle/metadata lookups and error types.
//!
//! Notes
//! - Internally delegates to the unified plugin host (`plugin_loader_unified`).
//! - Keep the API minimal; extend only when a new call-site needs a function.

use crate::bid::BidResult;
use crate::box_trait::NyashBox;

/// Create a plugin box of `box_type` via the active host.
pub fn create_box(box_type: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
    let guard = host.read().map_err(|_| crate::bid::BidError::PluginError)?;
    guard.create_box(box_type, args)
}

/// Invoke an instance method on a plugin-managed instance.
pub fn invoke_instance_method(
    box_type: &str,
    method_name: &str,
    instance_id: u32,
    args: &[Box<dyn NyashBox>],
) -> BidResult<Option<Box<dyn NyashBox>>> {
    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
    let guard = host.read().map_err(|_| crate::bid::BidError::PluginError)?;
    guard.invoke_instance_method(box_type, method_name, instance_id, args)
}

/// Resolve a method handle (type_id, method_id, returns_result) if available.
pub fn resolve_method_handle(
    box_type: &str,
    method_name: &str,
) -> BidResult<crate::runtime::plugin_loader_unified::MethodHandle> {
    crate::runtime::method_registry_box::resolve_method_handle(box_type, method_name)
}
