//! Core ops lowering (non-hostcall): BinOp, Compare, Branch, Jump
use super::builder::{IRBuilder, BinOpKind, CmpKind};
use crate::mir::{BinaryOp, CompareOp, ValueId, MirFunction, MirType};

use super::core::LowerCore;

impl LowerCore {
    fn is_string_like(&self, func: &MirFunction, v: &ValueId) -> bool {
        // Check per-value type metadata
        if let Some(mt) = func.metadata.value_types.get(v) {
            if matches!(mt, MirType::String) { return true; }
            if let MirType::Box(ref name) = mt { if name == "StringBox" { return true; } }
        }
        // Check if this value is a parameter with String or StringBox type
        if let Some(pidx) = self.param_index.get(v).copied() {
            if let Some(pt) = func.signature.params.get(pidx) {
                if matches!(pt, MirType::String) { return true; }
                if let MirType::Box(ref name) = pt { if name == "StringBox" { return true; } }
            }
        }
        // Check if it originates from a StringBox NewBox
        if let Some(name) = self.box_type_map.get(v) { if name == "StringBox" { return true; } }
        false
    }

    pub fn lower_binop(&mut self, b: &mut dyn IRBuilder, op: &BinaryOp, lhs: &ValueId, rhs: &ValueId, dst: &ValueId, func: &MirFunction) {
        // Route string-like addition to hostcall (handle,handle)
        if crate::jit::config::current().hostcall {
            if matches!(op, BinaryOp::Add) {
                if self.is_string_like(func, lhs) || self.is_string_like(func, rhs) {
                    self.push_value_if_known_or_param(b, lhs);
                    self.push_value_if_known_or_param(b, rhs);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_CONCAT_HH, 2, true);
                    // Track handle result for downstream usages
                    self.handle_values.insert(*dst);
                    let slot = *self.local_index.entry(*dst).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                    b.store_local_i64(slot);
                    return;
                }
            }
        }
        self.push_value_if_known_or_param(b, lhs);
        self.push_value_if_known_or_param(b, rhs);
        let kind = match op {
            BinaryOp::Add => BinOpKind::Add,
            BinaryOp::Sub => BinOpKind::Sub,
            BinaryOp::Mul => BinOpKind::Mul,
            BinaryOp::Div => BinOpKind::Div,
            BinaryOp::Mod => BinOpKind::Mod,
            _ => { return; }
        };
        b.emit_binop(kind);
        if let (Some(a), Some(bv)) = (self.known_i64.get(lhs), self.known_i64.get(rhs)) {
            let res = match op {
                BinaryOp::Add => a.wrapping_add(*bv),
                BinaryOp::Sub => a.wrapping_sub(*bv),
                BinaryOp::Mul => a.wrapping_mul(*bv),
                BinaryOp::Div => if *bv != 0 { a.wrapping_div(*bv) } else { 0 },
                BinaryOp::Mod => if *bv != 0 { a.wrapping_rem(*bv) } else { 0 },
                _ => 0,
            };
            self.known_i64.insert(*dst, res);
        }
    }

    pub fn lower_compare(&mut self, b: &mut dyn IRBuilder, op: &CompareOp, lhs: &ValueId, rhs: &ValueId, dst: &ValueId, func: &MirFunction) {
        // Route string-like comparisons (Eq/Lt) to hostcalls (i64 0/1)
        if crate::jit::config::current().hostcall {
            if matches!(op, CompareOp::Eq | CompareOp::Lt) {
                if self.is_string_like(func, lhs) || self.is_string_like(func, rhs) {
                    self.push_value_if_known_or_param(b, lhs);
                    self.push_value_if_known_or_param(b, rhs);
                    let sym = match op { CompareOp::Eq => crate::jit::r#extern::collections::SYM_STRING_EQ_HH, CompareOp::Lt => crate::jit::r#extern::collections::SYM_STRING_LT_HH, _ => unreachable!() };
                    b.emit_host_call(sym, 2, true);
                    self.bool_values.insert(*dst);
                    return;
                }
            }
        }
        self.push_value_if_known_or_param(b, lhs);
        self.push_value_if_known_or_param(b, rhs);
        let kind = match op {
            CompareOp::Eq => CmpKind::Eq,
            CompareOp::Ne => CmpKind::Ne,
            CompareOp::Lt => CmpKind::Lt,
            CompareOp::Le => CmpKind::Le,
            CompareOp::Gt => CmpKind::Gt,
            CompareOp::Ge => CmpKind::Ge,
        };
        b.emit_compare(kind);
        self.bool_values.insert(*dst);
    }

    pub fn lower_jump(&mut self, b: &mut dyn IRBuilder) { b.emit_jump(); }
    pub fn lower_branch(&mut self, b: &mut dyn IRBuilder) { b.emit_branch(); }
}
