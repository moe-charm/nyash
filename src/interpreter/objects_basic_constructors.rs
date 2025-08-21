//! Basic type constructors for execute_new
//! Handles StringBox, IntegerBox, BoolBox, ArrayBox, etc.

use crate::ast::ASTNode;
use crate::box_trait::*;
use crate::interpreter::core::{NyashInterpreter as Interpreter, RuntimeError};
use crate::boxes::FloatBox;
use crate::boxes::null_box::NullBox;
use crate::boxes::map_box::MapBox;

impl Interpreter {
    /// Create basic type boxes (StringBox, IntegerBox, BoolBox, etc.)
    pub(super) fn create_basic_box(
        &mut self, 
        class: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match class {
            "StringBox" => {
                // StringBoxは引数1個（文字列値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StringBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let value = self.execute_expression(&arguments[0])?;
                if let Some(s) = value.as_any().downcast_ref::<StringBox>() {
                    return Ok(Box::new(StringBox::new(s.value.clone())));
                } else if let Some(i) = value.as_any().downcast_ref::<IntegerBox>() {
                    return Ok(Box::new(StringBox::new(i.value.to_string())));
                } else if let Some(b) = value.as_any().downcast_ref::<BoolBox>() {
                    return Ok(Box::new(StringBox::new(b.value.to_string())));
                } else {
                    return Ok(Box::new(StringBox::new(value.to_string_box().value)));
                }
            }
            
            "IntegerBox" => {
                // IntegerBoxは引数1個（整数値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntegerBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let value = self.execute_expression(&arguments[0])?;
                if let Some(i) = value.as_any().downcast_ref::<IntegerBox>() {
                    return Ok(Box::new(IntegerBox::new(i.value)));
                } else if let Some(s) = value.as_any().downcast_ref::<StringBox>() {
                    match s.value.parse::<i64>() {
                        Ok(n) => return Ok(Box::new(IntegerBox::new(n))),
                        Err(_) => return Err(RuntimeError::TypeError {
                            message: format!("Cannot convert '{}' to integer", s.value),
                        }),
                    }
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "IntegerBox constructor requires integer or string argument".to_string(),
                    });
                }
            }
            
            "BoolBox" => {
                // BoolBoxは引数1個（ブール値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("BoolBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let value = self.execute_expression(&arguments[0])?;
                if let Some(b) = value.as_any().downcast_ref::<BoolBox>() {
                    return Ok(Box::new(BoolBox::new(b.value)));
                } else if let Some(s) = value.as_any().downcast_ref::<StringBox>() {
                    let val = match s.value.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => return Err(RuntimeError::TypeError {
                            message: format!("Cannot convert '{}' to boolean", s.value),
                        }),
                    };
                    return Ok(Box::new(BoolBox::new(val)));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "BoolBox constructor requires boolean or string argument".to_string(),
                    });
                }
            }
            
            "ArrayBox" => {
                // ArrayBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ArrayBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                return Ok(Box::new(ArrayBox::new()));
            }
            
            "NullBox" => {
                // NullBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("NullBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                return Ok(Box::new(NullBox::new()));
            }
            
            "MapBox" => {
                // MapBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MapBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let map_box = Box::new(MapBox::new()) as Box<dyn NyashBox>;
                return Ok(map_box);
            }
            
            "FloatBox" => {
                // FloatBoxは引数1個（浮動小数点数値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("FloatBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let value = self.execute_expression(&arguments[0])?;
                if let Some(f) = value.as_any().downcast_ref::<FloatBox>() {
                    return Ok(Box::new(FloatBox::new(f.value)));
                } else if let Some(i) = value.as_any().downcast_ref::<IntegerBox>() {
                    return Ok(Box::new(FloatBox::new(i.value as f64)));
                } else if let Some(s) = value.as_any().downcast_ref::<StringBox>() {
                    match s.value.parse::<f64>() {
                        Ok(n) => return Ok(Box::new(FloatBox::new(n))),
                        Err(_) => return Err(RuntimeError::TypeError {
                            message: format!("Cannot convert '{}' to float", s.value),
                        }),
                    }
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "FloatBox constructor requires float, integer, or string argument".to_string(),
                    });
                }
            }
            
            _ => {
                // Not a basic type
                Err(RuntimeError::TypeError {
                    message: format!("Not a basic type: {}", class),
                })
            }
        }
    }
}
