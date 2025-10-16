use crate::mir::MirFunction;
use crate::mir::verification_types::VerificationError;

pub fn check_no_parameter_reassignment(
    function: &MirFunction,
) -> Result<(), Vec<VerificationError>> {
    if function.params.is_empty() {
        return Ok(());
    }
    let mut errors = Vec::new();
    for (bb_id, block) in &function.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let Some(dst) = inst.dst_value() {
                if function.params.contains(&dst) {
                    errors.push(VerificationError::ParameterReassigned {
                        value: dst,
                        block: *bb_id,
                        instruction_index: idx,
                    });
                }
            }
        }
        if let Some(term) = &block.terminator {
            if let Some(dst) = term.dst_value() {
                if function.params.contains(&dst) {
                    errors.push(VerificationError::ParameterReassigned {
                        value: dst,
                        block: *bb_id,
                        instruction_index: block.instructions.len(),
                    });
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
