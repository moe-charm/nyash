use crate::backend::vm_types::{VMError, VMValue};
use crate::mir::types::CompareOp;

/// Guard ordered compares for BoxRef pairs. Eq/Ne allow higher-level equals semantics.
pub fn guard_ordered_boxref(op: CompareOp, a: &VMValue, b: &VMValue) -> Result<(), VMError> {
    match op {
        CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge => {
            let a_is_box = matches!(a, VMValue::BoxRef(_));
            let b_is_box = matches!(b, VMValue::BoxRef(_));
            if a_is_box && b_is_box {
                return Err(VMError::TypeError(
                    "ordered compare on BoxRef is unsupported (use .equals or numeric field)".into(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
