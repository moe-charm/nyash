/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;
pub mod wasm;
pub mod aot;

pub use vm::{VM, VMError, VMValue};
pub use wasm::{WasmBackend, WasmError};
pub use aot::{AotBackend, AotError, AotConfig, AotStats};