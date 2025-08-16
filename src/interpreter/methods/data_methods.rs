/*!
 * Data Processing Box Methods Module
 * 
 * Contains method implementations for data processing Box types:
 * - BufferBox (execute_buffer_method) - Binary data operations
 * - JSONBox (execute_json_method) - JSON parsing and manipulation
 * - RegexBox (execute_regex_method) - Regular expression operations
 */

use super::super::*;
use crate::box_trait::NyashBox;
use crate::boxes::{buffer::BufferBox, JSONBox, RegexBox};

impl NyashInterpreter {
    /// BufferBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_buffer_method(&mut self, buffer_box: &BufferBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let data = self.execute_expression(&arguments[0])?;
                Ok(buffer_box.write(data))
            }
            "readAll" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("readAll() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(buffer_box.readAll())
            }
            "read" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let count = self.execute_expression(&arguments[0])?;
                Ok(buffer_box.read(count))
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(buffer_box.clear())
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(buffer_box.length())
            }
            "append" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("append() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other = self.execute_expression(&arguments[0])?;
                Ok(buffer_box.append(other))
            }
            "slice" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("slice() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let start = self.execute_expression(&arguments[0])?;
                let end = self.execute_expression(&arguments[1])?;
                Ok(buffer_box.slice(start, end))
            }
            // ⭐ Phase 10: Zero-copy detection APIs
            "is_shared_with" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("is_shared_with() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other = self.execute_expression(&arguments[0])?;
                Ok(buffer_box.is_shared_with(other))
            }
            "share_reference" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("share_reference() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let data = self.execute_expression(&arguments[0])?;
                Ok(buffer_box.share_reference(data))
            }
            "memory_footprint" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("memory_footprint() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(buffer_box.memory_footprint())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for BufferBox", method),
            })
        }
    }

    /// JSONBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_json_method(&mut self, json_box: &JSONBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "parse" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("parse() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let data = self.execute_expression(&arguments[0])?;
                Ok(JSONBox::parse(data))
            }
            "stringify" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("stringify() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(json_box.stringify())
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key = self.execute_expression(&arguments[0])?;
                Ok(json_box.get(key))
            }
            "set" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("set() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let key = self.execute_expression(&arguments[0])?;
                let value = self.execute_expression(&arguments[1])?;
                Ok(json_box.set(key, value))
            }
            "has" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("has() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let key = self.execute_expression(&arguments[0])?;
                Ok(json_box.has(key))
            }
            "keys" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("keys() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(json_box.keys())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for JSONBox", method),
            })
        }
    }

    /// RegexBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_regex_method(&mut self, regex_box: &RegexBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "test" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("test() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let text = self.execute_expression(&arguments[0])?;
                Ok(regex_box.test(text))
            }
            "find" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("find() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let text = self.execute_expression(&arguments[0])?;
                Ok(regex_box.find(text))
            }
            "findAll" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("findAll() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let text = self.execute_expression(&arguments[0])?;
                Ok(regex_box.find_all(text))
            }
            "replace" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("replace() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let text = self.execute_expression(&arguments[0])?;
                let replacement = self.execute_expression(&arguments[1])?;
                Ok(regex_box.replace(text, replacement))
            }
            "split" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("split() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let text = self.execute_expression(&arguments[0])?;
                Ok(regex_box.split(text))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for RegexBox", method),
            })
        }
    }
}