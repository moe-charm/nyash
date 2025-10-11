//! methods.rs — BoxCall method helpers (extracted, behavior-preserving)

use super::super::*;

impl MirInterpreter {
    /// Void/VoidBox guard for common short‑circuit patterns.
    /// If handled, returns Some(Result<()>). Otherwise None.
    pub(crate) fn boxcall_void_guard_defaults(
        &mut self,
        dst: Option<ValueId>,
        recv: &VMValue,
        method: &str,
    ) -> Option<Result<(), VMError>> {
        // Treat both VMValue::Void and BoxRef(VoidBox) equally
        let is_void_like = match recv {
            VMValue::Void => true,
            VMValue::BoxRef(b) => b.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some(),
            _ => false,
        };
        if !is_void_like { return None; }

        let ret = match method {
            // booleans
            "is_eof" => Some(VMValue::Bool(true)),
            "starts_with" | "match_string" => Some(VMValue::Bool(false)),
            "is_whitespace_char" | "is_digit_char" | "is_hex_digit_char" | "is_alpha_char" | "is_alphanumeric_or_underscore" => Some(VMValue::Bool(false)),
            // integers
            "length" => Some(VMValue::Integer(0)),
            "indexOf" | "lastIndexOf" => Some(VMValue::Integer(-1)),
            "get_position" => Some(VMValue::Integer(0)),
            "get_line" => Some(VMValue::Integer(1)),
            "get_column" => Some(VMValue::Integer(1)),
            // strings
            "substring" | "current" | "peek" | "peek_at" | "advance" | "read_while" => Some(VMValue::String(String::new())),
            // voids
            "advance_by" | "skip_whitespace" | "push" => Some(VMValue::Void),
            _ => None,
        };
        if let Some(v) = ret {
            if let Some(d) = dst { self.regs.insert(d, v); }
            return Some(Ok(()));
        }
        None
    }
}
