/*!
 * WASM Code Generation - Core MIR to WASM instruction conversion
 *
 * Phase 8.2 PoC1: Basic operations (arithmetic, control flow, print)
 * Phase 8.3 PoC2: Reference operations (RefNew/RefGet/RefSet)
 */

mod boxcall;
mod instructions;
mod strings;
mod utils;
mod wat;

use crate::backend::wasm::{MemoryManager, RuntimeImports, WasmError};
use crate::mir::{BasicBlockId, MirFunction, MirModule, ValueId};
use std::collections::HashMap;

pub use wat::WasmModule;

/// WASM code generator
pub struct WasmCodegen {
    /// Current function context for local variable management
    current_locals: HashMap<ValueId, u32>,
    next_local_index: u32,
    /// String literals and their data segment offsets
    string_literals: HashMap<String, u32>,
    next_data_offset: u32,
}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {
            current_locals: HashMap::new(),
            next_local_index: 0,
            string_literals: HashMap::new(),
            next_data_offset: 0x1000, // Start data after initial heap space
        }
    }

    /// Generate WASM module from MIR module
    pub fn generate_module(
        &mut self,
        mir_module: MirModule,
        memory_manager: &MemoryManager,
        runtime: &RuntimeImports,
    ) -> Result<WasmModule, WasmError> {
        let mut wasm_module = WasmModule::new();

        // Add memory declaration (64KB initial)
        wasm_module.memory = "(memory (export \"memory\") 1)".to_string();

        // Add runtime imports (env.print for debugging)
        wasm_module.imports.extend(runtime.get_imports());

        // Add globals (heap pointer)
        wasm_module.globals.extend(memory_manager.get_globals());

        // Add memory management functions
        wasm_module
            .functions
            .push(memory_manager.get_malloc_function());
        wasm_module
            .functions
            .push(memory_manager.get_generic_box_alloc_function());

        // Add Box-specific allocation functions for known types
        for box_type in ["StringBox", "IntegerBox", "BoolBox", "DataBox", "ArrayBox"] {
            if let Ok(alloc_func) = memory_manager.get_box_alloc_function(box_type) {
                wasm_module.functions.push(alloc_func);
            }
        }

        // Generate functions
        for (name, function) in &mir_module.functions {
            let wasm_function = self.generate_function(name, function.clone())?;
            wasm_module.functions.push(wasm_function);
        }

        // Add ArrayBox helper functions (alloc wrapper and ops)
        if let Ok(funcs) = memory_manager.get_array_helpers() {
            wasm_module.functions.extend(funcs);
        }

        // Add string literal data segments
        wasm_module
            .data_segments
            .extend(self.generate_data_segments());

        // Add main function export if it exists
        if mir_module.functions.contains_key("main") {
            wasm_module
                .exports
                .push("(export \"main\" (func $main))".to_string());
        }

        Ok(wasm_module)
    }

    /// Generate WASM function from MIR function
    fn generate_function(
        &mut self,
        name: &str,
        mir_function: MirFunction,
    ) -> Result<String, WasmError> {
        // Reset local variable tracking for this function
        self.current_locals.clear();
        self.next_local_index = 0;

        let mut function_body = String::new();
        function_body.push_str(&format!("(func ${}", name));

        // Add return type if not void
        match mir_function.signature.return_type {
            crate::mir::MirType::Integer => function_body.push_str(" (result i32)"),
            crate::mir::MirType::Bool => function_body.push_str(" (result i32)"),
            crate::mir::MirType::Void => {} // No return type
            _ => {
                return Err(WasmError::UnsupportedInstruction(format!(
                    "Unsupported return type: {:?}",
                    mir_function.signature.return_type
                )))
            }
        }

        // Collect all local variables needed
        let local_count = self.count_locals(&mir_function)?;
        if local_count > 0 {
            // Declare individual local variables for each ValueId
            for i in 0..local_count {
                function_body.push_str(&format!(" (local ${} i32)", i));
            }
        }

        function_body.push('\n');

        // Generate body from entry block
        let entry_instructions =
            self.generate_basic_block(&mir_function, mir_function.entry_block)?;
        for instruction in entry_instructions {
            function_body.push_str(&format!("    {}\n", instruction));
        }

        function_body.push_str("  )");
        Ok(function_body)
    }

    /// Count local variables needed for the function
    fn count_locals(&mut self, mir_function: &MirFunction) -> Result<u32, WasmError> {
        let mut max_value_id = 0;

        for (_, block) in &mir_function.blocks {
            for instruction in &block.instructions {
                if let Some(value_id) = instruction.dst_value() {
                    max_value_id = max_value_id.max(value_id.as_u32());
                }
                for used_value in instruction.used_values() {
                    max_value_id = max_value_id.max(used_value.as_u32());
                }
            }
        }

        // Assign local indices to value IDs
        for i in 0..=max_value_id {
            let value_id = ValueId::new(i);
            self.current_locals.insert(value_id, self.next_local_index);
            self.next_local_index += 1;
        }

        Ok(self.next_local_index)
    }

    /// Generate WASM instructions for a basic block
    fn generate_basic_block(
        &mut self,
        mir_function: &MirFunction,
        block_id: BasicBlockId,
    ) -> Result<Vec<String>, WasmError> {
        let block = mir_function.blocks.get(&block_id).ok_or_else(|| {
            WasmError::CodegenError(format!("Basic block {:?} not found", block_id))
        })?;

        let mut instructions = Vec::new();

        // Process regular instructions
        for mir_instruction in &block.instructions {
            let wasm_instructions = self.generate_instruction(mir_instruction)?;
            instructions.extend(wasm_instructions);
        }

        // Process terminator instruction
        if let Some(ref terminator) = block.terminator {
            let wasm_instructions = self.generate_instruction(terminator)?;
            instructions.extend(wasm_instructions);
        }

        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        BasicBlock, BasicBlockId, EffectMask, FunctionSignature, MirFunction, MirModule, MirType,
        ValueId, ConstValue,
    };

    #[test]
    fn test_wasm_module_wat_generation() {
        let mut module = WasmModule::new();
        module.memory = "(memory (export \"memory\") 1)".to_string();
        module
            .imports
            .push("(import \"env\" \"print\" (func $print (param i32)))".to_string());

        let wat = module.to_wat();
        assert!(wat.contains("(module"));
        assert!(wat.contains("memory"));
        assert!(wat.contains("import"));
    }

    #[test]
    fn test_constant_generation() {
        let mut codegen = WasmCodegen::new();
        let dst = ValueId::new(0);

        let result = codegen.generate_const(dst, &ConstValue::Integer(42));
        assert!(result.is_err()); // Should fail without local mapping
    }
}
