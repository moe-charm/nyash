/*!
 * Operator Processing Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles binary and unary operations for all Box types
 * Core philosophy: "Everything is Box" with type-safe operations
 */

use super::*;
use crate::ast::UnaryOperator;
use crate::box_trait::{BoolBox, SharedNyashBox};
use crate::operator_traits::{DynamicAdd, DynamicSub, DynamicMul, DynamicDiv, OperatorError};

/// Helper function for addition operations between Box types
pub(super) fn try_add_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
    // IntegerBox + IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value + right_int.value)));
    }
    
    // StringBox + anything -> concatenation
    if let Some(left_str) = left.as_any().downcast_ref::<StringBox>() {
        let right_str = right.to_string_box();
        return Some(Box::new(StringBox::new(format!("{}{}", left_str.value, right_str.value))));
    }
    
    // BoolBox + BoolBox -> IntegerBox 
    if let (Some(left_bool), Some(right_bool)) = (
        left.as_any().downcast_ref::<BoolBox>(),
        right.as_any().downcast_ref::<BoolBox>()
    ) {
        return Some(Box::new(IntegerBox::new((left_bool.value as i64) + (right_bool.value as i64))));
    }
    
    None
}

/// Helper function for subtraction operations between Box types
pub(super) fn try_sub_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
    // IntegerBox - IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value - right_int.value)));
    }
    
    None
}

/// Helper function for multiplication operations between Box types
pub(super) fn try_mul_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
    // IntegerBox * IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value * right_int.value)));
    }
    
    // StringBox * IntegerBox -> repeated string
    if let (Some(left_str), Some(right_int)) = (
        left.as_any().downcast_ref::<StringBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        if right_int.value >= 0 {
            return Some(Box::new(StringBox::new(left_str.value.repeat(right_int.value as usize))));
        }
    }
    
    None
}

/// Helper function for division operations between Box types
pub(super) fn try_div_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Result<Box<dyn NyashBox>, String> {
    // IntegerBox / IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        if right_int.value == 0 {
            return Err("Division by zero".to_string());
        }
        return Ok(Box::new(IntegerBox::new(left_int.value / right_int.value)));
    }
    
    Err("Unsupported division operation".to_string())
}

/// Helper function for modulo operations between Box types
pub(super) fn try_mod_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Result<Box<dyn NyashBox>, String> {
    // IntegerBox % IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        if right_int.value == 0 {
            return Err("Modulo by zero".to_string());
        }
        return Ok(Box::new(IntegerBox::new(left_int.value % right_int.value)));
    }
    
    Err("Unsupported modulo operation".to_string())
}

impl NyashInterpreter {
    /// Execute binary operation between two expressions
    pub(super) fn execute_binary_op(&mut self, op: &BinaryOperator, left: &ASTNode, right: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let left_val = self.execute_expression(left)?;
        let right_val = self.execute_expression(right)?;
        
        match op {
            BinaryOperator::Add => {
                if let Some(result) = try_add_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result)
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Cannot add {} and {}", left_val.type_name(), right_val.type_name()),
                    })
                }
            }
            BinaryOperator::Subtract => {
                if let Some(result) = try_sub_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result)
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Cannot subtract {} from {}", right_val.type_name(), left_val.type_name()),
                    })
                }
            }
            BinaryOperator::Multiply => {
                if let Some(result) = try_mul_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result)
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Cannot multiply {} and {}", left_val.type_name(), right_val.type_name()),
                    })
                }
            }
            BinaryOperator::Divide => {
                match try_div_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(msg) => Err(RuntimeError::DivisionByZero { message: msg }),
                }
            }
            BinaryOperator::Modulo => {
                match try_mod_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(msg) => Err(RuntimeError::DivisionByZero { message: msg }),
                }
            }
            BinaryOperator::Equal => {
                let result = self.compare_values(&left_val, &right_val);
                Ok(Box::new(BoolBox::new(result)))
            }
            BinaryOperator::NotEqual => {
                let result = !self.compare_values(&left_val, &right_val);
                Ok(Box::new(BoolBox::new(result)))
            }
            BinaryOperator::LessThan => {
                let result = self.less_than_values(&left_val, &right_val)?;
                Ok(Box::new(BoolBox::new(result)))
            }
            BinaryOperator::LessThanOrEqual => {
                let less_than = self.less_than_values(&left_val, &right_val)?;
                let equal = self.compare_values(&left_val, &right_val);
                Ok(Box::new(BoolBox::new(less_than || equal)))
            }
            BinaryOperator::GreaterThan => {
                let less_than_or_equal = self.less_than_values(&left_val, &right_val)? || self.compare_values(&left_val, &right_val);
                Ok(Box::new(BoolBox::new(!less_than_or_equal)))
            }
            BinaryOperator::GreaterThanOrEqual => {
                let less_than = self.less_than_values(&left_val, &right_val)?;
                Ok(Box::new(BoolBox::new(!less_than)))
            }
            BinaryOperator::And => {
                let left_bool = self.to_bool(&left_val);
                if !left_bool {
                    Ok(Box::new(BoolBox::new(false)))
                } else {
                    let right_bool = self.to_bool(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
            BinaryOperator::Or => {
                let left_bool = self.to_bool(&left_val);
                if left_bool {
                    Ok(Box::new(BoolBox::new(true)))
                } else {
                    let right_bool = self.to_bool(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
        }
    }

    /// Execute unary operation on an expression
    pub(super) fn execute_unary_op(&mut self, operator: &UnaryOperator, operand: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let operand_val = self.execute_expression(operand)?;
        
        match operator {
            UnaryOperator::Not => {
                let bool_val = self.to_bool(&operand_val);
                Ok(Box::new(BoolBox::new(!bool_val)))
            }
            UnaryOperator::Minus => {
                if let Some(int_box) = operand_val.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(-int_box.value)))
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Cannot negate {}", operand_val.type_name()),
                    })
                }
            }
        }
    }
}