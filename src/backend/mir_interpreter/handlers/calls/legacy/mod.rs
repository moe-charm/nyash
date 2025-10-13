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
#[cfg(feature = "legacy-boxes")]
mod legacy_resolver;
#[cfg(feature = "legacy-boxes")]
mod extern_handler;

// Re-export for parent module
#[allow(unused_imports)] pub(crate) use callee_dispatcher::*;
#[allow(unused_imports)] pub(crate) use method_handler::*;
#[cfg(feature = "legacy-boxes")]
#[allow(unused_imports)] pub(crate) use legacy_resolver::*;
#[cfg(feature = "legacy-boxes")]
#[allow(unused_imports)] pub(crate) use extern_handler::*;

impl MirInterpreter {
    /// Entry point for Call instruction handling
    pub(crate) fn handle_call(
        &mut self,
        dst: Option<ValueId>,
        _func: ValueId,
        callee: Option<&Callee>,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // LocalSSA at call-site: prefer a materialized in-block SSA id for each arg
        let args2: Vec<ValueId> = self.materialize_args_in_current_block(args);
        let call_result = if let Some(callee_type) = callee {
            self.execute_callee_call(callee_type, &args2)?
        } else {
            #[cfg(feature = "legacy-boxes")]
            {
                self.execute_legacy_call(func, &args2)?
            }
            #[cfg(not(feature = "legacy-boxes"))]
            {
                return Err(VMError::InvalidInstruction("NameConst legacy call path disabled (legacy-boxes OFF)".into()));
            }
        };
        if let Some(d) = dst {
            self.regs.insert(d, call_result);
        }
        Ok(())
    }
}
