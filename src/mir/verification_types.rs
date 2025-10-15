/*!
 * Verification types (extracted from verification.rs)
 */

use super::{BasicBlockId, ValueId};
use crate::mir::MirType;

/// Verification error types
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationError {
    UndefinedValue {
        value: ValueId,
        block: BasicBlockId,
        instruction_index: usize,
    },
    /// Unified Call must specify typed callee (legacy func-only form is forbidden)
    LegacyCallMissingCallee {
        block: BasicBlockId,
        instruction_index: usize,
    },
    /// Method calls must have an explicit receiver (Known certainty)
    MethodReceiverMissing {
        block: BasicBlockId,
       instruction_index: usize,
       method: String,
    },
    /// Static ModuleFunction 呼び出しなのに receiver を渡していない
    ModuleFunctionStaticReceiverMissing {
        block: BasicBlockId,
        instruction_index: usize,
        function: String,
        expected_args: usize,
        actual_args: usize,
    },
    /// Static ModuleFunction の先頭引数が Box(Type) になっていない
    ModuleFunctionStaticReceiverTypeMismatch {
        block: BasicBlockId,
        instruction_index: usize,
        function: String,
        expected_box: String,
        actual_type: Option<MirType>,
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
    /// Forbid getField/setField on a constant Name (static box self) — indicates invalid static field semantics
    StaticSelfFieldForbidden {
        block: BasicBlockId,
        instruction_index: usize,
        method: String,
    },
    /// Method(StringBox.size/len/length) must have an in-block receiver Copy right before call (dev-gated)
    MethodReceiverMissingLocalCopy {
        block: BasicBlockId,
        instruction_index: usize,
        method: String,
        receiver: ValueId,
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
            VerificationError::ModuleFunctionStaticReceiverMissing {
                block,
                instruction_index,
                function,
                expected_args,
                actual_args,
            } => {
                write!(
                    f,
                    "ModuleFunction call '{}' in block {} at {} is missing static receiver: expected {} args (including singleton), found {}",
                    function, block, instruction_index, expected_args, actual_args
                )
            }
            VerificationError::ModuleFunctionStaticReceiverTypeMismatch {
                block,
                instruction_index,
                function,
                expected_box,
                actual_type,
            } => {
                let actual_desc = actual_type
                    .as_ref()
                    .map(|ty| format!("{:?}", ty))
                    .unwrap_or_else(|| "unknown".to_string());
                write!(
                    f,
                    "ModuleFunction call '{}' in block {} at {} has receiver type mismatch: expected Box({}), found {}",
                    function, block, instruction_index, expected_box, actual_desc
                )
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
            VerificationError::StaticSelfFieldForbidden { block, instruction_index, method } => {
                write!(
                    f,
                    "Static box field access is forbidden in block {} at {} via {} (use methods or instance boxes)",
                    block, instruction_index, method
                )
            }
            VerificationError::LegacyCallMissingCallee { block, instruction_index } => {
                write!(
                    f,
                    "Unified Call missing typed callee in block {} at {} (use MirCall with callee=...)",
                    block, instruction_index
                )
            }
            VerificationError::MethodReceiverMissing { block, instruction_index, method } => {
                write!(
                    f,
                    "Method receiver missing in block {} at {} for {} (receiver must be explicit when Known)",
                    block, instruction_index, method
                )
            }
            VerificationError::MethodReceiverMissingLocalCopy { block, instruction_index, method, receiver } => {
                write!(
                    f,
                    "String receiver not materialized locally in block {} at {} for {} (recv=%{:?} needs in-block Copy)",
                    block, instruction_index, method, receiver
                )
            }
        }
    }
}
