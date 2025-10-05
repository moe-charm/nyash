//! Callee type dispatcher
//!
//! Routes Callee variants to appropriate handlers (Global, ModuleFunction, Method, etc.)

use super::super::super::*;
use super::method_handler::*;

impl MirInterpreter {
    pub(crate) fn execute_callee_call(
        &mut self,
        callee: &Callee,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // Optional: emit one-line JSON call trace for parity checks (refactored helper)
        match callee {
            // Global: trace emission is handled inside handle_callee_global after bridge check
            Callee::Global(_name) => {}
            Callee::ModuleFunction(_name) => {
                // Trace emitted below in specific handler; keep quiet here
            }
            Callee::Method { box_name, method, receiver, .. } => {
                let label = format!("Method:{}.{}{}", box_name, method, format!("/{}", args.len()));
                let recv_id = receiver.map(|v| v.as_u32());
                self.emit_call_trace_label(&label, args.len(), recv_id);
            }
            Callee::Constructor { box_type } => {
                let label = format!("Ctor:{}", box_type);
                self.emit_call_trace_label(&label, args.len(), None);
            }
            Callee::Closure { .. } => {
                self.emit_call_trace_label("Closure", args.len(), None);
            }
            Callee::Value(_fid) => {
                self.emit_call_trace_label("Value", args.len(), None);
            }
            // Extern: trace emission is handled inside handle_callee_extern
            Callee::Extern(_name) => {}
        }
        match callee {
            Callee::Global(func_name) => self.handle_callee_global(func_name, args),
            Callee::ModuleFunction(func_name) => self.handle_callee_module_function(func_name, args),
            Callee::Method { box_name, method, receiver, certainty: _, } => {
                self.handle_method_call_legacy(box_name, method, *receiver, args)
            }
            Callee::Constructor { box_type } => Err(VMError::InvalidInstruction(format!(
                "Constructor calls not yet implemented for {}",
                box_type
            ))),
            Callee::Closure { .. } => Err(VMError::InvalidInstruction(
                "Closure creation not yet implemented in VM".into(),
            )),
            Callee::Value(func_val_id) => {
                let _func_val = self.reg_load(*func_val_id)?;
                Err(VMError::InvalidInstruction(
                    "First-class function calls not yet implemented in VM".into(),
                ))
            }
            Callee::Extern(extern_name) => self.handle_callee_extern(extern_name, args),
        }
    }
}
