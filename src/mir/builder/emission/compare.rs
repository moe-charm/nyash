//! CompareEmissionBox — 比較命令発行の薄いヘルパ（仕様不変）

use crate::mir::{CompareOp, MirInstruction, MirType, ValueId};
use crate::mir::builder::MirBuilder;

#[inline]
pub fn emit_to(b: &mut MirBuilder, dst: ValueId, op: CompareOp, lhs: ValueId, rhs: ValueId) -> Result<(), String> {
    b.emit_instruction(MirInstruction::Compare { dst, op, lhs, rhs })?;
    // 比較結果は Bool 型（既存実装と同じ振る舞い）
    b.value_types.insert(dst, MirType::Bool);
    Ok(())
}

