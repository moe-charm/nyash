use crate::backend::vm::ControlFlow;
use crate::backend::{VMError, VMValue, VM};
use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use std::sync::Arc;

impl VM {
    /// Execute NewBox instruction
    pub(crate) fn execute_newbox(
        &mut self,
        dst: ValueId,
        box_type: &str,
        args: &[ValueId],
    ) -> Result<ControlFlow, VMError> {
        // Convert args to NyashBox values
        let arg_values: Vec<Box<dyn NyashBox>> = args
            .iter()
            .map(|arg| {
                let val = self.get_value(*arg)?;
                Ok(val.to_nyash_box())
            })
            .collect::<Result<Vec<_>, VMError>>()?;

        // Create new box using runtime's registry
        let new_box = {
            let registry = self.runtime.box_registry.lock().map_err(|_| {
                VMError::InvalidInstruction("Failed to lock box registry".to_string())
            })?;
            registry.create_box(box_type, &arg_values).map_err(|e| {
                VMError::InvalidInstruction(format!("Failed to create {}: {}", box_type, e))
            })?
        };

        // 80/20: Basic boxes are stored as primitives in VMValue for simpler ops
        if box_type == "IntegerBox" {
            if let Some(ib) = new_box
                .as_any()
                .downcast_ref::<crate::box_trait::IntegerBox>()
            {
                self.set_value(dst, VMValue::Integer(ib.value));
                return Ok(ControlFlow::Continue);
            }
        } else if box_type == "BoolBox" {
            if let Some(bb) = new_box.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                self.set_value(dst, VMValue::Bool(bb.value));
                return Ok(ControlFlow::Continue);
            }
        } else if box_type == "StringBox" {
            if let Some(sb) = new_box
                .as_any()
                .downcast_ref::<crate::box_trait::StringBox>()
            {
                self.set_value(dst, VMValue::String(sb.value.clone()));
                return Ok(ControlFlow::Continue);
            }
        }

        self.set_value(dst, VMValue::BoxRef(Arc::from(new_box)));
        Ok(ControlFlow::Continue)
    }
}
