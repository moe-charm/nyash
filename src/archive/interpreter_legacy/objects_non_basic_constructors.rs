//! Non-basic type constructors for execute_new
//! Handles MathBox, ConsoleBox, GUI boxes, Network boxes, etc.

use crate::ast::ASTNode;
use crate::box_trait::*;
use crate::interpreter::{NyashInterpreter as Interpreter, RuntimeError};
use crate::boxes::math_box::MathBox;
use crate::boxes::random_box::RandomBox;
use crate::boxes::sound_box::SoundBox;
use crate::boxes::debug_box::DebugBox;
use crate::box_factory::BoxFactory;

impl Interpreter {
    /// Create non-basic type boxes (MathBox, ConsoleBox, GUI/Network boxes, etc.)
    pub(super) fn create_non_basic_box(
        &mut self, 
        class: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match class {
            "MathBox" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("MathBox constructor expects 0 arguments, got {}", arguments.len()) });
                }
                if let Ok(reg) = self.runtime.box_registry.lock() {
                    if let Ok(b) = reg.create_box("MathBox", &[]) { return Ok(b); }
                }
                // fallback to builtin
                return Ok(Box::new(MathBox::new()) as Box<dyn NyashBox>);
            }
            
            "ConsoleBox" => {
                // ConsoleBoxは引数なしで作成（可能なら統一レジストリ経由でプラグイン優先）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ConsoleBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                // Delegate to unified registry so env-based plugin overrides apply consistently
                if let Ok(reg) = self.runtime.box_registry.lock() {
                    if let Ok(b) = reg.create_box("ConsoleBox", &[]) {
                        return Ok(b);
                    }
                }
                // Fallback to builtin mock if registry path failed
                return Ok(Box::new(crate::box_trait::ConsoleBox::new()) as Box<dyn NyashBox>);
            }
            
            "RandomBox" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("RandomBox constructor expects 0 arguments, got {}", arguments.len()) });
                }
                if let Ok(reg) = self.runtime.box_registry.lock() {
                    if let Ok(b) = reg.create_box("RandomBox", &[]) { return Ok(b); }
                }
                return Ok(Box::new(RandomBox::new()) as Box<dyn NyashBox>);
            }
            
            "SoundBox" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("SoundBox constructor expects 0 arguments, got {}", arguments.len()) });
                }
                if let Ok(reg) = self.runtime.box_registry.lock() {
                    if let Ok(b) = reg.create_box("SoundBox", &[]) { return Ok(b); }
                }
                return Ok(Box::new(SoundBox::new()) as Box<dyn NyashBox>);
            }
            
            "DebugBox" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation { message: format!("DebugBox constructor expects 0 arguments, got {}", arguments.len()) });
                }
                if let Ok(reg) = self.runtime.box_registry.lock() {
                    if let Ok(b) = reg.create_box("DebugBox", &[]) { return Ok(b); }
                }
                return Ok(Box::new(DebugBox::new()) as Box<dyn NyashBox>);
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
