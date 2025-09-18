/*!
 * Operators Processing Module
 * 
 * Extracted from expressions.rs
 * Handles binary operations, unary operations, and operator helper functions
 * Core philosophy: "Everything is Box" with type-safe operator overloading
 */

use super::*;
use crate::ast::UnaryOperator;
use crate::box_trait::{BoolBox, SharedNyashBox};
use crate::operator_traits::{DynamicAdd, DynamicSub, DynamicMul, DynamicDiv, OperatorError};

// ========================================================================================
// Helper Functions for Binary Operations
// ========================================================================================

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

// ========================================================================================
// NyashInterpreter Implementation - Binary and Unary Operations
// ========================================================================================

impl NyashInterpreter {
    /// Try to extract a UTF-8 string from a NyashBox: supports internal StringBox,
    /// Result.Ok(String-like), and Plugin StringBox via toUtf8 (when plugins enabled).
    fn try_box_to_string(&self, b: &dyn NyashBox) -> Option<String> {
        // Internal StringBox
        if let Some(sb) = b.as_any().downcast_ref::<StringBox>() { return Some(sb.value.clone()); }
        // Result.Ok(inner) → recurse
        if let Some(res) = b.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
            if let crate::boxes::result::NyashResultBox::Ok(inner) = res { return self.try_box_to_string(inner.as_ref()); }
        }
        // Plugin StringBox via toUtf8
        #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
        {
            if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                if pb.box_type == "StringBox" {
                    let host = crate::runtime::get_global_plugin_host();
                    if let Ok(ro) = host.read() {
                        if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", pb.inner.instance_id, &[]) {
                            if let Some(vb) = val_opt {
                                if let Some(sb2) = vb.as_any().downcast_ref::<StringBox>() { return Some(sb2.value.clone()); }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    /// 二項演算を実行
    pub(super) fn execute_binary_op(&mut self, op: &BinaryOperator, left: &ASTNode, right: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> 
    {
        // 🎯 State-sharing evaluation for performance
        let left_shared = self.execute_expression_shared(left)?;
        let right_shared = self.execute_expression_shared(right)?;
        let left_val = &**left_shared;
        let right_val = &**right_shared;
        
        match op {
            BinaryOperator::Add => {
                // Prefer string-like concatenation when either side is string-like
                if let (Some(ls), Some(rs)) = (self.try_box_to_string(left_val), self.try_box_to_string(right_val)) {
                    return Ok(Box::new(StringBox::new(format!("{}{}", ls, rs))));
                }
                if let Some(ls) = self.try_box_to_string(left_val) {
                    return Ok(Box::new(StringBox::new(format!("{}{}", ls, right_val.to_string_box().value))));
                }
                if let Some(rs) = self.try_box_to_string(right_val) {
                    return Ok(Box::new(StringBox::new(format!("{}{}", left_val.to_string_box().value, rs))));
                }
                // Numeric fallback: if both sides stringify to valid integers, add them
                {
                    let ls = left_val.to_string_box().value;
                    let rs = right_val.to_string_box().value;
                    if let (Ok(li), Ok(ri)) = (ls.trim().parse::<i64>(), rs.trim().parse::<i64>()) {
                        return Ok(Box::new(IntegerBox::new(li + ri)));
                    }
                }
                if let Some(result) = try_add_operation(left_val, right_val) {
                    Ok(result)
                } else {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot add {} and {}", left_val.type_name(), right_val.type_name()),
                    })
                }
            },
            
            BinaryOperator::Subtract => {
                if let Some(result) = try_sub_operation(left_val, right_val) {
                    Ok(result)
                } else {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot subtract {} from {}", right_val.type_name(), left_val.type_name()),
                    })
                }
            },
            
            BinaryOperator::Multiply => {
                if let Some(result) = try_mul_operation(left_val, right_val) {
                    Ok(result)
                } else {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot multiply {} and {}", left_val.type_name(), right_val.type_name()),
                    })
                }
            },
            
            BinaryOperator::Divide => {
                match try_div_operation(left_val, right_val) {
                    Ok(result) => Ok(result),
                    Err(msg) => Err(RuntimeError::InvalidOperation { message: msg }),
                }
            },
            
            BinaryOperator::Modulo => {
                match try_mod_operation(left_val, right_val) {
                    Ok(result) => Ok(result),
                    Err(msg) => Err(RuntimeError::InvalidOperation { message: msg }),
                }
            },
            
            BinaryOperator::Equal => {
                let result = self.compare_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(result)))
            },
            
            BinaryOperator::NotEqual => {
                let result = self.compare_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(!result)))
            },
            
            BinaryOperator::LessThan => {
                let result = self.less_than_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(result)))
            },
            
            BinaryOperator::LessThanOrEqual => {
                let less = self.less_than_values(left_val, right_val)?;
                let equal = self.compare_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(less || equal)))
            },
            
            BinaryOperator::GreaterThan => {
                let less = self.less_than_values(left_val, right_val)?;
                let equal = self.compare_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(!less && !equal)))
            },
            
            BinaryOperator::GreaterThanOrEqual => {
                let less = self.less_than_values(left_val, right_val)?;
                Ok(Box::new(BoolBox::new(!less)))
            },
            
            BinaryOperator::And => {
                // Short-circuit evaluation
                if !self.is_truthy(left_val) {
                    Ok(Box::new(BoolBox::new(false)))
                } else {
                    Ok(Box::new(BoolBox::new(self.is_truthy(right_val))))
                }
            },
            
            BinaryOperator::Or => {
                // Short-circuit evaluation
                if self.is_truthy(left_val) {
                    Ok(Box::new(BoolBox::new(true)))
                } else {
                    Ok(Box::new(BoolBox::new(self.is_truthy(right_val))))
                }
            },
            BinaryOperator::Shl => {
                if let (Some(li), Some(ri)) = (
                    left_val.as_any().downcast_ref::<IntegerBox>(),
                    right_val.as_any().downcast_ref::<IntegerBox>(),
                ) {
                    Ok(Box::new(IntegerBox::new(li.value.wrapping_shl((ri.value as u32) & 63))))
                } else {
                    Err(RuntimeError::InvalidOperation { message: format!("Shift-left '<<' requires integers (got {} and {})", left_val.type_name(), right_val.type_name()) })
                }
            },
            BinaryOperator::Shr => {
                if let (Some(li), Some(ri)) = (
                    left_val.as_any().downcast_ref::<IntegerBox>(),
                    right_val.as_any().downcast_ref::<IntegerBox>(),
                ) {
                    Ok(Box::new(IntegerBox::new(((li.value as u64) >> ((ri.value as u32) & 63)) as i64)))
                } else {
                    Err(RuntimeError::InvalidOperation { message: format!("Shift-right '>>' requires integers (got {} and {})", left_val.type_name(), right_val.type_name()) })
                }
            },
            BinaryOperator::BitAnd => {
                if let (Some(li), Some(ri)) = (
                    left_val.as_any().downcast_ref::<IntegerBox>(),
                    right_val.as_any().downcast_ref::<IntegerBox>(),
                ) { Ok(Box::new(IntegerBox::new(li.value & ri.value))) } else { Err(RuntimeError::InvalidOperation { message: format!("Bitwise '&' requires integers (got {} and {})", left_val.type_name(), right_val.type_name()) }) }
            },
            BinaryOperator::BitOr => {
                if let (Some(li), Some(ri)) = (
                    left_val.as_any().downcast_ref::<IntegerBox>(),
                    right_val.as_any().downcast_ref::<IntegerBox>(),
                ) { Ok(Box::new(IntegerBox::new(li.value | ri.value))) } else { Err(RuntimeError::InvalidOperation { message: format!("Bitwise '|' requires integers (got {} and {})", left_val.type_name(), right_val.type_name()) }) }
            },
            BinaryOperator::BitXor => {
                if let (Some(li), Some(ri)) = (
                    left_val.as_any().downcast_ref::<IntegerBox>(),
                    right_val.as_any().downcast_ref::<IntegerBox>(),
                ) { Ok(Box::new(IntegerBox::new(li.value ^ ri.value))) } else { Err(RuntimeError::InvalidOperation { message: format!("Bitwise '^' requires integers (got {} and {})", left_val.type_name(), right_val.type_name()) }) }
            },
        }
    }

    /// 単項演算を実行
    pub(super) fn execute_unary_op(&mut self, operator: &UnaryOperator, operand: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> 
    {
        let operand_shared = self.execute_expression_shared(operand)?;
        let operand_val = &**operand_shared;
        
        match operator {
            UnaryOperator::Not => {
                let is_truthy = self.is_truthy(operand_val);
                Ok(Box::new(BoolBox::new(!is_truthy)))
            },
            UnaryOperator::Minus => {
                if let Some(int_val) = operand_val.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(-int_val.value)))
                } else {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot negate {}", operand_val.type_name()),
                    })
                }
            },
        }
    }

    // ========================================================================================
    // Helper Methods for Comparisons
    // ========================================================================================

    /// 値の等価性を比較
    pub(super) fn compare_values(&self, left: &dyn NyashBox, right: &dyn NyashBox) -> Result<bool, RuntimeError> {
        // IntegerBox comparison
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>()
        ) {
            return Ok(left_int.value == right_int.value);
        }
        
        // String-like comparison (internal/Result.Ok/plugin StringBox)
        if let (Some(ls), Some(rs)) = (self.try_box_to_string(left), self.try_box_to_string(right)) {
            return Ok(ls == rs);
        }
        
        // BoolBox comparison
        if let (Some(left_bool), Some(right_bool)) = (
            left.as_any().downcast_ref::<BoolBox>(),
            right.as_any().downcast_ref::<BoolBox>()
        ) {
            return Ok(left_bool.value == right_bool.value);
        }
        
        // NullBox comparison
        if left.type_name() == "NullBox" && right.type_name() == "NullBox" {
            return Ok(true);
        }
        
        // Different types are not equal
        Ok(false)
    }

    /// 値の大小関係を比較 (left < right)
    pub(super) fn less_than_values(&self, left: &dyn NyashBox, right: &dyn NyashBox) -> Result<bool, RuntimeError> {
        // IntegerBox comparison
        if let (Some(left_int), Some(right_int)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>()
        ) {
            return Ok(left_int.value < right_int.value);
        }
        
        // String-like comparison (lexicographic)
        if let (Some(ls), Some(rs)) = (self.try_box_to_string(left), self.try_box_to_string(right)) {
            return Ok(ls < rs);
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Cannot compare {} and {}", left.type_name(), right.type_name()),
        })
    }

    /// 値の真偽性を判定
    pub(super) fn is_truthy(&self, value: &dyn NyashBox) -> bool {
        // BoolBox
        if let Some(bool_val) = value.as_any().downcast_ref::<BoolBox>() {
            return bool_val.value;
        }
        
        // IntegerBox (0 is false, non-zero is true)
        if let Some(int_val) = value.as_any().downcast_ref::<IntegerBox>() {
            return int_val.value != 0;
        }
        
        // StringBox (empty string is false)
        if let Some(str_val) = value.as_any().downcast_ref::<StringBox>() {
            return !str_val.value.is_empty();
        }
        
        // NullBox is always false
        if value.type_name() == "NullBox" {
            return false;
        }
        
        // Everything else is true
        true
    }
}
