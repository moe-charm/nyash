//! Extern function execution (exit, panic, etc.)

use super::super::super::*;

impl MirInterpreter {
    pub(crate) fn execute_extern_function(
        &mut self,
        extern_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // Unified dotted extern support: e.g., "nyrt.ops.op_eq"
        if let Some((iface, method)) = extern_name.rsplit_once('.') {
            // Route selected iface.method pairs to dedicated handlers
            if iface == "nyrt.ops" && method == "op_eq" {
                if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.ops.op_eq requires 2 args".into())); }
                let a = self.reg_load(args[0])?;
                let b = self.reg_load(args[1])?;
                let ok = self.eval_equals(&a, &b)?;
                return Ok(VMValue::Bool(ok));
            }
            // Unknown dotted extern — return error (use legacy ExternCall path only for supported iface.method)
            return Err(VMError::InvalidInstruction(format!(
                "ExternCall {}.{} not supported",
                iface, method
            )));
        }
        match extern_name {
            "exit" => {
                let code = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.as_integer().unwrap_or(0)
                } else {
                    0
                };
                std::process::exit(code as i32);
            }
            "panic" => {
                let msg = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.to_string()
                } else {
                    "VM panic".to_string()
                };
                panic!("{}", msg);
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown extern function: {}",
                extern_name
            ))),
        }
    }
}
