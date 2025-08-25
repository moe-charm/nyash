/*!
 * VM Backend - Execute MIR instructions in a virtual machine
 * 
 * Simple stack-based VM for executing MIR code
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, UnaryOp, ValueId, BasicBlockId};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox};
use std::collections::HashMap;
use std::sync::Arc;
use crate::runtime::NyashRuntime;
use crate::scope_tracker::ScopeTracker;
// MirModule is already imported via crate::mir at top
use crate::instance_v2::InstanceBox;
use super::vm_phi::LoopExecutor;
use std::time::Instant;

// Phase 9.78a: Import necessary components for unified Box handling
// TODO: Re-enable when interpreter refactoring is complete
// use crate::box_factory::UnifiedBoxRegistry;
// use crate::instance_v2::InstanceBox;
// use crate::interpreter::BoxDeclaration;
// use crate::scope_tracker::ScopeTracker;
// #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
// use crate::runtime::plugin_loader_v2::PluginLoaderV2;

/// VM execution error
#[derive(Debug)]
pub enum VMError {
    InvalidValue(String),
    InvalidInstruction(String),
    InvalidBasicBlock(String),
    DivisionByZero,
    StackUnderflow,
    TypeError(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            VMError::InvalidInstruction(msg) => write!(f, "Invalid instruction: {}", msg),
            VMError::InvalidBasicBlock(msg) => write!(f, "Invalid basic block: {}", msg),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::TypeError(msg) => write!(f, "Type error: {}", msg),
        }
    }
}

impl std::error::Error for VMError {}

/// VM value representation
#[derive(Debug, Clone)]
pub enum VMValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Future(crate::boxes::future::FutureBox),
    Void,
    // Phase 9.78a: Add BoxRef for complex Box types
    BoxRef(Arc<dyn NyashBox>),
}

// Manual PartialEq implementation to avoid requiring PartialEq on FutureBox
impl PartialEq for VMValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VMValue::Integer(a), VMValue::Integer(b)) => a == b,
            (VMValue::Float(a), VMValue::Float(b)) => a == b,
            (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
            (VMValue::String(a), VMValue::String(b)) => a == b,
            (VMValue::Void, VMValue::Void) => true,
            // Future equality semantics are not defined; treat distinct futures as not equal
            (VMValue::Future(_), VMValue::Future(_)) => false,
            // BoxRef equality by reference
            (VMValue::BoxRef(_), VMValue::BoxRef(_)) => false,
            _ => false,
        }
    }
}

impl VMValue {
    /// Convert to NyashBox for output
    pub fn to_nyash_box(&self) -> Box<dyn NyashBox> {
        match self {
            VMValue::Integer(i) => Box::new(IntegerBox::new(*i)),
            VMValue::Float(f) => Box::new(StringBox::new(&f.to_string())), // Simplified for now
            VMValue::Bool(b) => Box::new(BoolBox::new(*b)),
            VMValue::String(s) => Box::new(StringBox::new(s)),
            VMValue::Future(f) => Box::new(f.clone()),
            VMValue::Void => Box::new(VoidBox::new()),
            // BoxRef returns a shared handle (do NOT birth a new instance)
            VMValue::BoxRef(arc_box) => arc_box.share_box(),
        }
    }
    
    
    /// Get string representation for printing
    pub fn to_string(&self) -> String {
        match self {
            VMValue::Integer(i) => i.to_string(),
            VMValue::Float(f) => f.to_string(),
            VMValue::Bool(b) => b.to_string(),
            VMValue::String(s) => s.clone(),
            VMValue::Future(f) => f.to_string_box().value,
            VMValue::Void => "void".to_string(),
            VMValue::BoxRef(arc_box) => arc_box.to_string_box().value,
        }
    }
    
    /// Attempt to convert to integer
    pub fn as_integer(&self) -> Result<i64, VMError> {
        match self {
            VMValue::Integer(i) => Ok(*i),
            _ => Err(VMError::TypeError(format!("Expected integer, got {:?}", self))),
        }
    }
    
    /// Attempt to convert to bool
    pub fn as_bool(&self) -> Result<bool, VMError> {
        match self {
            VMValue::Bool(b) => Ok(*b),
            VMValue::Integer(i) => Ok(*i != 0),
            // Pragmatic coercions for dynamic boxes
            VMValue::BoxRef(b) => {
                // BoolBox → bool
                if let Some(bb) = b.as_any().downcast_ref::<BoolBox>() {
                    return Ok(bb.value);
                }
                // VoidBox → false (nullish false)
                if b.as_any().downcast_ref::<VoidBox>().is_some() {
                    return Ok(false);
                }
                Err(VMError::TypeError(format!("Expected bool, got BoxRef({})", b.type_name())))
            }
            _ => Err(VMError::TypeError(format!("Expected bool, got {:?}", self))),
        }
    }
    
    /// Convert from NyashBox to VMValue  
    pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
        // Try to downcast to known types for optimization
        if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
            VMValue::Integer(int_box.value)
        } else if let Some(bool_box) = nyash_box.as_any().downcast_ref::<BoolBox>() {
            VMValue::Bool(bool_box.value)
        } else if let Some(string_box) = nyash_box.as_any().downcast_ref::<StringBox>() {
            VMValue::String(string_box.value.clone())
        } else if let Some(future_box) = nyash_box.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
            VMValue::Future(future_box.clone())
        } else {
            // Phase 9.78a: For all other Box types (user-defined, plugin), store as BoxRef
            VMValue::BoxRef(Arc::from(nyash_box))
        }
    }
}

impl From<&ConstValue> for VMValue {
    fn from(const_val: &ConstValue) -> Self {
        match const_val {
            ConstValue::Integer(i) => VMValue::Integer(*i),
            ConstValue::Float(f) => VMValue::Float(*f),
            ConstValue::Bool(b) => VMValue::Bool(*b),
            ConstValue::String(s) => VMValue::String(s.clone()),
            ConstValue::Null => VMValue::Void, // Simplified
            ConstValue::Void => VMValue::Void,
        }
    }
}

/// Virtual Machine state
pub struct VM {
    /// Value storage (uses ValueId as direct index into Vec for O(1) access)
    values: Vec<Option<VMValue>>,
    /// Current function being executed
    current_function: Option<String>,
    /// Current basic block
    current_block: Option<BasicBlockId>,
    /// Previous basic block (for phi node resolution)
    previous_block: Option<BasicBlockId>,
    /// Program counter within current block
    pc: usize,
    /// Return value from last execution
    #[allow(dead_code)]
    last_result: Option<VMValue>,
    /// Simple field storage for objects (maps reference -> field -> value)
    pub(super) object_fields: HashMap<ValueId, HashMap<String, VMValue>>,
    /// Class name mapping for objects (for visibility checks)
    pub(super) object_class: HashMap<ValueId, String>,
    /// Marks ValueIds that represent internal (me/this) references within the current function
    pub(super) object_internal: std::collections::HashSet<ValueId>,
    /// Loop executor for handling phi nodes and loop-specific logic
    loop_executor: LoopExecutor,
    /// Shared runtime for box creation and declarations
    pub(super) runtime: NyashRuntime,
    /// Scope tracker for calling fini on scope exit
    scope_tracker: ScopeTracker,
    /// Active MIR module during execution (for function calls)
    module: Option<MirModule>,
    /// Instruction execution counters (by MIR opcode)
    instr_counter: std::collections::HashMap<&'static str, usize>,
    /// Execution start time for optional stats
    exec_start: Option<Instant>,
    // Phase 9.78a: Add unified Box handling components
    // TODO: Re-enable when interpreter refactoring is complete
    // /// Box registry for creating all Box types
    // box_registry: Arc<UnifiedBoxRegistry>,
    // /// Plugin loader for external Box types
    // #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    // plugin_loader: Option<Arc<PluginLoaderV2>>,
    // Scope tracker for lifecycle management
    // scope_tracker: ScopeTracker,
    // /// Box declarations from the AST
    // box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            current_block: None,
            previous_block: None,
            pc: 0,
            last_result: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime: NyashRuntime::new(),
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
            // TODO: Re-enable when interpreter refactoring is complete
            // box_registry: Arc::new(UnifiedBoxRegistry::new()),
            // #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
            // plugin_loader: None,
            // scope_tracker: ScopeTracker::new(),
            // box_declarations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a VM with an external runtime (dependency injection)
    pub fn with_runtime(runtime: NyashRuntime) -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            current_block: None,
            previous_block: None,
            pc: 0,
            last_result: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime,
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
        }
    }
    
    // TODO: Re-enable when interpreter refactoring is complete
    /*
    /// Create a new VM instance with Box registry and declarations
    pub fn new_with_registry(
        box_registry: Arc<UnifiedBoxRegistry>, 
        box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>
    ) -> Self {
        // Implementation pending interpreter refactoring
        unimplemented!()
    }
    
    /// Phase 9.78a: Create VM with plugin support
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    pub fn new_with_plugins(
        box_registry: Arc<UnifiedBoxRegistry>,
        plugin_loader: Arc<PluginLoaderV2>,
        box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
    ) -> Self {
        // Implementation pending interpreter refactoring
        unimplemented!()
    }
    */
    
    /// Execute a MIR module
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Store module for nested calls
        self.module = Some(module.clone());
        // Reset stats
        self.instr_counter.clear();
        self.exec_start = Some(Instant::now());
        // Find main function
        let main_function = module.get_function("main")
            .ok_or_else(|| VMError::InvalidInstruction("No main function found".to_string()))?;
        
        // Execute main function
        let result = self.execute_function(main_function)?;
        
        // Optional: print VM stats
        self.maybe_print_stats();

        // Convert result to NyashBox
        Ok(result.to_nyash_box())
    }

    /// Call a MIR function by name with VMValue arguments
    pub(super) fn call_function_by_name(&mut self, func_name: &str, args: Vec<VMValue>) -> Result<VMValue, VMError> {
        let module_ref = self.module.as_ref().ok_or_else(|| VMError::InvalidInstruction("No active module".to_string()))?;
        let function_ref = module_ref.get_function(func_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("Function '{}' not found", func_name)))?;
        // Clone function to avoid borrowing conflicts during execution
        let function = function_ref.clone();

        // Save current frame
        let saved_values = std::mem::take(&mut self.values);
        let saved_current_function = self.current_function.clone();
        let saved_current_block = self.current_block;
        let saved_previous_block = self.previous_block;
        let saved_pc = self.pc;
        let saved_last_result = self.last_result.clone();

        // Bind parameters
        for (i, param_id) in function.params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                self.set_value(*param_id, arg.clone());
            }
        }

        // Heuristic: map `me` (first param) to class name parsed from function name (e.g., User.method/N)
        if let Some(first) = function.params.get(0) {
            if let Some((class_part, _rest)) = func_name.split_once('.') {
                // Record class for internal field visibility checks
                self.object_class.insert(*first, class_part.to_string());
                // Mark internal reference
                self.object_internal.insert(*first);
            }
        }

        // Execute the function
        let result = self.execute_function(&function);

        // Restore frame
        self.values = saved_values;
        self.current_function = saved_current_function;
        self.current_block = saved_current_block;
        self.previous_block = saved_previous_block;
        self.pc = saved_pc;
        self.last_result = saved_last_result;

        result
    }
    
    /// Execute a single function
    fn execute_function(&mut self, function: &MirFunction) -> Result<VMValue, VMError> {
        self.current_function = Some(function.signature.name.clone());
        
        // Initialize loop executor for this function
        self.loop_executor.initialize();
        
        // Enter a new scope for this function
        self.scope_tracker.push_scope();
        
        // Start at entry block
        let mut current_block = function.entry_block;
        
        loop {
            let block = function.get_block(current_block)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("Block {} not found", current_block)))?;
            
            self.current_block = Some(current_block);
            self.pc = 0;
            
            let mut next_block = None;
            let mut should_return = None;
            
            // Execute instructions in this block (including terminator)
            let all_instructions: Vec<_> = block.all_instructions().collect();
            for (index, instruction) in all_instructions.iter().enumerate() {
                self.pc = index;
                
                match self.execute_instruction(instruction)? {
                    ControlFlow::Continue => continue,
                    ControlFlow::Jump(target) => {
                        next_block = Some(target);
                        break;
                    },
                    ControlFlow::Return(value) => {
                        should_return = Some(value);
                        break;
                    },
                }
            }
            
            // Handle control flow
            if let Some(return_value) = should_return {
                // Exit scope before returning
                self.scope_tracker.pop_scope();
                return Ok(return_value);
            } else if let Some(target) = next_block {
                // Update previous block before jumping
                self.previous_block = Some(current_block);
                // Record the transition in loop executor
                self.loop_executor.record_transition(current_block, target);
                current_block = target;
            } else {
                // Block ended without terminator - this shouldn't happen in well-formed MIR
                // but let's handle it gracefully by returning void
                // Exit scope before returning
                self.scope_tracker.pop_scope();
                return Ok(VMValue::Void);
            }
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &MirInstruction) -> Result<ControlFlow, VMError> {
        // Record instruction for stats
        self.record_instruction(instruction);
        
        match instruction {
            // Basic operations
            MirInstruction::Const { dst, value } => 
                self.execute_const(*dst, value),
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                if std::env::var("NYASH_VM_DEBUG_ANDOR").ok().as_deref() == Some("1") {
                    eprintln!("[VM] execute_instruction -> BinOp({:?})", op);
                }
                self.execute_binop(*dst, op, *lhs, *rhs)
            },
            
            MirInstruction::UnaryOp { dst, op, operand } => 
                self.execute_unaryop(*dst, op, *operand),
            
            MirInstruction::Compare { dst, op, lhs, rhs } => 
                self.execute_compare(*dst, op, *lhs, *rhs),
            
            // I/O operations
            MirInstruction::Print { value, .. } => 
                self.execute_print(*value),
            
            // Type operations
            MirInstruction::TypeOp { dst, op, value, ty } => 
                self.execute_typeop(*dst, op, *value, ty),
            
            // Control flow
            MirInstruction::Return { value } => 
                self.execute_return(*value),
            
            MirInstruction::Jump { target } => 
                self.execute_jump(*target),
            
            MirInstruction::Branch { condition, then_bb, else_bb } => 
                self.execute_branch(*condition, *then_bb, *else_bb),
            
            MirInstruction::Phi { dst, inputs } => 
                self.execute_phi(*dst, inputs),
            
            // Memory operations
            MirInstruction::Load { dst, ptr } => 
                self.execute_load(*dst, *ptr),
            
            MirInstruction::Store { value, ptr } => 
                self.execute_store(*value, *ptr),
            
            MirInstruction::Copy { dst, src } => 
                self.execute_copy(*dst, *src),
            
            // Complex operations
            MirInstruction::Call { dst, func, args, effects: _ } => 
                self.execute_call(*dst, *func, args),
            
            MirInstruction::BoxCall { dst, box_val, method, args, effects: _ } => 
                self.execute_boxcall(*dst, *box_val, method, args),
            
            MirInstruction::NewBox { dst, box_type, args } => 
                self.execute_newbox(*dst, box_type, args),
            
            // Array operations
            MirInstruction::ArrayGet { dst, array, index } => 
                self.execute_array_get(*dst, *array, *index),
            
            MirInstruction::ArraySet { array, index, value } => 
                self.execute_array_set(*array, *index, *value),
            
            // Reference operations
            MirInstruction::RefNew { dst, box_val } => 
                self.execute_ref_new(*dst, *box_val),
            
            MirInstruction::RefGet { dst, reference, field } => 
                self.execute_ref_get(*dst, *reference, field),
            
            MirInstruction::RefSet { reference, field, value } => 
                self.execute_ref_set(*reference, field, *value),
            
            // Weak references
            MirInstruction::WeakNew { dst, box_val } => 
                self.execute_weak_new(*dst, *box_val),
            
            MirInstruction::WeakLoad { dst, weak_ref } => 
                self.execute_weak_load(*dst, *weak_ref),
            
            // Unified weak reference operations
            MirInstruction::WeakRef { dst, op, value } => {
                match op {
                    crate::mir::WeakRefOp::New => {
                        self.execute_weak_new(*dst, *value)
                    }
                    crate::mir::WeakRefOp::Load => {
                        self.execute_weak_load(*dst, *value)
                    }
                }
            }
            
            // Barriers
            MirInstruction::BarrierRead { ptr: _ } => {
                // Memory barrier read is a no-op for now
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::BarrierWrite { ptr: _ } => {
                // Memory barrier write is a no-op for now
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::Barrier { .. } => {
                // Unified barrier is a no-op for now
                Ok(ControlFlow::Continue)
            }
            
            // Exception handling
            MirInstruction::Throw { exception, effects: _ } => 
                self.execute_throw(*exception),
            
            MirInstruction::Catch { exception_type: _, exception_value, handler_bb: _ } => 
                self.execute_catch(*exception_value),
            
            // Future operations
            MirInstruction::FutureNew { dst, value } => {
                let initial_value = self.get_value(*value)?;
                let future = crate::boxes::future::FutureBox::new();
                let nyash_box = initial_value.to_nyash_box();
                future.set_result(nyash_box);
                self.set_value(*dst, VMValue::Future(future));
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::FutureSet { future, value } => {
                let future_val = self.get_value(*future)?;
                let new_value = self.get_value(*value)?;
                
                if let VMValue::Future(ref future_box) = future_val {
                    future_box.set_result(new_value.to_nyash_box());
                    Ok(ControlFlow::Continue)
                } else {
                    Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
                }
            }
            
            // Special operations
            MirInstruction::Await { dst, future } => 
                self.execute_await(*dst, *future),
            
            MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => 
                self.execute_extern_call(*dst, iface_name, method_name, args),
            
            // Deprecated/No-op instructions
            MirInstruction::TypeCheck { dst, value: _, expected_type: _ } => {
                // TypeCheck is deprecated in favor of TypeOp
                self.set_value(*dst, VMValue::Bool(true));
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::Cast { dst, value, target_type: _ } => {
                // Cast is deprecated in favor of TypeOp
                let val = self.get_value(*value)?;
                self.set_value(*dst, val);
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::Debug { value: _, message: _ } => {
                // Debug is a no-op in release mode
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::Nop => {
                // No operation
                Ok(ControlFlow::Continue)
            }
            
            MirInstruction::Safepoint => {
                // Safepoint for future GC/async support
                Ok(ControlFlow::Continue)
            }
        }
    }

    
    /// Get a value from storage
    pub(super) fn get_value(&self, value_id: ValueId) -> Result<VMValue, VMError> {
        let index = value_id.to_usize();
        if index < self.values.len() {
            if let Some(ref value) = self.values[index] {
                Ok(value.clone())
            } else {
                Err(VMError::InvalidValue(format!("Value {} not set", value_id)))
            }
        } else {
            Err(VMError::InvalidValue(format!("Value {} out of bounds", value_id)))
        }
    }
    
    /// Set a value in the VM storage
    pub(super) fn set_value(&mut self, value_id: ValueId, value: VMValue) {
        let index = value_id.to_usize();
        // Resize Vec if necessary
        if index >= self.values.len() {
            self.values.resize(index + 1, None);
        }
        self.values[index] = Some(value);
    }
    

    /// Record an instruction execution for statistics
    pub(super) fn record_instruction(&mut self, instruction: &MirInstruction) {
        let key: &'static str = match instruction {
            MirInstruction::Const { .. } => "Const",
            MirInstruction::BinOp { .. } => "BinOp",
            MirInstruction::UnaryOp { .. } => "UnaryOp",
            MirInstruction::Compare { .. } => "Compare",
            MirInstruction::Load { .. } => "Load",
            MirInstruction::Store { .. } => "Store",
            MirInstruction::Call { .. } => "Call",
            MirInstruction::BoxCall { .. } => "BoxCall",
            MirInstruction::Branch { .. } => "Branch",
            MirInstruction::Jump { .. } => "Jump",
            MirInstruction::Return { .. } => "Return",
            MirInstruction::Phi { .. } => "Phi",
            MirInstruction::NewBox { .. } => "NewBox",
            MirInstruction::TypeCheck { .. } => "TypeCheck",
            MirInstruction::Cast { .. } => "Cast",
            MirInstruction::TypeOp { .. } => "TypeOp",
            MirInstruction::ArrayGet { .. } => "ArrayGet",
            MirInstruction::ArraySet { .. } => "ArraySet",
            MirInstruction::Copy { .. } => "Copy",
            MirInstruction::Debug { .. } => "Debug",
            MirInstruction::Print { .. } => "Print",
            MirInstruction::Nop => "Nop",
            MirInstruction::Throw { .. } => "Throw",
            MirInstruction::Catch { .. } => "Catch",
            MirInstruction::Safepoint => "Safepoint",
            MirInstruction::RefNew { .. } => "RefNew",
            MirInstruction::RefGet { .. } => "RefGet",
            MirInstruction::RefSet { .. } => "RefSet",
            MirInstruction::WeakNew { .. } => "WeakNew",
            MirInstruction::WeakLoad { .. } => "WeakLoad",
            MirInstruction::BarrierRead { .. } => "BarrierRead",
            MirInstruction::BarrierWrite { .. } => "BarrierWrite",
            MirInstruction::WeakRef { .. } => "WeakRef",
            MirInstruction::Barrier { .. } => "Barrier",
            MirInstruction::FutureNew { .. } => "FutureNew",
            MirInstruction::FutureSet { .. } => "FutureSet",
            MirInstruction::Await { .. } => "Await",
            MirInstruction::ExternCall { .. } => "ExternCall",
        };
        *self.instr_counter.entry(key).or_insert(0) += 1;
    }

    
    /// Phase 9.78a: Unified method dispatch for all Box types
    fn call_unified_method(&self, box_value: Box<dyn NyashBox>, method: &str, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, we use the simplified method dispatch
        // In a full implementation, this would check for InstanceBox and dispatch appropriately
        self.call_box_method_impl(box_value, method, args)
    }
    
    /// Call a method on a Box - simplified version of interpreter method dispatch
    pub(super) fn call_box_method(&self, box_value: Box<dyn NyashBox>, method: &str, _args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, implement basic methods for common box types
        // This is a simplified version - real implementation would need full method dispatch

        // ResultBox (NyashResultBox - new)
        if let Some(result_box) = box_value.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
            match method {
                // Rust側の公開APIメソッド名に合わせたバリアントを許容
                "is_ok" | "isOk" => {
                    return Ok(result_box.is_ok());
                }
                "get_value" | "getValue" => {
                    return Ok(result_box.get_value());
                }
                "get_error" | "getError" => {
                    return Ok(result_box.get_error());
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // ResultBox (box_trait::ResultBox - legacy)
        {
            #[allow(deprecated)]
            if let Some(result_box_legacy) = box_value.as_any().downcast_ref::<crate::box_trait::ResultBox>() {
                match method {
                    "is_ok" | "isOk" => {
                        return Ok(result_box_legacy.is_ok());
                    }
                    "get_value" | "getValue" => {
                        return Ok(result_box_legacy.get_value());
                    }
                    "get_error" | "getError" => {
                        return Ok(result_box_legacy.get_error());
                    }
                    _ => return Ok(Box::new(VoidBox::new())),
                }
            }
        }

        // Generic fallback: toString for any Box type
        if method == "toString" {
            return Ok(Box::new(StringBox::new(box_value.to_string_box().value)));
        }

        // StringBox methods
        if let Some(string_box) = box_value.as_any().downcast_ref::<StringBox>() {
            match method {
                "length" | "len" => {
                    return Ok(Box::new(IntegerBox::new(string_box.value.len() as i64)));
                },
                "toString" => {
                    return Ok(Box::new(StringBox::new(string_box.value.clone())));
                },
                "substring" => {
                    // substring(start, end) - simplified implementation
                    if _args.len() >= 2 {
                        if let (Some(start_box), Some(end_box)) = (_args.get(0), _args.get(1)) {
                            if let (Some(start_int), Some(end_int)) = (
                                start_box.as_any().downcast_ref::<IntegerBox>(),
                                end_box.as_any().downcast_ref::<IntegerBox>()
                            ) {
                                let start = start_int.value.max(0) as usize;
                                let end = end_int.value.max(0) as usize;
                                let len = string_box.value.len();
                                
                                if start <= len {
                                    let end_idx = end.min(len);
                                    if start <= end_idx {
                                        let substr = &string_box.value[start..end_idx];
                                        return Ok(Box::new(StringBox::new(substr)));
                                    }
                                }
                            }
                        }
                    }
                    return Ok(Box::new(StringBox::new(""))); // Return empty string on error
                },
                "concat" => {
                    // concat(other) - concatenate with another string
                    if let Some(other_box) = _args.get(0) {
                        let other_str = other_box.to_string_box().value;
                        let result = string_box.value.clone() + &other_str;
                        return Ok(Box::new(StringBox::new(result)));
                    }
                    return Ok(Box::new(StringBox::new(string_box.value.clone())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // ArrayBox methods (minimal set)
        if let Some(array_box) = box_value.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match method {
                "push" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.push(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: push(value) requires 1 arg")));
                },
                "pop" => { return Ok(array_box.pop()); },
                "length" | "len" => { return Ok(array_box.length()); },
                "get" => {
                    if let Some(i) = _args.get(0) { return Ok(array_box.get(i.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: get(index) requires 1 arg")));
                },
                "set" => {
                    if _args.len() >= 2 { return Ok(array_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: set(index, value) requires 2 args")));
                },
                "remove" => {
                    if let Some(i) = _args.get(0) { return Ok(array_box.remove(i.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: remove(index) requires 1 arg")));
                },
                "contains" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.contains(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: contains(value) requires 1 arg")));
                },
                "indexOf" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.indexOf(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: indexOf(value) requires 1 arg")));
                },
                "clear" => { return Ok(array_box.clear()); },
                "join" => {
                    if let Some(sep) = _args.get(0) { return Ok(array_box.join(sep.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: join(sep) requires 1 arg")));
                },
                "sort" => { return Ok(array_box.sort()); },
                "reverse" => { return Ok(array_box.reverse()); },
                "slice" => {
                    if _args.len() >= 2 { return Ok(array_box.slice(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: slice(start, end) requires 2 args")));
                },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // MapBox methods (minimal set)
        if let Some(map_box) = box_value.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            match method {
                "set" => {
                    if _args.len() >= 2 { return Ok(map_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: set(key, value) requires 2 args")));
                },
                "get" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.get(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: get(key) requires 1 arg")));
                },
                "has" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.has(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: has(key) requires 1 arg")));
                },
                "delete" | "remove" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.delete(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: delete(key) requires 1 arg")));
                },
                "keys" => { return Ok(map_box.keys()); },
                "values" => { return Ok(map_box.values()); },
                "size" => { return Ok(map_box.size()); },
                "clear" => { return Ok(map_box.clear()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // SocketBox methods (minimal set + timeout variants)
        if let Some(sock) = box_value.as_any().downcast_ref::<crate::boxes::socket_box::SocketBox>() {
            match method {
                "bind" => {
                    if _args.len() >= 2 { return Ok(sock.bind(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: bind(address, port) requires 2 args")));
                },
                "listen" => {
                    if let Some(b) = _args.get(0) { return Ok(sock.listen(b.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: listen(backlog) requires 1 arg")));
                },
                "accept" => { return Ok(sock.accept()); },
                "acceptTimeout" | "accept_timeout" => {
                    if let Some(ms) = _args.get(0) { return Ok(sock.accept_timeout(ms.clone_or_share())); }
                    return Ok(Box::new(crate::box_trait::VoidBox::new()));
                },
                "connect" => {
                    if _args.len() >= 2 { return Ok(sock.connect(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: connect(address, port) requires 2 args")));
                },
                "read" => { return Ok(sock.read()); },
                "recvTimeout" | "recv_timeout" => {
                    if let Some(ms) = _args.get(0) { return Ok(sock.recv_timeout(ms.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("")));
                },
                "write" => {
                    if let Some(d) = _args.get(0) { return Ok(sock.write(d.clone_or_share())); }
                    return Ok(Box::new(crate::box_trait::BoolBox::new(false)));
                },
                "close" => { return Ok(sock.close()); },
                "isServer" | "is_server" => { return Ok(sock.is_server()); },
                "isConnected" | "is_connected" => { return Ok(sock.is_connected()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }
        
        // IntegerBox methods  
        if let Some(integer_box) = box_value.as_any().downcast_ref::<IntegerBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(integer_box.value.to_string())));
                },
                "abs" => {
                    return Ok(Box::new(IntegerBox::new(integer_box.value.abs())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // BoolBox methods
        if let Some(bool_box) = box_value.as_any().downcast_ref::<BoolBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(bool_box.value.to_string())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // ArrayBox methods - needed for kilo editor
        if let Some(array_box) = box_value.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match method {
                "length" | "len" => {
                    let items = array_box.items.read().unwrap();
                    return Ok(Box::new(IntegerBox::new(items.len() as i64)));
                },
                "get" => {
                    // get(index) - get element at index
                    if let Some(index_box) = _args.get(0) {
                        if let Some(index_int) = index_box.as_any().downcast_ref::<IntegerBox>() {
                            let items = array_box.items.read().unwrap();
                            let index = index_int.value as usize;
                            if index < items.len() {
                                return Ok(items[index].clone_box());
                            }
                        }
                    }
                    return Ok(Box::new(VoidBox::new())); // Return void for out of bounds
                },
                "set" => {
                    // set(index, value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    // In a real implementation, we'd need mutable access
                    return Ok(Box::new(VoidBox::new()));
                },
                "push" => {
                    // push(value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    return Ok(Box::new(VoidBox::new()));
                },
                "insert" => {
                    // insert(index, value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    return Ok(Box::new(VoidBox::new()));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // PluginBoxV2 support
        if let Some(plugin_box) = box_value.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            // For toString on plugins, return a descriptive string
            if method == "toString" {
                return Ok(Box::new(StringBox::new(format!("{}(id={})", plugin_box.box_type, plugin_box.inner.instance_id))));
            }
            
            // Other plugin methods should be called via BoxCall instruction
            // This path shouldn't normally be reached for plugin methods
            eprintln!("Warning: Plugin method '{}' called via call_box_method - should use BoxCall", method);
            return Ok(Box::new(VoidBox::new()));
        }
        
        // Default: return void for any unrecognized box type or method
        Ok(Box::new(VoidBox::new()))
    }
}

/// Control flow result from instruction execution
pub(super) enum ControlFlow {
    Continue,
    Jump(BasicBlockId),
    Return(VMValue),
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock};
    use crate::parser::NyashParser;
    use crate::runtime::NyashRuntime;
    use crate::core::model::BoxDeclaration as CoreBoxDecl;
    use crate::interpreter::SharedState;
    use crate::box_factory::user_defined::UserDefinedBoxFactory;
    use std::sync::Arc;
    use std::collections::HashMap;
    
    #[test]
    fn test_basic_vm_execution() {
        let mut vm = VM::new();
        
        // Test constant loading
        let const_instr = MirInstruction::Const {
            dst: ValueId(1),
            value: ConstValue::Integer(42),
        };
        
        let result = vm.execute_instruction(&const_instr);
        assert!(result.is_ok());
        
        let value = vm.get_value(ValueId(1)).unwrap();
        assert_eq!(value.as_integer().unwrap(), 42);
    }
    
    #[test]
    fn test_binary_operations() {
        let mut vm = VM::new();
        
        // Load constants
        vm.set_value(ValueId(1), VMValue::Integer(10));
        vm.set_value(ValueId(2), VMValue::Integer(32));
        
        // Test addition
        let add_instr = MirInstruction::BinOp {
            dst: ValueId(3),
            op: BinaryOp::Add,
            lhs: ValueId(1),
            rhs: ValueId(2),
        };
        
        let result = vm.execute_instruction(&add_instr);
        assert!(result.is_ok());
        
        let value = vm.get_value(ValueId(3)).unwrap();
        assert_eq!(value.as_integer().unwrap(), 42);
    }

    fn collect_box_declarations(ast: &crate::ast::ASTNode, runtime: &NyashRuntime) {
        fn walk(node: &crate::ast::ASTNode, runtime: &NyashRuntime) {
            match node {
                crate::ast::ASTNode::Program { statements, .. } => {
                    for st in statements { walk(st, runtime); }
                }
                crate::ast::ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    let decl = CoreBoxDecl {
                        name: name.clone(),
                        fields: fields.clone(),
                        public_fields: public_fields.clone(),
                        private_fields: private_fields.clone(),
                        methods: methods.clone(),
                        constructors: constructors.clone(),
                        init_fields: init_fields.clone(),
                        weak_fields: weak_fields.clone(),
                        is_interface: *is_interface,
                        extends: extends.clone(),
                        implements: implements.clone(),
                        type_parameters: type_parameters.clone(),
                    };
                    if let Ok(mut map) = runtime.box_declarations.write() {
                        map.insert(name.clone(), decl);
                    }
                }
                _ => {}
            }
        }
        walk(ast, runtime);
    }

    #[test]
    fn test_vm_user_box_birth_and_method() {
        let code = r#"
box Person {
  init { name }
  birth(n) {
    me.name = n
  }
  greet() {
    return "Hello, " + me.name
  }
}

return new Person("Alice").greet()
"#;

        // Parse to AST
        let ast = NyashParser::parse_from_string(code).expect("parse failed");

        // Prepare runtime with user-defined declarations and factory
        let runtime = {
            let rt = NyashRuntime::new();
            collect_box_declarations(&ast, &rt);
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };

        // Compile to MIR
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");

        // Debug: Print MIR
        println!("=== MIR Output ===");
        let mut printer = crate::mir::MirPrinter::verbose();
        println!("{}", printer.print_module(&compile_result.module));
        println!("==================");

        // Execute with VM
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "Hello, Alice");
    }

    #[test]
    fn test_vm_user_box_var_then_method() {
        let code = r#"
box Counter {
  init { x }
  birth(n) { me.x = n }
  inc() { me.x = me.x + 1 }
  get() { return me.x }
}

local c
c = new Counter(10)
c.inc()
c.get()
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse failed");
        let runtime = {
            let rt = NyashRuntime::new();
            collect_box_declarations(&ast, &rt);
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "11");
    }

    #[test]
    fn test_vm_extern_console_log() {
        let code = r#"
console.log("ok")
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse failed");
        let runtime = NyashRuntime::new();
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "void");
    }
}
