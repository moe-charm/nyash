//! Call helpers: centralizes call paths (PluginHost, functions)

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, VoidBox};
use super::{NyashInterpreter, RuntimeError};

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
use crate::runtime::plugin_loader_v2::PluginBoxV2;

impl NyashInterpreter {
    /// Invoke a method on a PluginBoxV2 via PluginHost facade.
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    pub(crate) fn call_plugin_method(
        &mut self,
        plugin_box: &PluginBoxV2,
        method: &str,
        arg_nodes: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        if plugin_box.is_finalized() {
            return Err(RuntimeError::RuntimeFailure { message: format!("Use after fini: {}", plugin_box.box_type) });
        }
        let mut arg_values: Vec<Box<dyn NyashBox>> = Vec::new();
        for arg in arg_nodes {
            arg_values.push(self.execute_expression(arg)?);
        }
        let host_guard = crate::runtime::get_global_plugin_host();
        let host = host_guard.read().map_err(|_| RuntimeError::RuntimeFailure { message: "Plugin host lock poisoned".into() })?;
        match host.invoke_instance_method(&plugin_box.box_type, method, plugin_box.instance_id(), &arg_values) {
            Ok(Some(result_box)) => Ok(result_box),
            Ok(None) => Ok(Box::new(VoidBox::new())),
            Err(e) => Err(RuntimeError::RuntimeFailure { message: format!("Plugin method {} failed: {:?}", method, e) }),
        }
    }

    /// Create a plugin box by type with arguments evaluated from AST.
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    pub(crate) fn create_plugin_box(
        &mut self,
        box_type: &str,
        arg_nodes: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut arg_values: Vec<Box<dyn NyashBox>> = Vec::new();
        for arg in arg_nodes {
            arg_values.push(self.execute_expression(arg)?);
        }
        let host_guard = crate::runtime::get_global_plugin_host();
        let host = host_guard.read().map_err(|_| RuntimeError::RuntimeFailure { message: "Plugin host lock poisoned".into() })?;
        host.create_box(box_type, &arg_values)
            .map_err(|e| RuntimeError::InvalidOperation { message: format!("Failed to construct plugin '{}': {:?}", box_type, e) })
    }

    /// Check if a given box type is provided by plugins (per current config).
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    pub(crate) fn is_plugin_box_type(&self, box_type: &str) -> bool {
        let host_guard = crate::runtime::get_global_plugin_host();
        if let Ok(host) = host_guard.read() {
            if let Some(cfg) = host.config_ref() {
                return cfg.find_library_for_box(box_type).is_some();
            }
        }
        false
    }
}
