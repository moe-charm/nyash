// Op Handlers - Operator implementations for nyrt.ops.*
//
// Consolidates operator logic previously duplicated across:
// - handlers/externals.rs
// - extern_adapter.rs

use crate::backend::mir_interpreter::VMValue;
use crate::backend::vm_types::VMError;

/// Handle op_eq(a, b) with user-defined equals() support
///
/// Equality semantics:
/// 1. Fast path: primitive equality (Integer, Float, Bool, String, Void)
/// 2. Box pointer equality check
/// 3. User-defined equals() dispatch for InstanceBox
/// 4. Fallback: not equal
///
/// # Recursion Prevention
/// User-defined equals() calls are made with CallMode::NoOperatorGuard
/// to prevent infinite recursion.
pub fn op_eq_static(a: &VMValue, b: &VMValue) -> Result<VMValue, VMError> {
    use VMValue::*;

    match (a, b) {
        // Fast path: primitives
        (Integer(x), Integer(y)) => Ok(Bool(x == y)),
        (Float(x), Float(y)) => Ok(Bool(x == y)),
        (Bool(x), Bool(y)) => Ok(Bool(x == y)),
        (String(x), String(y)) => Ok(Bool(x == y)),
        (Void, Void) => Ok(Bool(true)),

        // Box equality: pointer check only (no user-defined equals() support)
        // For full user-defined equals(), use op_eq_with_interpreter()
        (BoxRef(ax), BoxRef(bx)) => {
            use std::sync::Arc;
            Ok(Bool(Arc::ptr_eq(ax, bx)))
        }

        // Cross-type: false
        _ => Ok(Bool(false)),
    }
}

/// Handle op_eq(a, b) with full user-defined equals() support
///
/// This version requires access to MirInterpreter for calling user methods.
/// Used by handlers/externals.rs.
pub fn op_eq_with_interpreter<F>(
    a: &VMValue,
    b: &VMValue,
    call_user_equals: F,
) -> Result<VMValue, VMError>
where
    F: FnOnce(&str, Vec<VMValue>) -> Result<VMValue, VMError>,
{
    use VMValue::*;

    match (a, b) {
        // Fast path: primitives
        (Integer(x), Integer(y)) => Ok(Bool(x == y)),
        (Float(x), Float(y)) => Ok(Bool(x == y)),
        (Bool(x), Bool(y)) => Ok(Bool(x == y)),
        (String(x), String(y)) => Ok(Bool(x == y)),
        (Void, Void) => Ok(Bool(true)),

        // Box equality: pointer check first
        (BoxRef(ax), BoxRef(bx)) => {
            use std::sync::Arc;
            if Arc::ptr_eq(ax, bx) {
                return Ok(Bool(true));
            }

            // Try user-defined equals() for InstanceBox
            if let Some(inst) = ax.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                let class_name = &inst.class_name;
                let equals_sig = format!("{}.equals/1", class_name);

                // Call user-defined equals() with NoOperatorGuard
                let argv = vec![a.clone(), b.clone()];
                let result = call_user_equals(&equals_sig, argv)?;

                return match crate::backend::abi_util::to_bool_vm(&result) {
                    Ok(b) => Ok(Bool(b)),
                    Err(e) => Err(VMError::TypeError(e)),
                };
            }

            // Fallback: not equal
            Ok(Bool(false))
        }

        // Cross-type: false
        _ => Ok(Bool(false)),
    }
}
