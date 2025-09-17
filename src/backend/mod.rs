/*!
 * Backend module - Different execution backends for MIR
 */

// VM core types are always available
pub mod vm_types;

// Legacy VM execution pipeline (feature-gated)
#[cfg(feature = "vm-legacy")]
pub mod vm;
#[cfg(feature = "vm-legacy")]
pub mod vm_boxcall;
#[cfg(feature = "vm-legacy")]
pub mod vm_instructions;
#[cfg(feature = "vm-legacy")]
pub mod vm_phi;
#[cfg(feature = "vm-legacy")]
pub mod vm_stats;
#[cfg(feature = "vm-legacy")]
pub mod vm_values;

// When vm-legacy is disabled, provide a compatibility shim module so
// crate::backend::vm::VMValue etc. keep resolving to vm_types::*.
#[cfg(not(feature = "vm-legacy"))]
pub mod vm {
    pub use super::vm_types::{VMError, VMValue};
}
// Phase 9.78h: VM split scaffolding (control_flow/dispatch/frame)
pub mod abi_util; // Shared ABI/utility helpers
#[cfg(feature = "vm-legacy")]
pub mod control_flow;
#[cfg(feature = "vm-legacy")]
pub mod dispatch;
#[cfg(feature = "vm-legacy")]
pub mod frame;
pub mod gc_helpers;
pub mod mir_interpreter;
#[cfg(feature = "vm-legacy")]
pub mod vm_control_flow;
#[cfg(feature = "vm-legacy")]
mod vm_exec; // A3: execution loop extracted
#[cfg(feature = "vm-legacy")]
mod vm_gc; // A3: GC roots & diagnostics extracted
#[cfg(feature = "vm-legacy")]
mod vm_methods; // A3-S1: method dispatch wrappers extracted
#[cfg(feature = "vm-legacy")]
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
// Always re-export VMError/VMValue from vm_types; VM (executor) only when enabled
pub use vm_types::{VMError, VMValue};
#[cfg(feature = "vm-legacy")]
pub use vm::VM;

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
