/*!
 * Phase 8.3 PoC2 Integration Test - Box Operations in WASM
 * 
 * Tests end-to-end MIR→WASM compilation and execution for:
 * - RefNew: Box creation and reference assignment
 * - RefGet: Field reading from Box objects
 * - RefSet: Field writing to Box objects  
 * - NewBox: Direct Box allocation with type information
 * 
 * Validates the "Everything is Box" philosophy in WASM
 */

use nyash_rust::mir::{
    MirModule, MirFunction, FunctionSignature, MirType, EffectMask,
    BasicBlock, BasicBlockId, ValueId, MirInstruction, ConstValue
};
use nyash_rust::backend::wasm::WasmBackend;

#[test]
fn test_wasm_poc2_refnew_basic() {
    // Build MIR equivalent to:
    // function main() {
    //     %box = new_box "DataBox"(42)
    //     %ref = ref_new %box  
    //     return %ref  // Should return box pointer
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_refnew_mir_module();
    
    // Generate WAT text for debugging
    let wat_result = backend.compile_to_wat(mir_module.clone());
    assert!(wat_result.is_ok(), "WAT generation should succeed");
    
    let wat_text = wat_result.unwrap();
    
    // Verify WAT contains expected elements
    assert!(wat_text.contains("(module"), "Should contain module declaration");
    assert!(wat_text.contains("$malloc"), "Should contain malloc function");
    assert!(wat_text.contains("$alloc_databox"), "Should contain DataBox allocator");
    assert!(wat_text.contains("call $alloc_databox"), "Should call DataBox allocator");
    assert!(wat_text.contains("i32.store"), "Should store field values");
    
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
    // Should return a valid pointer (greater than heap start 0x800)
    assert!(return_value >= 0x800, "Should return valid Box pointer: {}", return_value);
}

#[test]
fn test_wasm_poc2_refget_refset() {
    // Build MIR equivalent to:
    // function main() {
    //     %box = new_box "DataBox"(10)
    //     %ref = ref_new %box
    //     ref_set %ref.value = 42
    //     %result = ref_get %ref.value
    //     return %result  // Should return 42
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_refget_refset_mir_module();
    
    let wasm_result = backend.compile_module(mir_module);
    assert!(wasm_result.is_ok(), "WASM compilation should succeed");
    
    let return_value = backend.execute_wasm(&wasm_result.unwrap()).unwrap();
    assert_eq!(return_value, 42, "Should return updated field value");
}

#[test]
fn test_wasm_poc2_complete_box_workflow() {
    // Build MIR equivalent to:
    // function main() {
    //     %box1 = new_box "DataBox"(100)
    //     %box2 = new_box "DataBox"(200)
    //     %ref1 = ref_new %box1
    //     %ref2 = ref_new %box2
    //     %val1 = ref_get %ref1.value
    //     %val2 = ref_get %ref2.value
    //     %sum = %val1 + %val2
    //     ref_set %ref1.value = %sum
    //     %result = ref_get %ref1.value
    //     return %result  // Should return 300
    // }
    
    let mut backend = WasmBackend::new();
    let mir_module = build_complete_workflow_mir_module();
    
    let wasm_result = backend.compile_module(mir_module);
    assert!(wasm_result.is_ok(), "WASM compilation should succeed");
    
    let return_value = backend.execute_wasm(&wasm_result.unwrap()).unwrap();
    assert_eq!(return_value, 300, "Should return sum of Box values");
}

/// Build MIR module for basic RefNew test
fn build_refnew_mir_module() -> MirModule {
    let mut module = MirModule::new("test_refnew".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    let init_val = ValueId::new(0);    // 42
    let box_ptr = ValueId::new(1);     // DataBox pointer
    let ref_ptr = ValueId::new(2);     // Reference to DataBox
    
    // Create constant for initialization
    block.add_instruction(MirInstruction::Const {
        dst: init_val,
        value: ConstValue::Integer(42),
    });
    
    // Create DataBox with initial value
    block.add_instruction(MirInstruction::NewBox {
        dst: box_ptr,
        box_type: "DataBox".to_string(),
        args: vec![init_val],
    });
    
    // Create reference to the Box
    block.add_instruction(MirInstruction::RefNew {
        dst: ref_ptr,
        box_val: box_ptr,
    });
    
    // Return the reference
    block.set_terminator(MirInstruction::Return {
        value: Some(ref_ptr),
    });
    
    main_function.add_block(block);
    module.add_function(main_function);
    
    module
}

/// Build MIR module for RefGet/RefSet test
fn build_refget_refset_mir_module() -> MirModule {
    let mut module = MirModule::new("test_refget_refset".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    let init_val = ValueId::new(0);    // 10
    let new_val = ValueId::new(1);     // 42
    let box_ptr = ValueId::new(2);     // DataBox pointer
    let ref_ptr = ValueId::new(3);     // Reference to DataBox
    let result = ValueId::new(4);      // Read back value
    
    // Create constants
    block.add_instruction(MirInstruction::Const {
        dst: init_val,
        value: ConstValue::Integer(10),
    });
    
    block.add_instruction(MirInstruction::Const {
        dst: new_val,
        value: ConstValue::Integer(42),
    });
    
    // Create DataBox with initial value
    block.add_instruction(MirInstruction::NewBox {
        dst: box_ptr,
        box_type: "DataBox".to_string(),
        args: vec![init_val],
    });
    
    // Create reference to the Box
    block.add_instruction(MirInstruction::RefNew {
        dst: ref_ptr,
        box_val: box_ptr,
    });
    
    // Set field value
    block.add_instruction(MirInstruction::RefSet {
        reference: ref_ptr,
        field: "value".to_string(),
        value: new_val,
    });
    
    // Get field value
    block.add_instruction(MirInstruction::RefGet {
        dst: result,
        reference: ref_ptr,
        field: "value".to_string(),
    });
    
    // Return the result
    block.set_terminator(MirInstruction::Return {
        value: Some(result),
    });
    
    main_function.add_block(block);
    module.add_function(main_function);
    
    module
}

/// Build MIR module for complete Box workflow test
fn build_complete_workflow_mir_module() -> MirModule {
    let mut module = MirModule::new("test_complete_workflow".to_string());
    
    let main_signature = FunctionSignature {
        name: "main".to_string(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    
    let entry_block = BasicBlockId::new(0);
    let mut main_function = MirFunction::new(main_signature, entry_block);
    let mut block = BasicBlock::new(entry_block);
    
    let val1_init = ValueId::new(0);   // 100
    let val2_init = ValueId::new(1);   // 200
    let box1_ptr = ValueId::new(2);    // DataBox 1 pointer
    let box2_ptr = ValueId::new(3);    // DataBox 2 pointer
    let ref1_ptr = ValueId::new(4);    // Reference to DataBox 1
    let ref2_ptr = ValueId::new(5);    // Reference to DataBox 2
    let val1 = ValueId::new(6);        // Value from box1
    let val2 = ValueId::new(7);        // Value from box2
    let sum = ValueId::new(8);         // Sum of values
    let result = ValueId::new(9);      // Final result
    
    // Create constants
    block.add_instruction(MirInstruction::Const {
        dst: val1_init,
        value: ConstValue::Integer(100),
    });
    
    block.add_instruction(MirInstruction::Const {
        dst: val2_init,
        value: ConstValue::Integer(200),
    });
    
    // Create DataBoxes
    block.add_instruction(MirInstruction::NewBox {
        dst: box1_ptr,
        box_type: "DataBox".to_string(),
        args: vec![val1_init],
    });
    
    block.add_instruction(MirInstruction::NewBox {
        dst: box2_ptr,
        box_type: "DataBox".to_string(),
        args: vec![val2_init],
    });
    
    // Create references
    block.add_instruction(MirInstruction::RefNew {
        dst: ref1_ptr,
        box_val: box1_ptr,
    });
    
    block.add_instruction(MirInstruction::RefNew {
        dst: ref2_ptr,
        box_val: box2_ptr,
    });
    
    // Get values from both boxes
    block.add_instruction(MirInstruction::RefGet {
        dst: val1,
        reference: ref1_ptr,
        field: "value".to_string(),
    });
    
    block.add_instruction(MirInstruction::RefGet {
        dst: val2,
        reference: ref2_ptr,
        field: "value".to_string(),
    });
    
    // Add values
    block.add_instruction(MirInstruction::BinOp {
        dst: sum,
        op: nyash_rust::mir::BinaryOp::Add,
        lhs: val1,
        rhs: val2,
    });
    
    // Store sum back to first box
    block.add_instruction(MirInstruction::RefSet {
        reference: ref1_ptr,
        field: "value".to_string(),
        value: sum,
    });
    
    // Read back the result
    block.add_instruction(MirInstruction::RefGet {
        dst: result,
        reference: ref1_ptr,
        field: "value".to_string(),
    });
    
    // Return the result
    block.set_terminator(MirInstruction::Return {
        value: Some(result),
    });
    
    main_function.add_block(block);
    module.add_function(main_function);
    
    module
}