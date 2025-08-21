//! Non-basic type constructors for execute_new
//! Handles MathBox, ConsoleBox, GUI boxes, Network boxes, etc.

use crate::ast::ASTNode;
use crate::box_trait::*;
use crate::interpreter::core::{NyashInterpreter as Interpreter, RuntimeError};
use crate::boxes::math_box::MathBox;
use crate::boxes::random_box::RandomBox;
use crate::boxes::sound_box::SoundBox;
use crate::boxes::debug_box::DebugBox;

impl Interpreter {
    /// Create non-basic type boxes (MathBox, ConsoleBox, GUI/Network boxes, etc.)
    pub(super) fn create_non_basic_box(
        &mut self, 
        class: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match class {
            "MathBox" => {
                // MathBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MathBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let math_box = Box::new(MathBox::new()) as Box<dyn NyashBox>;
                return Ok(math_box);
            }
            
            "ConsoleBox" => {
                // ConsoleBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ConsoleBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let console_box = Box::new(crate::box_trait::ConsoleBox::new()) as Box<dyn NyashBox>;
                return Ok(console_box);
            }
            
            "RandomBox" => {
                // RandomBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("RandomBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let random_box = Box::new(RandomBox::new()) as Box<dyn NyashBox>;
                return Ok(random_box);
            }
            
            "SoundBox" => {
                // SoundBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("SoundBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let sound_box = Box::new(SoundBox::new()) as Box<dyn NyashBox>;
                return Ok(sound_box);
            }
            
            "DebugBox" => {
                // DebugBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let debug_box = Box::new(DebugBox::new()) as Box<dyn NyashBox>;
                return Ok(debug_box);
            }
            
            _ => {
                // Not a non-basic type handled here
                Err(RuntimeError::TypeError {
                    message: format!("Not a non-basic type handled in this method: {}", class),
                })
            }
        }
    }
}