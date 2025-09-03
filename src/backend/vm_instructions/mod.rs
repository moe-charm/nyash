/*!
 * VM Instruction Handlers (split modules)
 *
 * This module was split to keep files under ~1000 lines while preserving
 * behavior and public APIs. Methods remain as `impl VM` across submodules.
 */

pub mod core;           // const/binop/unary/compare/print/ctrl/type/phi/mem/array/refs/weak/barriers/exn/await
pub mod call;           // execute_call (Function name or FunctionBox)
pub mod newbox;         // execute_newbox
pub mod function_new;   // execute_function_new
pub mod extern_call;    // execute_extern_call
pub mod boxcall;        // execute_boxcall + vtable stub
pub mod plugin_invoke;  // execute_plugin_invoke + helpers

