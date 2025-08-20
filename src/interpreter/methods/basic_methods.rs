/*!
 * Basic Box Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains method implementations for:
 * - StringBox (execute_string_method)
 * - IntegerBox (execute_integer_method) 
 * - BoolBox (execute_bool_method)
 * - FloatBox (execute_float_method)
 */

use super::super::*;
use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
use crate::boxes::FloatBox;

impl NyashInterpreter {
    /// StringBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_string_method(&mut self, string_box: &StringBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "split" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("split() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let delimiter_value = self.execute_expression(&arguments[0])?;
                if let Some(delimiter_str) = delimiter_value.as_any().downcast_ref::<StringBox>() {
                    Ok(string_box.split(&delimiter_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "split() requires string delimiter".to_string(),
                    })
                }
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                // StringBoxは自分自身を返す
                Ok(Box::new(string_box.clone()))
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.length())
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                if let Some(index_int) = index_value.as_any().downcast_ref::<IntegerBox>() {
                    match string_box.get(index_int.value as usize) {
                        Some(char_box) => Ok(char_box),
                        None => Ok(Box::new(VoidBox::new())),
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "get() requires integer index".to_string(),
                    })
                }
            }
            "find" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("find() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let search_value = self.execute_expression(&arguments[0])?;
                if let Some(search_str) = search_value.as_any().downcast_ref::<StringBox>() {
                    Ok(string_box.find(&search_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "find() requires string argument".to_string(),
                    })
                }
            }
            "replace" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("replace() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let old_value = self.execute_expression(&arguments[0])?;
                let new_value = self.execute_expression(&arguments[1])?;
                if let (Some(old_str), Some(new_str)) = (
                    old_value.as_any().downcast_ref::<StringBox>(),
                    new_value.as_any().downcast_ref::<StringBox>()
                ) {
                    Ok(string_box.replace(&old_str.value, &new_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "replace() requires string arguments".to_string(),
                    })
                }
            }
            "trim" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("trim() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.trim())
            }
            "toUpper" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toUpper() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.to_upper())
            }
            "toLower" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toLower() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.to_lower())
            }
            "toInteger" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toInteger() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.to_integer())
            }
            "substring" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("substring() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let start = self.execute_expression(&arguments[0])?;
                let end = self.execute_expression(&arguments[1])?;
                
                // Convert arguments to integers
                let start_int = if let Some(int_box) = start.as_any().downcast_ref::<IntegerBox>() {
                    int_box.value as usize
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "substring() expects integer arguments".to_string(),
                    });
                };
                
                let end_int = if let Some(int_box) = end.as_any().downcast_ref::<IntegerBox>() {
                    int_box.value as usize
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "substring() expects integer arguments".to_string(),
                    });
                };
                
                Ok(string_box.substring(start_int, end_int))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for StringBox", method),
                })
            }
        }
    }

    /// IntegerBoxのメソッド呼び出しを実行  
    pub(in crate::interpreter) fn execute_integer_method(&mut self, integer_box: &IntegerBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(integer_box.value.to_string())))
            }
            "abs" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("abs() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(integer_box.value.abs())))
            }
            "max" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("max() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(integer_box.value.max(other_int.value))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "max() requires integer argument".to_string(),
                    })
                }
            }
            "min" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("min() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(integer_box.value.min(other_int.value))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "min() requires integer argument".to_string(),
                    })
                }
            }
            "toFloat" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toFloat() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(integer_box.value as f64)))
            }
            "pow" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pow() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let exponent_value = self.execute_expression(&arguments[0])?;
                if let Some(exponent_int) = exponent_value.as_any().downcast_ref::<IntegerBox>() {
                    if exponent_int.value >= 0 {
                        let result = (integer_box.value as f64).powf(exponent_int.value as f64);
                        Ok(Box::new(FloatBox::new(result)))
                    } else {
                        let result = (integer_box.value as f64).powf(exponent_int.value as f64);
                        Ok(Box::new(FloatBox::new(result)))
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "pow() requires integer exponent".to_string(),
                    })
                }
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for IntegerBox", method),
                })
            }
        }
    }

    /// BoolBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_bool_method(&mut self, bool_box: &BoolBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(bool_box.to_string_box()))
            }
            "not" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("not() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(!bool_box.value)))
            }
            "and" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("and() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_bool) = other_value.as_any().downcast_ref::<BoolBox>() {
                    Ok(Box::new(BoolBox::new(bool_box.value && other_bool.value)))
                } else {
                    // Support truthiness evaluation for non-boolean types
                    let is_truthy = self.is_truthy(&other_value);
                    Ok(Box::new(BoolBox::new(bool_box.value && is_truthy)))
                }
            }
            "or" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("or() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_bool) = other_value.as_any().downcast_ref::<BoolBox>() {
                    Ok(Box::new(BoolBox::new(bool_box.value || other_bool.value)))
                } else {
                    // Support truthiness evaluation for non-boolean types
                    let is_truthy = self.is_truthy(&other_value);
                    Ok(Box::new(BoolBox::new(bool_box.value || is_truthy)))
                }
            }
            "equals" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                Ok(Box::new(bool_box.equals(&*other_value)))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for BoolBox", method),
                })
            }
        }
    }

    /// FloatBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_float_method(&mut self, float_box: &FloatBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(float_box.value.to_string())))
            }
            "abs" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("abs() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(float_box.value.abs())))
            }
            "floor" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("floor() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(float_box.value.floor() as i64)))
            }
            "ceil" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ceil() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(float_box.value.ceil() as i64)))
            }
            "round" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("round() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(float_box.value.round() as i64)))
            }
            "toInteger" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toInteger() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(float_box.value as i64)))
            }
            "max" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("max() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_float) = other_value.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.max(other_float.value))))
                } else if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.max(other_int.value as f64))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "max() requires numeric argument".to_string(),
                    })
                }
            }
            "min" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("min() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_float) = other_value.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.min(other_float.value))))
                } else if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.min(other_int.value as f64))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "min() requires numeric argument".to_string(),
                    })
                }
            }
            "pow" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pow() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let exponent_value = self.execute_expression(&arguments[0])?;
                if let Some(exponent_float) = exponent_value.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.powf(exponent_float.value))))
                } else if let Some(exponent_int) = exponent_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(FloatBox::new(float_box.value.powf(exponent_int.value as f64))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "pow() requires numeric exponent".to_string(),
                    })
                }
            }
            "sqrt" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sqrt() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                if float_box.value < 0.0 {
                    Err(RuntimeError::InvalidOperation {
                        message: "Cannot take square root of negative number".to_string(),
                    })
                } else {
                    Ok(Box::new(FloatBox::new(float_box.value.sqrt())))
                }
            }
            "sin" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sin() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(float_box.value.sin())))
            }
            "cos" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("cos() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(float_box.value.cos())))
            }
            "tan" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("tan() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(float_box.value.tan())))
            }
            "log" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("log() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                if float_box.value <= 0.0 {
                    Err(RuntimeError::InvalidOperation {
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    })
                } else {
                    Ok(Box::new(FloatBox::new(float_box.value.ln())))
                }
            }
            "log10" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("log10() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                if float_box.value <= 0.0 {
                    Err(RuntimeError::InvalidOperation {
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    })
                } else {
                    Ok(Box::new(FloatBox::new(float_box.value.log10())))
                }
            }
            "exp" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("exp() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(FloatBox::new(float_box.value.exp())))
            }
            "isNaN" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isNaN() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(float_box.value.is_nan())))
            }
            "isInfinite" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isInfinite() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(float_box.value.is_infinite())))
            }
            "isFinite" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isFinite() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(float_box.value.is_finite())))
            }
            "equals" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                Ok(Box::new(float_box.equals(&*other_value)))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for FloatBox", method),
                })
            }
        }
    }
}