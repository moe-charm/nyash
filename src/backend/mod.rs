/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;
pub mod vm_phi;
pub mod wasm;
pub mod aot;

#[cfg(feature = "llvm")]
pub mod llvm;

pub use vm::{VM, VMError, VMValue};
pub use wasm::{WasmBackend, WasmError};
pub use aot::{AotBackend, AotError, AotConfig, AotStats};

#[cfg(feature = "llvm")]
pub use llvm::{compile_and_execute as llvm_compile_and_execute, compile_to_object as llvm_compile_to_object};