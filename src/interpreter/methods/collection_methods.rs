/*!
 * Collection Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains method implementations for collection types:
 * - ArrayBox (execute_array_method)
 * - MapBox (execute_map_method)
 */

use super::super::*;
use crate::box_trait::{StringBox, IntegerBox, NyashBox, BoolBox};
use crate::boxes::{ArrayBox, MapBox};

impl NyashInterpreter {
    /// ArrayBoxのメソッド呼び出しを実行  
    pub(in crate::interpreter) fn execute_array_method(&mut self, array_box: &ArrayBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "push" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("push() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let element = self.execute_expression(&arguments[0])?;
                Ok(array_box.push(element))
            }
            "pop" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pop() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.pop())
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.length())
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                Ok(array_box.get(index_value))
            }
            "set" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("set() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                let element_value = self.execute_expression(&arguments[1])?;
                Ok(array_box.set(index_value, element_value))
            }
            "remove" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("remove() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                Ok(array_box.remove(index_value))
            }
            "indexOf" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("indexOf() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let element = self.execute_expression(&arguments[0])?;
                Ok(array_box.indexOf(element))
            }
            "contains" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("contains() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let element = self.execute_expression(&arguments[0])?;
                Ok(array_box.contains(element))
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.clear())
            }
            "join" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("join() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let delimiter_value = self.execute_expression(&arguments[0])?;
                Ok(array_box.join(delimiter_value))
            }
            "isEmpty" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isEmpty() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let length = array_box.length();
                if let Some(int_box) = length.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(BoolBox::new(int_box.value == 0)))
                } else {
                    Ok(Box::new(BoolBox::new(false)))
                }
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(array_box.to_string_box()))
            }
            "sort" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sort() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.sort())
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("reverse() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.reverse())
            }
            "slice" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("slice() expects 2 arguments (start, end), got {}", arguments.len()),
                    });
                }
                let start_value = self.execute_expression(&arguments[0])?;
                let end_value = self.execute_expression(&arguments[1])?;
                Ok(array_box.slice(start_value, end_value))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for ArrayBox", method),
                })
            }
        }
    }

    /// MapBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_map_method(&mut self, map_box: &MapBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // メソッドを実行（必要時評価方式）
        match method {
            "set" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("set() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let key_value = self.execute_expression(&arguments[0])?;
                let value_value = self.execute_expression(&arguments[1])?;
                Ok(map_box.set(key_value, value_value))
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key_value = self.execute_expression(&arguments[0])?;
                Ok(map_box.get(key_value))
            }
            "has" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("has() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key_value = self.execute_expression(&arguments[0])?;
                Ok(map_box.has(key_value))
            }
            "delete" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key_value = self.execute_expression(&arguments[0])?;
                Ok(map_box.delete(key_value))
            }
            "keys" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("keys() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(map_box.keys())
            }
            "values" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("values() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(map_box.values())
            }
            "size" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("size() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(map_box.size())
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(map_box.clear())
            }
            "isEmpty" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isEmpty() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let size = map_box.size();
                if let Some(int_box) = size.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(BoolBox::new(int_box.value == 0)))
                } else {
                    Ok(Box::new(BoolBox::new(false)))
                }
            }
            "containsKey" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("containsKey() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key_value = self.execute_expression(&arguments[0])?;
                Ok(map_box.has(key_value))
            }
            "containsValue" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("containsValue() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let _value = self.execute_expression(&arguments[0])?;
                // Simple implementation: check if any value equals the given value
                Ok(Box::new(BoolBox::new(false))) // TODO: implement proper value search
            }
            "forEach" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("forEach() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let callback_value = self.execute_expression(&arguments[0])?;
                Ok(map_box.forEach(callback_value))
            }
            "toJSON" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toJSON() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(map_box.toJSON())
            }
            // Note: merge, filter, map methods not implemented in MapBox yet
            // These would require more complex callback handling
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(map_box.to_string_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown MapBox method: {}", method),
                })
            }
        }
    }
}