use crate::ast::Span;
use crate::parser::ParseError;
use thiserror::Error;

/// 実行時エラー
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Undefined variable '{name}'")]
    UndefinedVariable { name: String },

    #[error("Undefined function '{name}'")]
    UndefinedFunction { name: String },

    #[error("Undefined class '{name}'")]
    UndefinedClass { name: String },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Break outside of loop")]
    BreakOutsideLoop,

    #[error("Return outside of function")]
    ReturnOutsideFunction,

    #[error("Uncaught exception")]
    UncaughtException,

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Environment error: {0}")]
    EnvironmentError(String),

    // === 🔥 Enhanced Errors with Span Information ===

    #[error("Undefined variable '{name}' at {span}")]
    UndefinedVariableAt { name: String, span: Span },

    #[error("Type error: {message} at {span}")]
    TypeErrorAt { message: String, span: Span },

    #[error("Invalid operation: {message} at {span}")]
    InvalidOperationAt { message: String, span: Span },

    #[error("Break outside of loop at {span}")]
    BreakOutsideLoopAt { span: Span },

    #[error("Return outside of function at {span}")]
    ReturnOutsideFunctionAt { span: Span },

    #[error("Runtime failure: {message}")]
    RuntimeFailure { message: String },
}

impl RuntimeError {
    /// エラーの詳細な文脈付きメッセージを生成
    pub fn detailed_message(&self, source: Option<&str>) -> String {
        match self {
            // Enhanced errors with span information
            RuntimeError::UndefinedVariableAt { name, span } => {
                let mut msg = format!("⚠️  Undefined variable '{}'", name);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }

            RuntimeError::TypeErrorAt { message, span } => {
                let mut msg = format!("⚠️  Type error: {}", message);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }

            RuntimeError::InvalidOperationAt { message, span } => {
                let mut msg = format!("⚠️  Invalid operation: {}", message);
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }

            RuntimeError::BreakOutsideLoopAt { span } => {
                let mut msg = "⚠️  Break statement outside of loop".to_string();
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }

            RuntimeError::ReturnOutsideFunctionAt { span } => {
                let mut msg = "⚠️  Return statement outside of function".to_string();
                if let Some(src) = source {
                    msg.push('\n');
                    msg.push_str(&span.error_context(src));
                } else {
                    msg.push_str(&format!(" at {}", span));
                }
                msg
            }

            // Fallback for old error variants without span
            _ => format!("⚠️  {}", self),
        }
    }
}
