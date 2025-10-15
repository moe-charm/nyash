use crate::mir::{MirFunction, MirInstruction, MirType};
use crate::mir::verification_types::VerificationError;

/// Forbid Compare(Eq/Ne) when either operand is Box-typed.
pub fn check_no_box_compare(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors = Vec::new();
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let MirInstruction::Compare { op, lhs, rhs, .. } = inst {
                // Only Eq/Ne are redirected to op_eq at MIR; Lt/Le/Gt/Ge remain allowed.
                if matches!(op, crate::mir::CompareOp::Eq | crate::mir::CompareOp::Ne) {
                    let lty = function.metadata.value_types.get(lhs);
                    let rty = function.metadata.value_types.get(rhs);
                    let is_box = |t: Option<&MirType>| matches!(t, Some(MirType::Box(_)));
                    if is_box(lty) || is_box(rty) {
                        errors.push(VerificationError::BoxCompareForbidden {
                            block: *bid,
                            instruction_index: idx,
                            lhs: *lhs,
                            rhs: *rhs,
                        });
                    }
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

