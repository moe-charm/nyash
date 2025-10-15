// Op Handlers - Operator implementations for nyrt.ops.*
//
// Phase 1 Cleanup (2025-10-11):
// - ✅ op_eq_with_interpreter() removed (52 lines) - unused after handle_op_eq removal
// - ✅ op_eq_static() retained - used by extern_adapter.rs
//
// Note: User-defined equals() support now handled via MIR builder's Callee::Extern path.

use crate::backend::mir_interpreter::VMValue;
use crate::backend::vm_types::VMError;

/// Handle op_eq(a, b) - static equality check
///
/// Equality semantics:
/// 1. Fast path: primitive equality (Integer, Float, Bool, String, Void)
/// 2. Box pointer equality check (Arc::ptr_eq only)
/// 3. Cross-type: false
///
/// Note: User-defined equals() for InstanceBox is handled via MIR builder's
/// Callee::Extern("nyrt.ops.op_eq") path, not this function.
pub fn op_eq_static(a: &VMValue, b: &VMValue) -> Result<VMValue, VMError> {
    use VMValue::*;

    match (a, b) {
        // Fast path: primitives
        (Integer(x), Integer(y)) => Ok(Bool(x == y)),
        (Float(x), Float(y)) => Ok(Bool(x == y)),
        (Bool(x), Bool(y)) => Ok(Bool(x == y)),
        (String(x), String(y)) => Ok(Bool(x == y)),
        (Void, Void) => Ok(Bool(true)),

        // Box equality: pointer check only
        (BoxRef(ax), BoxRef(bx)) => {
            use std::sync::Arc;
            Ok(Bool(Arc::ptr_eq(ax, bx)))
        }

        // Cross-type: false
        _ => Ok(Bool(false)),
    }
}
