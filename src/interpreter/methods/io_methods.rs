/*!
 * I/O Operations Box Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains method implementations for I/O and error handling operations:
 * - FileBox (execute_file_method) - File I/O operations
 * - ResultBox (execute_result_method) - Error handling and result operations
 */

use super::super::*;
use crate::box_trait::{FileBox, ResultBox, StringBox, NyashBox};

impl NyashInterpreter {
    /// FileBoxのメソッド呼び出しを実行
    /// Handles file I/O operations including read, write, exists, delete, and copy
    pub(in crate::interpreter) fn execute_file_method(&mut self, file_box: &FileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "read" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.read())
            }
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let content = self.execute_expression(&arguments[0])?;
                Ok(file_box.write(content))
            }
            "exists" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("exists() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.exists())
            }
            "delete" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.delete())
            }
            "copy" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("copy() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let dest_value = self.execute_expression(&arguments[0])?;
                if let Some(dest_str) = dest_value.as_any().downcast_ref::<StringBox>() {
                    Ok(file_box.copy(&dest_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "copy() requires string destination path".to_string(),
                    })
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for FileBox", method),
            })
        }
    }

    /// ResultBoxのメソッド呼び出しを実行
    /// Handles result/error checking operations for error handling patterns
    pub(in crate::interpreter) fn execute_result_method(&mut self, result_box: &ResultBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "isOk" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isOk() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.is_ok())
            }
            "getValue" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getValue() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_value())
            }
            "getError" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getError() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_error())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for ResultBox", method),
            })
        }
    }
}