//! Adapter to route legacy call-like MIR instructions through unified Callee path (opt-in).

use super::super::*;

impl MirInterpreter {
    /// Try to execute a legacy call-like instruction via `execute_callee_call` by
    /// converting it to a `Callee`. Enabled only when `NYASH_VM_CALL_ADAPTER=1`.
    pub(crate) fn try_execute_via_callee(&mut self, inst: &crate::mir::MirInstruction) -> Option<Result<VMValue, VMError>> {
        if std::env::var("NYASH_VM_CALL_ADAPTER").ok().as_deref() != Some("1") {
            return None;
        }
        use crate::mir::MirInstruction as I;
        use crate::mir::Callee;
        match inst {
            I::ExternCall { iface_name, method_name, args, .. } => {
                let name = format!("{}.{}", iface_name, method_name);
                Some(self.handle_callee_extern(&name, args))
            }
            I::BoxCall { box_val, method, args, .. } | I::PluginInvoke { box_val, method, args, .. } => {
                // Infer receiver box type for trace purposes; dispatch relies on execute_method_call
                let box_name = match self.reg_load(*box_val) {
                    Ok(VMValue::BoxRef(bx)) => bx.type_name().to_string(),
                    Ok(VMValue::String(_)) => "StringBox".to_string(),
                    _ => "<unknown>".to_string(),
                };
                let cal = Callee::Method { box_name, method: method.clone(), receiver: Some(*box_val), certainty: crate::mir::definitions::call_unified::TypeCertainty::Known };
                Some(self.execute_callee_call(&cal, args))
            }
            _ => None,
        }
    }
}

