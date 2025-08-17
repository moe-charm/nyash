/*!
 * MIR Instruction Set - 20 Core Instructions per ChatGPT5 Design
 * 
 * SSA-form instructions with effect tracking for optimization
 */

use super::{ValueId, EffectMask, Effect};
// use crate::value::NyashValue;  // Commented out to avoid circular dependency
use std::fmt;

/// MIR instruction types - limited to 20 core instructions
#[derive(Debug, Clone, PartialEq)]
pub enum MirInstruction {
    // === Constants and Values ===
    /// Load a constant value
    /// `%dst = const value`
    Const {
        dst: ValueId,
        value: ConstValue,
    },
    
    // === Arithmetic Operations ===
    /// Binary arithmetic operation
    /// `%dst = %lhs op %rhs`
    BinOp {
        dst: ValueId,
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    
    /// Unary operation
    /// `%dst = op %operand`
    UnaryOp {
        dst: ValueId,
        op: UnaryOp,
        operand: ValueId,
    },
    
    // === Comparison Operations ===
    /// Compare two values
    /// `%dst = %lhs cmp %rhs`
    Compare {
        dst: ValueId,
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    
    // === Memory Operations ===
    /// Load from memory/variable
    /// `%dst = load %ptr`
    Load {
        dst: ValueId,
        ptr: ValueId,
    },
    
    /// Store to memory/variable
    /// `store %value -> %ptr`
    Store {
        value: ValueId,
        ptr: ValueId,
    },
    
    // === Function Calls ===
    /// Call a function
    /// `%dst = call %func(%args...)`
    Call {
        dst: Option<ValueId>,
        func: ValueId,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    /// Box method invocation
    /// `%dst = invoke %box.method(%args...)`
    BoxCall {
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    // === Control Flow ===
    /// Conditional branch
    /// `br %condition -> %then_bb, %else_bb`
    Branch {
        condition: ValueId,
        then_bb: super::BasicBlockId,
        else_bb: super::BasicBlockId,
    },
    
    /// Unconditional jump
    /// `jmp %target_bb`
    Jump {
        target: super::BasicBlockId,
    },
    
    /// Return from function
    /// `ret %value` or `ret void`
    Return {
        value: Option<ValueId>,
    },
    
    // === SSA Phi Function ===
    /// SSA phi function for merging values from different paths
    /// `%dst = phi [%val1 from %bb1, %val2 from %bb2, ...]`
    Phi {
        dst: ValueId,
        inputs: Vec<(super::BasicBlockId, ValueId)>,
    },
    
    // === Box Operations ===
    /// Create a new Box instance
    /// `%dst = new_box "BoxType"(%args...)`
    NewBox {
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
    },
    
    /// Check Box type
    /// `%dst = type_check %box "BoxType"`
    TypeCheck {
        dst: ValueId,
        value: ValueId,
        expected_type: String,
    },
    
    // === Type Conversion ===
    /// Convert between types
    /// `%dst = cast %value as Type`
    Cast {
        dst: ValueId,
        value: ValueId,
        target_type: MirType,
    },
    
    // === Array Operations ===
    /// Get array element
    /// `%dst = %array[%index]`
    ArrayGet {
        dst: ValueId,
        array: ValueId,
        index: ValueId,
    },
    
    /// Set array element
    /// `%array[%index] = %value`
    ArraySet {
        array: ValueId,
        index: ValueId,
        value: ValueId,
    },
    
    // === Special Operations ===
    /// Copy a value (for optimization passes)
    /// `%dst = copy %src`
    Copy {
        dst: ValueId,
        src: ValueId,
    },
    
    /// Debug/introspection instruction
    /// `debug %value "message"`
    Debug {
        value: ValueId,
        message: String,
    },
    
    /// Print instruction for console output
    /// `print %value`
    Print {
        value: ValueId,
        effects: EffectMask,
    },
    
    /// No-op instruction (for optimization placeholders)
    Nop,
    
    // === Control Flow & Exception Handling (Phase 5) ===
    
    /// Throw an exception
    /// `throw %exception_value`
    Throw {
        exception: ValueId,
        effects: EffectMask,
    },
    
    /// Catch handler setup (landing pad for exceptions)
    /// `catch %exception_type -> %handler_bb`
    Catch {
        exception_type: Option<String>, // None = catch-all
        exception_value: ValueId,       // Where to store caught exception
        handler_bb: super::BasicBlockId,
    },
    
    /// Safepoint instruction (no-op for now, can be used for GC/debugging)
    /// `safepoint`
    Safepoint,
    
    // === Phase 6: Box Reference Operations ===
    
    /// Create a new reference to a Box
    /// `%dst = ref_new %box`
    RefNew {
        dst: ValueId,
        box_val: ValueId,
    },
    
    /// Get/dereference a Box field through reference
    /// `%dst = ref_get %ref.field`
    RefGet {
        dst: ValueId,
        reference: ValueId,
        field: String,
    },
    
    /// Set/assign Box field through reference
    /// `ref_set %ref.field = %value`
    RefSet {
        reference: ValueId,
        field: String,
        value: ValueId,
    },
    
    /// Create a weak reference to a Box
    /// `%dst = weak_new %box`
    WeakNew {
        dst: ValueId,
        box_val: ValueId,
    },
    
    /// Load from weak reference (if still alive)
    /// `%dst = weak_load %weak_ref`
    WeakLoad {
        dst: ValueId,
        weak_ref: ValueId,
    },
    
    /// Memory barrier read (no-op for now, proper effect annotation)
    /// `barrier_read %ptr`
    BarrierRead {
        ptr: ValueId,
    },
    
    /// Memory barrier write (no-op for now, proper effect annotation)
    /// `barrier_write %ptr`
    BarrierWrite {
        ptr: ValueId,
    },
    
    // === Phase 7: Async/Future Operations ===
    
    /// Create a new Future with initial value
    /// `%dst = future_new %value`
    FutureNew {
        dst: ValueId,
        value: ValueId,
    },
    
    /// Set Future value and mark as ready
    /// `future_set %future = %value`
    FutureSet {
        future: ValueId,
        value: ValueId,
    },
    
    /// Wait for Future completion and get value
    /// `%dst = await %future`
    Await {
        dst: ValueId,
        future: ValueId,
    },
    
    // === Phase 9.7: External Function Calls (Box FFI/ABI) ===
    
    /// External function call through Box FFI/ABI
    /// `%dst = extern_call interface.method(%args...)`
    ExternCall {
        dst: Option<ValueId>,
        iface_name: String,         // e.g., "env.console"
        method_name: String,        // e.g., "log"
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    // === Phase 8.5: MIR 26-instruction reduction (NEW) ===
    
    /// Box field load operation (replaces Load)
    /// `%dst = %box.field`
    BoxFieldLoad {
        dst: ValueId,
        box_val: ValueId,
        field: String,
    },
    
    /// Box field store operation (replaces Store)
    /// `%box.field = %value`
    BoxFieldStore {
        box_val: ValueId,
        field: String,
        value: ValueId,
    },
    
    /// Check weak reference validity
    /// `%dst = weak_check %weak_ref`
    WeakCheck {
        dst: ValueId,
        weak_ref: ValueId,
    },
    
    /// Send data via Bus
    /// `send %data -> %target`
    Send {
        data: ValueId,
        target: ValueId,
    },
    
    /// Receive data from Bus
    /// `%dst = recv %source`
    Recv {
        dst: ValueId,
        source: ValueId,
    },
    
    /// Tail call optimization
    /// `tail_call %func(%args...)`
    TailCall {
        func: ValueId,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    /// Adopt ownership (parent takes child)
    /// `adopt %parent <- %child`
    Adopt {
        parent: ValueId,
        child: ValueId,
    },
    
    /// Release strong ownership
    /// `release %ref`
    Release {
        reference: ValueId,
    },
    
    /// Memory copy optimization
    /// `memcopy %dst <- %src, %size`
    MemCopy {
        dst: ValueId,
        src: ValueId,
        size: ValueId,
    },
    
    /// Atomic memory fence
    /// `atomic_fence %ordering`
    AtomicFence {
        ordering: AtomicOrdering,
    },
}

/// Constant values in MIR
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    Void,
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    
    // Logical
    And, Or,
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    // Arithmetic
    Neg,
    
    // Logical
    Not,
    
    // Bitwise
    BitNot,
}

/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq, Ne, Lt, Le, Gt, Ge,
}

/// MIR type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirType {
    Integer,
    Float,
    Bool,
    String,
    Box(String), // Box type with name
    Array(Box<MirType>),
    Future(Box<MirType>), // Future containing a type
    Void,
    Unknown,
}

/// Atomic memory ordering for fence operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicOrdering {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

impl MirInstruction {
    /// Get the effect mask for this instruction
    pub fn effects(&self) -> EffectMask {
        match self {
            // Pure operations
            MirInstruction::Const { .. } |
            MirInstruction::BinOp { .. } |
            MirInstruction::UnaryOp { .. } |
            MirInstruction::Compare { .. } |
            MirInstruction::Cast { .. } |
            MirInstruction::Copy { .. } |
            MirInstruction::Phi { .. } |
            MirInstruction::TypeCheck { .. } |
            MirInstruction::Nop => EffectMask::PURE,
            
            // Memory operations
            MirInstruction::Load { .. } => EffectMask::READ,
            MirInstruction::Store { .. } |
            MirInstruction::ArraySet { .. } => EffectMask::WRITE,
            MirInstruction::ArrayGet { .. } => EffectMask::READ,
            
            // Function calls use provided effect mask
            MirInstruction::Call { effects, .. } |
            MirInstruction::BoxCall { effects, .. } => *effects,
            
            // Control flow (pure but affects execution)
            MirInstruction::Branch { .. } |
            MirInstruction::Jump { .. } |
            MirInstruction::Return { .. } => EffectMask::PURE,
            
            // Box creation may allocate
            MirInstruction::NewBox { .. } => EffectMask::PURE.add(Effect::Alloc),
            
            // Debug has debug effect
            MirInstruction::Debug { .. } => EffectMask::PURE.add(Effect::Debug),
            
            // Print has external write effect
            MirInstruction::Print { effects, .. } => *effects,
            
            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { effects, .. } => *effects,
            MirInstruction::Catch { .. } => EffectMask::PURE, // Setting up handler is pure
            MirInstruction::Safepoint => EffectMask::PURE,    // No-op for now
            
            // Phase 6: Box reference operations
            MirInstruction::RefNew { .. } => EffectMask::PURE, // Creating reference is pure
            MirInstruction::RefGet { .. } => EffectMask::READ, // Reading field has read effects
            MirInstruction::RefSet { .. } => EffectMask::WRITE, // Writing field has write effects
            MirInstruction::WeakNew { .. } => EffectMask::PURE, // Creating weak ref is pure
            MirInstruction::WeakLoad { .. } => EffectMask::READ, // Loading weak ref has read effects
            MirInstruction::BarrierRead { .. } => EffectMask::READ.add(Effect::Barrier), // Memory barrier with read
            MirInstruction::BarrierWrite { .. } => EffectMask::WRITE.add(Effect::Barrier), // Memory barrier with write
            
            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { .. } => EffectMask::PURE.add(Effect::Alloc), // Creating future may allocate
            MirInstruction::FutureSet { .. } => EffectMask::WRITE, // Setting future has write effects
            MirInstruction::Await { .. } => EffectMask::READ.add(Effect::Async), // Await blocks and reads
            
            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { effects, .. } => *effects, // Use provided effect mask
            
            // Phase 8.5: MIR 26-instruction reduction (NEW)
            MirInstruction::BoxFieldLoad { .. } => EffectMask::READ, // Box field read
            MirInstruction::BoxFieldStore { .. } => EffectMask::WRITE, // Box field write
            MirInstruction::WeakCheck { .. } => EffectMask::PURE, // Check is pure
            MirInstruction::Send { .. } => EffectMask::IO, // Bus send has IO effects
            MirInstruction::Recv { .. } => EffectMask::IO, // Bus recv has IO effects
            MirInstruction::TailCall { effects, .. } => *effects, // Use provided effect mask
            MirInstruction::Adopt { .. } => EffectMask::WRITE, // Ownership change has write effects
            MirInstruction::Release { .. } => EffectMask::WRITE, // Ownership release has write effects
            MirInstruction::MemCopy { .. } => EffectMask::WRITE, // Memory copy has write effects
            MirInstruction::AtomicFence { .. } => EffectMask::IO.add(Effect::Barrier), // Fence has barrier + IO
        }
    }
    
    /// Get the destination ValueId if this instruction produces a value
    pub fn dst_value(&self) -> Option<ValueId> {
        match self {
            MirInstruction::Const { dst, .. } |
            MirInstruction::BinOp { dst, .. } |
            MirInstruction::UnaryOp { dst, .. } |
            MirInstruction::Compare { dst, .. } |
            MirInstruction::Load { dst, .. } |
            MirInstruction::Phi { dst, .. } |
            MirInstruction::NewBox { dst, .. } |
            MirInstruction::TypeCheck { dst, .. } |
            MirInstruction::Cast { dst, .. } |
            MirInstruction::ArrayGet { dst, .. } |
            MirInstruction::Copy { dst, .. } |
            MirInstruction::RefNew { dst, .. } |
            MirInstruction::RefGet { dst, .. } |
            MirInstruction::WeakNew { dst, .. } |
            MirInstruction::WeakLoad { dst, .. } |
            MirInstruction::FutureNew { dst, .. } |
            MirInstruction::Await { dst, .. } => Some(*dst),
            
            MirInstruction::Call { dst, .. } |
            MirInstruction::BoxCall { dst, .. } |
            MirInstruction::ExternCall { dst, .. } => *dst,
            
            // Phase 8.5: MIR 26-instruction reduction (NEW)
            MirInstruction::BoxFieldLoad { dst, .. } |
            MirInstruction::WeakCheck { dst, .. } |
            MirInstruction::Recv { dst, .. } |
            MirInstruction::MemCopy { dst, .. } => Some(*dst),
            
            MirInstruction::Store { .. } |
            MirInstruction::Branch { .. } |
            MirInstruction::Jump { .. } |
            MirInstruction::Return { .. } |
            MirInstruction::ArraySet { .. } |
            MirInstruction::Debug { .. } |
            MirInstruction::Print { .. } |
            MirInstruction::Throw { .. } |
            MirInstruction::RefSet { .. } |
            MirInstruction::BarrierRead { .. } |
            MirInstruction::BarrierWrite { .. } |
            MirInstruction::FutureSet { .. } |
            MirInstruction::Safepoint |
            MirInstruction::Nop => None,
            
            // Phase 8.5: Non-value producing instructions
            MirInstruction::BoxFieldStore { .. } |
            MirInstruction::Send { .. } |
            MirInstruction::TailCall { .. } |
            MirInstruction::Adopt { .. } |
            MirInstruction::Release { .. } |
            MirInstruction::AtomicFence { .. } => None,
            
            MirInstruction::Catch { exception_value, .. } => Some(*exception_value),
        }
    }
    
    /// Get all ValueIds used by this instruction
    pub fn used_values(&self) -> Vec<ValueId> {
        match self {
            MirInstruction::Const { .. } |
            MirInstruction::Jump { .. } |
            MirInstruction::Nop => Vec::new(),
            
            MirInstruction::UnaryOp { operand, .. } |
            MirInstruction::Load { ptr: operand, .. } |
            MirInstruction::TypeCheck { value: operand, .. } |
            MirInstruction::Cast { value: operand, .. } |
            MirInstruction::Copy { src: operand, .. } |
            MirInstruction::Debug { value: operand, .. } |
            MirInstruction::Print { value: operand, .. } => vec![*operand],
            
            MirInstruction::BinOp { lhs, rhs, .. } |
            MirInstruction::Compare { lhs, rhs, .. } |
            MirInstruction::Store { value: lhs, ptr: rhs, .. } => vec![*lhs, *rhs],
            
            MirInstruction::ArrayGet { array, index, .. } => vec![*array, *index],
            
            MirInstruction::ArraySet { array, index, value } => vec![*array, *index, *value],
            
            MirInstruction::Branch { condition, .. } => vec![*condition],
            
            MirInstruction::Return { value } => {
                value.map(|v| vec![v]).unwrap_or_default()
            },
            
            MirInstruction::Call { func, args, .. } => {
                let mut used = vec![*func];
                used.extend(args);
                used
            },
            
            MirInstruction::BoxCall { box_val, args, .. } => {
                let mut used = vec![*box_val];
                used.extend(args);
                used
            },
            
            MirInstruction::NewBox { args, .. } => args.clone(),
            
            MirInstruction::Phi { inputs, .. } => {
                inputs.iter().map(|(_, value)| *value).collect()
            },
            
            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { exception, .. } => vec![*exception],
            MirInstruction::Catch { .. } => Vec::new(), // Handler setup doesn't use values
            MirInstruction::Safepoint => Vec::new(),
            
            // Phase 6: Box reference operations
            MirInstruction::RefNew { box_val, .. } => vec![*box_val],
            MirInstruction::RefGet { reference, .. } => vec![*reference],
            MirInstruction::RefSet { reference, value, .. } => vec![*reference, *value],
            MirInstruction::WeakNew { box_val, .. } => vec![*box_val],
            MirInstruction::WeakLoad { weak_ref, .. } => vec![*weak_ref],
            MirInstruction::BarrierRead { ptr } => vec![*ptr],
            MirInstruction::BarrierWrite { ptr } => vec![*ptr],
            
            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { value, .. } => vec![*value],
            MirInstruction::FutureSet { future, value } => vec![*future, *value],
            MirInstruction::Await { future, .. } => vec![*future],
            
            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { args, .. } => args.clone(),
            
            // Phase 8.5: MIR 26-instruction reduction (NEW)
            MirInstruction::BoxFieldLoad { box_val, .. } => vec![*box_val],
            MirInstruction::BoxFieldStore { box_val, value, .. } => vec![*box_val, *value],
            MirInstruction::WeakCheck { weak_ref, .. } => vec![*weak_ref],
            MirInstruction::Send { data, target } => vec![*data, *target],
            MirInstruction::Recv { source, .. } => vec![*source],
            MirInstruction::TailCall { func, args, .. } => {
                let mut used = vec![*func];
                used.extend(args);
                used
            },
            MirInstruction::Adopt { parent, child } => vec![*parent, *child],
            MirInstruction::Release { reference } => vec![*reference],
            MirInstruction::MemCopy { dst, src, size } => vec![*dst, *src, *size],
            MirInstruction::AtomicFence { .. } => Vec::new(), // Fence doesn't use values
        }
    }
}

impl ConstValue {
    /*
    /// Convert to NyashValue
    pub fn to_nyash_value(&self) -> NyashValue {
        match self {
            ConstValue::Integer(n) => NyashValue::new_integer(*n),
            ConstValue::Float(f) => NyashValue::new_float(*f),
            ConstValue::Bool(b) => NyashValue::new_bool(*b),
            ConstValue::String(s) => NyashValue::new_string(s.clone()),
            ConstValue::Null => NyashValue::new_null(),
            ConstValue::Void => NyashValue::new_void(),
        }
    }
    
    /// Create from NyashValue
    pub fn from_nyash_value(value: &NyashValue) -> Option<Self> {
        match value {
            NyashValue::Integer(n) => Some(ConstValue::Integer(*n)),
            NyashValue::Float(f) => Some(ConstValue::Float(*f)),
            NyashValue::Bool(b) => Some(ConstValue::Bool(*b)),
            NyashValue::String(s) => Some(ConstValue::String(s.clone())),
            NyashValue::Null => Some(ConstValue::Null),
            NyashValue::Void => Some(ConstValue::Void),
            _ => None, // Collections and Boxes can't be constants
        }
    }
    */
}

impl fmt::Display for MirInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirInstruction::Const { dst, value } => {
                write!(f, "{} = const {}", dst, value)
            },
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                write!(f, "{} = {} {:?} {}", dst, lhs, op, rhs)
            },
            MirInstruction::UnaryOp { dst, op, operand } => {
                write!(f, "{} = {:?} {}", dst, op, operand)
            },
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                write!(f, "{} = {} {:?} {}", dst, lhs, op, rhs)
            },
            MirInstruction::Load { dst, ptr } => {
                write!(f, "{} = load {}", dst, ptr)
            },
            MirInstruction::Store { value, ptr } => {
                write!(f, "store {} -> {}", value, ptr)
            },
            MirInstruction::Call { dst, func, args, effects } => {
                if let Some(dst) = dst {
                    write!(f, "{} = call {}({}); effects: {}", dst, func, 
                           args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", "),
                           effects)
                } else {
                    write!(f, "call {}({}); effects: {}", func, 
                           args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", "),
                           effects)
                }
            },
            MirInstruction::Return { value } => {
                if let Some(value) = value {
                    write!(f, "ret {}", value)
                } else {
                    write!(f, "ret void")
                }
            },
            MirInstruction::ExternCall { dst, iface_name, method_name, args, effects } => {
                if let Some(dst) = dst {
                    write!(f, "{} = extern_call {}.{}({}); effects: {}", dst, iface_name, method_name,
                           args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", "),
                           effects)
                } else {
                    write!(f, "extern_call {}.{}({}); effects: {}", iface_name, method_name,
                           args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", "),
                           effects)
                }
            },
            // Phase 8.5: MIR 26-instruction reduction (NEW)
            MirInstruction::BoxFieldLoad { dst, box_val, field } => {
                write!(f, "{} = {}.{}", dst, box_val, field)
            },
            MirInstruction::BoxFieldStore { box_val, field, value } => {
                write!(f, "{}.{} = {}", box_val, field, value)
            },
            MirInstruction::WeakCheck { dst, weak_ref } => {
                write!(f, "{} = weak_check {}", dst, weak_ref)
            },
            MirInstruction::Send { data, target } => {
                write!(f, "send {} -> {}", data, target)
            },
            MirInstruction::Recv { dst, source } => {
                write!(f, "{} = recv {}", dst, source)
            },
            MirInstruction::TailCall { func, args, effects } => {
                write!(f, "tail_call {}({}); effects: {}", func,
                       args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", "),
                       effects)
            },
            MirInstruction::Adopt { parent, child } => {
                write!(f, "adopt {} <- {}", parent, child)
            },
            MirInstruction::Release { reference } => {
                write!(f, "release {}", reference)
            },
            MirInstruction::MemCopy { dst, src, size } => {
                write!(f, "memcopy {} <- {}, {}", dst, src, size)
            },
            MirInstruction::AtomicFence { ordering } => {
                write!(f, "atomic_fence {:?}", ordering)
            },
            _ => write!(f, "{:?}", self), // Fallback for other instructions
        }
    }
}

impl fmt::Display for ConstValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstValue::Integer(n) => write!(f, "{}", n),
            ConstValue::Float(fl) => write!(f, "{}", fl),
            ConstValue::Bool(b) => write!(f, "{}", b),
            ConstValue::String(s) => write!(f, "\"{}\"", s),
            ConstValue::Null => write!(f, "null"),
            ConstValue::Void => write!(f, "void"),
        }
    }
}

impl fmt::Display for AtomicOrdering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomicOrdering::Relaxed => write!(f, "relaxed"),
            AtomicOrdering::Acquire => write!(f, "acquire"),
            AtomicOrdering::Release => write!(f, "release"),
            AtomicOrdering::AcqRel => write!(f, "acq_rel"),
            AtomicOrdering::SeqCst => write!(f, "seq_cst"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_const_instruction() {
        let dst = ValueId::new(0);
        let inst = MirInstruction::Const {
            dst,
            value: ConstValue::Integer(42),
        };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert!(inst.used_values().is_empty());
        assert!(inst.effects().is_pure());
    }
    
    #[test]
    fn test_binop_instruction() {
        let dst = ValueId::new(0);
        let lhs = ValueId::new(1);
        let rhs = ValueId::new(2);
        
        let inst = MirInstruction::BinOp {
            dst, op: BinaryOp::Add, lhs, rhs
        };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![lhs, rhs]);
        assert!(inst.effects().is_pure());
    }
    
    #[test]
    fn test_call_instruction() {
        let dst = ValueId::new(0);
        let func = ValueId::new(1);
        let arg1 = ValueId::new(2);
        let arg2 = ValueId::new(3);
        
        let inst = MirInstruction::Call {
            dst: Some(dst),
            func,
            args: vec![arg1, arg2],
            effects: EffectMask::IO,
        };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![func, arg1, arg2]);
        assert_eq!(inst.effects(), EffectMask::IO);
    }
    
    /*
    #[test]
    fn test_const_value_conversion() {
        let const_val = ConstValue::Integer(42);
        let nyash_val = const_val.to_nyash_value();
        
        assert_eq!(nyash_val, NyashValue::new_integer(42));
        
        let back = ConstValue::from_nyash_value(&nyash_val).unwrap();
        assert_eq!(back, const_val);
    }
    */
    
    #[test]
    fn test_ref_new_instruction() {
        let dst = ValueId::new(0);
        let box_val = ValueId::new(1);
        let inst = MirInstruction::RefNew { dst, box_val };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![box_val]);
        assert!(inst.effects().is_pure());
    }
    
    #[test]
    fn test_ref_get_instruction() {
        let dst = ValueId::new(0);
        let reference = ValueId::new(1);
        let field = "name".to_string();
        let inst = MirInstruction::RefGet { dst, reference, field };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![reference]);
        assert!(!inst.effects().is_pure());
        assert!(inst.effects().contains(super::super::effect::Effect::ReadHeap));
    }
    
    #[test]
    fn test_ref_set_instruction() {
        let reference = ValueId::new(0);
        let field = "value".to_string();
        let value = ValueId::new(1);
        let inst = MirInstruction::RefSet { reference, field, value };
        
        assert_eq!(inst.dst_value(), None);
        assert_eq!(inst.used_values(), vec![reference, value]);
        assert!(!inst.effects().is_pure());
        assert!(inst.effects().contains(super::super::effect::Effect::WriteHeap));
    }
    
    #[test] 
    fn test_weak_new_instruction() {
        let dst = ValueId::new(0);
        let box_val = ValueId::new(1);
        let inst = MirInstruction::WeakNew { dst, box_val };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![box_val]);
        assert!(inst.effects().is_pure());
    }
    
    #[test]
    fn test_weak_load_instruction() {
        let dst = ValueId::new(0);
        let weak_ref = ValueId::new(1);
        let inst = MirInstruction::WeakLoad { dst, weak_ref };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![weak_ref]);
        assert!(!inst.effects().is_pure());
        assert!(inst.effects().contains(super::super::effect::Effect::ReadHeap));
    }
    
    #[test]
    fn test_barrier_instructions() {
        let ptr = ValueId::new(0);
        
        let read_barrier = MirInstruction::BarrierRead { ptr };
        assert_eq!(read_barrier.dst_value(), None);
        assert_eq!(read_barrier.used_values(), vec![ptr]);
        assert!(read_barrier.effects().contains(super::super::effect::Effect::Barrier));
        assert!(read_barrier.effects().contains(super::super::effect::Effect::ReadHeap));
        
        let write_barrier = MirInstruction::BarrierWrite { ptr };
        assert_eq!(write_barrier.dst_value(), None);
        assert_eq!(write_barrier.used_values(), vec![ptr]);
        assert!(write_barrier.effects().contains(super::super::effect::Effect::Barrier));
        assert!(write_barrier.effects().contains(super::super::effect::Effect::WriteHeap));
    }
    
    #[test]
    fn test_extern_call_instruction() {
        let dst = ValueId::new(0);
        let arg1 = ValueId::new(1);
        let arg2 = ValueId::new(2);
        let inst = MirInstruction::ExternCall {
            dst: Some(dst),
            iface_name: "env.console".to_string(),
            method_name: "log".to_string(),
            args: vec![arg1, arg2],
            effects: super::super::effect::EffectMask::IO,
        };
        
        assert_eq!(inst.dst_value(), Some(dst));
        assert_eq!(inst.used_values(), vec![arg1, arg2]);
        assert_eq!(inst.effects(), super::super::effect::EffectMask::IO);
        
        // Test void extern call
        let void_inst = MirInstruction::ExternCall {
            dst: None,
            iface_name: "env.canvas".to_string(),
            method_name: "fillRect".to_string(),
            args: vec![arg1],
            effects: super::super::effect::EffectMask::IO,
        };
        
        assert_eq!(void_inst.dst_value(), None);
        assert_eq!(void_inst.used_values(), vec![arg1]);
    }
}