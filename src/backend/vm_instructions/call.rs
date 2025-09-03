use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use crate::backend::vm::ControlFlow;
use crate::backend::{VM, VMError, VMValue};

impl VM {
    /// Execute Call instruction (supports function name or FunctionBox value)
    pub(crate) fn execute_call(&mut self, dst: Option<ValueId>, func: ValueId, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Evaluate function value
        let fval = self.get_value(func)?;
        match fval {
            VMValue::String(func_name) => {
                // Legacy: call function by name
                let arg_values: Vec<VMValue> = args.iter().map(|arg| self.get_value(*arg)).collect::<Result<Vec<_>, _>>()?;
                let result = self.call_function_by_name(&func_name, arg_values)?;
                if let Some(dst_id) = dst { self.set_value(dst_id, result); }
                Ok(ControlFlow::Continue)
            }
            VMValue::BoxRef(arc_box) => {
                // FunctionBox call path
                if let Some(fun) = arc_box.as_any().downcast_ref::<crate::boxes::function_box::FunctionBox>() {
                    // Convert args to NyashBox for interpreter helper
                    let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
                        .map(|a| self.get_value(*a).map(|v| v.to_nyash_box()))
                        .collect::<Result<Vec<_>, VMError>>()?;
                    // Execute via interpreter helper
                    match crate::interpreter::run_function_box(fun, nyash_args) {
                        Ok(out) => {
                            if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            Ok(ControlFlow::Continue)
                        }
                        Err(e) => Err(VMError::InvalidInstruction(format!("FunctionBox call failed: {:?}", e)))
                    }
                } else {
                    Err(VMError::TypeError(format!("Call target not callable: {}", arc_box.type_name())))
                }
            }
            other => Err(VMError::TypeError(format!("Call target must be function name or FunctionBox, got {:?}", other))),
        }
    }
}
