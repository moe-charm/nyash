/*!
 * Binary and unary operator evaluation
 */

// Removed super::* import - specific imports below
use crate::ast::{ASTNode, BinaryOperator, UnaryOperator};
use crate::box_trait::{NyashBox, IntegerBox, StringBox, BoolBox, CompareBox};
use crate::boxes::FloatBox;
use crate::interpreter::core::{NyashInterpreter, RuntimeError};

// Local helper functions to bypass import issues
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

pub(super) fn try_mul_operation(left: &dyn NyashBox, right: &dyn NyashBox) -> Option<Box<dyn NyashBox>> {
    // IntegerBox * IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value * right_int.value)));
    }
    
    // StringBox * IntegerBox -> repetition
    if let (Some(str_box), Some(count_int)) = (
        left.as_any().downcast_ref::<StringBox>(),
        right.as_any().downcast_ref::<IntegerBox>()
    ) {
        return Some(Box::new(StringBox::new(str_box.value.repeat(count_int.value as usize))));
    }
    
    None
}

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
    
    Err(format!("Division not supported between {} and {}", left.type_name(), right.type_name()))
}

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
    
    Err(format!("Modulo not supported between {} and {}", left.type_name(), right.type_name()))
}

impl NyashInterpreter {
    /// 二項演算を実行 - Binary operation processing
    pub(super) fn execute_binary_op(&mut self, op: &BinaryOperator, left: &ASTNode, right: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let left_val = self.execute_expression(left)?;
        let right_val = self.execute_expression(right)?;
        
        match op {
            BinaryOperator::Add => {
                // 🚀 Direct trait-based operator resolution (temporary workaround)
                // Use helper function instead of trait methods
                if let Some(result) = try_add_operation(left_val.as_ref(), right_val.as_ref()) {
                    return Ok(result);
                }
                
                Err(RuntimeError::InvalidOperation { 
                    message: format!("Addition not supported between {} and {}", 
                                   left_val.type_name(), right_val.type_name()) 
                })
            }
            
            BinaryOperator::Equal => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::NotEqual => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(BoolBox::new(!result.value)))
            }
            
            BinaryOperator::And => {
                let left_bool = self.is_truthy(&left_val);
                if !left_bool {
                    Ok(Box::new(BoolBox::new(false)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
            
            BinaryOperator::Or => {
                let left_bool = self.is_truthy(&left_val);
                if left_bool {
                    Ok(Box::new(BoolBox::new(true)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
            
            BinaryOperator::Subtract => {
                // Use helper function instead of trait methods
                if let Some(result) = try_sub_operation(left_val.as_ref(), right_val.as_ref()) {
                    return Ok(result);
                }
                
                Err(RuntimeError::InvalidOperation { 
                    message: format!("Subtraction not supported between {} and {}", 
                                   left_val.type_name(), right_val.type_name()) 
                })
            }
            
            BinaryOperator::Multiply => {
                // Use helper function instead of trait methods
                if let Some(result) = try_mul_operation(left_val.as_ref(), right_val.as_ref()) {
                    return Ok(result);
                }
                
                Err(RuntimeError::InvalidOperation { 
                    message: format!("Multiplication not supported between {} and {}", 
                                   left_val.type_name(), right_val.type_name()) 
                })
            }
            
            BinaryOperator::Divide => {
                // Use helper function instead of trait methods
                match try_div_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(error_msg) => Err(RuntimeError::InvalidOperation { 
                        message: error_msg 
                    })
                }
            }
            
            BinaryOperator::Modulo => {
                // Use helper function for modulo operation
                match try_mod_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(error_msg) => Err(RuntimeError::InvalidOperation { 
                        message: error_msg 
                    })
                }
            }
            
            BinaryOperator::Less => {
                let result = CompareBox::less(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::Greater => {
                let result = CompareBox::greater(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::LessEqual => {
                let result = CompareBox::less_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::GreaterEqual => {
                let result = CompareBox::greater_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
        }
    }
    
    /// 単項演算を実行 - Unary operation processing
    pub(super) fn execute_unary_op(&mut self, operator: &UnaryOperator, operand: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let operand_val = self.execute_expression(operand)?;
        
        match operator {
            UnaryOperator::Minus => {
                // 数値の符号反転
                if let Some(int_box) = operand_val.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(-int_box.value)))
                } else if let Some(float_box) = operand_val.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(-float_box.value)))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "Unary minus can only be applied to Integer or Float".to_string(),
                    })
                }
            }
            UnaryOperator::Not => {
                // 論理否定
                if let Some(bool_box) = operand_val.as_any().downcast_ref::<BoolBox>() {
                    Ok(Box::new(BoolBox::new(!bool_box.value)))
                } else {
                    // どんな値でもtruthyness判定してnot演算を適用
                    let is_truthy = self.is_truthy(&operand_val);
                    Ok(Box::new(BoolBox::new(!is_truthy)))
                }
            }
        }
    }
}