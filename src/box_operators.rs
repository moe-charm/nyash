/*!
 * Box Operator Implementations - Trait-Based Operator Overloading
 * 
 * This module implements the new trait-based operator system for basic Box types.
 * It provides implementations of NyashAdd, NyashSub, etc. for IntegerBox, StringBox,
 * and other fundamental types.
 * 
 * Based on AI consultation decision (2025-08-10): Rust-style traits with
 * static/dynamic hybrid dispatch for optimal performance.
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::boxes::FloatBox;
use crate::operator_traits::{
    NyashAdd, NyashSub, NyashMul, NyashDiv,
    DynamicAdd, DynamicSub, DynamicMul, DynamicDiv,
    OperatorError
};

// ===== IntegerBox Operator Implementations =====

/// IntegerBox + IntegerBox -> IntegerBox
impl NyashAdd<IntegerBox> for IntegerBox {
    type Output = IntegerBox;
    
    fn add(self, rhs: IntegerBox) -> Self::Output {
        IntegerBox::new(self.value + rhs.value)
    }
}

/// IntegerBox - IntegerBox -> IntegerBox
impl NyashSub<IntegerBox> for IntegerBox {
    type Output = IntegerBox;
    
    fn sub(self, rhs: IntegerBox) -> Self::Output {
        IntegerBox::new(self.value - rhs.value)
    }
}

/// IntegerBox * IntegerBox -> IntegerBox
impl NyashMul<IntegerBox> for IntegerBox {
    type Output = IntegerBox;
    
    fn mul(self, rhs: IntegerBox) -> Self::Output {
        IntegerBox::new(self.value * rhs.value)
    }
}

/// IntegerBox / IntegerBox -> IntegerBox (with zero check)
impl NyashDiv<IntegerBox> for IntegerBox {
    type Output = Result<IntegerBox, OperatorError>;
    
    fn div(self, rhs: IntegerBox) -> Self::Output {
        if rhs.value == 0 {
            Err(OperatorError::DivisionByZero)
        } else {
            Ok(IntegerBox::new(self.value / rhs.value))
        }
    }
}

/// Dynamic dispatch implementation for IntegerBox
impl DynamicAdd for IntegerBox {
    fn try_add(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // IntegerBox + IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(IntegerBox::new(self.value + other_int.value)));
        }
        
        // IntegerBox + FloatBox -> FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value as f64 + other_float.value)));
        }
        
        // Fallback: Convert both to strings and concatenate
        // This preserves the existing AddBox behavior
        let left_str = self.to_string_box();
        let right_str = other.to_string_box();
        Some(Box::new(StringBox::new(format!("{}{}", left_str.value, right_str.value))))
    }
    
    fn can_add_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox" | "FloatBox" | "StringBox")
    }
}

impl DynamicSub for IntegerBox {
    fn try_sub(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // IntegerBox - IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(IntegerBox::new(self.value - other_int.value)));
        }
        
        // IntegerBox - FloatBox -> FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value as f64 - other_float.value)));
        }
        
        None // Subtraction not supported for other types
    }
    
    fn can_sub_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox" | "FloatBox")
    }
}

impl DynamicMul for IntegerBox {
    fn try_mul(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // IntegerBox * IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(IntegerBox::new(self.value * other_int.value)));
        }
        
        // IntegerBox * FloatBox -> FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value as f64 * other_float.value)));
        }
        
        // IntegerBox * StringBox -> Repeated string
        if let Some(other_str) = other.as_any().downcast_ref::<StringBox>() {
            if self.value >= 0 && self.value <= 10000 { // Safety limit
                let repeated = other_str.value.repeat(self.value as usize);
                return Some(Box::new(StringBox::new(repeated)));
            }
        }
        
        None
    }
    
    fn can_mul_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox" | "FloatBox" | "StringBox")
    }
}

impl DynamicDiv for IntegerBox {
    fn try_div(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // IntegerBox / IntegerBox -> FloatBox (for precision)
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            if other_int.value == 0 {
                return None; // Division by zero
            }
            return Some(Box::new(FloatBox::new(self.value as f64 / other_int.value as f64)));
        }
        
        // IntegerBox / FloatBox -> FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            if other_float.value == 0.0 {
                return None; // Division by zero
            }
            return Some(Box::new(FloatBox::new(self.value as f64 / other_float.value)));
        }
        
        None
    }
    
    fn can_div_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox" | "FloatBox")
    }
}

// ===== FloatBox Operator Implementations =====

/// FloatBox + FloatBox -> FloatBox
impl NyashAdd<FloatBox> for FloatBox {
    type Output = FloatBox;
    
    fn add(self, rhs: FloatBox) -> Self::Output {
        FloatBox::new(self.value + rhs.value)
    }
}

/// FloatBox - FloatBox -> FloatBox
impl NyashSub<FloatBox> for FloatBox {
    type Output = FloatBox;
    
    fn sub(self, rhs: FloatBox) -> Self::Output {
        FloatBox::new(self.value - rhs.value)
    }
}

/// FloatBox * FloatBox -> FloatBox
impl NyashMul<FloatBox> for FloatBox {
    type Output = FloatBox;
    
    fn mul(self, rhs: FloatBox) -> Self::Output {
        FloatBox::new(self.value * rhs.value)
    }
}

/// FloatBox / FloatBox -> FloatBox (with zero check)
impl NyashDiv<FloatBox> for FloatBox {
    type Output = Result<FloatBox, OperatorError>;
    
    fn div(self, rhs: FloatBox) -> Self::Output {
        if rhs.value == 0.0 {
            Err(OperatorError::DivisionByZero)
        } else {
            Ok(FloatBox::new(self.value / rhs.value))
        }
    }
}

// ===== FloatBox Dynamic Operator Implementations =====

impl DynamicAdd for FloatBox {
    fn try_add(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // FloatBox + FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value + other_float.value)));
        }
        
        // FloatBox + IntegerBox -> FloatBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(FloatBox::new(self.value + other_int.value as f64)));
        }
        
        // Fallback: Convert both to strings and concatenate
        let left_str = self.to_string_box();
        let right_str = other.to_string_box();
        Some(Box::new(StringBox::new(format!("{}{}", left_str.value, right_str.value))))
    }
    
    fn can_add_with(&self, other_type: &str) -> bool {
        matches!(other_type, "FloatBox" | "IntegerBox" | "StringBox")
    }
}

impl DynamicSub for FloatBox {
    fn try_sub(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // FloatBox - FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value - other_float.value)));
        }
        
        // FloatBox - IntegerBox -> FloatBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(FloatBox::new(self.value - other_int.value as f64)));
        }
        
        None // Subtraction not supported for other types
    }
    
    fn can_sub_with(&self, other_type: &str) -> bool {
        matches!(other_type, "FloatBox" | "IntegerBox")
    }
}

impl DynamicMul for FloatBox {
    fn try_mul(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // FloatBox * FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            return Some(Box::new(FloatBox::new(self.value * other_float.value)));
        }
        
        // FloatBox * IntegerBox -> FloatBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            return Some(Box::new(FloatBox::new(self.value * other_int.value as f64)));
        }
        
        None
    }
    
    fn can_mul_with(&self, other_type: &str) -> bool {
        matches!(other_type, "FloatBox" | "IntegerBox")
    }
}

impl DynamicDiv for FloatBox {
    fn try_div(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // FloatBox / FloatBox
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            if other_float.value == 0.0 {
                return None; // Division by zero
            }
            return Some(Box::new(FloatBox::new(self.value / other_float.value)));
        }
        
        // FloatBox / IntegerBox -> FloatBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            if other_int.value == 0 {
                return None; // Division by zero
            }
            return Some(Box::new(FloatBox::new(self.value / other_int.value as f64)));
        }
        
        None
    }
    
    fn can_div_with(&self, other_type: &str) -> bool {
        matches!(other_type, "FloatBox" | "IntegerBox")
    }
}

// ===== StringBox Operator Implementations =====

/// StringBox + StringBox -> StringBox (concatenation)
impl NyashAdd<StringBox> for StringBox {
    type Output = StringBox;
    
    fn add(self, rhs: StringBox) -> Self::Output {
        StringBox::new(format!("{}{}", self.value, rhs.value))
    }
}

/// StringBox * IntegerBox -> StringBox (repetition)
impl NyashMul<IntegerBox> for StringBox {
    type Output = StringBox;
    
    fn mul(self, rhs: IntegerBox) -> Self::Output {
        if rhs.value >= 0 && rhs.value <= 10000 { // Safety limit
            StringBox::new(self.value.repeat(rhs.value as usize))
        } else {
            StringBox::new(String::new()) // Empty string for invalid repetition
        }
    }
}

/// Dynamic dispatch implementation for StringBox
impl DynamicAdd for StringBox {
    fn try_add(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // StringBox + StringBox
        if let Some(other_str) = other.as_any().downcast_ref::<StringBox>() {
            return Some(Box::new(StringBox::new(format!("{}{}", self.value, other_str.value))));
        }
        
        // StringBox + any other type -> Convert to string and concatenate
        let other_str = other.to_string_box();
        Some(Box::new(StringBox::new(format!("{}{}", self.value, other_str.value))))
    }
    
    fn can_add_with(&self, _other_type: &str) -> bool {
        true // StringBox can concatenate with anything via to_string_box()
    }
}

impl DynamicSub for StringBox {
    fn try_sub(&self, _other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        None // Subtraction not defined for strings
    }
    
    fn can_sub_with(&self, _other_type: &str) -> bool {
        false
    }
}

impl DynamicMul for StringBox {
    fn try_mul(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // StringBox * IntegerBox -> Repeated string
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            if other_int.value >= 0 && other_int.value <= 10000 { // Safety limit
                let repeated = self.value.repeat(other_int.value as usize);
                return Some(Box::new(StringBox::new(repeated)));
            }
        }
        
        None
    }
    
    fn can_mul_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox")
    }
}

impl DynamicDiv for StringBox {
    fn try_div(&self, _other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        None // Division not defined for strings
    }
    
    fn can_div_with(&self, _other_type: &str) -> bool {
        false
    }
}

// ===== BoolBox Operator Implementations =====

/// BoolBox + BoolBox -> IntegerBox (logical OR as addition)
impl NyashAdd<BoolBox> for BoolBox {
    type Output = IntegerBox;
    
    fn add(self, rhs: BoolBox) -> Self::Output {
        let result = (self.value as i64) + (rhs.value as i64);
        IntegerBox::new(result)
    }
}

impl DynamicAdd for BoolBox {
    fn try_add(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // BoolBox + BoolBox
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            let result = (self.value as i64) + (other_bool.value as i64);
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        // BoolBox + IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            let result = (self.value as i64) + other_int.value;
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        // Fallback to string concatenation
        let left_str = self.to_string_box();
        let right_str = other.to_string_box();
        Some(Box::new(StringBox::new(format!("{}{}", left_str.value, right_str.value))))
    }
    
    fn can_add_with(&self, other_type: &str) -> bool {
        matches!(other_type, "BoolBox" | "IntegerBox" | "StringBox")
    }
}

impl DynamicSub for BoolBox {
    fn try_sub(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // BoolBox - BoolBox
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            let result = (self.value as i64) - (other_bool.value as i64);
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        // BoolBox - IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            let result = (self.value as i64) - other_int.value;
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        None
    }
    
    fn can_sub_with(&self, other_type: &str) -> bool {
        matches!(other_type, "BoolBox" | "IntegerBox")
    }
}

impl DynamicMul for BoolBox {
    fn try_mul(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // BoolBox * BoolBox -> logical AND
        if let Some(other_bool) = other.as_any().downcast_ref::<BoolBox>() {
            let result = (self.value as i64) * (other_bool.value as i64);
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        // BoolBox * IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            let result = (self.value as i64) * other_int.value;
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        None
    }
    
    fn can_mul_with(&self, other_type: &str) -> bool {
        matches!(other_type, "BoolBox" | "IntegerBox")
    }
}

impl DynamicDiv for BoolBox {
    fn try_div(&self, other: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
        // BoolBox / IntegerBox
        if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            if other_int.value == 0 {
                return None; // Division by zero
            }
            let result = (self.value as i64) / other_int.value;
            return Some(Box::new(IntegerBox::new(result)));
        }
        
        None
    }
    
    fn can_div_with(&self, other_type: &str) -> bool {
        matches!(other_type, "IntegerBox")
    }
}

// ===== High-level Operator Resolution =====

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
        
        if let Some(float_box) = right.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
            if let Some(result) = float_box.try_add(right) {
                return Ok(result);
            }
        }
        
        if let Some(bool_box) = left.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            if let Some(result) = bool_box.try_add(right) {
                return Ok(result);
            }
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_addition() {
        let a = IntegerBox::new(5);
        let b = IntegerBox::new(3);
        let result = a.add(b);
        assert_eq!(result.value, 8);
    }

    #[test]
    fn test_string_concatenation() {
        let a = StringBox::new("Hello");
        let b = StringBox::new(" World");
        let result = a.add(b);
        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn test_string_repetition() {
        let s = StringBox::new("Hi");
        let n = IntegerBox::new(3);
        let result = s.mul(n);
        assert_eq!(result.value, "HiHiHi");
    }

    #[test]
    fn test_dynamic_addition() {
        let a = IntegerBox::new(10);
        let b = StringBox::new("20");
        
        // Test dynamic dispatch
        let result = a.try_add(&b).unwrap();
        let result_str = result.to_string_box();
        assert_eq!(result_str.value, "1020"); // String concatenation fallback
    }

    #[test]
    fn test_boolean_arithmetic() {
        let a = BoolBox::new(true);
        let b = BoolBox::new(false);
        let result = a.add(b);
        assert_eq!(result.value, 1); // true + false = 1 + 0 = 1
    }

    #[test]
    fn test_can_add_with() {
        let int_box = IntegerBox::new(42);
        assert!(int_box.can_add_with("IntegerBox"));
        assert!(int_box.can_add_with("StringBox"));
        
        let str_box = StringBox::new("test");
        assert!(str_box.can_add_with("IntegerBox"));
        assert!(str_box.can_add_with("StringBox"));
    }
}