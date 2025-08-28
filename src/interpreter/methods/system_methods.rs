/*!
 * System Methods Module
 * 
 * Contains system-level Box type method implementations:
 * - GcConfigBox: Garbage collector configuration
 * - DebugConfigBox: Debug and observability configuration
 */

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, VoidBox, StringBox, BoolBox};
use crate::boxes::gc_config_box::GcConfigBox;
use crate::boxes::debug_config_box::DebugConfigBox;
use crate::interpreter::{NyashInterpreter, RuntimeError};

impl NyashInterpreter {
    /// Execute GcConfigBox methods
    pub(crate) fn execute_gc_config_method(
        &mut self,
        gc_box: &GcConfigBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "setFlag" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("GcConfigBox.setFlag expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let on = self.execute_expression(&arguments[1])?;
                
                let name_str = name.to_string_box().value;
                let on_bool = if let Some(b) = on.as_any().downcast_ref::<BoolBox>() {
                    b.value
                } else {
                    on.to_string_box().value.to_lowercase() == "true"
                };
                
                let mut gc_clone = gc_box.clone();
                gc_clone.set_flag(&name_str, on_bool);
                Ok(Box::new(gc_clone))
            }
            
            "getFlag" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("GcConfigBox.getFlag expects 1 argument, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let name_str = name.to_string_box().value;
                Ok(gc_box.get_flag(&name_str))
            }
            
            "apply" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("GcConfigBox.apply expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(gc_box.apply())
            }
            
            "summary" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("GcConfigBox.summary expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(gc_box.summary())
            }
            
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("GcConfigBox has no method '{}'", method),
            }),
        }
    }
    
    /// Execute DebugConfigBox methods
    pub(crate) fn execute_debug_config_method(
        &mut self,
        debug_box: &DebugConfigBox,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "setFlag" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.setFlag expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let on = self.execute_expression(&arguments[1])?;
                
                let name_str = name.to_string_box().value;
                let on_bool = if let Some(b) = on.as_any().downcast_ref::<BoolBox>() {
                    b.value
                } else {
                    on.to_string_box().value.to_lowercase() == "true"
                };
                
                let mut debug_clone = debug_box.clone();
                debug_clone.set_flag(&name_str, on_bool);
                Ok(Box::new(debug_clone))
            }
            
            "setPath" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.setPath expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let path = self.execute_expression(&arguments[1])?;
                
                let name_str = name.to_string_box().value;
                let path_str = path.to_string_box().value;
                
                let mut debug_clone = debug_box.clone();
                debug_clone.set_path(&name_str, &path_str);
                Ok(Box::new(debug_clone))
            }
            
            "getFlag" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.getFlag expects 1 argument, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let name_str = name.to_string_box().value;
                Ok(debug_box.get_flag(&name_str))
            }
            
            "getPath" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.getPath expects 1 argument, got {}", arguments.len()),
                    });
                }
                let name = self.execute_expression(&arguments[0])?;
                let name_str = name.to_string_box().value;
                Ok(debug_box.get_path(&name_str))
            }
            
            "apply" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.apply expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(debug_box.apply())
            }
            
            "summary" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugConfigBox.summary expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(debug_box.summary())
            }
            
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("DebugConfigBox has no method '{}'", method),
            }),
        }
    }
}