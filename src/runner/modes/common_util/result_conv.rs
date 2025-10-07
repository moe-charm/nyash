//! Result type conversion utilities
//!
//! Unified helper for converting Box results to string representations
//! based on MIR type information. Consolidates duplicated logic from
//! vm.rs and mir_interpreter.rs.

use nyash_rust::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox};
use nyash_rust::boxes::FloatBox;
use nyash_rust::mir::MirType;

/// Convert a Box result to (type_name, string_value) tuple based on MIR type
///
/// Parameters:
/// - `result`: The Box result to convert
/// - `mir_type`: Expected MIR type (from function signature)
/// - `use_coercion`: Whether to use coercion fallbacks (vm.rs needs this)
///
/// Returns: (type_name, string_representation)
pub fn convert_box_result_to_string(
    result: &dyn NyashBox,
    mir_type: &MirType,
    use_coercion: bool,
) -> (&'static str, String) {
    match mir_type {
        MirType::Float => {
            if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                ("Float", format!("{}", fb.value))
            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                ("Float", format!("{}", ib.value as f64))
            } else if use_coercion {
                if let Some(s) = nyash_rust::runtime::semantics::coerce_to_string(result) {
                    ("String", s)
                } else {
                    (result.type_name(), result.to_string_box().value)
                }
            } else {
                ("Float", result.to_string_box().value)
            }
        }
        MirType::Integer => {
            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                ("Integer", ib.value.to_string())
            } else if use_coercion {
                if let Some(i) = nyash_rust::runtime::semantics::coerce_to_i64(result) {
                    ("Integer", i.to_string())
                } else {
                    (result.type_name(), result.to_string_box().value)
                }
            } else {
                ("Integer", result.to_string_box().value)
            }
        }
        MirType::Bool => {
            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                ("Bool", bb.value.to_string())
            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                ("Bool", (ib.value != 0).to_string())
            } else {
                ("Bool", result.to_string_box().value)
            }
        }
        MirType::String => {
            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                ("String", sb.value.clone())
            } else if use_coercion {
                if let Some(s) = nyash_rust::runtime::semantics::coerce_to_string(result) {
                    ("String", s)
                } else {
                    (result.type_name(), result.to_string_box().value)
                }
            } else {
                ("String", result.to_string_box().value)
            }
        }
        _ => {
            if use_coercion {
                if let Some(i) = nyash_rust::runtime::semantics::coerce_to_i64(result) {
                    ("Integer", i.to_string())
                } else if let Some(s) = nyash_rust::runtime::semantics::coerce_to_string(result) {
                    ("String", s)
                } else {
                    (result.type_name(), result.to_string_box().value)
                }
            } else {
                (result.type_name(), result.to_string_box().value)
            }
        }
    }
}

/// Fallback conversion without type hint (always uses coercion)
pub fn convert_box_result_fallback(result: &dyn NyashBox) -> (&'static str, String) {
    if let Some(i) = nyash_rust::runtime::semantics::coerce_to_i64(result) {
        ("Integer", i.to_string())
    } else if let Some(s) = nyash_rust::runtime::semantics::coerce_to_string(result) {
        ("String", s)
    } else {
        (result.type_name(), result.to_string_box().value)
    }
}
