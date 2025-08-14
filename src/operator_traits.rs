/*!
 * Nyash Operator Traits System - Rust-Style Trait-Based Overloading
 * 
 * This module implements the new operator overloading system based on the
 * AI consultation decision (2025-08-10). It follows Rust's trait pattern
 * with static/dynamic hybrid dispatch for maximum performance and flexibility.
 * 
 * Design Principles:
 * - Static dispatch when types are known at compile time
 * - Dynamic dispatch (vtable) when types are unknown
 * - Full compatibility with "Everything is Box" philosophy
 * - Coherence rules (orphan rule) to prevent conflicts
 */

use crate::box_trait::NyashBox;
use std::sync::Arc;

// Forward declaration - traits defined in this module are implemented in box_operators
// We need to ensure trait implementations are loaded when this module is used

// ===== Core Operator Traits =====

/// Addition operator trait - equivalent to Rust's std::ops::Add
/// This replaces the old AddBox with a proper trait-based system
pub trait NyashAdd<Rhs = Self> {
    /// The resulting type after applying the `+` operator
    type Output;
    
    /// Performs the `+` operation
    fn add(self, rhs: Rhs) -> Self::Output;
}

/// Subtraction operator trait - equivalent to Rust's std::ops::Sub
pub trait NyashSub<Rhs = Self> {
    /// The resulting type after applying the `-` operator
    type Output;
    
    /// Performs the `-` operation
    fn sub(self, rhs: Rhs) -> Self::Output;
}

/// Multiplication operator trait - equivalent to Rust's std::ops::Mul
pub trait NyashMul<Rhs = Self> {
    /// The resulting type after applying the `*` operator
    type Output;
    
    /// Performs the `*` operation
    fn mul(self, rhs: Rhs) -> Self::Output;
}

/// Division operator trait - equivalent to Rust's std::ops::Div
pub trait NyashDiv<Rhs = Self> {
    /// The resulting type after applying the `/` operator
    type Output;
    
    /// Performs the `/` operation
    fn div(self, rhs: Rhs) -> Self::Output;
}

// ===== Dynamic Dispatch Support for Box<dyn NyashBox> =====

/// Trait for boxes that can be used in addition operations
/// This enables dynamic dispatch when static types are not available
pub trait DynamicAdd: NyashBox {
    /// Try to add this box with another box dynamically
    /// Returns None if the operation is not supported
    fn try_add(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>>;
    
    /// Check if this box can be added with another box type
    fn can_add_with(&self, other_type: &str) -> bool;
}

/// Trait for boxes that can be used in subtraction operations
pub trait DynamicSub: NyashBox {
    /// Try to subtract another box from this box dynamically
    fn try_sub(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>>;
    
    /// Check if this box can be subtracted with another box type
    fn can_sub_with(&self, other_type: &str) -> bool;
}

/// Trait for boxes that can be used in multiplication operations
pub trait DynamicMul: NyashBox {
    /// Try to multiply this box with another box dynamically
    fn try_mul(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>>;
    
    /// Check if this box can be multiplied with another box type
    fn can_mul_with(&self, other_type: &str) -> bool;
}

/// Trait for boxes that can be used in division operations
pub trait DynamicDiv: NyashBox {
    /// Try to divide this box by another box dynamically
    fn try_div(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>>;
    
    /// Check if this box can be divided by another box type
    fn can_div_with(&self, other_type: &str) -> bool;
}

// ===== Error Types =====

/// Errors that can occur during operator resolution
#[derive(Debug, Clone)]
pub enum OperatorError {
    /// The operation is not supported between these types
    UnsupportedOperation {
        operator: String,
        left_type: String,
        right_type: String,
    },
    
    /// Division by zero
    DivisionByZero,
    
    /// Ambiguous operation (multiple implementations match)
    AmbiguousOperation {
        operator: String,
        candidates: Vec<String>,
    },
}

impl std::fmt::Display for OperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorError::UnsupportedOperation { operator, left_type, right_type } => {
                write!(f, "Operator '{}' is not supported between {} and {}", 
                       operator, left_type, right_type)
            }
            OperatorError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            OperatorError::AmbiguousOperation { operator, candidates } => {
                write!(f, "Ambiguous operator '{}': multiple candidates found: {}", 
                       operator, candidates.join(", "))
            }
        }
    }
}

impl std::error::Error for OperatorError {}

// Note: OperatorResolver is now defined in box_operators.rs
// Import it directly from there if needed

