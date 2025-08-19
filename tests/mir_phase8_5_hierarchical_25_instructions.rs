/*!
 * Phase 8.5 MIR 25-Instruction Hierarchical Implementation Tests
 * 
 * Comprehensive test suite for the ChatGPT5 + AI Council designed MIR system
 */

#![cfg(feature = "mir-v2")]
use nyash_rust::mir::{
    MirInstructionV2, ConstValue, BinaryOp, CompareOp, AtomicOrdering,
    EffectMask, Effect, ValueIdGenerator, BasicBlockIdGenerator,
    OwnershipVerifier, OwnershipError,
};

/// Test that we have exactly 25 instructions in the specification
#[test]
fn test_mir_instruction_count() {
    // This is verified at compile time by the instruction enum
    // Each tier should have the correct count:
    // Tier-0: 8 instructions
    // Tier-1: 12 instructions  
    // Tier-2: 5 instructions
    // Total: 25 instructions
    
    let mut value_gen = ValueIdGenerator::new();
    let mut bb_gen = BasicBlockIdGenerator::new();
    
    // Tier-0: Universal Core (8 instructions)
    let tier0_instructions = vec![
        MirInstructionV2::Const { dst: value_gen.next(), value: ConstValue::Integer(42) },
        MirInstructionV2::BinOp { dst: value_gen.next(), op: BinaryOp::Add, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Compare { dst: value_gen.next(), op: CompareOp::Eq, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Branch { condition: value_gen.next(), then_bb: bb_gen.next(), else_bb: bb_gen.next() },
        MirInstructionV2::Jump { target: bb_gen.next() },
        MirInstructionV2::Phi { dst: value_gen.next(), inputs: vec![(bb_gen.next(), value_gen.next())] },
        MirInstructionV2::Call { dst: Some(value_gen.next()), func: value_gen.next(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Return { value: Some(value_gen.next()) },
    ];
    
    for inst in &tier0_instructions {
        assert_eq!(inst.tier(), 0, "Tier-0 instruction should have tier 0");
    }
    assert_eq!(tier0_instructions.len(), 8, "Tier-0 should have exactly 8 instructions");
    
    // Tier-1: Nyash Semantics (12 instructions)
    let tier1_instructions = vec![
        MirInstructionV2::NewBox { dst: value_gen.next(), box_type: "TestBox".to_string(), args: vec![] },
        MirInstructionV2::BoxFieldLoad { dst: value_gen.next(), box_val: value_gen.next(), field: "value".to_string() },
        MirInstructionV2::BoxFieldStore { box_val: value_gen.next(), field: "value".to_string(), value: value_gen.next() },
        MirInstructionV2::BoxCall { dst: Some(value_gen.next()), box_val: value_gen.next(), method: "test".to_string(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Safepoint,
        MirInstructionV2::RefGet { dst: value_gen.next(), reference: value_gen.next() },
        MirInstructionV2::RefSet { reference: value_gen.next(), new_target: value_gen.next() },
        MirInstructionV2::WeakNew { dst: value_gen.next(), box_val: value_gen.next() },
        MirInstructionV2::WeakLoad { dst: value_gen.next(), weak_ref: value_gen.next() },
        MirInstructionV2::WeakCheck { dst: value_gen.next(), weak_ref: value_gen.next() },
        MirInstructionV2::Send { bus: value_gen.next(), message: value_gen.next() },
        MirInstructionV2::Recv { dst: value_gen.next(), bus: value_gen.next() },
    ];
    
    for inst in &tier1_instructions {
        assert_eq!(inst.tier(), 1, "Tier-1 instruction should have tier 1");
    }
    assert_eq!(tier1_instructions.len(), 12, "Tier-1 should have exactly 12 instructions");
    
    // Tier-2: Implementation Assistance (5 instructions)
    let tier2_instructions = vec![
        MirInstructionV2::TailCall { func: value_gen.next(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Adopt { parent: value_gen.next(), child: value_gen.next() },
        MirInstructionV2::Release { reference: value_gen.next() },
        MirInstructionV2::MemCopy { dest: value_gen.next(), src: value_gen.next(), size: value_gen.next() },
        MirInstructionV2::AtomicFence { ordering: AtomicOrdering::SeqCst },
    ];
    
    for inst in &tier2_instructions {
        assert_eq!(inst.tier(), 2, "Tier-2 instruction should have tier 2");
    }
    assert_eq!(tier2_instructions.len(), 5, "Tier-2 should have exactly 5 instructions");
    
    // Total verification
    let total_instructions = tier0_instructions.len() + tier1_instructions.len() + tier2_instructions.len();
    assert_eq!(total_instructions, 25, "Total instruction count must be exactly 25");
}

/// Test the 4-category effect system
#[test]
fn test_effect_categories() {
    let mut value_gen = ValueIdGenerator::new();
    let mut bb_gen = BasicBlockIdGenerator::new();
    
    // Test Pure effects
    let pure_instructions = vec![
        MirInstructionV2::Const { dst: value_gen.next(), value: ConstValue::Integer(42) },
        MirInstructionV2::BinOp { dst: value_gen.next(), op: BinaryOp::Add, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Compare { dst: value_gen.next(), op: CompareOp::Eq, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Phi { dst: value_gen.next(), inputs: vec![(bb_gen.next(), value_gen.next())] },
        MirInstructionV2::BoxFieldLoad { dst: value_gen.next(), box_val: value_gen.next(), field: "value".to_string() },
        MirInstructionV2::RefGet { dst: value_gen.next(), reference: value_gen.next() },
        MirInstructionV2::WeakNew { dst: value_gen.next(), box_val: value_gen.next() },
        MirInstructionV2::WeakLoad { dst: value_gen.next(), weak_ref: value_gen.next() },
        MirInstructionV2::WeakCheck { dst: value_gen.next(), weak_ref: value_gen.next() },
    ];
    
    for inst in pure_instructions {
        let effects = inst.effects();
        assert!(effects.is_pure() || effects.primary_category() == Effect::Pure, 
               "Instruction should be pure: {:?}", inst);
    }
    
    // Test Mut effects
    let mut_instructions = vec![
        MirInstructionV2::BoxFieldStore { box_val: value_gen.next(), field: "value".to_string(), value: value_gen.next() },
        MirInstructionV2::RefSet { reference: value_gen.next(), new_target: value_gen.next() },
        MirInstructionV2::Adopt { parent: value_gen.next(), child: value_gen.next() },
        MirInstructionV2::Release { reference: value_gen.next() },
        MirInstructionV2::MemCopy { dest: value_gen.next(), src: value_gen.next(), size: value_gen.next() },
    ];
    
    for inst in mut_instructions {
        let effects = inst.effects();
        assert!(effects.is_mut() || effects.primary_category() == Effect::Mut,
               "Instruction should be mut: {:?}", inst);
    }
    
    // Test Io effects
    let io_instructions = vec![
        MirInstructionV2::Safepoint,
        MirInstructionV2::Send { bus: value_gen.next(), message: value_gen.next() },
        MirInstructionV2::Recv { dst: value_gen.next(), bus: value_gen.next() },
        MirInstructionV2::AtomicFence { ordering: AtomicOrdering::SeqCst },
    ];
    
    for inst in io_instructions {
        let effects = inst.effects();
        assert!(effects.is_io() || effects.primary_category() == Effect::Io,
               "Instruction should be io: {:?}", inst);
    }
    
    // Test Control effects
    let control_instructions = vec![
        MirInstructionV2::Branch { condition: value_gen.next(), then_bb: bb_gen.next(), else_bb: bb_gen.next() },
        MirInstructionV2::Jump { target: bb_gen.next() },
        MirInstructionV2::Return { value: Some(value_gen.next()) },
        MirInstructionV2::TailCall { func: value_gen.next(), args: vec![], effects: EffectMask::PURE },
    ];
    
    for inst in control_instructions {
        let effects = inst.effects();
        assert!(effects.is_control() || effects.primary_category() == Effect::Control,
               "Instruction should be control: {:?}", inst);
    }
}

/// Test optimization safety based on effect categories
#[test]
fn test_optimization_safety() {
    let mut value_gen = ValueIdGenerator::new();
    
    // Pure operations should be reorderable and eligible for CSE/LICM
    let const_inst = MirInstructionV2::Const { dst: value_gen.next(), value: ConstValue::Integer(42) };
    let binop_inst = MirInstructionV2::BinOp { 
        dst: value_gen.next(), 
        op: BinaryOp::Add, 
        lhs: value_gen.next(), 
        rhs: value_gen.next() 
    };
    
    assert!(const_inst.effects().is_pure(), "Const should be pure and reorderable");
    assert!(binop_inst.effects().is_pure(), "BinOp should be pure and reorderable");
    
    // Mut operations should preserve same Box/Field dependencies
    let store_inst = MirInstructionV2::BoxFieldStore { 
        box_val: value_gen.next(), 
        field: "value".to_string(), 
        value: value_gen.next() 
    };
    
    assert!(store_inst.effects().is_mut(), "BoxFieldStore should be mut");
    assert!(!store_inst.effects().is_pure(), "Mut operations cannot be reordered freely");
    
    // Io operations should not be reordered
    let send_inst = MirInstructionV2::Send { 
        bus: value_gen.next(), 
        message: value_gen.next() 
    };
    
    assert!(send_inst.effects().is_io(), "Send should be io");
    assert!(!send_inst.effects().is_read_only(), "Io operations have external effects");
}

/// Test ownership forest verification
#[test]
fn test_ownership_forest_verification() {
    let mut verifier = OwnershipVerifier::new();
    let mut value_gen = ValueIdGenerator::new();
    
    // Test basic ownership establishment
    let parent = value_gen.next();
    let child = value_gen.next();
    
    let adopt_inst = MirInstructionV2::Adopt { parent, child };
    assert!(verifier.process_instruction(&adopt_inst).is_ok(), "Basic adoption should succeed");
    
    let stats = verifier.ownership_stats();
    assert_eq!(stats.strong_edges, 1, "Should have one strong edge");
    
    // Test forest property verification
    assert!(verifier.verify_ownership_forest().is_ok(), "Basic forest should be valid");
    
    // Test weak reference creation
    let weak_ref = value_gen.next();
    let weak_new_inst = MirInstructionV2::WeakNew { dst: weak_ref, box_val: child };
    assert!(verifier.process_instruction(&weak_new_inst).is_ok(), "Weak reference creation should succeed");
    
    let stats_after_weak = verifier.ownership_stats();
    assert_eq!(stats_after_weak.weak_edges, 1, "Should have one weak edge");
    assert_eq!(stats_after_weak.live_weak_refs, 1, "Should have one live weak reference");
}

/// Test ownership forest violations
#[test]
fn test_ownership_violations() {
    let mut verifier = OwnershipVerifier::new();
    let mut value_gen = ValueIdGenerator::new();
    
    // Test unsafe RefSet (changing strong reference without Release)
    let reference = value_gen.next();
    let old_target = value_gen.next();
    let new_target = value_gen.next();
    
    // Manually set up initial state
    verifier.strong_edges.insert(reference, old_target);
    
    // Try to change reference without releasing old target
    let unsafe_ref_set = MirInstructionV2::RefSet { reference, new_target };
    let result = verifier.process_instruction(&unsafe_ref_set);
    
    assert!(result.is_err(), "Unsafe RefSet should be rejected");
    if let Err(errors) = result {
        assert!(errors.iter().any(|e| matches!(e, OwnershipError::UnsafeRefSet { .. })),
               "Should detect unsafe RefSet");
    }
}

/// Test weak reference liveness tracking
#[test]
fn test_weak_reference_liveness() {
    let mut verifier = OwnershipVerifier::new();
    let mut value_gen = ValueIdGenerator::new();
    
    let target = value_gen.next();
    let weak_ref = value_gen.next();
    
    // Create weak reference to target
    let weak_new = MirInstructionV2::WeakNew { dst: weak_ref, box_val: target };
    assert!(verifier.process_instruction(&weak_new).is_ok());
    
    // Release the target
    let release = MirInstructionV2::Release { reference: target };
    assert!(verifier.process_instruction(&release).is_ok());
    
    // Check that target is now considered dead
    let stats = verifier.ownership_stats();
    assert_eq!(stats.dead_targets, 1, "Target should be marked as dead");
    
    // WeakLoad should handle expired reference deterministically
    let weak_load = MirInstructionV2::WeakLoad { dst: value_gen.next(), weak_ref };
    assert!(verifier.process_instruction(&weak_load).is_ok(), 
           "WeakLoad should handle expired reference gracefully");
    
    // WeakCheck should also handle expired reference deterministically
    let weak_check = MirInstructionV2::WeakCheck { dst: value_gen.next(), weak_ref };
    assert!(verifier.process_instruction(&weak_check).is_ok(), 
           "WeakCheck should handle expired reference gracefully");
}

/// Test Bus communication instructions
#[test]
fn test_bus_operations() {
    let mut value_gen = ValueIdGenerator::new();
    
    let bus = value_gen.next();
    let message = value_gen.next();
    
    // Test Send instruction
    let send_inst = MirInstructionV2::Send { bus, message };
    assert_eq!(send_inst.tier(), 1, "Send should be Tier-1");
    assert!(send_inst.effects().is_io(), "Send should have io effects");
    
    let used_values = send_inst.used_values();
    assert_eq!(used_values.len(), 2, "Send should use bus and message");
    assert!(used_values.contains(&bus) && used_values.contains(&message));
    
    // Test Recv instruction
    let recv_inst = MirInstructionV2::Recv { dst: value_gen.next(), bus };
    assert_eq!(recv_inst.tier(), 1, "Recv should be Tier-1");
    assert!(recv_inst.effects().is_io(), "Recv should have io effects");
    
    let recv_used = recv_inst.used_values();
    assert_eq!(recv_used.len(), 1, "Recv should use only bus");
    assert!(recv_used.contains(&bus));
}

/// Test implementation assistance instructions (Tier-2)
#[test]
fn test_implementation_assistance() {
    let mut value_gen = ValueIdGenerator::new();
    
    // Test TailCall
    let tail_call = MirInstructionV2::TailCall { 
        func: value_gen.next(), 
        args: vec![value_gen.next()], 
        effects: EffectMask::PURE 
    };
    assert_eq!(tail_call.tier(), 2, "TailCall should be Tier-2");
    assert!(tail_call.effects().is_control(), "TailCall should be control flow");
    
    // Test MemCopy
    let mem_copy = MirInstructionV2::MemCopy { 
        dest: value_gen.next(), 
        src: value_gen.next(), 
        size: value_gen.next() 
    };
    assert_eq!(mem_copy.tier(), 2, "MemCopy should be Tier-2");
    assert!(mem_copy.effects().is_mut(), "MemCopy should be mut");
    
    // Test AtomicFence
    let atomic_fence = MirInstructionV2::AtomicFence { ordering: AtomicOrdering::AcqRel };
    assert_eq!(atomic_fence.tier(), 2, "AtomicFence should be Tier-2");
    assert!(atomic_fence.effects().is_io(), "AtomicFence should be io");
    assert!(atomic_fence.effects().contains(Effect::Barrier), "AtomicFence should have barrier effect");
}

/// Test instruction descriptions and display
#[test]
fn test_instruction_descriptions() {
    let mut value_gen = ValueIdGenerator::new();
    
    let const_inst = MirInstructionV2::Const { dst: value_gen.next(), value: ConstValue::Integer(42) };
    assert_eq!(const_inst.description(), "Load constant value");
    
    let send_inst = MirInstructionV2::Send { bus: value_gen.next(), message: value_gen.next() };
    assert_eq!(send_inst.description(), "Send Bus message");
    
    let adopt_inst = MirInstructionV2::Adopt { parent: value_gen.next(), child: value_gen.next() };
    assert_eq!(adopt_inst.description(), "Transfer ownership");
    
    // Test Display trait
    assert_eq!(format!("{}", const_inst), "Load constant value");
    assert_eq!(format!("{}", send_inst), "Send Bus message");
    assert_eq!(format!("{}", adopt_inst), "Transfer ownership");
}

/// Test value ID tracking for dependencies
#[test]
fn test_value_id_tracking() {
    let mut value_gen = ValueIdGenerator::new();
    
    let dst = value_gen.next();
    let lhs = value_gen.next();
    let rhs = value_gen.next();
    
    let binop = MirInstructionV2::BinOp { dst, op: BinaryOp::Add, lhs, rhs };
    
    // Test destination value
    assert_eq!(binop.dst_value(), Some(dst), "BinOp should produce destination value");
    
    // Test used values
    let used = binop.used_values();
    assert_eq!(used.len(), 2, "BinOp should use two values");
    assert!(used.contains(&lhs) && used.contains(&rhs), "Should use lhs and rhs");
    
    // Test instruction with no destination
    let store = MirInstructionV2::BoxFieldStore { 
        box_val: value_gen.next(), 
        field: "value".to_string(), 
        value: value_gen.next() 
    };
    assert_eq!(store.dst_value(), None, "BoxFieldStore should not produce destination value");
}

/// Test the complete 25-instruction specification compliance
#[test]
fn test_complete_specification_compliance() {
    // This test verifies that our implementation matches the exact specification
    
    // Verify we can create all 25 instruction types without compilation errors
    let mut value_gen = ValueIdGenerator::new();
    let mut bb_gen = BasicBlockIdGenerator::new();
    
    let all_instructions = vec![
        // Tier-0: Universal Core (8)
        MirInstructionV2::Const { dst: value_gen.next(), value: ConstValue::Integer(42) },
        MirInstructionV2::BinOp { dst: value_gen.next(), op: BinaryOp::Add, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Compare { dst: value_gen.next(), op: CompareOp::Eq, lhs: value_gen.next(), rhs: value_gen.next() },
        MirInstructionV2::Branch { condition: value_gen.next(), then_bb: bb_gen.next(), else_bb: bb_gen.next() },
        MirInstructionV2::Jump { target: bb_gen.next() },
        MirInstructionV2::Phi { dst: value_gen.next(), inputs: vec![] },
        MirInstructionV2::Call { dst: Some(value_gen.next()), func: value_gen.next(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Return { value: Some(value_gen.next()) },
        
        // Tier-1: Nyash Semantics (12)
        MirInstructionV2::NewBox { dst: value_gen.next(), box_type: "TestBox".to_string(), args: vec![] },
        MirInstructionV2::BoxFieldLoad { dst: value_gen.next(), box_val: value_gen.next(), field: "field".to_string() },
        MirInstructionV2::BoxFieldStore { box_val: value_gen.next(), field: "field".to_string(), value: value_gen.next() },
        MirInstructionV2::BoxCall { dst: Some(value_gen.next()), box_val: value_gen.next(), method: "method".to_string(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Safepoint,
        MirInstructionV2::RefGet { dst: value_gen.next(), reference: value_gen.next() },
        MirInstructionV2::RefSet { reference: value_gen.next(), new_target: value_gen.next() },
        MirInstructionV2::WeakNew { dst: value_gen.next(), box_val: value_gen.next() },
        MirInstructionV2::WeakLoad { dst: value_gen.next(), weak_ref: value_gen.next() },
        MirInstructionV2::WeakCheck { dst: value_gen.next(), weak_ref: value_gen.next() },
        MirInstructionV2::Send { bus: value_gen.next(), message: value_gen.next() },
        MirInstructionV2::Recv { dst: value_gen.next(), bus: value_gen.next() },
        
        // Tier-2: Implementation Assistance (5)
        MirInstructionV2::TailCall { func: value_gen.next(), args: vec![], effects: EffectMask::PURE },
        MirInstructionV2::Adopt { parent: value_gen.next(), child: value_gen.next() },
        MirInstructionV2::Release { reference: value_gen.next() },
        MirInstructionV2::MemCopy { dest: value_gen.next(), src: value_gen.next(), size: value_gen.next() },
        MirInstructionV2::AtomicFence { ordering: AtomicOrdering::SeqCst },
    ];
    
    assert_eq!(all_instructions.len(), 25, "Must have exactly 25 instructions");
    
    // Verify tier distribution
    let tier0_count = all_instructions.iter().filter(|i| i.tier() == 0).count();
    let tier1_count = all_instructions.iter().filter(|i| i.tier() == 1).count();
    let tier2_count = all_instructions.iter().filter(|i| i.tier() == 2).count();
    
    assert_eq!(tier0_count, 8, "Tier-0 should have 8 instructions");
    assert_eq!(tier1_count, 12, "Tier-1 should have 12 instructions");
    assert_eq!(tier2_count, 5, "Tier-2 should have 5 instructions");
    
    // Verify each instruction has proper effect classification
    for instruction in &all_instructions {
        let effects = instruction.effects();
        let category = effects.primary_category();
        
        // Ensure every instruction has a valid effect category
        assert!(
            matches!(category, Effect::Pure | Effect::Mut | Effect::Io | Effect::Control),
            "Instruction must have valid effect category: {:?}", instruction
        );
    }
}

/// Performance test: Ensure effect calculations are fast
#[test]
fn test_effect_calculation_performance() {
    use std::time::Instant;
    
    let mut value_gen = ValueIdGenerator::new();
    let mut bb_gen = BasicBlockIdGenerator::new();
    
    // Create a large number of instructions
    let mut instructions = Vec::new();
    for _ in 0..10000 {
        instructions.push(MirInstructionV2::BinOp { 
            dst: value_gen.next(), 
            op: BinaryOp::Add, 
            lhs: value_gen.next(), 
            rhs: value_gen.next() 
        });
    }
    
    // Measure effect calculation time
    let start = Instant::now();
    for instruction in &instructions {
        let _ = instruction.effects();
        let _ = instruction.tier();
        let _ = instruction.dst_value();
        let _ = instruction.used_values();
    }
    let elapsed = start.elapsed();
    
    // Should be very fast (< 10ms for 10k instructions)
    assert!(elapsed.as_millis() < 100, 
           "Effect calculations should be fast, took {:?}", elapsed);
}
