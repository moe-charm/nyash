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
}
