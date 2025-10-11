use crate::backend::vm_types::VMError;

/// TypeRegistry を参照し、(Type, method, arity) の組が利用可能かを早期検証する。
/// 利用可能でなければ、一貫した診断を返す。
pub fn arity_guard_for(type_name: &str, method: &str, arity: usize) -> Result<(), VMError> {
    if crate::runtime::type_registry::resolve_typebox_by_name(type_name).is_some() {
        if crate::runtime::type_registry::resolve_slot_by_name(type_name, method, arity).is_none() {
            if let Some(known) = crate::runtime::type_registry::known_arities_for(type_name, method) {
                if !known.is_empty() {
                    return Err(VMError::InvalidInstruction(format!(
                        "No matching method: {}.{}({} args). Available arities: {:?}",
                        type_name, method, arity, known
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Preflight for BoxCall: unborn guard unification (skips for birth).
pub fn preflight_unborn(
    interp: &crate::backend::mir_interpreter::MirInterpreter,
    recv_id: crate::mir::ValueId,
    method: &str,
) -> Result<(), crate::backend::vm_types::VMError> {
    interp.boxcall_unborn_guard_public(recv_id, method)
}

/// Build a unified Unknown-method error for BoxCall.
pub fn unknown_method_err(type_name: &str, method: &str, arity: usize) -> crate::backend::vm_types::VMError {
    crate::backend::vm_types::VMError::InvalidInstruction(format!(
        "Unknown method: {}.{} (arity={})",
        type_name, method, arity
    ))
}


/// Build a unified downcast-failed error for BoxCall receivers.
pub fn downcast_failed(type_name: &str) -> crate::backend::vm_types::VMError {
    crate::backend::vm_types::VMError::TypeError(format!("downcast failed: {}", type_name))
}

fn vmvalue_kind(recv: &crate::backend::vm_types::VMValue) -> String {
    use crate::backend::vm_types::VMValue as V;
    match recv {
        V::Void => "Void".into(),
        V::Integer(_) => "Integer".into(),
        V::Float(_) => "Float".into(),
        V::Bool(_) => "Bool".into(),
        V::String(_) => "String".into(),
        V::Future(_) => "Future".into(),
        V::BoxRef(bx) => format!("BoxRef({})", bx.type_name()),
    }
}

/// Build a unified method-not-supported error using a stable kind printer.
pub fn method_not_supported(method: &str, receiver: &crate::backend::vm_types::VMValue) -> crate::backend::vm_types::VMError {
    crate::backend::vm_types::VMError::InvalidInstruction(format!(
        "Method {} not supported on {}",
        method,
        vmvalue_kind(receiver)
    ))
}
