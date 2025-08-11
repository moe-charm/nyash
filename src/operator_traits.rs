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

// ===== Operator Resolution System =====

/// High-level operator resolution that tries static dispatch first,
/// then falls back to dynamic dispatch
pub struct OperatorResolver;

impl OperatorResolver {
    /// Resolve addition operation with hybrid dispatch
    pub fn resolve_add(
        left: &dyn NyashBox,
        right: &dyn NyashBox,
    ) -> Result<Box<dyn NyashBox>, OperatorError> {
        // Try to cast to concrete types first and use their DynamicAdd implementation
        // This approach uses the concrete types rather than trait objects
        
        // Check if left implements DynamicAdd by trying common types
        if let Some(int_box) = left.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            if let Some(result) = int_box.try_add(right) {
                return Ok(result);
            }
        }
        
        if let Some(str_box) = left.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            if let Some(result) = str_box.try_add(right) {
                return Ok(result);
            }
        }
        
        if let Some(float_box) = left.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
            if let Some(result) = float_box.try_add(right) {
                return Ok(result);
            }
        }
        
        if let Some(bool_box) = left.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            if let Some(result) = bool_box.try_add(right) {
                return Ok(result);
            }
        }
        
        // If no specific implementation found, return error
        Err(OperatorError::UnsupportedOperation {
            operator: "+".to_string(),
            left_type: left.type_name().to_string(),
            right_type: right.type_name().to_string(),
        })
    }
    
    /// Resolve subtraction operation with hybrid dispatch
    pub fn resolve_sub(
        left: &dyn NyashBox,
        right: &dyn NyashBox,
    ) -> Result<Box<dyn NyashBox>, OperatorError> {
        // Try concrete types for DynamicSub
        if let Some(int_box) = left.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            if let Some(result) = int_box.try_sub(right) {
                return Ok(result);
            }
        }
        
        if let Some(float_box) = left.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
            if let Some(result) = float_box.try_sub(right) {
                return Ok(result);
            }
        }
        
        if let Some(bool_box) = left.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            if let Some(result) = bool_box.try_sub(right) {
                return Ok(result);
            }
        }
        
        Err(OperatorError::UnsupportedOperation {
            operator: "-".to_string(),
            left_type: left.type_name().to_string(),
            right_type: right.type_name().to_string(),
        })
    }
    
    /// Resolve multiplication operation with hybrid dispatch
    pub fn resolve_mul(
        left: &dyn NyashBox,
        right: &dyn NyashBox,
    ) -> Result<Box<dyn NyashBox>, OperatorError> {
        // Try concrete types for DynamicMul
        if let Some(int_box) = left.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            if let Some(result) = int_box.try_mul(right) {
                return Ok(result);
            }
        }
        
        if let Some(str_box) = left.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            if let Some(result) = str_box.try_mul(right) {
                return Ok(result);
            }
        }
        
        if let Some(float_box) = left.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
            if let Some(result) = float_box.try_mul(right) {
                return Ok(result);
            }
        }
        
        if let Some(bool_box) = left.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            if let Some(result) = bool_box.try_mul(right) {
                return Ok(result);
            }
        }
        
        Err(OperatorError::UnsupportedOperation {
            operator: "*".to_string(),
            left_type: left.type_name().to_string(),
            right_type: right.type_name().to_string(),
        })
    }
    
    /// Resolve division operation with hybrid dispatch
    pub fn resolve_div(
        left: &dyn NyashBox,
        right: &dyn NyashBox,
    ) -> Result<Box<dyn NyashBox>, OperatorError> {
        // Try concrete types for DynamicDiv
        if let Some(int_box) = left.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            if let Some(result) = int_box.try_div(right) {
                return Ok(result);
            } else {
                // If try_div returns None, it might be division by zero
                return Err(OperatorError::DivisionByZero);
            }
        }
        
        if let Some(float_box) = left.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
            if let Some(result) = float_box.try_div(right) {
                return Ok(result);
            } else {
                // If try_div returns None, it might be division by zero
                return Err(OperatorError::DivisionByZero);
            }
        }
        
        if let Some(bool_box) = left.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            if let Some(result) = bool_box.try_div(right) {
                return Ok(result);
            } else {
                return Err(OperatorError::DivisionByZero);
            }
        }
        
        Err(OperatorError::UnsupportedOperation {
            operator: "/".to_string(),
            left_type: left.type_name().to_string(),
            right_type: right.type_name().to_string(),
        })
    }
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
                write!(f, "Operator '{}' is not supported between {} and {}. Consider explicit type conversion.", 
                       operator, left_type, right_type)
            }
            OperatorError::DivisionByZero => {
                write!(f, "Division by zero is not allowed")
            }
            OperatorError::AmbiguousOperation { operator, candidates } => {
                write!(f, "Ambiguous operator '{}'. Multiple implementations available: {}", 
                       operator, candidates.join(", "))
            }
        }
    }
}

impl std::error::Error for OperatorError {}

// ===== Performance Optimization Support =====

/// Signature for function overloading (future expansion)
#[derive(Debug, Clone, PartialEq)]
pub struct OperatorSignature {
    pub left_type: String,
    pub right_type: String,
    pub output_type: String,
    pub specificity: u32,  // Higher = more specific
}

impl OperatorSignature {
    pub fn new(left_type: &str, right_type: &str, output_type: &str) -> Self {
        Self {
            left_type: left_type.to_string(),
            right_type: right_type.to_string(),
            output_type: output_type.to_string(),
            specificity: Self::calculate_specificity(left_type, right_type),
        }
    }
    
    /// Calculate specificity for tie-breaking
    /// More specific types get higher scores
    fn calculate_specificity(left_type: &str, right_type: &str) -> u32 {
        // Simple heuristic: exact types are more specific than generic ones
        let mut score = 0;
        
        // Prefer primitive types over complex ones
        if matches!(left_type, "IntegerBox" | "FloatBox" | "StringBox" | "BoolBox") {
            score += 10;
        }
        
        if matches!(right_type, "IntegerBox" | "FloatBox" | "StringBox" | "BoolBox") {
            score += 10;
        }
        
        // Same types are more specific than mixed types
        if left_type == right_type {
            score += 5;
        }
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_signature_specificity() {
        let int_int = OperatorSignature::new("IntegerBox", "IntegerBox", "IntegerBox");
        let int_float = OperatorSignature::new("IntegerBox", "FloatBox", "FloatBox");
        let str_str = OperatorSignature::new("StringBox", "StringBox", "StringBox");
        
        // Same types should be more specific than mixed types
        assert!(int_int.specificity > int_float.specificity);
        assert!(str_str.specificity > int_float.specificity);
        
        // All should have reasonable specificity scores
        assert!(int_int.specificity >= 25); // 10 + 10 + 5
        assert!(int_float.specificity >= 20); // 10 + 10
        assert!(str_str.specificity >= 25); // 10 + 10 + 5
    }
    
    #[test]
    fn test_operator_error_display() {
        let error = OperatorError::UnsupportedOperation {
            operator: "+".to_string(),
            left_type: "StringBox".to_string(),
            right_type: "IntegerBox".to_string(),
        };
        
        let message = format!("{}", error);
        assert!(message.contains("not supported"));
        assert!(message.contains("StringBox"));
        assert!(message.contains("IntegerBox"));
    }
}