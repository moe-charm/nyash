use super::super::*;

impl MirInterpreter {
    /// Compute a stable key for an object receiver to store fields across functions.
    /// Prefer Arc ptr address for BoxRef; else fall back to ValueId number cast.
    pub(in crate::backend::mir_interpreter) fn object_key_for(&self, id: crate::mir::ValueId) -> u64 {
        if let Ok(v) = self.reg_load(id) {
            if let crate::backend::vm::VMValue::BoxRef(arc) = v {
                let ptr = std::sync::Arc::as_ptr(&arc) as *const ();
                return ptr as usize as u64;
            }
        }
        id.as_u32() as u64
    }

    /// Register a BoxRef into the current scope if it is a release candidate
    /// (user InstanceBox, PluginBoxV2, or identity handle box).
    pub(in crate::backend::mir_interpreter) fn maybe_register_scope_value(&mut self, v: &VMValue) {
        let bx = match v { VMValue::BoxRef(b) => b.clone(), _ => return };
        // InstanceBox
        if bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() {
            self.scope.register_box(bx);
            return;
        }
        // PluginBoxV2
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        if bx
            .as_any()
            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
            .is_some()
        {
            self.scope.register_box(bx);
            return;
        }
        // Identity handle (e.g., external/stateful)
        if bx.is_identity() {
            self.scope.register_box(bx);
        }
    }
}
