//! BranchEmissionBox — 分岐/ジャンプ命令発行の薄いヘルパ（仕様不変）

use crate::mir::{BasicBlockId, MirInstruction};
use crate::mir::builder::MirBuilder;

#[inline]
pub fn emit_conditional(b: &mut MirBuilder, cond: crate::mir::ValueId, then_bb: BasicBlockId, else_bb: BasicBlockId) -> Result<(), String> {
    b.emit_instruction(MirInstruction::Branch { condition: cond, then_bb, else_bb })
}

#[inline]
pub fn emit_jump(b: &mut MirBuilder, target: BasicBlockId) -> Result<(), String> {
    b.emit_instruction(MirInstruction::Jump { target })
}

