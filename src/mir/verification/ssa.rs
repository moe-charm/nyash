use crate::mir::function::MirFunction;
use crate::mir::{ValueId};
use crate::mir::verification_types::VerificationError;

/// Verify SSA form: single assignment and all uses defined
pub fn check_ssa_form(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    use std::collections::HashMap;
    let mut errors = Vec::new();
    let mut definitions: HashMap<ValueId, (crate::mir::BasicBlockId, usize)> = HashMap::new();

    for (block_id, block) in &function.blocks {
        for (inst_idx, instruction) in block.all_instructions().enumerate() {
            if let Some(dst) = instruction.dst_value() {
                if let Some((first_block, _)) = definitions.insert(dst, (*block_id, inst_idx)) {
                    errors.push(VerificationError::MultipleDefinition {
                        value: dst,
                        first_block,
                        second_block: *block_id,
                    });
                }
            }
        }
    }

    for (block_id, block) in &function.blocks {
        for (inst_idx, instruction) in block.all_instructions().enumerate() {
            for used_value in instruction.used_values() {
                if !definitions.contains_key(&used_value) {
                    errors.push(VerificationError::UndefinedValue {
                        value: used_value,
                        block: *block_id,
                        instruction_index: inst_idx,
                    });
                }
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

