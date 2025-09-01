/*!
 * Async Operations Module
 * 
 * Extracted from expressions.rs lines 1020-1031 (~11 lines)
 * Handles await expression processing for asynchronous operations
 * Core philosophy: "Everything is Box" with async support
 */

use super::*;
use crate::boxes::result::NyashResultBox;
use crate::box_trait::StringBox;

impl NyashInterpreter {
    /// await式を実行 - 非同期操作の結果を待機
    pub(super) fn execute_await(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value = self.execute_expression(expression)?;
        
        // FutureBoxなら協調待機して Result.Ok/Err を返す
        if let Some(future) = value.as_any().downcast_ref::<FutureBox>() {
            let max_ms: u64 = std::env::var("NYASH_AWAIT_MAX_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(5000);
            let start = std::time::Instant::now();
            let mut spins = 0usize;
            while !future.ready() {
                crate::runtime::global_hooks::safepoint_and_poll();
                std::thread::yield_now();
                spins += 1;
                if spins % 1024 == 0 { std::thread::sleep(std::time::Duration::from_millis(1)); }
                if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                    let err = Box::new(StringBox::new("Timeout"));
                    return Ok(Box::new(NyashResultBox::new_err(err)));
                }
            }
            let v = future.get();
            Ok(Box::new(NyashResultBox::new_ok(v)))
        } else {
            // FutureBoxでなければ Ok(value) で返す
            Ok(Box::new(NyashResultBox::new_ok(value)))
        }
    }
}
