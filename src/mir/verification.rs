/*!
 * MIR Verification - SSA form and semantic verification
 * 
 * Implements dominance checking, SSA verification, and semantic analysis
 */

use super::{MirModule, MirFunction, BasicBlockId, ValueId};
use crate::mir::verification_types::VerificationError;
use crate::debug::log as dlog;
use std::collections::{HashSet, HashMap};
mod legacy;
mod barrier;
mod awaits;
mod utils;

// VerificationError moved to crate::mir::verification_types

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
        // 5. Minimal checks for WeakRef/Barrier
        if let Err(mut weak_barrier_errors) = self.verify_weakref_and_barrier(function) {
            local_errors.append(&mut weak_barrier_errors);
        }
        // 6. Light barrier-context diagnostic (strict mode only)
        if let Err(mut barrier_ctx) = self.verify_barrier_context(function) {
            local_errors.append(&mut barrier_ctx);
        }
        // 7. Forbid legacy instructions (must be rewritten to Core-15)
        if let Err(mut legacy_errors) = self.verify_no_legacy_ops(function) {
            local_errors.append(&mut legacy_errors);
        }
        // 8. Async semantics: ensure checkpoints around await
        if let Err(mut await_cp) = self.verify_await_checkpoints(function) {
            local_errors.append(&mut await_cp);
        }
        
        if local_errors.is_empty() {
            Ok(())
        } else {
            if dlog::on("NYASH_DEBUG_VERIFIER") {
                eprintln!("[VERIFY] {} errors in function {}", local_errors.len(), function.signature.name);
                for e in &local_errors {
                    match e {
                        VerificationError::MergeUsesPredecessorValue { value, merge_block, pred_block } => {
                            eprintln!(
                                "  • MergeUsesPredecessorValue: value=%{:?} merge_bb={:?} pred_bb={:?} -- hint: insert/use Phi in merge block for values from predecessors",
                                value, merge_block, pred_block
                            );
                        }
                        VerificationError::DominatorViolation { value, use_block, def_block } => {
                            eprintln!(
                                "  • DominatorViolation: value=%{:?} use_bb={:?} def_bb={:?} -- hint: ensure definition dominates use, or route via Phi",
                                value, use_block, def_block
                            );
                        }
                        VerificationError::InvalidPhi { phi_value, block, reason } => {
                            eprintln!(
                                "  • InvalidPhi: phi_dst=%{:?} in bb={:?} reason={} -- hint: check inputs cover all predecessors and placed at block start",
                                phi_value, block, reason
                            );
                        }
                        VerificationError::InvalidWeakRefSource { weak_ref, block, instruction_index, reason } => {
                            eprintln!(
                                "  • InvalidWeakRefSource: weak=%{:?} at {}:{} reason='{}' -- hint: source must be WeakRef(new)/WeakNew; ensure creation precedes load and value flows correctly",
                                weak_ref, block, instruction_index, reason
                            );
                        }
                        VerificationError::InvalidBarrierPointer { ptr, block, instruction_index, reason } => {
                            eprintln!(
                                "  • InvalidBarrierPointer: ptr=%{:?} at {}:{} reason='{}' -- hint: barrier pointer must be a valid ref (not void/null); ensure it is defined and non-void",
                                ptr, block, instruction_index, reason
                            );
                        }
                        VerificationError::SuspiciousBarrierContext { block, instruction_index, note } => {
                            eprintln!(
                                "  • SuspiciousBarrierContext: at {}:{} note='{}' -- hint: place barrier within ±2 of load/store/ref ops in same block or disable strict check",
                                block, instruction_index, note
                            );
                        }
                        other => {
                            eprintln!("  • {:?}", other);
                        }
                    }
                }
            }
            Err(local_errors)
        }
    }

    /// Reject legacy instructions that should be rewritten to Core-15 equivalents
    /// Skips check when NYASH_VERIFY_ALLOW_LEGACY=1
    fn verify_no_legacy_ops(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        legacy::check_no_legacy_ops(function)
    }

    /// Ensure that each Await instruction (or ExternCall(env.future.await)) is immediately
    /// preceded and followed by a checkpoint.
    /// A checkpoint is either MirInstruction::Safepoint or ExternCall("env.runtime", "checkpoint").
    fn verify_await_checkpoints(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        awaits::check_await_checkpoints(function)
    }

    /// Verify WeakRef/Barrier minimal semantics
    fn verify_weakref_and_barrier(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        barrier::check_weakref_and_barrier(function)
    }

    /// Light diagnostic: Barrier should be near memory ops in the same block (best-effort)
    /// Enabled only when NYASH_VERIFY_BARRIER_STRICT=1
    fn verify_barrier_context(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        barrier::check_barrier_context(function)
    }
    
    /// Verify SSA form properties
    fn verify_ssa_form(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        // Allow non-SSA (edge-copy) mode for PHI-less MIR when enabled via env
        if crate::config::env::verify_allow_no_phi()
        {
            return Ok(());
        }
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
    
    /// Verify dominance relations (def must dominate use across blocks)
    fn verify_dominance(&self, function: &MirFunction) -> Result<(), Vec<VerificationError>> {
        // Allow non-SSA (edge-copy) mode for PHI-less MIR when enabled via env
        if crate::config::env::verify_allow_no_phi()
        {
            return Ok(());
        }
        let mut errors = Vec::new();

        // Build def -> block map and dominators
        let def_block = utils::compute_def_blocks(function);
        let dominators = utils::compute_dominators(function);

        for (use_block_id, block) in &function.blocks {
            for instruction in block.all_instructions() {
                // Phi inputs are special: they are defined in predecessors; skip dominance check for them
                if let super::MirInstruction::Phi { .. } = instruction { continue; }
                for used_value in instruction.used_values() {
                    if let Some(&def_bb) = def_block.get(&used_value) {
                        if def_bb != *use_block_id {
                            let doms = dominators.get(use_block_id).unwrap();
                            if !doms.contains(&def_bb) {
                                errors.push(VerificationError::DominatorViolation {
                                    value: used_value,
                                    use_block: *use_block_id,
                                    def_block: def_bb,
                                });
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
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
        let reachable = utils::compute_reachable_blocks(function);
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
        // Allow non-SSA (edge-copy) mode for PHI-less MIR when enabled via env
        if crate::config::env::verify_allow_no_phi()
        {
            return Ok(());
        }
        let mut errors = Vec::new();
        let preds = utils::compute_predecessors(function);
        let def_block = utils::compute_def_blocks(function);
        let dominators = utils::compute_dominators(function);
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
            let doms_of_block = dominators.get(bid).unwrap();
            // check instructions including terminator
            for inst in block.all_instructions() {
                // Skip Phi: its inputs are allowed to come from predecessors by SSA definition
                if let super::MirInstruction::Phi { .. } = inst { continue; }
                for used in inst.used_values() {
                    if let Some(&db) = def_block.get(&used) {
                        // If def doesn't dominate merge block, it must be routed via phi
                        if !doms_of_block.contains(&db) {
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
    
    /// Get all verification errors from the last run
    pub fn get_errors(&self) -> &[VerificationError] {
        &self.errors
    }
    
    /// Clear verification errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    // Wrapper helpers removed; use verification::utils directly at call sites
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
            VerificationError::InvalidWeakRefSource { weak_ref, block, instruction_index, reason } => {
                write!(f, "Invalid WeakRef source {} in block {} at {}: {}", weak_ref, block, instruction_index, reason)
            },
            VerificationError::InvalidBarrierPointer { ptr, block, instruction_index, reason } => {
                write!(f, "Invalid Barrier pointer {} in block {} at {}: {}", ptr, block, instruction_index, reason)
            },
            VerificationError::SuspiciousBarrierContext { block, instruction_index, note } => {
                write!(f, "Suspicious Barrier context in block {} at {}: {}", block, instruction_index, note)
            },
            VerificationError::UnsupportedLegacyInstruction { block, instruction_index, name } => {
                write!(f, "Unsupported legacy instruction '{}' in block {} at {} (enable rewrite passes)", name, block, instruction_index)
            },
            VerificationError::MissingCheckpointAroundAwait { block, instruction_index, position } => {
                write!(f, "Missing {} checkpoint around await in block {} at instruction {}", position, block, instruction_index)
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

    #[test]
    fn test_merge_use_before_phi_detected() {
        use crate::mir::{MirInstruction, ConstValue};

        // Construct a function with a bad merge use (no phi)
        let signature = FunctionSignature {
            name: "merge_bad".to_string(),
            params: vec![],
            return_type: MirType::String,
            effects: EffectMask::PURE,
        };

        let entry = BasicBlockId::new(0);
        let mut f = MirFunction::new(signature, entry);

        let then_bb = BasicBlockId::new(1);
        let else_bb = BasicBlockId::new(2);
        let merge_bb = BasicBlockId::new(3);

        let cond = f.next_value_id(); // %0
        {
            let b0 = f.get_block_mut(entry).unwrap();
            b0.add_instruction(MirInstruction::Const { dst: cond, value: ConstValue::Bool(true) });
            b0.add_instruction(MirInstruction::Branch { condition: cond, then_bb, else_bb });
        }

        let v1 = f.next_value_id(); // %1
        let mut b1 = BasicBlock::new(then_bb);
        b1.add_instruction(MirInstruction::Const { dst: v1, value: ConstValue::String("A".to_string()) });
        b1.add_instruction(MirInstruction::Jump { target: merge_bb });
        f.add_block(b1);

        let v2 = f.next_value_id(); // %2
        let mut b2 = BasicBlock::new(else_bb);
        b2.add_instruction(MirInstruction::Const { dst: v2, value: ConstValue::String("B".to_string()) });
        b2.add_instruction(MirInstruction::Jump { target: merge_bb });
        f.add_block(b2);

        let mut b3 = BasicBlock::new(merge_bb);
        // Illegal: directly use v1 from predecessor
        b3.add_instruction(MirInstruction::Return { value: Some(v1) });
        f.add_block(b3);

        f.update_cfg();

        let mut verifier = MirVerifier::new();
        let res = verifier.verify_function(&f);
        assert!(res.is_err(), "Verifier should error on merge use without phi");
        let errs = res.err().unwrap();
        assert!(errs.iter().any(|e| matches!(e, VerificationError::MergeUsesPredecessorValue{..} | VerificationError::DominatorViolation{..})),
            "Expected merge/dominator error, got: {:?}", errs);
    }

    #[test]
    fn test_loop_phi_normalization() {
        // Program:
        // local i = 0
        // loop(false) { i = 1 }
        // i
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::Local {
                    variables: vec!["i".to_string()],
                    initial_values: vec![Some(Box::new(ASTNode::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))],
                    span: Span::unknown(),
                },
                ASTNode::Loop {
                    condition: Box::new(ASTNode::Literal { value: LiteralValue::Bool(false), span: Span::unknown() }),
                    body: vec![ ASTNode::Assignment {
                        target: Box::new(ASTNode::Variable { name: "i".to_string(), span: Span::unknown() }),
                        value: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }),
                        span: Span::unknown(),
                    }],
                    span: Span::unknown(),
                },
                ASTNode::Variable { name: "i".to_string(), span: Span::unknown() },
            ],
            span: Span::unknown(),
        };

        let mut builder = MirBuilder::new();
        let module = builder.build_module(ast).expect("build mir");

        // Verify SSA/dominance: should pass
        let mut verifier = MirVerifier::new();
        let res = verifier.verify_module(&module);
        if let Err(errs) = &res { eprintln!("Verifier errors: {:?}", errs); }
        assert!(res.is_ok(), "MIR loop with phi normalization should pass verification");

        // Ensure phi is printed (header phi for variable i)
        let printer = MirPrinter::verbose();
        let mir_text = printer.print_module(&module);
        assert!(mir_text.contains("phi"), "Printed MIR should contain a phi for loop header\n{}", mir_text);
    }

    #[test]
    fn test_loop_nested_if_phi() {
        // Program:
        // local x = 0
        // loop(false) { if true { x = 1 } else { x = 2 } }
        // x
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::Local {
                    variables: vec!["x".to_string()],
                    initial_values: vec![Some(Box::new(ASTNode::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))],
                    span: Span::unknown(),
                },
                ASTNode::Loop {
                    condition: Box::new(ASTNode::Literal { value: LiteralValue::Bool(false), span: Span::unknown() }),
                    body: vec![ ASTNode::If {
                        condition: Box::new(ASTNode::Literal { value: LiteralValue::Bool(true), span: Span::unknown() }),
                        then_body: vec![ ASTNode::Assignment {
                            target: Box::new(ASTNode::Variable { name: "x".to_string(), span: Span::unknown() }),
                            value: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }),
                            span: Span::unknown(),
                        }],
                        else_body: Some(vec![ ASTNode::Assignment {
                            target: Box::new(ASTNode::Variable { name: "x".to_string(), span: Span::unknown() }),
                            value: Box::new(ASTNode::Literal { value: LiteralValue::Integer(2), span: Span::unknown() }),
                            span: Span::unknown(),
                        }]),
                        span: Span::unknown(),
                    }],
                    span: Span::unknown(),
                },
                ASTNode::Variable { name: "x".to_string(), span: Span::unknown() },
            ],
            span: Span::unknown(),
        };

        let mut builder = MirBuilder::new();
        let module = builder.build_module(ast).expect("build mir");

        let mut verifier = MirVerifier::new();
        let res = verifier.verify_module(&module);
        if let Err(errs) = &res { eprintln!("Verifier errors: {:?}", errs); }
        assert!(res.is_ok(), "Nested if in loop should pass verification with proper phis");

        let printer = MirPrinter::verbose();
        let mir_text = printer.print_module(&module);
        assert!(mir_text.contains("phi"), "Printed MIR should contain phi nodes for nested if/loop\n{}", mir_text);
    }
}
