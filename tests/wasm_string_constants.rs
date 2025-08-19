#![cfg(feature = "wasm-backend")]
/*!
 * WASM String Constants Test - Validates Issue #65 implementation
 * 
 * Tests string constant support in WASM backend:
 * - ConstValue::String handling in generate_const
 * - Data segment generation for string literals
 * - StringBox creation with proper layout
 * - WAT generation includes data segments and string allocation
 */

use nyash_rust::mir::{
    MirModule, MirFunction, FunctionSignature, MirType, EffectMask,
    BasicBlock, BasicBlockId, ValueId, MirInstruction, ConstValue
};
use nyash_rust::backend::wasm::WasmBackend;

#[test]
fn test_wasm_string_constant_basic() {
    // Build MIR equivalent to:
    // function main() {
    //     %str = const "Hello, WASM!"
    //     return %str  // Should return StringBox pointer
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_string_const_mir_module();
    
    // Generate WAT text for debugging
    let wat_result = backend.compile_to_wat(mir_module.clone());
    assert!(wat_result.is_ok(), "WAT generation should succeed for string constants");
    
    let wat_text = wat_result.unwrap();
    
    // Verify WAT contains expected elements for string support
    assert!(wat_text.contains("(module"), "Should contain module declaration");
    assert!(wat_text.contains("memory"), "Should contain memory declaration");
    assert!(wat_text.contains("data"), "Should contain data segment for string literal");
    assert!(wat_text.contains("\\48\\65\\6c\\6c\\6f"), "Should contain UTF-8 bytes for 'Hello'");
    assert!(wat_text.contains("$alloc_stringbox"), "Should contain StringBox allocator");
    assert!(wat_text.contains("print_str"), "Should contain print_str import");
    
    // Verify string literal is properly embedded
    // (The assertion for UTF-8 bytes is above)
    
    // Compile to WASM binary 
    let wasm_result = backend.compile_module(mir_module);
    if let Err(e) = &wasm_result {
        println!("WASM compilation error: {}", e);
        println!("Generated WAT:\n{}", wat_text);
    }
    assert!(wasm_result.is_ok(), "WASM compilation should succeed for string constants");
}

#[test]
fn test_wasm_string_constant_multiple() {
    // Test multiple string constants to verify data segment management
    // function main() {
    //     %str1 = const "First"
    //     %str2 = const "Second"  
    //     %str3 = const "First"   // Duplicate should reuse data segment
    //     return %str1
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_multiple_string_const_mir_module();
    
    let wat_result = backend.compile_to_wat(mir_module.clone());
    assert!(wat_result.is_ok(), "WAT generation should succeed for multiple strings");
    
    let wat_text = wat_result.unwrap();
    
    // Should contain both unique strings (in hex format)
    assert!(wat_text.contains("\\46\\69\\72\\73\\74"), "Should contain 'First' string in hex");
    assert!(wat_text.contains("\\53\\65\\63\\6f\\6e\\64"), "Should contain 'Second' string in hex");
    
    // Should have 2 data segments (First and Second, duplicate First reused)
    let data_count = wat_text.matches("(data").count();
    assert_eq!(data_count, 2, "Should have exactly 2 data segments for 2 unique strings");
    
    let wasm_result = backend.compile_module(mir_module);
    assert!(wasm_result.is_ok(), "WASM compilation should succeed for multiple strings");
}

/// Build a MIR module with a single string constant
fn build_string_const_mir_module() -> MirModule {
    let mut module = MirModule::new("test_string_const".to_string());
    
    // Create main function signature
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer, // StringBox pointer as i32
        effects: EffectMask::PURE,
    };
    
    // Create basic block
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    // %str = const "Hello, WASM!"
    let str_value = ValueId::new(0);
    block.instructions.push(MirInstruction::Const {
        dst: str_value,
        value: ConstValue::String("Hello, WASM!".to_string()),
    });
    
    // return %str
    block.terminator = Some(MirInstruction::Return {
        value: Some(str_value),
    });
    
    main_function.blocks.insert(entry_block, block);
    
    module.functions.insert("main".to_string(), main_function);
    module
}

/// Build a MIR module with multiple string constants
fn build_multiple_string_const_mir_module() -> MirModule {
    let mut module = MirModule::new("test_multiple_strings".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    // %str1 = const "First"
    let str1_value = ValueId::new(0);
    block.instructions.push(MirInstruction::Const {
        dst: str1_value,
        value: ConstValue::String("First".to_string()),
    });
    
    // %str2 = const "Second"
    let str2_value = ValueId::new(1);
    block.instructions.push(MirInstruction::Const {
        dst: str2_value,
        value: ConstValue::String("Second".to_string()),
    });
    
    // %str3 = const "First" (duplicate)
    let str3_value = ValueId::new(2);
    block.instructions.push(MirInstruction::Const {
        dst: str3_value,
        value: ConstValue::String("First".to_string()),
    });
    
    // return %str1
    block.terminator = Some(MirInstruction::Return {
        value: Some(str1_value),
    });
    
    main_function.blocks.insert(entry_block, block);
    
    module.functions.insert("main".to_string(), main_function);
    module
}
