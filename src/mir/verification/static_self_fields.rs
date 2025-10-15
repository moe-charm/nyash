use crate::mir::{MirFunction, MirInstruction};
use crate::mir::verification_types::VerificationError;

/// Verify that BoxCall getField/setField is not performed on a constant string receiver
/// This typically indicates `me` lowered to a NameConst (static box self), which is unsupported.
pub fn check_no_static_self_field_calls(function: &MirFunction) -> Result<(), Vec<VerificationError>> {
    let mut errors: Vec<VerificationError> = Vec::new();

    // Build a quick lookup of const-string values: ValueId -> Some(&str)
    // We scan on the fly for each use to keep this pass simple and local.
    for (bid, block) in &function.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let MirInstruction::BoxCall { box_val, method, .. } = inst {
                if method == "getField" || method == "setField" {
                    if is_const_string_value(function, *box_val) {
                        errors.push(VerificationError::StaticSelfFieldForbidden {
                            block: *bid,
                            instruction_index: idx,
                            method: method.clone(),
                        });
                    }
                }
            }
        }
        // Also inspect terminator, though BoxCall should not appear there normally
        if let Some(term) = &block.terminator {
            if let MirInstruction::BoxCall { box_val, method, .. } = term {
                if method == "getField" || method == "setField" {
                    if is_const_string_value(function, *box_val) {
                        errors.push(VerificationError::StaticSelfFieldForbidden {
                            block: *bid,
                            instruction_index: block.instructions.len(),
                            method: method.clone(),
                        });
                    }
                }
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn is_const_string_value(function: &MirFunction, vid: crate::mir::ValueId) -> bool {
    for (_bid, bb) in &function.blocks {
        for inst in bb.all_instructions() {
            if let MirInstruction::Const { dst, value } = inst {
                if *dst == vid {
                    return matches!(value, crate::mir::types::ConstValue::String(_));
                }
            }
        }
    }
    false
}

