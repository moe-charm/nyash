/*!
 * Backend module - Different execution backends for MIR
 */

pub mod vm;
pub mod vm_boxcall;
pub mod vm_instructions;
pub mod vm_phi;
pub mod vm_stats;
pub mod vm_types;
pub mod vm_values;
// Phase 9.78h: VM split scaffolding (control_flow/dispatch/frame)
pub mod abi_util; // Shared ABI/utility helpers
pub mod control_flow;
pub mod dispatch;
pub mod frame;
pub mod gc_helpers;
pub mod mir_interpreter;
pub mod vm_control_flow;
mod vm_exec; // A3: execution loop extracted
mod vm_gc; // A3: GC roots & diagnostics extracted
mod vm_methods; // A3-S1: method dispatch wrappers extracted
mod vm_state; // A3: state & basic helpers extracted // Lightweight MIR interpreter

#[cfg(feature = "wasm-backend")]
pub mod aot;
#[cfg(feature = "wasm-backend")]
pub mod wasm;
#[cfg(feature = "wasm-backend")]
pub mod wasm_v2;

#[cfg(feature = "llvm-inkwell-legacy")]
pub mod llvm_legacy;
// Back-compat shim so existing paths crate::backend::llvm::* keep working
#[cfg(feature = "cranelift-jit")]
pub mod cranelift;
#[cfg(feature = "llvm-inkwell-legacy")]
pub mod llvm;

pub use mir_interpreter::MirInterpreter;
pub use vm::{VMError, VMValue, VM};

#[cfg(feature = "wasm-backend")]
pub use aot::{AotBackend, AotConfig, AotError, AotStats};
#[cfg(feature = "wasm-backend")]
pub use wasm::{WasmBackend, WasmError};

#[cfg(feature = "cranelift-jit")]
pub use cranelift::{
    compile_and_execute as cranelift_compile_and_execute,
    compile_to_object as cranelift_compile_to_object,
};
#[cfg(feature = "llvm-inkwell-legacy")]
pub use llvm_legacy::{
    compile_and_execute as llvm_compile_and_execute, compile_to_object as llvm_compile_to_object,
};
