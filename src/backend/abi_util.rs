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
            // Dev-only narrow fallback: ParserBox.* 内での誤束縛により条件が BoxRef(ParserBox) になる場合、
            // gpos を 0/非0 で真偽化。環境変数 NYASH_VM_PARSERBOX_BOOL=1 のときのみ有効。
            if std::env::var("NYASH_VM_PARSERBOX_BOOL").ok().as_deref() == Some("1") {
                if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    if inst.class_name == "ParserBox" {
                        if let Some(ny) = inst.get_field_ng("gpos") {
                            let truth = match ny {
                                crate::value::NyashValue::Integer(i) => i != 0,
                                crate::value::NyashValue::Float(f) => f != 0.0,
                                crate::value::NyashValue::String(s) => !s.is_empty(),
                                _ => false,
                            };
                            return Ok(truth);
                        }
                        return Ok(false);
                    }
                }
            }
            // Default truthiness for BoxRef (align with NyashValue::to_bool semantics):
            // Arrays, Maps, Boxes are truthy (except explicit Void/nullish cases handled above).
            Ok(true)
        }
        VMValue::Float(f) => Ok(*f != 0.0),
        #[cfg(feature = "legacy-boxes")]
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
        // Treat BoxRef(VoidBox/MissingBox) as equal to Void (null) for backward compatibility
        (BoxRef(bx), Void) => {
            let is_void = bx.as_any().downcast_ref::<VoidBox>().is_some();
            #[cfg(feature = "legacy-boxes")]
            let is_missing = bx.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some();
            #[cfg(not(feature = "legacy-boxes"))]
            let is_missing = false;
            is_void || is_missing
        }
        (Void, BoxRef(bx)) => {
            let is_void = bx.as_any().downcast_ref::<VoidBox>().is_some();
            #[cfg(feature = "legacy-boxes")]
            let is_missing = bx.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some();
            #[cfg(not(feature = "legacy-boxes"))]
            let is_missing = false;
            is_void || is_missing
        }
        _ => false,
    }
}

/// Convert a VMValue to String for string concatenation semantics.
/// Development-friendly: Void→"null", BoxRef→to_string_box().value, primitives→Rust-style string.
pub fn to_string_vm(v: &VMValue) -> String {
    match v {
        VMValue::String(s) => s.clone(),
        VMValue::Integer(i) => i.to_string(),
        VMValue::Float(f) => {
            // Avoid trailing .0 noise when possible
            let s = format!("{}", f);
            s
        }
        VMValue::Bool(b) => b.to_string(),
        VMValue::Void => "null".to_string(),
        VMValue::BoxRef(bx) => bx.to_string_box().value,
        #[cfg(feature = "legacy-boxes")]
        VMValue::Future(_) => "<future>".to_string(),
    }
}

/// Obtain a human-readable tag/type name for a VMValue.
pub fn tag_of_vm(v: &VMValue) -> &'static str {
    match v {
        VMValue::Integer(_) => "Integer",
        VMValue::Float(_) => "Float",
        VMValue::Bool(_) => "Bool",
        VMValue::String(_) => "String",
        #[cfg(feature = "legacy-boxes")]
        VMValue::Future(_) => "Future",
        VMValue::Void => "Void",
        VMValue::BoxRef(_) => "BoxRef",
    }
}

/// Wrap a NyashBox object into a handle using runtime handle registry.
/// This keeps a single handle mechanism across backends.
/// ARCHIVED: JIT handle implementation moved to archive/jit-cranelift/ during Phase 15
pub fn handle_of(_boxref: Arc<dyn NyashBox>) -> Handle {
    // TODO: Implement handle registry for Phase 15 VM/LLVM backends
    // For now, use a simple 0-handle placeholder
    0
}

/// Try to resolve a handle back to a Box object.
/// ARCHIVED: JIT handle implementation moved to archive/jit-cranelift/ during Phase 15
pub fn handle_get(_h: Handle) -> Option<Arc<dyn NyashBox>> {
    // TODO: Implement handle registry for Phase 15 VM/LLVM backends
    None
}
