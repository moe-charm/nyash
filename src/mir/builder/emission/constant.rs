//! ConstantEmissionBox — Const 命令の発行を集約（仕様不変）

use crate::mir::{ConstValue, MirInstruction, ValueId};
use crate::mir::builder::MirBuilder;

#[inline]
pub fn emit_integer(b: &mut MirBuilder, val: i64) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(val) });
    dst
}

#[inline]
pub fn emit_bool(b: &mut MirBuilder, val: bool) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Bool(val) });
    dst
}

#[inline]
pub fn emit_float(b: &mut MirBuilder, val: f64) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Float(val) });
    dst
}

#[inline]
pub fn emit_string<S: Into<String>>(b: &mut MirBuilder, s: S) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::String(s.into()) });
    dst
}

#[inline]
pub fn emit_null(b: &mut MirBuilder) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Null });
    dst
}

#[inline]
pub fn emit_void(b: &mut MirBuilder) -> ValueId {
    let dst = b.value_gen.next();
    let _ = b.emit_instruction(MirInstruction::Const { dst, value: ConstValue::Void });
    dst
}
