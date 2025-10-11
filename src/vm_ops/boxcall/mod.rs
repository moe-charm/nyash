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
