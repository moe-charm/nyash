/*!
 * MIR Core Types — split from instruction.rs for clarity (behavior-preserving)
 */

use std::fmt;

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

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Logical
    And,
    Or,
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
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
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

/// Kind of unified type operation (PoC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeOpKind {
    Check,
    Cast,
}

/// Kind of unified weak reference operation (PoC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeakRefOp {
    New,
    Load,
}

/// Kind of unified barrier operation (PoC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierOp {
    Read,
    Write,
}

