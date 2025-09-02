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
pub mod gc_helpers;
pub mod abi_util; // Shared ABI/utility helpers
pub mod mir_interpreter; // Lightweight MIR interpreter

#[cfg(feature = "wasm-backend")]
pub mod wasm;
#[cfg(feature = "wasm-backend")]
pub mod aot;
#[cfg(feature = "wasm-backend")]
pub mod wasm_v2;

#[cfg(feature = "llvm")]
pub mod llvm;
#[cfg(feature = "cranelift-jit")]
pub mod cranelift;

pub use vm::{VM, VMError, VMValue};
pub use mir_interpreter::MirInterpreter;

#[cfg(feature = "wasm-backend")]
pub use wasm::{WasmBackend, WasmError};
#[cfg(feature = "wasm-backend")]
pub use aot::{AotBackend, AotError, AotConfig, AotStats};

#[cfg(feature = "llvm")]
pub use llvm::{compile_and_execute as llvm_compile_and_execute, compile_to_object as llvm_compile_to_object};
#[cfg(feature = "cranelift-jit")]
pub use cranelift::{compile_and_execute as cranelift_compile_and_execute, compile_to_object as cranelift_compile_to_object};
