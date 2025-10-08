/*!
 * Verification types (extracted from verification.rs)
 */

use super::{BasicBlockId, ValueId};

/// Verification error types
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationError {
    UndefinedValue {
        value: ValueId,
        block: BasicBlockId,
        instruction_index: usize,
    },
    MultipleDefinition {
        value: ValueId,
        first_block: BasicBlockId,
        second_block: BasicBlockId,
    },
    InvalidPhi {
        phi_value: ValueId,
        block: BasicBlockId,
        reason: String,
    },
    UnreachableBlock {
        block: BasicBlockId,
    },
    ControlFlowError {
        block: BasicBlockId,
        reason: String,
    },
    DominatorViolation {
        value: ValueId,
        use_block: BasicBlockId,
        def_block: BasicBlockId,
    },
    MergeUsesPredecessorValue {
        value: ValueId,
        merge_block: BasicBlockId,
        pred_block: BasicBlockId,
    },
    InvalidWeakRefSource {
        weak_ref: ValueId,
        block: BasicBlockId,
        instruction_index: usize,
        reason: String,
    },
    InvalidBarrierPointer {
        ptr: ValueId,
        block: BasicBlockId,
        instruction_index: usize,
        reason: String,
    },
    SuspiciousBarrierContext {
        block: BasicBlockId,
        instruction_index: usize,
        note: String,
    },
    UnsupportedLegacyInstruction {
        block: BasicBlockId,
        instruction_index: usize,
        name: String,
    },
    MissingCheckpointAroundAwait {
        block: BasicBlockId,
        instruction_index: usize,
        position: &'static str,
    },
    /// PHI-off strict policy violation (edge-copy rules)
    EdgeCopyStrictViolation {
        block: BasicBlockId,
        value: ValueId,
        pred_block: Option<BasicBlockId>,
        reason: String,
    },
    /// Forbid Compare(Eq/Ne) on Box-typed operands (semantic is fixed via op_eq at MIR)
    BoxCompareForbidden {
        block: BasicBlockId,
        instruction_index: usize,
        lhs: ValueId,
        rhs: ValueId,
    },
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationError::UndefinedValue {
                value,
                block,
                instruction_index,
            } => {
                write!(
                    f,
                    "Undefined value {} used in block {} at instruction {}",
                    value, block, instruction_index
                )
            }
            VerificationError::MultipleDefinition {
                value,
                first_block,
                second_block,
            } => {
                write!(
                    f,
                    "Value {} defined multiple times: first in block {}, again in block {}",
                    value, first_block, second_block
                )
            }
            VerificationError::InvalidPhi {
                phi_value,
                block,
                reason,
            } => {
                write!(
                    f,
                    "Invalid phi function {} in block {}: {}",
                    phi_value, block, reason
                )
            }
            VerificationError::UnreachableBlock { block } => {
                write!(f, "Unreachable block {}", block)
            }
            VerificationError::ControlFlowError { block, reason } => {
                write!(f, "Control flow error in block {}: {}", block, reason)
            }
            VerificationError::DominatorViolation {
                value,
                use_block,
                def_block,
            } => {
                write!(
                    f,
                    "Value {} used in block {} but defined in non-dominating block {}",
                    value, use_block, def_block
                )
            }
            VerificationError::MergeUsesPredecessorValue {
                value,
                merge_block,
                pred_block,
            } => {
                write!(
                    f,
                    "Merge block {} uses predecessor-defined value {} from block {} without Phi",
                    merge_block, value, pred_block
                )
            }
            VerificationError::InvalidWeakRefSource {
                weak_ref,
                block,
                instruction_index,
                reason,
            } => {
                write!(
                    f,
                    "Invalid WeakRef source {} in block {} at {}: {}",
                    weak_ref, block, instruction_index, reason
                )
            }
            VerificationError::InvalidBarrierPointer {
                ptr,
                block,
                instruction_index,
                reason,
            } => {
                write!(
                    f,
                    "Invalid Barrier pointer {} in block {} at {}: {}",
                    ptr, block, instruction_index, reason
                )
            }
            VerificationError::SuspiciousBarrierContext {
                block,
                instruction_index,
                note,
            } => {
                write!(
                    f,
                    "Suspicious Barrier context in block {} at {}: {}",
                    block, instruction_index, note
                )
            }
            VerificationError::UnsupportedLegacyInstruction {
                block,
                instruction_index,
                name,
            } => {
                write!(
                    f,
                    "Unsupported legacy instruction '{}' in block {} at {} (enable rewrite passes)",
                    name, block, instruction_index
                )
            }
            VerificationError::MissingCheckpointAroundAwait {
                block,
                instruction_index,
                position,
            } => {
                write!(
                    f,
                    "Missing {} checkpoint around await in block {} at instruction {}",
                    position, block, instruction_index
                )
            }
            VerificationError::EdgeCopyStrictViolation { block, value, pred_block, reason } => {
                if let Some(pb) = pred_block {
                    write!(
                        f,
                        "EdgeCopyStrictViolation for value {} at merge block {} from pred {}: {}",
                        value, block, pb, reason
                    )
                } else {
                    write!(
                        f,
                        "EdgeCopyStrictViolation for value {} at merge block {}: {}",
                        value, block, reason
                    )
                }
            }
            VerificationError::BoxCompareForbidden { block, instruction_index, lhs, rhs } => {
                write!(
                    f,
                    "Box Compare forbidden in block {} at {} (lhs=%{:?}, rhs=%{:?}); use nyrt.ops.op_eq via MirCall::Extern",
                    block, instruction_index, lhs, rhs
                )
            }
        }
    }
}
