//! Core ops lowering (non-hostcall): BinOp, Compare, Branch, Jump
use super::builder::{IRBuilder, BinOpKind, CmpKind};
use crate::mir::{BinaryOp, CompareOp, ValueId};

use super::core::LowerCore;

impl LowerCore {
    pub fn lower_binop(&mut self, b: &mut dyn IRBuilder, op: &BinaryOp, lhs: &ValueId, rhs: &ValueId, dst: &ValueId) {
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

    pub fn lower_compare(&mut self, b: &mut dyn IRBuilder, op: &CompareOp, lhs: &ValueId, rhs: &ValueId, dst: &ValueId) {
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
