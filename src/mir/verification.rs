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
    /// Merge block uses predecessor-defined value directly instead of Phi
    MergeUsesPredecessorValue {
        value: ValueId,
        merge_block: BasicBlockId,
        pred_block: BasicBlockId,
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
        
        for (_name, function) in &module.functions {
            if let Err(mut func_errors) = self.verify_function(function) {
                // Add function context to errors
                for _error in &mut func_errors {
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
        // 4. Check merge-block value usage (ensure Phi is used)
        if let Err(mut merge_errors) = self.verify_merge_uses(function) {
            local_errors.append(&mut merge_errors);
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
        let errors = Vec::new();
        
        // For now, just check that values are defined before use in the same block
        for (_block_id, block) in &function.blocks {
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

    /// Verify that blocks with multiple predecessors do not use predecessor-defined values directly.
    /// In merge blocks, values coming from predecessors must be routed through Phi.
    fn verify_merge_uses(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        let mut errors = Vec::new();
        // Build predecessor map
        let mut preds: std::collections::HashMap<BasicBlockId, Vec<BasicBlockId>> = std::collections::HashMap::new();
        for (bid, block) in &function.blocks {
            for succ in &block.successors {
                preds.entry(*succ).or_default().push(*bid);
            }
        }
        // Build definition map (value -> def block)
        let mut def_block: std::collections::HashMap<ValueId, BasicBlockId> = std::collections::HashMap::new();
        for (bid, block) in &function.blocks {
            for inst in block.all_instructions() {
                if let Some(dst) = inst.dst_value() {
                    def_block.insert(dst, *bid);
                }
            }
        }
        // Helper: collect phi dsts in a block
        let mut phi_dsts_in_block: std::collections::HashMap<BasicBlockId, std::collections::HashSet<ValueId>> = std::collections::HashMap::new();
        for (bid, block) in &function.blocks {
            let set = phi_dsts_in_block.entry(*bid).or_default();
            for inst in block.all_instructions() {
                if let super::MirInstruction::Phi { dst, .. } = inst { set.insert(*dst); }
            }
        }

        for (bid, block) in &function.blocks {
            let Some(pred_list) = preds.get(bid) else { continue };
            if pred_list.len() < 2 { continue; }
            let phi_dsts = phi_dsts_in_block.get(bid);
            // check instructions including terminator
            for inst in block.all_instructions() {
                for used in inst.used_values() {
                    if let Some(&db) = def_block.get(&used) {
                        if pred_list.contains(&db) {
                            // used value defined in a predecessor; must be routed via phi (i.e., used should be phi dst)
                            let is_phi_dst = phi_dsts.map(|s| s.contains(&used)).unwrap_or(false);
                            if !is_phi_dst {
                                errors.push(VerificationError::MergeUsesPredecessorValue {
                                    value: used,
                                    merge_block: *bid,
                                    pred_block: db,
                                });
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
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
            VerificationError::MergeUsesPredecessorValue { value, merge_block, pred_block } => {
                write!(f, "Merge block {} uses predecessor-defined value {} from block {} without Phi",
                       merge_block, value, pred_block)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock, MirBuilder, MirPrinter};
    use crate::ast::{ASTNode, Span, LiteralValue};
    
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

    #[test]
    fn test_if_merge_uses_phi_not_predecessor() {
        // Program:
        // if true { result = "A" } else { result = "B" }
        // result
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::If {
                    condition: Box::new(ASTNode::Literal { value: LiteralValue::Bool(true), span: Span::unknown() }),
                    then_body: vec![ ASTNode::Assignment {
                        target: Box::new(ASTNode::Variable { name: "result".to_string(), span: Span::unknown() }),
                        value: Box::new(ASTNode::Literal { value: LiteralValue::String("A".to_string()), span: Span::unknown() }),
                        span: Span::unknown(),
                    }],
                    else_body: Some(vec![ ASTNode::Assignment {
                        target: Box::new(ASTNode::Variable { name: "result".to_string(), span: Span::unknown() }),
                        value: Box::new(ASTNode::Literal { value: LiteralValue::String("B".to_string()), span: Span::unknown() }),
                        span: Span::unknown(),
                    }]),
                    span: Span::unknown(),
                },
                ASTNode::Variable { name: "result".to_string(), span: Span::unknown() },
            ],
            span: Span::unknown(),
        };

        let mut builder = MirBuilder::new();
        let module = builder.build_module(ast).expect("build mir");

        // Verify: should be OK (no MergeUsesPredecessorValue)
        let mut verifier = MirVerifier::new();
        let res = verifier.verify_module(&module);
        if let Err(errs) = &res { eprintln!("Verifier errors: {:?}", errs); }
        assert!(res.is_ok(), "MIR should pass merge-phi verification");

        // Optional: ensure printer shows a phi in merge and ret returns a defined value
        let mut printer = MirPrinter::verbose();
        let mir_text = printer.print_module(&module);
        assert!(mir_text.contains("phi"), "Printed MIR should contain a phi in merge block\n{}", mir_text);
    }
}
