/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;
pub mod vm_phi;
pub mod vm_instructions;
pub mod vm_values;
pub mod vm_boxcall;
pub mod vm_stats;
// Phase 9.78h: VM split scaffolding (control_flow/dispatch/frame)
pub mod control_flow;
pub mod dispatch;
pub mod frame;

#[cfg(feature = "wasm-backend")]
pub mod wasm;
#[cfg(feature = "wasm-backend")]
pub mod aot;

#[cfg(feature = "llvm")]
pub mod llvm;

pub use vm::{VM, VMError, VMValue};

#[cfg(feature = "wasm-backend")]
pub use wasm::{WasmBackend, WasmError};
#[cfg(feature = "wasm-backend")]
pub use aot::{AotBackend, AotError, AotConfig, AotStats};

#[cfg(feature = "llvm")]
pub use llvm::{compile_and_execute as llvm_compile_and_execute, compile_to_object as llvm_compile_to_object};
