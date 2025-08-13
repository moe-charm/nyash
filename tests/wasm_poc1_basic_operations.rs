/*!
 * Phase 8.2 PoC1 Integration Test - Basic WASM Arithmetic Operations
 * 
 * Tests end-to-end MIR→WASM compilation and execution for:
 * - Constant loading
 * - Binary arithmetic (addition, subtraction, multiplication) 
 * - Print output
 * - Return values
 */

use nyash_rust::mir::{
    MirModule, MirFunction, FunctionSignature, MirType, EffectMask,
    BasicBlock, BasicBlockId, ValueId, MirInstruction, ConstValue, BinaryOp
};
use nyash_rust::backend::wasm::WasmBackend;

#[test]
fn test_wasm_poc1_basic_arithmetic() {
    // Build MIR equivalent to: 
    // function main() {
    //     %a = const 42
    //     %b = const 8  
    //     %result = %a + %b
    //     print %result
    //     return %result
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_arithmetic_mir_module();
    
    // Generate WAT text for debugging
    let wat_result = backend.compile_to_wat(mir_module.clone());
    assert!(wat_result.is_ok(), "WAT generation should succeed");
    
    let wat_text = wat_result.unwrap();
    println!("Generated WAT:\n{}", wat_text);
    
    // Debug: Print MIR instructions
    if let Some(main_func) = mir_module.functions.get("main") {
        if let Some(entry_block) = main_func.blocks.get(&main_func.entry_block) {
            println!("MIR instructions ({}:", entry_block.instructions.len());
            for (i, instruction) in entry_block.instructions.iter().enumerate() {
                println!("  {}: {:?}", i, instruction);
            }
            
            // Check if any instruction is a Return
            let has_return = entry_block.instructions.iter().any(|instr| matches!(instr, MirInstruction::Return { .. }));
            println!("Has return instruction: {}", has_return);
        }
    }
    
    // Verify WAT contains expected elements
    assert!(wat_text.contains("(module"), "Should contain module declaration");
    assert!(wat_text.contains("memory"), "Should contain memory declaration");
    assert!(wat_text.contains("import"), "Should contain imports");
    assert!(wat_text.contains("$main"), "Should contain main function");
    assert!(wat_text.contains("i32.const 42"), "Should contain constant 42");
    assert!(wat_text.contains("i32.const 8"), "Should contain constant 8");
    assert!(wat_text.contains("i32.add"), "Should contain addition operation");
    assert!(wat_text.contains("call $print"), "Should contain print call");
    
    // Compile to WASM binary and execute 
    let wasm_result = backend.compile_module(mir_module);
    if let Err(e) = &wasm_result {
        println!("WASM compilation error: {}", e);
    }
    assert!(wasm_result.is_ok(), "WASM compilation should succeed");
    
    let wasm_bytes = wasm_result.unwrap();
    assert!(!wasm_bytes.is_empty(), "WASM bytes should not be empty");
    
    // Execute with wasmtime
    let execution_result = backend.execute_wasm(&wasm_bytes);
    assert!(execution_result.is_ok(), "WASM execution should succeed");
    
    let return_value = execution_result.unwrap();
    assert_eq!(return_value, 50, "Should return 42 + 8 = 50");
}

#[test]  
fn test_wasm_poc1_multiplication() {
    // Test: 6 * 7 = 42
    let mut backend = WasmBackend::new();
    let mir_module = build_multiplication_mir_module();
    
    let wasm_result = backend.compile_module(mir_module);
    assert!(wasm_result.is_ok(), "WASM compilation should succeed");
    
    let return_value = backend.execute_wasm(&wasm_result.unwrap()).unwrap();
    assert_eq!(return_value, 42, "Should return 6 * 7 = 42");
}

#[test]
fn test_wasm_poc1_subtraction() {
    // Test: 50 - 8 = 42
    let mut backend = WasmBackend::new();
    let mir_module = build_subtraction_mir_module();
    
    let wasm_result = backend.compile_module(mir_module);
    assert!(wasm_result.is_ok(), "WASM compilation should succeed");
    
    let return_value = backend.execute_wasm(&wasm_result.unwrap()).unwrap();
    assert_eq!(return_value, 42, "Should return 50 - 8 = 42");
}

/// Build MIR module for: 42 + 8
fn build_arithmetic_mir_module() -> MirModule {
    let mut module = MirModule::new("test_arithmetic".to_string());
    
    // Create main function signature
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    // Create entry block
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    
    // Create basic block
    let mut block = BasicBlock::new(entry_block);
    
    // Generate value IDs
    let val_a = ValueId::new(0);     // 42
    let val_b = ValueId::new(1);     // 8
    let result = ValueId::new(2);    // 42 + 8
    
    // Add instructions
    block.add_instruction(MirInstruction::Const {
        dst: val_a,
        value: ConstValue::Integer(42),
    });
    
    block.add_instruction(MirInstruction::Const {
        dst: val_b,
        value: ConstValue::Integer(8),
    });
    
    block.add_instruction(MirInstruction::BinOp {
        dst: result,
        op: BinaryOp::Add,
        lhs: val_a,
        rhs: val_b,
    });
    
    block.add_instruction(MirInstruction::Print {
        value: result,
        effects: EffectMask::IO,
    });
    
    block.add_instruction(MirInstruction::Return {
        value: Some(result),
    });
    
    // Debug: Print number of instructions
    println!("Total instructions added: {}", block.instructions.len());
    
    // Add block to function
    main_function.add_block(block);
    
    // Add function to module
    module.add_function(main_function);
    
    module
}

/// Build MIR module for: 6 * 7
fn build_multiplication_mir_module() -> MirModule {
    let mut module = MirModule::new("test_multiplication".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    let val_a = ValueId::new(0);     // 6
    let val_b = ValueId::new(1);     // 7
    let result = ValueId::new(2);    // 6 * 7
    
    block.add_instruction(MirInstruction::Const {
        dst: val_a,
        value: ConstValue::Integer(6),
    });
    
    block.add_instruction(MirInstruction::Const {
        dst: val_b,
        value: ConstValue::Integer(7),
    });
    
    block.add_instruction(MirInstruction::BinOp {
        dst: result,
        op: BinaryOp::Mul,
        lhs: val_a,
        rhs: val_b,
    });
    
    block.add_instruction(MirInstruction::Return {
        value: Some(result),
    });
    
    main_function.add_block(block);
    module.add_function(main_function);
    
    module
}

/// Build MIR module for: 50 - 8 
fn build_subtraction_mir_module() -> MirModule {
    let mut module = MirModule::new("test_subtraction".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    let val_a = ValueId::new(0);     // 50
    let val_b = ValueId::new(1);     // 8  
    let result = ValueId::new(2);    // 50 - 8
    
    block.add_instruction(MirInstruction::Const {
        dst: val_a,
        value: ConstValue::Integer(50),
    });
    
    block.add_instruction(MirInstruction::Const {
        dst: val_b,
        value: ConstValue::Integer(8),
    });
    
    block.add_instruction(MirInstruction::BinOp {
        dst: result,
        op: BinaryOp::Sub,
        lhs: val_a,
        rhs: val_b,
    });
    
    block.add_instruction(MirInstruction::Return {
        value: Some(result),
    });
    
    main_function.add_block(block);
    module.add_function(main_function);
    
    module
}