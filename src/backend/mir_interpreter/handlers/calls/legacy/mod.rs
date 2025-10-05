//! Legacy call handling (string-based function resolution + Method calls)
//!
//! This module provides backward compatibility for legacy MIR instructions
//! that use string-based function names (NameConst) instead of structured Callee.
//!
//! ## Module Structure
//! - `callee_dispatcher`: Routes Callee variants to handlers
//! - `method_handler`: Method call receiver resolution
//! - `legacy_resolver`: String-based function name resolution
//! - `extern_handler`: Extern function execution (exit, panic)

use super::super::*;

mod callee_dispatcher;
mod method_handler;
mod legacy_resolver;
mod extern_handler;

// Re-export for parent module
pub(crate) use callee_dispatcher::*;
pub(crate) use method_handler::*;
pub(crate) use legacy_resolver::*;
pub(crate) use extern_handler::*;

impl MirInterpreter {
    /// Entry point for Call instruction handling
    pub(crate) fn handle_call(
        &mut self,
        dst: Option<ValueId>,
        func: ValueId,
        callee: Option<&Callee>,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // LocalSSA at call-site: prefer a materialized in-block SSA id for each arg
        let args2: Vec<ValueId> = self.materialize_args_in_current_block(args);
        let call_result = if let Some(callee_type) = callee {
            self.execute_callee_call(callee_type, &args2)?
        } else {
            self.execute_legacy_call(func, &args2)?
        };
        if let Some(d) = dst {
            self.regs.insert(d, call_result);
        }
        Ok(())
    }
}
