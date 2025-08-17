/*!
 * WASM Code Generation - Core MIR to WASM instruction conversion
 * 
 * Phase 8.2 PoC1: Basic operations (arithmetic, control flow, print)
 * Phase 8.3 PoC2: Reference operations (RefNew/RefGet/RefSet)
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, ValueId, BasicBlockId};
use super::{WasmError, MemoryManager, RuntimeImports};
use std::collections::HashMap;

/// WASM module representation for WAT generation
pub struct WasmModule {
    pub imports: Vec<String>,
    pub memory: String,
    pub data_segments: Vec<String>,
    pub globals: Vec<String>,
    pub functions: Vec<String>,
    pub exports: Vec<String>,
}

impl WasmModule {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            memory: String::new(),
            data_segments: Vec::new(),
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
        
        // Add data segments (must come after memory)
        for data_segment in &self.data_segments {
            wat.push_str(&format!("  {}\n", data_segment));
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
        runtime: &RuntimeImports
    ) -> Result<WasmModule, WasmError> {
        let mut wasm_module = WasmModule::new();
        
        // Add memory declaration (64KB initial)
        wasm_module.memory = "(memory (export \"memory\") 1)".to_string();
        
        // Add runtime imports (env.print for debugging)
        wasm_module.imports.extend(runtime.get_imports());
        
        // Add globals (heap pointer)
        wasm_module.globals.extend(memory_manager.get_globals());
        
        // Add memory management functions
        wasm_module.functions.push(memory_manager.get_malloc_function());
        wasm_module.functions.push(memory_manager.get_generic_box_alloc_function());
        
        // Add Box-specific allocation functions for known types
        for box_type in ["StringBox", "IntegerBox", "BoolBox", "DataBox"] {
            if let Ok(alloc_func) = memory_manager.get_box_alloc_function(box_type) {
                wasm_module.functions.push(alloc_func);
            }
        }
        
        // Generate functions
        for (name, function) in &mir_module.functions {
            let wasm_function = self.generate_function(name, function.clone())?;
            wasm_module.functions.push(wasm_function);
        }
        
        // Add string literal data segments
        wasm_module.data_segments.extend(self.generate_data_segments());
        
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
    fn generate_basic_block(&mut self, mir_function: &MirFunction, block_id: BasicBlockId) -> Result<Vec<String>, WasmError> {
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
    fn generate_instruction(&mut self, instruction: &MirInstruction) -> Result<Vec<String>, WasmError> {
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
            
            // Phase 3: Print removed - now handled by Call intrinsic (@print)
            
            // Phase 8.3 PoC2: Reference operations
            MirInstruction::RefNew { dst, box_val } => {
                // Create a new reference to a Box by copying the Box value
                // This assumes box_val contains a Box pointer already
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            // Phase 3: RefGet/RefSet removed - now handled by BoxFieldLoad/BoxFieldStore
            
            MirInstruction::NewBox { dst, box_type, args } => {
                // Create a new Box using the generic allocator
                match box_type.as_str() {
                    "DataBox" => {
                        // Use specific allocator for known types
                        let mut instructions = vec![
                            "call $alloc_databox".to_string(),
                            format!("local.set ${}", self.get_local_index(*dst)?),
                        ];
                        
                        // Initialize fields with arguments if provided
                        for (i, arg) in args.iter().enumerate() {
                            instructions.extend(vec![
                                format!("local.get ${}", self.get_local_index(*dst)?),
                                format!("i32.const {}", 12 + i * 4), // Field offset
                                "i32.add".to_string(),
                                format!("local.get ${}", self.get_local_index(*arg)?),
                                "i32.store".to_string(),
                            ]);
                        }
                        
                        Ok(instructions)
                    },
                    _ => {
                        // Use generic allocator for unknown types
                        // This is a fallback - in a real implementation, all Box types should be known
                        Ok(vec![
                            "i32.const 8192".to_string(), // Default unknown type ID
                            format!("i32.const {}", args.len()),
                            "call $box_alloc".to_string(),
                            format!("local.set ${}", self.get_local_index(*dst)?),
                        ])
                    }
                }
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
            MirInstruction::FutureSet { .. } |
            MirInstruction::Safepoint => {
                // No-op for now
                Ok(vec!["nop".to_string()])
            },
            
            // Control Flow Instructions (Critical for loops and conditions)
            MirInstruction::Jump { target } => {
                // Unconditional jump to target basic block
                // Use WASM br instruction to break to the target block
                Ok(vec![
                    format!("br $block_{}", target.as_u32()),
                ])
            },
            
            MirInstruction::Branch { condition, then_bb, else_bb } => {
                // Conditional branch based on condition value
                // Load condition value and branch accordingly
                Ok(vec![
                    // Load condition value onto stack
                    format!("local.get ${}", self.get_local_index(*condition)?),
                    // If condition is true (non-zero), branch to then_bb
                    format!("br_if $block_{}", then_bb.as_u32()),
                    // Otherwise, fall through to else_bb
                    format!("br $block_{}", else_bb.as_u32()),
                ])
            },
            
            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                // Generate call to external function import
                let call_target = match (iface_name.as_str(), method_name.as_str()) {
                    ("env.console", "log") => "console_log",
                    ("env.canvas", "fillRect") => "canvas_fillRect", 
                    ("env.canvas", "fillText") => "canvas_fillText",
                    _ => return Err(WasmError::UnsupportedInstruction(
                        format!("Unsupported extern call: {}.{}", iface_name, method_name)
                    )),
                };
                
                let mut instructions = Vec::new();
                
                // Load all arguments onto stack in order
                for arg in args {
                    instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
                }
                
                // Call the external function
                instructions.push(format!("call ${}", call_target));
                
                // Store result if destination is provided
                if let Some(dst) = dst {
                    // For void functions, we still need to provide a dummy value
                    instructions.push("i32.const 0".to_string()); // Void result
                    instructions.push(format!("local.set ${}", self.get_local_index(*dst)?));
                }
                
                Ok(instructions)
            },
            
            // Phase 9.77: BoxCall Implementation - Critical Box method calls
            MirInstruction::BoxCall { dst, box_val, method, args, effects: _ } => {
                self.generate_box_call(*dst, *box_val, method, args)
            },
            
            // Phase 8.5: MIR 26-instruction reduction (NEW)
            MirInstruction::BoxFieldLoad { dst, box_val, field: _ } => {
                // Load field from box (similar to RefGet but with explicit Box semantics)
                // For now, assume all fields are at offset 12 (first field after header)
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    "i32.const 12".to_string(), // Box header + first field offset
                    "i32.add".to_string(),
                    "i32.load".to_string(),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::BoxFieldStore { box_val, field: _, value } => {
                // Store field to box (similar to RefSet but with explicit Box semantics)
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    "i32.const 12".to_string(), // Box header + first field offset
                    "i32.add".to_string(),
                    format!("local.get ${}", self.get_local_index(*value)?),
                    "i32.store".to_string(),
                ])
            },
            
            MirInstruction::WeakCheck { dst, weak_ref } => {
                // Check if weak reference is still alive
                // For now, always return 1 (true) - in full implementation,
                // this would check actual weak reference validity
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*weak_ref)?), // Touch the ref
                    "drop".to_string(), // Ignore the actual value
                    "i32.const 1".to_string(), // Always alive for now
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::Send { data, target } => {
                // Send data via Bus system - no-op for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*data)?),
                    format!("local.get ${}", self.get_local_index(*target)?),
                    "drop".to_string(), // Drop target
                    "drop".to_string(), // Drop data
                    "nop".to_string(),  // No actual send operation
                ])
            },
            
            MirInstruction::Recv { dst, source } => {
                // Receive data from Bus system - return constant for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*source)?), // Touch source
                    "drop".to_string(), // Ignore source
                    "i32.const 42".to_string(), // Placeholder received data
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            },
            
            MirInstruction::TailCall { func, args, effects: _ } => {
                // Tail call optimization - simplified as regular call for now
                let mut instructions = Vec::new();
                
                // Load all arguments
                for arg in args {
                    instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
                }
                
                // Call function (assuming it's a function index)
                instructions.push(format!("local.get ${}", self.get_local_index(*func)?));
                instructions.push("call_indirect".to_string());
                
                Ok(instructions)
            },
            
            MirInstruction::Adopt { parent, child } => {
                // Adopt ownership - no-op for now in WASM
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*parent)?),
                    format!("local.get ${}", self.get_local_index(*child)?),
                    "drop".to_string(), // Drop child
                    "drop".to_string(), // Drop parent
                    "nop".to_string(),  // No actual adoption
                ])
            },
            
            MirInstruction::Release { reference } => {
                // Release strong ownership - no-op for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*reference)?),
                    "drop".to_string(), // Drop reference
                    "nop".to_string(),  // No actual release
                ])
            },
            
            MirInstruction::MemCopy { dst, src, size } => {
                // Memory copy optimization - simple copy for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*src)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                    // Size is ignored for now - in full implementation,
                    // this would use memory.copy instruction
                    format!("local.get ${}", self.get_local_index(*size)?),
                    "drop".to_string(),
                ])
            },
            
            MirInstruction::AtomicFence { ordering: _ } => {
                // Atomic memory fence - no-op for now
                // WASM doesn't have direct memory fence instructions
                // In full implementation, this might use atomic wait/notify
                Ok(vec!["nop".to_string()])
            },
            
            // Phase 4: Call instruction for intrinsic functions
            MirInstruction::Call { dst, func, args, effects: _ } => {
                self.generate_call_instruction(dst.as_ref(), *func, args)
            },
            
            // Unsupported instructions
            _ => Err(WasmError::UnsupportedInstruction(
                format!("Instruction not yet supported: {:?}", instruction)
            )),
        }
    }
    
    /// Generate constant loading
    fn generate_const(&mut self, dst: ValueId, value: &ConstValue) -> Result<Vec<String>, WasmError> {
        let const_instruction = match value {
            ConstValue::Integer(n) => format!("i32.const {}", n),
            ConstValue::Bool(b) => format!("i32.const {}", if *b { 1 } else { 0 }),
            ConstValue::Void => "i32.const 0".to_string(),
            ConstValue::String(s) => {
                // Register the string literal and get its offset
                let data_offset = self.register_string_literal(s);
                let string_len = s.len() as u32;
                
                // Generate code to allocate a StringBox and return its pointer
                // This is more complex and will need StringBox allocation
                return self.generate_string_box_const(dst, data_offset, string_len);
            },
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
    
    /// Generate StringBox allocation for a string constant
    fn generate_string_box_const(&self, dst: ValueId, data_offset: u32, string_len: u32) -> Result<Vec<String>, WasmError> {
        // Allocate a StringBox using the StringBox allocator
        // StringBox layout: [type_id:0x1001][ref_count:1][field_count:2][data_ptr:offset][length:len]
        Ok(vec![
            // Call StringBox allocator function
            "call $alloc_stringbox".to_string(),
            // Store the result (StringBox pointer) in local variable
            format!("local.set ${}", self.get_local_index(dst)?),
            
            // Initialize StringBox fields
            // Get StringBox pointer back
            format!("local.get ${}", self.get_local_index(dst)?),
            // Set data_ptr field (offset 12 from StringBox pointer)
            "i32.const 12".to_string(),
            "i32.add".to_string(),
            format!("i32.const {}", data_offset),
            "i32.store".to_string(),
            
            // Get StringBox pointer again  
            format!("local.get ${}", self.get_local_index(dst)?),
            // Set length field (offset 16 from StringBox pointer)
            "i32.const 16".to_string(),
            "i32.add".to_string(),
            format!("i32.const {}", string_len),
            "i32.store".to_string(),
        ])
    }
    
    /// Generate print instruction (calls env.print import)
    fn generate_print(&self, value: ValueId) -> Result<Vec<String>, WasmError> {
        Ok(vec![
            format!("local.get ${}", self.get_local_index(value)?),
            "call $print".to_string(),
        ])
    }
    
    /// Register a string literal and return its data offset
    fn register_string_literal(&mut self, string: &str) -> u32 {
        if let Some(&offset) = self.string_literals.get(string) {
            return offset;
        }
        
        let offset = self.next_data_offset;
        let string_bytes = string.as_bytes();
        self.string_literals.insert(string.to_string(), offset);
        self.next_data_offset += string_bytes.len() as u32;
        
        offset
    }
    
    /// Generate data segments for all registered string literals
    fn generate_data_segments(&self) -> Vec<String> {
        let mut segments = Vec::new();
        
        for (string, &offset) in &self.string_literals {
            let string_bytes = string.as_bytes();
            
            // Convert to hex-escaped string for WAT
            let byte_string = string_bytes.iter()
                .map(|b| format!("\\{:02x}", b))
                .collect::<String>();
            
            let data_segment = format!(
                "(data (i32.const {}) \"{}\")",
                offset,
                byte_string
            );
            
            segments.push(data_segment);
        }
        
        segments
    }
    
    /// Get WASM local variable index for ValueId
    fn get_local_index(&self, value_id: ValueId) -> Result<u32, WasmError> {
        self.current_locals.get(&value_id)
            .copied()
            .ok_or_else(|| WasmError::CodegenError(format!("Local variable not found for ValueId: {:?}", value_id)))
    }
    
    /// Phase 9.77: Generate BoxCall method invocation
    /// Implements critical Box methods: toString, print, equals, clone
    fn generate_box_call(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, args: &[ValueId]) -> Result<Vec<String>, WasmError> {
        match method {
            "toString" => self.generate_to_string_call(dst, box_val),
            "print" => self.generate_print_call(dst, box_val),
            "equals" => self.generate_equals_call(dst, box_val, args),
            "clone" => self.generate_clone_call(dst, box_val),
            "log" => self.generate_log_call(dst, box_val, args),
            _ => Err(WasmError::UnsupportedInstruction(
                format!("Unsupported BoxCall method: {}", method)
            ))
        }
    }
    
    /// Generate toString() method call - Box → String conversion
    fn generate_to_string_call(&mut self, dst: Option<ValueId>, box_val: ValueId) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError("toString() requires destination".to_string()));
        };
        
        Ok(vec![
            format!(";; toString() implementation for ValueId({})", box_val.as_u32()),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_to_string".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate print() method call - Basic output
    fn generate_print_call(&mut self, dst: Option<ValueId>, box_val: ValueId) -> Result<Vec<String>, WasmError> {
        let mut instructions = vec![
            format!(";; print() implementation for ValueId({})", box_val.as_u32()),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_print".to_string(),
        ];
        
        // Store void result if destination is provided
        if let Some(dst) = dst {
            instructions.extend(vec![
                "i32.const 0".to_string(), // Void result
                format!("local.set ${}", self.get_local_index(dst)?),
            ]);
        }
        
        Ok(instructions)
    }
    
    /// Generate equals() method call - Box comparison
    fn generate_equals_call(&mut self, dst: Option<ValueId>, box_val: ValueId, args: &[ValueId]) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError("equals() requires destination".to_string()));
        };
        
        if args.len() != 1 {
            return Err(WasmError::CodegenError(
                format!("equals() expects 1 argument, got {}", args.len())
            ));
        }
        
        Ok(vec![
            format!(";; equals() implementation for ValueId({}) == ValueId({})", box_val.as_u32(), args[0].as_u32()),
            format!("local.get ${}", self.get_local_index(box_val)?),
            format!("local.get ${}", self.get_local_index(args[0])?),
            "call $box_equals".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate clone() method call - Box duplication
    fn generate_clone_call(&mut self, dst: Option<ValueId>, box_val: ValueId) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError("clone() requires destination".to_string()));
        };
        
        Ok(vec![
            format!(";; clone() implementation for ValueId({})", box_val.as_u32()),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_clone".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }
    
    /// Generate log() method call - Console logging (ConsoleBox.log)
    fn generate_log_call(&mut self, dst: Option<ValueId>, box_val: ValueId, args: &[ValueId]) -> Result<Vec<String>, WasmError> {
        let mut instructions = vec![
            format!(";; log() implementation for ValueId({})", box_val.as_u32()),
        ];
        
        // Load box_val (ConsoleBox instance)
        instructions.push(format!("local.get ${}", self.get_local_index(box_val)?));
        
        // Load all arguments
        for arg in args {
            instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
        }
        
        // Call console log function
        instructions.push("call $console_log".to_string());
        
        // Store void result if destination is provided
        if let Some(dst) = dst {
            instructions.extend(vec![
                "i32.const 0".to_string(), // Void result
                format!("local.set ${}", self.get_local_index(dst)?),
            ]);
        }
        
        Ok(instructions)
    }
    
    /// Generate Call instruction for intrinsic functions (Phase 4)
    fn generate_call_instruction(&mut self, dst: Option<&ValueId>, func: ValueId, args: &[ValueId]) -> Result<Vec<String>, WasmError> {
        // Get the function name from the func ValueId
        // In MIR, intrinsic function names are stored as string constants
        let mut instructions = Vec::new();
        
        // For intrinsic functions, we handle them based on their name
        // The func ValueId should contain a string constant like "@print"
        
        // For now, assume all calls are @print intrinsic
        // TODO: Implement proper function name resolution from ValueId
        
        // Load all arguments onto stack in order
        for arg in args {
            instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
        }
        
        // Call the print function (assuming it's imported as $print)
        instructions.push("call $print".to_string());
        
        // Store result if destination is provided
        if let Some(dst) = dst {
            // Intrinsic functions typically return void, but we provide a dummy value
            instructions.push("i32.const 0".to_string()); // Void result
            instructions.push(format!("local.set ${}", self.get_local_index(*dst)?));
        }
        
        Ok(instructions)
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
        let mut codegen = WasmCodegen::new();
        let dst = ValueId::new(0);
        
        let result = codegen.generate_const(dst, &ConstValue::Integer(42));
        assert!(result.is_err()); // Should fail without local mapping
    }
}