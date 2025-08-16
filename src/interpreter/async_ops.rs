/*!
 * Async Operations Module
 * 
 * Extracted from expressions.rs lines 1020-1031 (~11 lines)
 * Handles await expression processing for asynchronous operations
 * Core philosophy: "Everything is Box" with async support
 */

use super::*;

impl NyashInterpreter {
    /// await式を実行 - 非同期操作の結果を待機
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