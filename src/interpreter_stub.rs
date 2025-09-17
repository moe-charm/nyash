//! Minimal stubs for legacy interpreter APIs when the feature is disabled.
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, VoidBox};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("invalid operation: {message}")]
    InvalidOperation { message: String },
    #[error("type error: {message}")]
    TypeError { message: String },
}

#[derive(Debug, Default, Clone)]
pub struct SharedState;
impl SharedState { pub fn new() -> Self { Self } }

#[derive(Debug, Default)]
pub struct NyashInterpreter;
impl NyashInterpreter {
    pub fn new() -> Self { Self }
    pub fn with_shared(_s: SharedState) -> Self { Self }
    pub fn execute(&mut self, _ast: ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Ok(Box::new(VoidBox::new()))
    }
    pub fn execute_expression(&mut self, _expr: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Ok(Box::new(VoidBox::new()))
    }
}

pub fn run_function_box(
    _fun: &crate::boxes::function_box::FunctionBox,
    _args: Vec<Box<dyn NyashBox>>,
) -> Result<Box<dyn NyashBox>, RuntimeError> {
    Ok(Box::new(VoidBox::new()))
}

