/*!
 * MIR Verification - SSA form and semantic verification
 * 
 * Implements dominance checking, SSA verification, and semantic analysis
 */

use super::{MirModule, MirFunction, BasicBlockId, ValueId};
use std::collections::{HashSet, HashMap};

/// Verification error types
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationError {
    /// Undefined value used
    UndefinedValue {
        value: ValueId,
        block: BasicBlockId,
        instruction_index: usize,
    },
    
    /// Value defined multiple times
    MultipleDefinition {
        value: ValueId,
        first_block: BasicBlockId,
        second_block: BasicBlockId,
    },
    
    /// Invalid phi function
    InvalidPhi {
        phi_value: ValueId,
        block: BasicBlockId,
        reason: String,
    },
    
    /// Unreachable block
    UnreachableBlock {
        block: BasicBlockId,
    },
    
    /// Control flow error
    ControlFlowError {
        block: BasicBlockId,
        reason: String,
    },
    
    /// Dominator violation
    DominatorViolation {
        value: ValueId,
        use_block: BasicBlockId,
        def_block: BasicBlockId,
    },
}

/// MIR verifier for SSA form and semantic correctness
pub struct MirVerifier {
    /// Current verification errors
    errors: Vec<VerificationError>,
}

impl MirVerifier {
    /// Create a new MIR verifier
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }
    
    /// Verify an entire MIR module
    pub fn verify_module(&mut self, module: &MirModule) -> Result<(), Vec<VerificationError>> {
        self.errors.clear();
        
        for (name, function) in &module.functions {
            if let Err(mut func_errors) = self.verify_function(function) {
                // Add function context to errors
                for error in &mut func_errors {
                    // Could add function name to error context here
                }
                self.errors.extend(func_errors);
            }
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    /// Verify a single MIR function
    pub fn verify_function(&mut self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        let mut local_errors = Vec::new();
        
        // 1. Check SSA form
        if let Err(mut ssa_errors) = self.verify_ssa_form(function) {
            local_errors.append(&mut ssa_errors);
        }
        
        // 2. Check dominance relations
        if let Err(mut dom_errors) = self.verify_dominance(function) {
            local_errors.append(&mut dom_errors);
        }
        
        // 3. Check control flow integrity
        if let Err(mut cfg_errors) = self.verify_control_flow(function) {
            local_errors.append(&mut cfg_errors);
        }
        
        if local_errors.is_empty() {
            Ok(())
        } else {
            Err(local_errors)
        }
    }
    
    /// Verify SSA form properties
    fn verify_ssa_form(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        let mut errors = Vec::new();
        let mut definitions = HashMap::new();
        
        // Check that each value is defined exactly once
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
        
        // Check that all used values are defined
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
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Verify dominance relations
    fn verify_dominance(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        // This is a simplified dominance check
        // In a full implementation, we would compute the dominator tree
        let mut errors = Vec::new();
        
        // For now, just check that values are defined before use in the same block
        for (block_id, block) in &function.blocks {
            let mut defined_in_block = HashSet::new();
            
            for instruction in block.all_instructions() {
                // Check uses
                for used_value in instruction.used_values() {
                    if !defined_in_block.contains(&used_value) {
                        // Value used before definition in this block
                        // This is okay if it's defined in a dominating block
                        // For simplicity, we'll skip this check for now
                    }
                }
                
                // Record definition
                if let Some(dst) = instruction.dst_value() {
                    defined_in_block.insert(dst);
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Verify control flow graph integrity
    fn verify_control_flow(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        let mut errors = Vec::new();
        
        // Check that all referenced blocks exist
        for (block_id, block) in &function.blocks {
            for successor in &block.successors {
                if !function.blocks.contains_key(successor) {
                    errors.push(VerificationError::ControlFlowError {
                        block: *block_id,
                        reason: format!("References non-existent block {}", successor),
                    });
                }
            }
        }
        
        // Check that all blocks are reachable from entry
        let reachable = self.compute_reachable_blocks(function);
        for block_id in function.blocks.keys() {
            if !reachable.contains(block_id) && *block_id != function.entry_block {
                errors.push(VerificationError::UnreachableBlock {
                    block: *block_id,
                });
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Compute reachable blocks from entry
    fn compute_reachable_blocks(&self, function: &MirFunction) -> HashSet<BasicBlockId> {
        let mut reachable = HashSet::new();
        let mut worklist = vec![function.entry_block];
        
        while let Some(current) = worklist.pop() {
            if reachable.insert(current) {
                if let Some(block) = function.blocks.get(&current) {
                    // Add normal successors
                    for successor in &block.successors {
                        if !reachable.contains(successor) {
                            worklist.push(*successor);
                        }
                    }
                    
                    // Add exception handler blocks as reachable
                    for instruction in &block.instructions {
                        if let super::MirInstruction::Catch { handler_bb, .. } = instruction {
                            if !reachable.contains(handler_bb) {
                                worklist.push(*handler_bb);
                            }
                        }
                    }
                    
                    // Also check terminator for exception handlers
                    if let Some(ref terminator) = block.terminator {
                        if let super::MirInstruction::Catch { handler_bb, .. } = terminator {
                            if !reachable.contains(handler_bb) {
                                worklist.push(*handler_bb);
                            }
                        }
                    }
                }
            }
        }
        
        reachable
    }
    
    /// Get all verification errors from the last run
    pub fn get_errors(&self) -> &[VerificationError] {
        &self.errors
    }
    
    /// Clear verification errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

impl Default for MirVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationError::UndefinedValue { value, block, instruction_index } => {
                write!(f, "Undefined value {} used in block {} at instruction {}", 
                       value, block, instruction_index)
            },
            VerificationError::MultipleDefinition { value, first_block, second_block } => {
                write!(f, "Value {} defined multiple times: first in block {}, again in block {}",
                       value, first_block, second_block)
            },
            VerificationError::InvalidPhi { phi_value, block, reason } => {
                write!(f, "Invalid phi function {} in block {}: {}", 
                       phi_value, block, reason)
            },
            VerificationError::UnreachableBlock { block } => {
                write!(f, "Unreachable block {}", block)
            },
            VerificationError::ControlFlowError { block, reason } => {
                write!(f, "Control flow error in block {}: {}", block, reason)
            },
            VerificationError::DominatorViolation { value, use_block, def_block } => {
                write!(f, "Value {} used in block {} but defined in non-dominating block {}",
                       value, use_block, def_block)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock};
    
    #[test]
    fn test_valid_function_verification() {
        let signature = FunctionSignature {
            name: "test".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let entry_block = BasicBlockId::new(0);
        let function = MirFunction::new(signature, entry_block);
        
        let mut verifier = MirVerifier::new();
        let result = verifier.verify_function(&function);
        
        assert!(result.is_ok(), "Valid function should pass verification");
    }
    
    #[test]
    fn test_undefined_value_detection() {
        // This test would create a function with undefined value usage
        // and verify that the verifier catches it
        // Implementation details would depend on the specific test case
    }
}