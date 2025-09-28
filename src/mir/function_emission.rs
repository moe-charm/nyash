//! FunctionEmissionBox — MirFunction 直編集時の発行ヘルパ（仕様不変・dev補助）
//!
//! MirBuilder 経由ではなく MirFunction/BasicBlock を直接編集する箇所（dev 補助）向けに、
//! よく使う Const/Return/Jump の発行を薄い関数で提供する。

use crate::mir::{BasicBlockId, ConstValue, MirFunction, MirInstruction, ValueId};

#[inline]
pub fn emit_const_integer(f: &mut MirFunction, bb: BasicBlockId, val: i64) -> ValueId {
    let dst = f.next_value_id();
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(val) });
    }
    dst
}

#[inline]
pub fn emit_const_bool(f: &mut MirFunction, bb: BasicBlockId, val: bool) -> ValueId {
    let dst = f.next_value_id();
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Const { dst, value: ConstValue::Bool(val) });
    }
    dst
}

#[inline]
pub fn emit_const_string<S: Into<String>>(f: &mut MirFunction, bb: BasicBlockId, s: S) -> ValueId {
    let dst = f.next_value_id();
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Const { dst, value: ConstValue::String(s.into()) });
    }
    dst
}

#[inline]
pub fn emit_const_void(f: &mut MirFunction, bb: BasicBlockId) -> ValueId {
    let dst = f.next_value_id();
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Const { dst, value: ConstValue::Void });
    }
    dst
}

#[inline]
pub fn emit_return_value(f: &mut MirFunction, bb: BasicBlockId, value: ValueId) {
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Return { value: Some(value) });
    }
}

#[inline]
pub fn emit_jump(f: &mut MirFunction, bb: BasicBlockId, target: BasicBlockId) {
    if let Some(block) = f.get_block_mut(bb) {
        block.add_instruction(MirInstruction::Jump { target });
    }
}

