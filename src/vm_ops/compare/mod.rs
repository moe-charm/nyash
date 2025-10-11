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

/// Centralized Eq/Ne routing (box-first). For now delegates to VM's eval_equals,
/// but this is the single choke point to swap to Extern(op_eq) in the future.
pub fn equals_route(
    interp: &mut crate::backend::mir_interpreter::MirInterpreter,
    a: &crate::backend::vm_types::VMValue,
    b: &crate::backend::vm_types::VMValue,
) -> Result<bool, crate::backend::vm_types::VMError> {
    // Prefer Extern("nyrt.ops.op_eq") via interpreter public facade for unified semantics
    if let Some(res) = interp.extern_call_public("nyrt.ops", "op_eq", &[a.clone(), b.clone()]) {
        match res? {
            crate::backend::vm_types::VMValue::Bool(x) => return Ok(x),
            other => return Err(crate::backend::vm_types::VMError::TypeError(format!(
                "nyrt.ops.op_eq returned non-bool: {:?}", other
            ))),
        }
    }
    // Fallback to interpreter equals (should be rare)
    interp.equals_route_public(a, b)
}


