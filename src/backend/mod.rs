/*!
 * Backend module - Different execution backends for MIR
 */

// VM core types are always available
pub mod vm_types;

// Legacy VM execution pipeline removed from archive

// Compatibility shim module - always provide vm module with core types
pub mod vm {
    pub use super::vm_types::{VMError, VMValue};
}
// Core backend modules
pub mod abi_util; // Shared ABI/utility helpers
pub mod gc_helpers;
pub mod mir_interpreter; // Lightweight MIR interpreter (Rust VM core)

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

// Public aliases to make the role of the VM clear in runner/tests
pub use mir_interpreter::MirInterpreter;
/// Primary Rust VM executor alias (preferred name)
pub type NyashVm = mir_interpreter::MirInterpreter;
/// Back-compat shim used across runner/tests
pub type VM = NyashVm;
// Always re-export VMError/VMValue from vm_types
pub use vm_types::{VMError, VMValue};

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
