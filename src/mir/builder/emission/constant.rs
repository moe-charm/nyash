//! ConstantEmissionBox — Const 命令の発行を集約（仕様不変）

use crate::mir::{ConstValue, MirInstruction, MirType, ValueId};
use crate::mir::builder::MirBuilder;

#[inline]
pub fn emit_integer(b: &mut MirBuilder, val: i64) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(val) });
    // Dev-friendly: annotate primitive type (behavior-preserving)
    b.value_types.insert(dst, MirType::Integer);
    dst
}

#[inline]
pub fn emit_bool(b: &mut MirBuilder, val: bool) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Bool(val) });
    b.value_types.insert(dst, MirType::Bool);
    dst
}

#[inline]
pub fn emit_float(b: &mut MirBuilder, val: f64) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Float(val) });
    b.value_types.insert(dst, MirType::Float);
    dst
}

#[inline]
pub fn emit_string<S: Into<String>>(b: &mut MirBuilder, s: S) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::String(s.into()) });
    b.value_types.insert(dst, MirType::String);
    dst
}

#[inline]
pub fn emit_null(b: &mut MirBuilder) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Null });
    // Null keeps Unknown type (no annotation)
    dst
}

#[inline]
pub fn emit_void(b: &mut MirBuilder) -> ValueId {
    let dst = b.safe_next_value();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Void });
    // Void keeps Unknown/Void semantics (no annotation)
    dst
}
