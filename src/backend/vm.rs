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
    object_fields: HashMap<ValueId, HashMap<String, VMValue>>,
    /// Class name mapping for objects (for visibility checks)
    object_class: HashMap<ValueId, String>,
    /// Marks ValueIds that represent internal (me/this) references within the current function
    object_internal: std::collections::HashSet<ValueId>,
    /// Loop executor for handling phi nodes and loop-specific logic
    loop_executor: LoopExecutor,
    /// Shared runtime for box creation and declarations
    runtime: NyashRuntime,
    /// Scope tracker for calling fini on scope exit
    scope_tracker: ScopeTracker,
    /// Active MIR module during execution (for function calls)
    module: Option<MirModule>,
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
        // Find main function
        let main_function = module.get_function("main")
            .ok_or_else(|| VMError::InvalidInstruction("No main function found".to_string()))?;
        
        // Execute main function
        let result = self.execute_function(main_function)?;
        
        // Convert result to NyashBox
        Ok(result.to_nyash_box())
    }

    /// Call a MIR function by name with VMValue arguments
    fn call_function_by_name(&mut self, func_name: &str, args: Vec<VMValue>) -> Result<VMValue, VMError> {
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
        match instruction {
            MirInstruction::Const { dst, value } => {
                let vm_value = VMValue::from(value);
                self.set_value(*dst, vm_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                let left = self.get_value(*lhs)?;
                let right = self.get_value(*rhs)?;
                let result = self.execute_binary_op(op, &left, &right)?;
                self.set_value(*dst, result);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::UnaryOp { dst, op, operand } => {
                let operand_val = self.get_value(*operand)?;
                let result = self.execute_unary_op(op, &operand_val)?;
                self.set_value(*dst, result);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                let left = self.get_value(*lhs)?;
                let right = self.get_value(*rhs)?;
                let result = self.execute_compare_op(op, &left, &right)?;
                self.set_value(*dst, VMValue::Bool(result));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Print { value, .. } => {
                let val = self.get_value(*value)?;
                println!("{}", val.to_string());
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Return { value } => {
                let return_value = if let Some(val_id) = value {
                    let val = self.get_value(*val_id)?;
                    val
                } else {
                    VMValue::Void
                };
                Ok(ControlFlow::Return(return_value))
            },
            
            MirInstruction::Jump { target } => {
                Ok(ControlFlow::Jump(*target))
            },
            
            MirInstruction::Branch { condition, then_bb, else_bb } => {
                let cond_val = self.get_value(*condition)?;
                let cond_bool = cond_val.as_bool()?;
                
                if cond_bool {
                    Ok(ControlFlow::Jump(*then_bb))
                } else {
                    Ok(ControlFlow::Jump(*else_bb))
                }
            },
            
            MirInstruction::Phi { dst, inputs } => {
                // Create a closure that captures self immutably
                let values = &self.values;
                let get_value_fn = |value_id: ValueId| -> Result<VMValue, VMError> {
                    let index = value_id.to_usize();
                    if index < values.len() {
                        if let Some(ref value) = values[index] {
                            Ok(value.clone())
                        } else {
                            Err(VMError::InvalidValue(format!("Value {} not set", value_id)))
                        }
                    } else {
                        Err(VMError::InvalidValue(format!("Value {} out of bounds", value_id)))
                    }
                };
                
                // Delegate phi node execution to loop executor
                let selected_value = self.loop_executor.execute_phi(
                    *dst,
                    inputs,
                    get_value_fn
                )?;
                
                self.set_value(*dst, selected_value);
                Ok(ControlFlow::Continue)
            },
            
            // Missing instructions that need basic implementations
            MirInstruction::Load { dst, ptr } => {
                // For now, loading is the same as getting the value
                let value = self.get_value(*ptr)?;
                self.set_value(*dst, value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Store { value, ptr } => {
                // For now, storing just updates the ptr with the value
                let val = self.get_value(*value)?;
                self.set_value(*ptr, val);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Call { dst, func, args, effects: _ } => {
                // Resolve function name from func value (expects Const String)
                let func_val = self.get_value(*func)?;
                let func_name = match func_val {
                    VMValue::String(s) => s,
                    _ => return Err(VMError::InvalidInstruction("Call expects func to be a String name".to_string())),
                };
                // Gather argument VM values
                let mut vm_args = Vec::new();
                for arg_id in args {
                    vm_args.push(self.get_value(*arg_id)?);
                }
                let result = self.call_function_by_name(&func_name, vm_args)?;
                if let Some(dst_id) = dst {
                    self.set_value(*dst_id, result);
                }
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BoxCall { dst, box_val, method, args, effects: _ } => {
                // Phase 9.78a: Unified method dispatch for all Box types
                
                // Get the box value
                let box_vm_value = self.get_value(*box_val)?;
                
                // Handle BoxRef for proper method dispatch
                let box_nyash = match &box_vm_value {
                    // Use shared handle to avoid unintended constructor calls
                    VMValue::BoxRef(arc_box) => arc_box.share_box(),
                    _ => box_vm_value.to_nyash_box(),
                };
                
                // Fast path: birth() for user-defined boxes is lowered to a MIR function
                if method == "birth" {
                    if let Some(instance) = box_nyash.as_any().downcast_ref::<InstanceBox>() {
                        let class_name = instance.class_name.clone();
                        let func_name = format!("{}.birth/{}", class_name, args.len());

                        // Prepare VMValue args: me + evaluated arguments
                        let mut vm_args: Vec<VMValue> = Vec::new();
                        vm_args.push(VMValue::from_nyash_box(box_nyash.clone_box()));
                        for arg_id in args {
                            let arg_vm_value = self.get_value(*arg_id)?;
                            vm_args.push(arg_vm_value);
                        }

                        // Call the lowered function (ignore return)
                        let _ = self.call_function_by_name(&func_name, vm_args)?;

                        // birth returns void; only set dst if specified (rare for birth)
                        if let Some(dst_id) = dst {
                            self.set_value(*dst_id, VMValue::Void);
                        }
                        return Ok(ControlFlow::Continue);
                    }
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg_id in args {
                    let arg_vm_value = self.get_value(*arg_id)?;
                    arg_values.push(arg_vm_value.to_nyash_box());
                }
                
                // PluginBoxV2 method dispatch via BID-FFI (zero-arg minimal)
                #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
                if let Some(plugin) = box_nyash.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    let loader = crate::runtime::get_global_loader_v2();
                    let loader = loader.read().map_err(|_| VMError::InvalidInstruction("Plugin loader lock poisoned".into()))?;
                    match loader.invoke_instance_method(&plugin.box_type, method, plugin.instance_id, &arg_values) {
                        Ok(Some(result_box)) => {
                            if let Some(dst_id) = dst {
                                self.set_value(*dst_id, VMValue::from_nyash_box(result_box));
                            }
                        }
                        Ok(None) => {
                            if let Some(dst_id) = dst {
                                self.set_value(*dst_id, VMValue::Void);
                            }
                        }
                        Err(_) => {
                            return Err(VMError::InvalidInstruction(format!("Plugin method call failed: {}", method)));
                        }
                    }
                    return Ok(ControlFlow::Continue);
                }

                // Call the method - unified dispatch for all Box types
                // If user-defined InstanceBox: dispatch to lowered MIR function `{Class}.{method}/{argc}`
                if let Some(instance) = box_nyash.as_any().downcast_ref::<InstanceBox>() {
                    let class_name = instance.class_name.clone();
                    let func_name = format!("{}.{}{}", class_name, method, format!("/{}", args.len()));
                    // Prepare VMValue args: me + evaluated arguments (use original VM args for value-level fidelity)
                    let mut vm_args: Vec<VMValue> = Vec::new();
                    vm_args.push(VMValue::from_nyash_box(box_nyash.clone_box()));
                    for arg_id in args {
                        let arg_vm_value = self.get_value(*arg_id)?;
                        vm_args.push(arg_vm_value);
                    }
                    let call_result = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst {
                        self.set_value(*dst_id, call_result);
                    }
                    return Ok(ControlFlow::Continue);
                }

                let result = self.call_unified_method(box_nyash, method, arg_values)?;
                
                // Store result if destination is specified
                if let Some(dst_id) = dst {
                    let vm_result = VMValue::from_nyash_box(result);
                    self.set_value(*dst_id, vm_result);
                }
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::NewBox { dst, box_type, args } => {
                // Evaluate arguments into NyashBox for unified factory
                let mut nyash_args: Vec<Box<dyn NyashBox>> = Vec::new();
                for arg_id in args {
                    let arg_value = self.get_value(*arg_id)?;
                    nyash_args.push(arg_value.to_nyash_box());
                }
                // Create via unified registry from runtime
                let registry = self.runtime.box_registry.clone();
                let created = {
                    let guard = registry.lock().map_err(|_| VMError::InvalidInstruction("Registry lock poisoned".into()))?;
                    guard.create_box(box_type, &nyash_args)
                };
                match created {
                    Ok(b) => {
                        // Register for scope-based finalization (share; keep same instance)
                        let reg_arc = std::sync::Arc::from(b.share_box());
                        self.scope_tracker.register_box(reg_arc);
                        // Record class name for visibility checks
                        self.object_class.insert(*dst, box_type.clone());
                        // Store value in VM
                        self.set_value(*dst, VMValue::from_nyash_box(b));
                        Ok(ControlFlow::Continue)
                    }
                    Err(e) => Err(VMError::InvalidInstruction(format!("NewBox failed for {}: {}", box_type, e)))
                }
            },
            
            MirInstruction::TypeCheck { dst, value: _, expected_type: _ } => {
                // For now, type checks always return true
                // TODO: Implement proper type checking
                self.set_value(*dst, VMValue::Bool(true));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Cast { dst, value, target_type: _ } => {
                // For now, casting just copies the value
                // TODO: Implement proper type casting
                let val = self.get_value(*value)?;
                self.set_value(*dst, val);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::ArrayGet { dst, array: _, index: _ } => {
                // For now, array access returns a placeholder
                // TODO: Implement proper array access
                self.set_value(*dst, VMValue::Integer(0));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::ArraySet { array: _, index: _, value: _ } => {
                // For now, array setting is a no-op
                // TODO: Implement proper array setting
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Copy { dst, src } => {
                // Copy instruction - duplicate the source value
                let val = self.get_value(*src)?;
                self.set_value(*dst, val);
                // Propagate class mapping for references (helps track `me` copies)
                if let Some(class_name) = self.object_class.get(src).cloned() {
                    self.object_class.insert(*dst, class_name);
                }
                // Propagate internal marker (me/this lineage)
                if self.object_internal.contains(src) {
                    self.object_internal.insert(*dst);
                }
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Debug { value: _, message: _ } => {
                // Debug instruction - skip debug output for performance
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Nop => {
                // No-op instruction
                Ok(ControlFlow::Continue)
            },
            
            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { exception, effects: _ } => {
                let exception_val = self.get_value(*exception)?;
                // For now, convert throw to error return (simplified exception handling)
                // In a full implementation, this would unwind the stack looking for catch handlers
                println!("Exception thrown: {}", exception_val.to_string());
                Err(VMError::InvalidInstruction(format!("Unhandled exception: {}", exception_val.to_string())))
            },
            
            MirInstruction::Catch { exception_type: _, exception_value, handler_bb: _ } => {
                // For now, catch is a no-op since we don't have full exception handling
                // In a real implementation, this would set up exception handling metadata
                self.set_value(*exception_value, VMValue::Void);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Safepoint => {
                // Safepoint is a no-op for now
                // In a real implementation, this could trigger GC, debugging, etc.
                Ok(ControlFlow::Continue)
            },
            
            // Phase 6: Box reference operations
            MirInstruction::RefNew { dst, box_val } => {
                // For now, a reference is just the same as the box value
                // In a real implementation, this would create a proper reference
                let box_value = self.get_value(*box_val)?;
                self.set_value(*dst, box_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::RefGet { dst, reference, field } => {
                // Visibility check (if class known and visibility declared). Skip for internal refs.
                let is_internal = self.object_internal.contains(reference);
                if !is_internal {
                    if let Some(class_name) = self.object_class.get(reference) {
                        if let Ok(decls) = self.runtime.box_declarations.read() {
                            if let Some(decl) = decls.get(class_name) {
                                let has_vis = !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                                if has_vis && !decl.public_fields.contains(field) {
                                    return Err(VMError::TypeError(format!("Field '{}' is private in {}", field, class_name)));
                                }
                            }
                        }
                    }
                }
                // Get field value from object
                let field_value = if let Some(fields) = self.object_fields.get(reference) {
                    if let Some(value) = fields.get(field) {
                        value.clone()
                    } else {
                        // Field not set yet, return default
                        VMValue::Integer(0)
                    }
                } else {
                    // Object has no fields yet, return default
                    VMValue::Integer(0)
                };
                
                self.set_value(*dst, field_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::RefSet { reference, field, value } => {
                // Get the value to set
                let new_value = self.get_value(*value)?;
                // Visibility check (Skip for internal refs; otherwise enforce public)
                let is_internal = self.object_internal.contains(reference);
                if !is_internal {
                    if let Some(class_name) = self.object_class.get(reference) {
                        if let Ok(decls) = self.runtime.box_declarations.read() {
                            if let Some(decl) = decls.get(class_name) {
                                let has_vis = !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                                if has_vis && !decl.public_fields.contains(field) {
                                    return Err(VMError::TypeError(format!("Field '{}' is private in {}", field, class_name)));
                                }
                            }
                        }
                    }
                }
                
                // Ensure object has field storage
                if !self.object_fields.contains_key(reference) {
                    self.object_fields.insert(*reference, HashMap::new());
                }
                
                // Set the field
                if let Some(fields) = self.object_fields.get_mut(reference) {
                    fields.insert(field.clone(), new_value);
                }
                
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::WeakNew { dst, box_val } => {
                // For now, a weak reference is just a copy of the value
                // In a real implementation, this would create a proper weak reference
                let box_value = self.get_value(*box_val)?;
                self.set_value(*dst, box_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::WeakLoad { dst, weak_ref } => {
                // For now, loading from weak ref is the same as getting the value
                // In a real implementation, this would check if the weak ref is still valid
                let weak_value = self.get_value(*weak_ref)?;
                self.set_value(*dst, weak_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BarrierRead { ptr: _ } => {
                // Memory barrier read is a no-op for now
                // In a real implementation, this would ensure memory ordering
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BarrierWrite { ptr: _ } => {
                // Memory barrier write is a no-op for now
                // In a real implementation, this would ensure memory ordering
                Ok(ControlFlow::Continue)
            },
            
            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { dst, value } => {
                let initial_value = self.get_value(*value)?;
                let future = crate::boxes::future::FutureBox::new();
                // Convert VMValue to NyashBox and set it in the future
                let nyash_box = initial_value.to_nyash_box();
                future.set_result(nyash_box);
                self.set_value(*dst, VMValue::Future(future));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::FutureSet { future, value } => {
                let future_val = self.get_value(*future)?;
                let new_value = self.get_value(*value)?;
                
                if let VMValue::Future(ref future_box) = future_val {
                    future_box.set_result(new_value.to_nyash_box());
                    Ok(ControlFlow::Continue)
                } else {
                    Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
                }
            },
            
            MirInstruction::Await { dst, future } => {
                let future_val = self.get_value(*future)?;
                
                if let VMValue::Future(ref future_box) = future_val {
                    // This blocks until the future is ready
                    let result = future_box.get();
                    // Convert NyashBox back to VMValue
                    let vm_value = VMValue::from_nyash_box(result);
                    self.set_value(*dst, vm_value);
                    Ok(ControlFlow::Continue)
                } else {
                    Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
                }
            },
            
            // Phase 9.7: External Function Calls  
            MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                // Evaluate arguments as NyashBox for loader
                let mut nyash_args: Vec<Box<dyn NyashBox>> = Vec::new();
                for arg_id in args {
                    let arg_value = self.get_value(*arg_id)?;
                    nyash_args.push(arg_value.to_nyash_box());
                }
                // Route through plugin loader v2 (also handles env.* stubs)
                let loader = crate::runtime::get_global_loader_v2();
                let loader = loader.read().map_err(|_| VMError::InvalidInstruction("Plugin loader lock poisoned".into()))?;
                match loader.extern_call(iface_name, method_name, &nyash_args) {
                    Ok(Some(result_box)) => {
                        if let Some(dst_id) = dst {
                            self.set_value(*dst_id, VMValue::from_nyash_box(result_box));
                        }
                    }
                    Ok(None) => {
                        if let Some(dst_id) = dst {
                            self.set_value(*dst_id, VMValue::Void);
                        }
                    }
                    Err(_) => {
                        return Err(VMError::InvalidInstruction(format!("ExternCall failed: {}.{}", iface_name, method_name)));
                    }
                }
                Ok(ControlFlow::Continue)
            },
        }
    }
    
    /// Get a value from storage
    fn get_value(&self, value_id: ValueId) -> Result<VMValue, VMError> {
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
    fn set_value(&mut self, value_id: ValueId, value: VMValue) {
        let index = value_id.to_usize();
        // Resize Vec if necessary
        if index >= self.values.len() {
            self.values.resize(index + 1, None);
        }
        self.values[index] = Some(value);
    }
    
    /// Execute binary operation
    fn execute_binary_op(&self, op: &BinaryOp, left: &VMValue, right: &VMValue) -> Result<VMValue, VMError> {
        match (left, right) {
            (VMValue::Integer(l), VMValue::Integer(r)) => {
                let result = match op {
                    BinaryOp::Add => *l + *r,
                    BinaryOp::Sub => *l - *r,
                    BinaryOp::Mul => *l * *r,
                    BinaryOp::Div => {
                        if *r == 0 {
                            return Err(VMError::DivisionByZero);
                        }
                        *l / *r
                    },
                    _ => return Err(VMError::InvalidInstruction(format!("Unsupported integer operation: {:?}", op))),
                };
                Ok(VMValue::Integer(result))
            },
            
            (VMValue::String(l), VMValue::Integer(r)) => {
                // String + Integer concatenation
                match op {
                    BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r))),
                    _ => Err(VMError::TypeError("String-integer operations only support addition".to_string())),
                }
            },
            
            (VMValue::String(l), VMValue::String(r)) => {
                // String concatenation
                match op {
                    BinaryOp::Add => Ok(VMValue::String(format!("{}{}", l, r))),
                    _ => Err(VMError::TypeError("String operations only support addition".to_string())),
                }
            },
            
            _ => Err(VMError::TypeError(format!("Unsupported binary operation: {:?} on {:?} and {:?}", op, left, right))),
        }
    }
    
    /// Execute unary operation
    fn execute_unary_op(&self, op: &UnaryOp, operand: &VMValue) -> Result<VMValue, VMError> {
        match (op, operand) {
            (UnaryOp::Neg, VMValue::Integer(i)) => Ok(VMValue::Integer(-i)),
            (UnaryOp::Not, VMValue::Bool(b)) => Ok(VMValue::Bool(!b)),
            _ => Err(VMError::TypeError(format!("Unsupported unary operation: {:?} on {:?}", op, operand))),
        }
    }
    
    /// Execute comparison operation
    fn execute_compare_op(&self, op: &CompareOp, left: &VMValue, right: &VMValue) -> Result<bool, VMError> {
        match (left, right) {
            (VMValue::Integer(l), VMValue::Integer(r)) => {
                let result = match op {
                    CompareOp::Eq => l == r,
                    CompareOp::Ne => l != r,
                    CompareOp::Lt => l < r,
                    CompareOp::Le => l <= r,
                    CompareOp::Gt => l > r,
                    CompareOp::Ge => l >= r,
                };
                Ok(result)
            },
            
            (VMValue::String(l), VMValue::String(r)) => {
                let result = match op {
                    CompareOp::Eq => l == r,
                    CompareOp::Ne => l != r,
                    CompareOp::Lt => l < r,
                    CompareOp::Le => l <= r,
                    CompareOp::Gt => l > r,
                    CompareOp::Ge => l >= r,
                };
                Ok(result)
            },
            
            _ => Err(VMError::TypeError(format!("Unsupported comparison: {:?} on {:?} and {:?}", op, left, right))),
        }
    }
    
    /// Phase 9.78a: Unified method dispatch for all Box types
    fn call_unified_method(&self, box_value: Box<dyn NyashBox>, method: &str, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, we use the simplified method dispatch
        // In a full implementation, this would check for InstanceBox and dispatch appropriately
        self.call_box_method(box_value, method, args)
    }
    
    /// Call a method on a Box - simplified version of interpreter method dispatch
    fn call_box_method(&self, box_value: Box<dyn NyashBox>, method: &str, _args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, implement basic methods for common box types
        // This is a simplified version - real implementation would need full method dispatch
        
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
        
        // Default: return void for any unrecognized box type or method
        Ok(Box::new(VoidBox::new()))
    }
}

/// Control flow result from instruction execution
enum ControlFlow {
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
