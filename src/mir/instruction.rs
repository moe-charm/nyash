/*!
 * MIR Instruction Set - 20 Core Instructions per ChatGPT5 Design
 * 
 * SSA-form instructions with effect tracking for optimization
 */

use super::{ValueId, LocalId, EffectMask, Effect};
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
    
    /// No-op instruction (for optimization placeholders)
    Nop,
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
    Void,
    Unknown,
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
            MirInstruction::Copy { dst, .. } => Some(*dst),
            
            MirInstruction::Call { dst, .. } |
            MirInstruction::BoxCall { dst, .. } => *dst,
            
            MirInstruction::Store { .. } |
            MirInstruction::Branch { .. } |
            MirInstruction::Jump { .. } |
            MirInstruction::Return { .. } |
            MirInstruction::ArraySet { .. } |
            MirInstruction::Debug { .. } |
            MirInstruction::Nop => None,
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
            MirInstruction::Debug { value: operand, .. } => vec![*operand],
            
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
}