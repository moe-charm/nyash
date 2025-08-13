/*!
 * VM Backend - Execute MIR instructions in a virtual machine
 * 
 * Simple stack-based VM for executing MIR code
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, UnaryOp, ValueId, BasicBlockId};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox};
use std::collections::HashMap;

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
        // Try to downcast to known types
        if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
            VMValue::Integer(int_box.value)
        } else if let Some(bool_box) = nyash_box.as_any().downcast_ref::<BoolBox>() {
            VMValue::Bool(bool_box.value)
        } else if let Some(string_box) = nyash_box.as_any().downcast_ref::<StringBox>() {
            VMValue::String(string_box.value.clone())
        } else if let Some(future_box) = nyash_box.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
            VMValue::Future(future_box.clone())
        } else {
            // For any other type, convert to string representation
            VMValue::String(nyash_box.to_string_box().value)
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
    /// Value storage (maps ValueId to actual values)
    values: HashMap<ValueId, VMValue>,
    /// Current function being executed
    current_function: Option<String>,
    /// Current basic block
    current_block: Option<BasicBlockId>,
    /// Program counter within current block
    pc: usize,
    /// Return value from last execution
    last_result: Option<VMValue>,
    /// Simple field storage for objects (maps reference -> field -> value)
    object_fields: HashMap<ValueId, HashMap<String, VMValue>>,
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            current_function: None,
            current_block: None,
            pc: 0,
            last_result: None,
            object_fields: HashMap::new(),
        }
    }
    
    /// Execute a MIR module
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Find main function
        let main_function = module.get_function("main")
            .ok_or_else(|| VMError::InvalidInstruction("No main function found".to_string()))?;
        
        // Execute main function
        let result = self.execute_function(main_function)?;
        
        // Convert result to NyashBox
        Ok(result.to_nyash_box())
    }
    
    /// Execute a single function
    fn execute_function(&mut self, function: &MirFunction) -> Result<VMValue, VMError> {
        self.current_function = Some(function.signature.name.clone());
        
        // Start at entry block
        let mut current_block = function.entry_block;
        
        loop {
            let block = function.get_block(current_block)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("Block {} not found", current_block)))?;
            
            self.current_block = Some(current_block);
            self.pc = 0;
            
            let mut next_block = None;
            let mut should_return = None;
            
            // Execute instructions in this block
            for (index, instruction) in block.instructions.iter().enumerate() {
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
                return Ok(return_value);
            } else if let Some(target) = next_block {
                current_block = target;
            } else {
                // Block ended without terminator - this shouldn't happen in well-formed MIR
                // but let's handle it gracefully by returning void
                return Ok(VMValue::Void);
            }
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &MirInstruction) -> Result<ControlFlow, VMError> {
        match instruction {
            MirInstruction::Const { dst, value } => {
                let vm_value = VMValue::from(value);
                self.values.insert(*dst, vm_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                let left = self.get_value(*lhs)?;
                let right = self.get_value(*rhs)?;
                let result = self.execute_binary_op(op, &left, &right)?;
                self.values.insert(*dst, result);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::UnaryOp { dst, op, operand } => {
                let operand_val = self.get_value(*operand)?;
                let result = self.execute_unary_op(op, &operand_val)?;
                self.values.insert(*dst, result);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                let left = self.get_value(*lhs)?;
                let right = self.get_value(*rhs)?;
                let result = self.execute_compare_op(op, &left, &right)?;
                self.values.insert(*dst, VMValue::Bool(result));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Print { value, .. } => {
                let val = self.get_value(*value)?;
                println!("{}", val.to_string());
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Return { value } => {
                let return_value = if let Some(val_id) = value {
                    self.get_value(*val_id)?
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
                // For now, simplified phi - use first available input
                // In a real implementation, we'd need to track which block we came from
                if let Some((_, value_id)) = inputs.first() {
                    let value = self.get_value(*value_id)?;
                    self.values.insert(*dst, value);
                }
                Ok(ControlFlow::Continue)
            },
            
            // Missing instructions that need basic implementations
            MirInstruction::Load { dst, ptr } => {
                // For now, loading is the same as getting the value
                let value = self.get_value(*ptr)?;
                self.values.insert(*dst, value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Store { value, ptr } => {
                // For now, storing just updates the ptr with the value
                let val = self.get_value(*value)?;
                self.values.insert(*ptr, val);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Call { dst, func: _, args: _, effects: _ } => {
                // For now, function calls return void
                // TODO: Implement proper function call handling
                if let Some(dst_id) = dst {
                    self.values.insert(*dst_id, VMValue::Void);
                }
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::BoxCall { dst, box_val: _, method: _, args: _, effects: _ } => {
                // For now, box method calls return void
                // TODO: Implement proper box method call handling
                if let Some(dst_id) = dst {
                    self.values.insert(*dst_id, VMValue::Void);
                }
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::NewBox { dst, box_type: _, args: _ } => {
                // For now, new box creates a placeholder string value
                // TODO: Implement proper box creation
                self.values.insert(*dst, VMValue::String("NewBox".to_string()));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::TypeCheck { dst, value: _, expected_type: _ } => {
                // For now, type checks always return true
                // TODO: Implement proper type checking
                self.values.insert(*dst, VMValue::Bool(true));
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Cast { dst, value, target_type: _ } => {
                // For now, casting just copies the value
                // TODO: Implement proper type casting
                let val = self.get_value(*value)?;
                self.values.insert(*dst, val);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::ArrayGet { dst, array: _, index: _ } => {
                // For now, array access returns a placeholder
                // TODO: Implement proper array access
                self.values.insert(*dst, VMValue::Integer(0));
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
                self.values.insert(*dst, val);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::Debug { value, message: _ } => {
                // Debug instruction - print value for debugging
                let val = self.get_value(*value)?;
                println!("DEBUG: {}", val.to_string());
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
                self.values.insert(*exception_value, VMValue::Void);
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
                self.values.insert(*dst, box_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::RefGet { dst, reference, field } => {
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
                
                self.values.insert(*dst, field_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::RefSet { reference, field, value } => {
                // Get the value to set
                let new_value = self.get_value(*value)?;
                
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
                self.values.insert(*dst, box_value);
                Ok(ControlFlow::Continue)
            },
            
            MirInstruction::WeakLoad { dst, weak_ref } => {
                // For now, loading from weak ref is the same as getting the value
                // In a real implementation, this would check if the weak ref is still valid
                let weak_value = self.get_value(*weak_ref)?;
                self.values.insert(*dst, weak_value);
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
                future.set_result(initial_value.to_nyash_box());
                self.values.insert(*dst, VMValue::Future(future));
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
                    self.values.insert(*dst, vm_value);
                    Ok(ControlFlow::Continue)
                } else {
                    Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
                }
            },
        }
    }
    
    /// Get a value from storage
    fn get_value(&self, value_id: ValueId) -> Result<VMValue, VMError> {
        self.values.get(&value_id)
            .cloned()
            .ok_or_else(|| VMError::InvalidValue(format!("Value {} not found", value_id)))
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
        vm.values.insert(ValueId(1), VMValue::Integer(10));
        vm.values.insert(ValueId(2), VMValue::Integer(32));
        
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
}