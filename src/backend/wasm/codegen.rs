/*!
 * WASM Code Generation - Core MIR to WASM instruction conversion
 * 
 * Phase 8.2 PoC1: Basic operations (arithmetic, control flow, print)
 * Phase 8.3 PoC2: Reference operations (RefNew/RefGet/RefSet)
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, UnaryOp, ValueId, BasicBlockId};
use super::{WasmError, MemoryManager, RuntimeImports};
use std::collections::HashMap;

/// WASM module representation for WAT generation
pub struct WasmModule {
    pub imports: Vec<String>,
    pub memory: String,
    pub globals: Vec<String>,
    pub functions: Vec<String>,
    pub exports: Vec<String>,
}

impl WasmModule {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            memory: String::new(),
            globals: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }
    
    /// Generate WAT text format
    pub fn to_wat(&self) -> String {
        let mut wat = String::new();
        wat.push_str("(module\n");
        
        // Add imports first (must come before other definitions in WASM)
        for import in &self.imports {
            wat.push_str(&format!("  {}\n", import));
        }
        
        // Add memory declaration 
        if !self.memory.is_empty() {
            wat.push_str(&format!("  {}\n", self.memory));
        }
        
        // Add globals
        for global in &self.globals {
            wat.push_str(&format!("  {}\n", global));
        }
        
        // Add functions
        for function in &self.functions {
            wat.push_str(&format!("  {}\n", function));
        }
        
        // Add exports
        for export in &self.exports {
            wat.push_str(&format!("  {}\n", export));
        }
        
        wat.push_str(")\n");
        wat
    }
}

/// WASM code generator
pub struct WasmCodegen {
    /// Current function context for local variable management
    current_locals: HashMap<ValueId, u32>,
    next_local_index: u32,
}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {
            current_locals: HashMap::new(),
            next_local_index: 0,
        }
    }
    
    /// Generate WASM module from MIR module
    pub fn generate_module(
        &mut self, 
        mir_module: MirModule, 
        memory_manager: &MemoryManager, 
        runtime: &RuntimeImports
    ) -> Result<WasmModule, WasmError> {
        let mut wasm_module = WasmModule::new();
        
        // Add memory declaration (64KB initial)
        wasm_module.memory = "(memory (export \"memory\") 1)".to_string();
        
        // Add runtime imports (env.print for debugging)
        wasm_module.imports.extend(runtime.get_imports());
        
        // Add globals (heap pointer)
        wasm_module.globals.extend(memory_manager.get_globals());
        
        // Generate functions
        for (name, function) in &mir_module.functions {
            let wasm_function = self.generate_function(name, function.clone())?;
            wasm_module.functions.push(wasm_function);
        }
        
        // Add main function export if it exists
        if mir_module.functions.contains_key("main") {
            wasm_module.exports.push("(export \"main\" (func $main))".to_string());
        }
        
        Ok(wasm_module)
    }
    
    /// Generate WASM function from MIR function
    fn generate_function(&mut self, name: &str, mir_function: MirFunction) -> Result<String, WasmError> {
        // Reset local variable tracking for this function
        self.current_locals.clear();
        self.next_local_index = 0;
        
        let mut function_body = String::new();
        function_body.push_str(&format!("(func ${}", name));
        
        // Add return type if not void
        match mir_function.signature.return_type {
            crate::mir::MirType::Integer => function_body.push_str(" (result i32)"),
            crate::mir::MirType::Bool => function_body.push_str(" (result i32)"),
            crate::mir::MirType::Void => {}, // No return type
            _ => return Err(WasmError::UnsupportedInstruction(
                format!("Unsupported return type: {:?}", mir_function.signature.return_type)
            )),
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
        let entry_instructions = self.generate_basic_block(&mir_function, mir_function.entry_block)?;
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
    fn generate_basic_block(&self, mir_function: &MirFunction, block_id: BasicBlockId) -> Result<Vec<String>, WasmError> {
        let block = mir_function.blocks.get(&block_id)
            .ok_or_else(|| WasmError::CodegenError(format!("Basic block {:?} not found", block_id)))?;
        
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
    
    /// Generate WASM instructions for a single MIR instruction
    fn generate_instruction(&self, instruction: &MirInstruction) -> Result<Vec<String>, WasmError> {
        match instruction {
            // Phase 8.2 PoC1: Basic operations
            MirInstruction::Const { dst, value } => {
                self.generate_const(*dst, value)
            },
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                self.generate_binop(*dst, *op, *lhs, *rhs)
            },
            
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                self.generate_compare(*dst, *op, *lhs, *rhs)
            },
            
            MirInstruction::Return { value } => {
                self.generate_return(value.as_ref())
            },
            
            MirInstruction::Print { value, .. } => {
                self.generate_print(*value)
            },
            
            // Phase 8.3 PoC2: Reference operations (stub for now)
            MirInstruction::RefNew { dst, box_val } => {
                // For now, just copy the value (TODO: implement heap allocation)
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::RefGet { dst, reference, field: _ } => {
                // For now, just copy the reference (TODO: implement field access)
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*reference)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::RefSet { reference: _, field: _, value: _ } => {
                // For now, no-op (TODO: implement field assignment)
                Ok(vec!["nop".to_string()])
            },
            
            // Phase 8.4 PoC3: Extension stubs
            MirInstruction::WeakNew { dst, box_val } |
            MirInstruction::FutureNew { dst, value: box_val } => {
                // Treat as regular reference for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::WeakLoad { dst, weak_ref } |
            MirInstruction::Await { dst, future: weak_ref } => {
                // Always succeed for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*weak_ref)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::BarrierRead { .. } |
            MirInstruction::BarrierWrite { .. } |
            MirInstruction::FutureSet { .. } => {
                // No-op for now
                Ok(vec!["nop".to_string()])
            },
            
            // Control flow and debugging
            MirInstruction::Safepoint => {
                // Safepoint is a no-op in WASM (used for GC/debugging in other backends)
                Ok(vec!["nop".to_string()])
            },
            
            // Unsupported instructions
            _ => Err(WasmError::UnsupportedInstruction(
                format!("Instruction not yet supported: {:?}", instruction)
            )),
        }
    }
    
    /// Generate constant loading
    fn generate_const(&self, dst: ValueId, value: &ConstValue) -> Result<Vec<String>, WasmError> {
        let const_instruction = match value {
            ConstValue::Integer(n) => format!("i32.const {}", n),
            ConstValue::Bool(b) => format!("i32.const {}", if *b { 1 } else { 0 }),
            ConstValue::Void => "i32.const 0".to_string(),
            _ => return Err(WasmError::UnsupportedInstruction(
                format!("Unsupported constant type: {:?}", value)
            )),
        };
        
        Ok(vec![
            const_instruction,
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate binary operation
    fn generate_binop(&self, dst: ValueId, op: BinaryOp, lhs: ValueId, rhs: ValueId) -> Result<Vec<String>, WasmError> {
        let wasm_op = match op {
            BinaryOp::Add => "i32.add",
            BinaryOp::Sub => "i32.sub", 
            BinaryOp::Mul => "i32.mul",
            BinaryOp::Div => "i32.div_s",
            BinaryOp::And => "i32.and",
            BinaryOp::Or => "i32.or",
            _ => return Err(WasmError::UnsupportedInstruction(
                format!("Unsupported binary operation: {:?}", op)
            )),
        };
        
        Ok(vec![
            format!("local.get ${}", self.get_local_index(lhs)?),
            format!("local.get ${}", self.get_local_index(rhs)?),
            wasm_op.to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate comparison operation
    fn generate_compare(&self, dst: ValueId, op: CompareOp, lhs: ValueId, rhs: ValueId) -> Result<Vec<String>, WasmError> {
        let wasm_op = match op {
            CompareOp::Eq => "i32.eq",
            CompareOp::Ne => "i32.ne",
            CompareOp::Lt => "i32.lt_s",
            CompareOp::Le => "i32.le_s",
            CompareOp::Gt => "i32.gt_s",
            CompareOp::Ge => "i32.ge_s",
        };
        
        Ok(vec![
            format!("local.get ${}", self.get_local_index(lhs)?),
            format!("local.get ${}", self.get_local_index(rhs)?),
            wasm_op.to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate return instruction
    fn generate_return(&self, value: Option<&ValueId>) -> Result<Vec<String>, WasmError> {
        if let Some(value_id) = value {
            Ok(vec![
                format!("local.get ${}", self.get_local_index(*value_id)?),
                "return".to_string(),
            ])
        } else {
            Ok(vec!["return".to_string()])
        }
    }
    
    /// Generate print instruction (calls env.print import)
    fn generate_print(&self, value: ValueId) -> Result<Vec<String>, WasmError> {
        Ok(vec![
            format!("local.get ${}", self.get_local_index(value)?),
            "call $print".to_string(),
        ])
    }
    
    /// Get WASM local variable index for ValueId
    fn get_local_index(&self, value_id: ValueId) -> Result<u32, WasmError> {
        self.current_locals.get(&value_id)
            .copied()
            .ok_or_else(|| WasmError::CodegenError(format!("Local variable not found for ValueId: {:?}", value_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock, BasicBlockId, ValueId};
    
    #[test]
    fn test_wasm_module_wat_generation() {
        let mut module = WasmModule::new();
        module.memory = "(memory (export \"memory\") 1)".to_string();
        module.imports.push("(import \"env\" \"print\" (func $print (param i32)))".to_string());
        
        let wat = module.to_wat();
        assert!(wat.contains("(module"));
        assert!(wat.contains("memory"));
        assert!(wat.contains("import"));
    }
    
    #[test]
    fn test_constant_generation() {
        let codegen = WasmCodegen::new();
        let dst = ValueId::new(0);
        
        let result = codegen.generate_const(dst, &ConstValue::Integer(42));
        assert!(result.is_err()); // Should fail without local mapping
    }
}