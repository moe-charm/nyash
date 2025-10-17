//! LoopFormVerifierBox — Loop structure verification for PHI bug prevention
//!
//! **Purpose**: Verify loop structure conformance to prevent PHI bugs
//!
//! **Verification Rules**:
//! 1. **PHI Placement**: PHI instructions must be at block start
//! 2. **Header Structure**: Header = PHI group + (Jump|Branch) only
//! 3. **No Variable Binding**: Header prohibits Const/Copy/BinOp/etc (except PHI)
//!
//! **Integration with LoopFormBox**:
//! - Called from LoopFormBox::verify_structure() (Step 6)
//! - Fail-Fast: Returns error immediately on rule violation
//! - Trace support: HAKO_TRACE_LOOPFORM / NYASH_TRACE_LOOPFORM

use crate::mir::{BasicBlock, BasicBlockId, MirBuilder, MirFunction, MirInstruction};

/// LoopForm verification result
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Verification passed
    Ok,
    /// Rule 1 violation: PHI not at block start
    PhiPlacementViolation {
        block_id: BasicBlockId,
        phi_index: usize,
        message: String,
    },
    /// Rule 2 violation: Invalid header structure
    HeaderStructureViolation {
        block_id: BasicBlockId,
        message: String,
    },
    /// Rule 3 violation: Variable binding in header
    VariableBindingViolation {
        block_id: BasicBlockId,
        instruction: String,
        message: String,
    },
}

impl VerificationResult {
    /// Convert to Result<(), String>
    pub fn to_result(self) -> Result<(), String> {
        match self {
            VerificationResult::Ok => Ok(()),
            VerificationResult::PhiPlacementViolation { block_id, phi_index, message } => {
                Err(format!(
                    "PHI placement violation at bb{} index {}: {}",
                    block_id.as_u32(),
                    phi_index,
                    message
                ))
            }
            VerificationResult::HeaderStructureViolation { block_id, message } => {
                Err(format!(
                    "Header structure violation at bb{}: {}",
                    block_id.as_u32(),
                    message
                ))
            }
            VerificationResult::VariableBindingViolation { block_id, instruction, message } => {
                Err(format!(
                    "Variable binding violation at bb{} ({}): {}",
                    block_id.as_u32(),
                    instruction,
                    message
                ))
            }
        }
    }
}

/// LoopForm structure verifier Box
pub struct LoopFormVerifierBox;

impl LoopFormVerifierBox {
    /// Verify loop header structure (main entry point)
    ///
    /// **builder**: MIR builder
    /// **header_bb**: Header block ID
    ///
    /// **Returns**: VerificationResult
    pub fn verify_loop_header(
        builder: &MirBuilder,
        header_bb: BasicBlockId,
    ) -> VerificationResult {
        // Get header block
        let function = match &builder.current_function {
            Some(f) => f,
            None => {
                return VerificationResult::HeaderStructureViolation {
                    block_id: header_bb,
                    message: "No current function".to_string(),
                };
            }
        };

        let block = match function.get_block(header_bb) {
            Some(b) => b,
            None => {
                return VerificationResult::HeaderStructureViolation {
                    block_id: header_bb,
                    message: "Header block not found".to_string(),
                };
            }
        };

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform-verify] 🔍 verify_loop_header: bb{} ({} instructions)",
                header_bb.as_u32(),
                block.instructions.len()
            );
        }

        // Rule 1: PHI placement verification
        let result = Self::verify_phi_placement(header_bb, block);
        if result != VerificationResult::Ok {
            return result;
        }

        // Rule 2: Header structure verification
        let result = Self::verify_header_structure(header_bb, block);
        if result != VerificationResult::Ok {
            return result;
        }

        // Rule 3: No variable binding verification
        let result = Self::verify_no_variable_binding(header_bb, block);
        if result != VerificationResult::Ok {
            return result;
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!("[loopform-verify] ✅ verify_loop_header: All rules passed");
        }

        VerificationResult::Ok
    }

    // ========== Verification Rules ==========

    /// Rule 1: Verify PHI instructions are at block start
    ///
    /// **Purpose**: Ensure PHI nodes appear before any non-PHI instructions
    fn verify_phi_placement(
        block_id: BasicBlockId,
        block: &BasicBlock,
    ) -> VerificationResult {
        let mut in_phi_group = true;
        let mut phi_count = 0;

        for (index, inst) in block.instructions.iter().enumerate() {
            match inst {
                MirInstruction::Phi { .. } => {
                    if !in_phi_group {
                        // PHI after non-PHI → violation
                        return VerificationResult::PhiPlacementViolation {
                            block_id,
                            phi_index: index,
                            message: format!(
                                "PHI instruction at index {} appears after non-PHI instruction",
                                index
                            ),
                        };
                    }
                    phi_count += 1;
                }
                _ => {
                    // First non-PHI instruction marks end of PHI group
                    in_phi_group = false;
                }
            }
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform-verify] Rule 1 ✅: {} PHI nodes at block start",
                phi_count
            );
        }

        VerificationResult::Ok
    }

    /// Rule 2: Verify header structure (PHI + terminator only)
    ///
    /// **Purpose**: Ensure Header = PHI group + (Jump|Branch) only
    fn verify_header_structure(
        block_id: BasicBlockId,
        block: &BasicBlock,
    ) -> VerificationResult {
        // Header instructions must be exclusively PHI nodes
        for inst in block.instructions.iter() {
            if !matches!(inst, MirInstruction::Phi { .. }) {
                return VerificationResult::HeaderStructureViolation {
                    block_id,
                    message: format!(
                        "Header contains non-PHI instruction before terminator: {:?}",
                        inst
                    ),
                };
            }
        }

        // Expect a terminator (Jump or Branch) stored separately
        match &block.terminator {
            Some(MirInstruction::Jump { .. }) | Some(MirInstruction::Branch { .. }) => {
                // OK
            }
            Some(other) => {
                return VerificationResult::HeaderStructureViolation {
                    block_id,
                    message: format!(
                        "Header terminator must be Jump or Branch, got: {:?}",
                        other
                    ),
                };
            }
            None => {
                return VerificationResult::HeaderStructureViolation {
                    block_id,
                    message: "Header missing terminator (expected Jump or Branch)".to_string(),
                };
            }
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform-verify] Rule 2 ✅: Header = {} PHI + 1 terminator",
                block.instructions.len()
            );
        }

        VerificationResult::Ok
    }

    /// Rule 3: Verify no variable binding in header (except PHI)
    ///
    /// **Purpose**: Prohibit Const/Copy/BinOp/etc in header block
    fn verify_no_variable_binding(
        block_id: BasicBlockId,
        block: &BasicBlock,
    ) -> VerificationResult {
        for (index, inst) in block.instructions.iter().enumerate() {
            // Check if instruction binds a variable (has dst field, excluding PHI)
            let is_binding = match inst {
                // Allowed: PHI (handled separately)
                MirInstruction::Phi { .. } => false,
                // Allowed: Terminators (no dst)
                MirInstruction::Jump { .. } | MirInstruction::Branch { .. } => false,
                MirInstruction::Return { .. } => false,

                // Forbidden: All other instructions with dst
                MirInstruction::Const { .. } => true,
                MirInstruction::Copy { .. } => true,
                MirInstruction::BinOp { .. } => true,
                MirInstruction::Compare { .. } => true,
                MirInstruction::BoxCall { .. } => true,
                MirInstruction::Call { .. } => true,
                MirInstruction::NewBox { .. } => true,
                MirInstruction::Load { .. } => true,
                MirInstruction::TypeOp { .. } => true,
                MirInstruction::UnaryOp { .. } => true,
                MirInstruction::TypeCheck { .. } => true,
                MirInstruction::Cast { .. } => true,
                MirInstruction::NewClosure { .. } => true,
                MirInstruction::RefNew { .. } => true,
                MirInstruction::WeakNew { .. } => true,
                MirInstruction::WeakLoad { .. } => true,
                MirInstruction::FutureNew { .. } => true,
                MirInstruction::Await { .. } => true,

                // Allowed: No dst
                MirInstruction::Store { .. } => false,
                MirInstruction::FutureSet { .. } => false,
                MirInstruction::Safepoint => false,
                MirInstruction::Barrier { .. } => false,
                MirInstruction::BarrierRead { .. } => false,
                MirInstruction::BarrierWrite { .. } => false,
                MirInstruction::Nop => false,
                MirInstruction::Debug { .. } => false,
                MirInstruction::Print { .. } => false,
                MirInstruction::Throw { .. } => false,
                MirInstruction::Catch { .. } => false,
                MirInstruction::WeakRef { .. } => false,
            };

            if is_binding {
                let inst_name = match inst {
                    MirInstruction::Const { .. } => "Const",
                    MirInstruction::Copy { .. } => "Copy",
                    MirInstruction::BinOp { .. } => "BinOp",
                    MirInstruction::Compare { .. } => "Compare",
                    MirInstruction::BoxCall { .. } => "BoxCall",
                    MirInstruction::Call { .. } => "Call",
                    MirInstruction::NewBox { .. } => "NewBox",
                    MirInstruction::Load { .. } => "Load",
                    MirInstruction::TypeOp { .. } => "TypeOp",
                    MirInstruction::UnaryOp { .. } => "UnaryOp",
                    MirInstruction::TypeCheck { .. } => "TypeCheck",
                    MirInstruction::Cast { .. } => "Cast",
                    MirInstruction::NewClosure { .. } => "NewClosure",
                    MirInstruction::RefNew { .. } => "RefNew",
                    MirInstruction::WeakNew { .. } => "WeakNew",
                    MirInstruction::WeakLoad { .. } => "WeakLoad",
                    MirInstruction::FutureNew { .. } => "FutureNew",
                    MirInstruction::Await { .. } => "Await",
                    _ => "Unknown",
                };

                return VerificationResult::VariableBindingViolation {
                    block_id,
                    instruction: inst_name.to_string(),
                    message: format!(
                        "{} at index {} binds a variable (prohibited in header)",
                        inst_name, index
                    ),
                };
            }
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!("[loopform-verify] Rule 3 ✅: No variable binding in header");
        }

        VerificationResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_ok() {
        let result = VerificationResult::Ok;
        assert!(result.to_result().is_ok());
    }

    #[test]
    fn test_verification_result_phi_placement_violation() {
        let result = VerificationResult::PhiPlacementViolation {
            block_id: BasicBlockId::new(1),
            phi_index: 3,
            message: "PHI after non-PHI".to_string(),
        };
        assert!(result.to_result().is_err());
    }

    #[test]
    fn test_verification_result_header_structure_violation() {
        let result = VerificationResult::HeaderStructureViolation {
            block_id: BasicBlockId::new(1),
            message: "Non-PHI instruction in header".to_string(),
        };
        assert!(result.to_result().is_err());
    }

    #[test]
    fn test_verification_result_variable_binding_violation() {
        let result = VerificationResult::VariableBindingViolation {
            block_id: BasicBlockId::new(1),
            instruction: "Const".to_string(),
            message: "Variable binding in header".to_string(),
        };
        assert!(result.to_result().is_err());
    }
}
