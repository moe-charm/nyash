use crate::mir::{MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp};
use super::builder::{IRBuilder, BinOpKind, CmpKind};

/// Lower(Core-1): Minimal lowering skeleton for Const/Move/BinOp/Cmp/Branch/Ret
/// This does not emit real CLIF yet; it only walks MIR and validates coverage.
pub struct LowerCore {
    pub unsupported: usize,
    pub covered: usize,
}

impl LowerCore {
    pub fn new() -> Self { Self { unsupported: 0, covered: 0 } }

    /// Walk the MIR function and count supported/unsupported instructions.
    /// In the future, this will build CLIF via Cranelift builders.
    pub fn lower_function(&mut self, func: &MirFunction, builder: &mut dyn IRBuilder) -> Result<(), String> {
        builder.begin_function(&func.signature.name);
        for (_bb_id, bb) in func.blocks.iter() {
            for instr in bb.instructions.iter() {
                self.cover_if_supported(instr);
                self.try_emit(builder, instr);
            }
            if let Some(term) = &bb.terminator {
                self.cover_if_supported(term);
                self.try_emit(builder, term);
            }
        }
        builder.end_function();
        Ok(())
    }

    fn cover_if_supported(&mut self, instr: &MirInstruction) {
        use crate::mir::MirInstruction as I;
        let supported = matches!(
            instr,
            I::Const { .. }
                | I::Copy { .. }
                | I::BinOp { .. }
                | I::Compare { .. }
                | I::Jump { .. }
                | I::Branch { .. }
                | I::Return { .. }
                | I::ArrayGet { .. }
                | I::ArraySet { .. }
        );
        if supported { self.covered += 1; } else { self.unsupported += 1; }
    }

    fn try_emit(&mut self, b: &mut dyn IRBuilder, instr: &MirInstruction) {
        use crate::mir::MirInstruction as I;
        match instr {
            I::Const { value, .. } => match value {
                ConstValue::Integer(i) => b.emit_const_i64(*i),
                ConstValue::Float(f) => b.emit_const_f64(*f),
                ConstValue::Bool(_)
                | ConstValue::String(_) | ConstValue::Null | ConstValue::Void => {
                    // leave unsupported for now
                }
            },
            I::Copy { .. } => { /* no-op for now */ }
            I::BinOp { op, .. } => {
                let kind = match op {
                    BinaryOp::Add => BinOpKind::Add,
                    BinaryOp::Sub => BinOpKind::Sub,
                    BinaryOp::Mul => BinOpKind::Mul,
                    BinaryOp::Div => BinOpKind::Div,
                    BinaryOp::Mod => BinOpKind::Mod,
                    // Not yet supported in Core-1
                    BinaryOp::And | BinaryOp::Or
                    | BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => { return; }
                };
                b.emit_binop(kind);
            }
            I::Compare { op, .. } => {
                let kind = match op {
                    CompareOp::Eq => CmpKind::Eq,
                    CompareOp::Ne => CmpKind::Ne,
                    CompareOp::Lt => CmpKind::Lt,
                    CompareOp::Le => CmpKind::Le,
                    CompareOp::Gt => CmpKind::Gt,
                    CompareOp::Ge => CmpKind::Ge,
                };
                b.emit_compare(kind);
            }
            I::Jump { .. } => b.emit_jump(),
            I::Branch { .. } => b.emit_branch(),
            I::Return { .. } => b.emit_return(),
            I::ArrayGet { .. } => {
                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET, 2, true);
            }
            I::ArraySet { .. } => {
                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET, 3, false);
            }
            _ => {}
        }
    }
}
