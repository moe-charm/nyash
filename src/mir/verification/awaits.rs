use crate::mir::{function::MirFunction, MirInstruction};
use crate::mir::verification_types::VerificationError;

/// Ensure that each Await instruction (or ExternCall(env.future.await)) is immediately
/// preceded and followed by a checkpoint.
/// A checkpoint is either MirInstruction::Safepoint or ExternCall("env.runtime", "checkpoint").
pub fn check_await_checkpoints(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors = Vec::new();
    let is_cp = |inst: &MirInstruction| match inst {
        MirInstruction::Safepoint => true,
        MirInstruction::ExternCall { iface_name, method_name, .. } => iface_name == "env.runtime" && method_name == "checkpoint",
        _ => false,
    };
    for (bid, block) in &function.blocks {
        let instrs = &block.instructions;
        for (idx, inst) in instrs.iter().enumerate() {
            let is_await_like = match inst {
                MirInstruction::Await { .. } => true,
                MirInstruction::ExternCall { iface_name, method_name, .. } => iface_name == "env.future" && method_name == "await",
                _ => false,
            };
            if is_await_like {
                if idx == 0 || !is_cp(&instrs[idx - 1]) {
                    errors.push(VerificationError::MissingCheckpointAroundAwait { block: *bid, instruction_index: idx, position: "before" });
                }
                if idx + 1 >= instrs.len() || !is_cp(&instrs[idx + 1]) {
                    errors.push(VerificationError::MissingCheckpointAroundAwait { block: *bid, instruction_index: idx, position: "after" });
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

