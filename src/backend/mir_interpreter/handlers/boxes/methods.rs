//! methods.rs — BoxCall method helpers (extracted, behavior-preserving)

use super::super::*;
use crate::box_trait::NyashBox;

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

    /// Generic toString fallback for any non-plugin box.
    pub(crate) fn to_string_fallback(
        &mut self,
        dst: Option<ValueId>,
        recv_box: &Box<dyn NyashBox>,
        method: &str,
    ) -> Option<Result<(), VMError>> {
        if method != "toString" { return None; }
        if let Some(d) = dst {
            // Map VoidBox.toString → "null" for JSON-friendly semantics
            let s = if recv_box.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                "null".to_string()
            } else {
                recv_box.to_string_box().value
            };
            self.regs.insert(d, VMValue::String(s));
        }
        Some(Ok(()))
    }

    /// Minimal InstanceBox current() fallback used by some scanners.
    pub(crate) fn instance_current_fallback(
        &mut self,
        dst: Option<ValueId>,
        recv_box: &Box<dyn NyashBox>,
        method: &str,
        args: &[ValueId],
    ) -> Option<Result<(), VMError>> {
        if method != "current" || !args.is_empty() { return None; }
        if let Some(inst) = recv_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
            if let Some(crate::value::NyashValue::Integer(pos)) = inst.get_field_ng("position") {
                if let Some(crate::value::NyashValue::String(text)) = inst.get_field_ng("text") {
                    let s = if pos < 0 || (pos as usize) >= text.len() {
                        String::new()
                    } else {
                        let bytes = text.as_bytes();
                        let i = pos as usize;
                        let j = (i + 1).min(bytes.len());
                        String::from_utf8(bytes[i..j].to_vec()).unwrap_or_default()
                    };
                    if let Some(d) = dst { self.regs.insert(d, VMValue::String(s)); }
                    return Some(Ok(()));
                }
            }
        }
        None
    }

    /// ParserBox.* context: string-like fallbacks for InstanceBox when coercion is allowed.
    pub(crate) fn parserbox_strlike_coerce(
        &mut self,
        dst: Option<ValueId>,
        recv_box: &Box<dyn NyashBox>,
        method: &str,
        args: &[ValueId],
    ) -> Option<Result<(), VMError>> {
        let cur = match &self.cur_fn { Some(s) => s, None => return None };
        if !cur.starts_with("ParserBox.") { return None; }
        if std::env::var("NYASH_VM_STRLIKE_INSTANCE_COERCE").ok().as_deref() != Some("1") {
            return None;
        }
        match method {
            "indexOf" | "lastIndexOf" => {
                let s = recv_box.to_string_box().value;
                let sub = if let Some(a0) = args.get(0) {
                    match self.reg_load(*a0) {
                        Ok(VMValue::String(t)) => t,
                        Ok(VMValue::Integer(i)) => i.to_string(),
                        Ok(VMValue::Float(f)) => format!("{}", f),
                        Ok(VMValue::Bool(b)) => b.to_string(),
                        Ok(VMValue::Void) => String::new(),
                        Ok(VMValue::BoxRef(bx)) => bx.to_string_box().value,
                        Ok(VMValue::Future(_)) | Err(_) => String::new(),
                    }
                } else { String::new() };
                let from = if let Some(a1) = args.get(1) {
                    match self.reg_load(*a1) { Ok(VMValue::Integer(i)) => i as isize, Ok(VMValue::Float(f)) => f as isize, _ => 0 }
                } else { if method == "lastIndexOf" { (s.len() as isize) - 1 } else { 0 } };
                let idx = if method == "indexOf" {
                    if sub.is_empty() { 0 } else { s.find(&sub).map(|i| i as isize).unwrap_or(-1) }
                } else {
                    if sub.is_empty() { (s.len() as isize).min(from.max(0)) } else {
                        let bound = if from < 0 { 0 } else { (from as usize + 1).min(s.len()) };
                        let slice = &s[..bound];
                        slice.rfind(&sub).map(|i| i as isize).unwrap_or(-1)
                    }
                };
                if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(idx as i64)); }
                Some(Ok(()))
            }
            "substring" => {
                let s = recv_box.to_string_box().value;
                let start = if let Some(a0) = args.get(0) {
                    self.reg_load(*a0).ok().map_or(0, |v| v.as_integer().unwrap_or(0))
                } else { 0 };
                let end = if let Some(a1) = args.get(1) {
                    self.reg_load(*a1).ok().map_or(s.len() as i64, |v| v.as_integer().unwrap_or(s.len() as i64))
                } else { s.len() as i64 };
                let len = s.len() as i64;
                let i0 = start.max(0).min(len) as usize;
                let i1 = end.max(0).min(len) as usize;
                if i0 > i1 {
                    if let Some(d) = dst { self.regs.insert(d, VMValue::String(String::new())); }
                    return Some(Ok(()));
                }
                let bytes = s.as_bytes();
                let sub = String::from_utf8(bytes[i0..i1].to_vec()).unwrap_or_default();
                if let Some(d) = dst { self.regs.insert(d, VMValue::String(sub)); }
                Some(Ok(()))
            }
            _ => None,
        }
    }
}
