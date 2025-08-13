/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;
pub mod wasm;

pub use vm::{VM, VMError, VMValue};
pub use wasm::{WasmBackend, WasmError};