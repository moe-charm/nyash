/*!
 * Backend ABI/utility consolidation (minimal)
 *
 * Shared helpers for handle/ptr/to_bool/compare/tag/invoke scaffolding.
 * Initial scope focuses on value coercions used by the MIR interpreter and JIT.
 */

use crate::backend::vm::VMValue;
use crate::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox, VoidBox};
use std::sync::Arc;

/// Opaque handle type used by JIT/runtime bridges.
pub type Handle = u64;

/// Convert a VMValue to boolean using unified, permissive semantics.
pub fn to_bool_vm(v: &VMValue) -> Result<bool, String> {
    match v {
        VMValue::Bool(b) => Ok(*b),
        VMValue::Integer(i) => Ok(*i != 0),
        VMValue::Void => Ok(false),
        VMValue::String(s) => Ok(!s.is_empty()),
        VMValue::BoxRef(b) => {
            if let Some(bb) = b.as_any().downcast_ref::<BoolBox>() {
                return Ok(bb.value);
            }
            if let Some(ib) = b.as_any().downcast_ref::<IntegerBox>() {
                return Ok(ib.value != 0);
            }
            if let Some(sb) = b.as_any().downcast_ref::<StringBox>() {
                return Ok(!sb.value.is_empty());
            }
            if b.as_any().downcast_ref::<VoidBox>().is_some() {
                return Ok(false);
            }
            Err(format!("cannot coerce BoxRef({}) to bool", b.type_name()))
        }
        VMValue::Float(f) => Ok(*f != 0.0),
        VMValue::Future(_) => Err("cannot coerce Future to bool".to_string()),
    }
}

/// Nyash-style equality on VMValue (best-effort for core primitives).
pub fn eq_vm(a: &VMValue, b: &VMValue) -> bool {
    use VMValue::*;
    match (a, b) {
        (Integer(x), Integer(y)) => x == y,
        (Float(x), Float(y)) => x == y,
        (Bool(x), Bool(y)) => x == y,
        (String(x), String(y)) => x == y,
        (Void, Void) => true,
        // Cross-kind simple coercions commonly used in MIR compare
        (Integer(x), Bool(y)) | (Bool(y), Integer(x)) => (*x != 0) == *y,
        (Integer(x), Float(y)) => (*x as f64) == *y,
        (Float(x), Integer(y)) => *x == (*y as f64),
        (BoxRef(ax), BoxRef(by)) => Arc::ptr_eq(ax, by),
        _ => false,
    }
}

/// Obtain a human-readable tag/type name for a VMValue.
pub fn tag_of_vm(v: &VMValue) -> &'static str {
    match v {
        VMValue::Integer(_) => "Integer",
        VMValue::Float(_) => "Float",
        VMValue::Bool(_) => "Bool",
        VMValue::String(_) => "String",
        VMValue::Future(_) => "Future",
        VMValue::Void => "Void",
        VMValue::BoxRef(_) => "BoxRef",
    }
}

/// Wrap a NyashBox object into a handle using JIT handle registry.
/// This keeps a single handle mechanism across backends.
pub fn handle_of(boxref: Arc<dyn NyashBox>) -> Handle {
    crate::jit::rt::handles::to_handle(boxref)
}

/// Try to resolve a handle back to a Box object.
pub fn handle_get(h: Handle) -> Option<Arc<dyn NyashBox>> {
    crate::jit::rt::handles::get(h)
}
