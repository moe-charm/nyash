/*!
 * Async Operations Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles await expressions and async operations
 * Core philosophy: "Everything is Box" with async support
 */

use super::*;
use crate::boxes::FutureBox;

impl NyashInterpreter {
    /// await式を実行 - Execute await expression
    pub(super) fn execute_await(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value = self.execute_expression(expression)?;
        
        // FutureBoxなら待機して結果を取得
        if let Some(future) = value.as_any().downcast_ref::<FutureBox>() {
            future.wait_and_get()
                .map_err(|msg| RuntimeError::InvalidOperation { message: msg })
        } else {
            // FutureBoxでなければそのまま返す
            Ok(value)
        }
    }
}