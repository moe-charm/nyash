use crate::mir::{MirFunction, MirInstruction};
use crate::mir::definitions::call_unified::Callee;
use crate::mir::verification_types::VerificationError;

/// Verify that StringBox.(size|len|length) method calls have an in-block receiver copy
/// immediately before the call (Fail-Fast when env gate enabled).
pub fn check_string_len_receiver_materialized(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors = Vec::new();
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let MirInstruction::Call { callee: Some(Callee::Method { method, receiver: Some(r), .. }), args, .. } = inst {
                if (method == "size" || method == "len" || method == "length") && args.is_empty() {
                    // Check if previous instruction is Copy to this receiver id
                    let ok = if idx == 0 { false } else { matches!(block.instructions[idx-1], MirInstruction::Copy{ dst, src: _ } if dst == *r) };
                    if !ok {
                        errors.push(VerificationError::MethodReceiverMissingLocalCopy {
                            block: *bid,
                            instruction_index: idx,
                            method: method.clone(),
                            receiver: *r,
                        });
                    }
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

