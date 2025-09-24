//! LLVM Legacy Backend (Deprecated)
//!
//! This module has been archived and is no longer supported.
//! Please use the Python LLVM harness instead.

#[cfg(feature = "llvm-inkwell-legacy")]
compile_error!(
    "LLVM Inkwell Legacy backend is no longer supported. \
    Please use the Python LLVM harness with --backend llvm or NYASH_LLVM_USE_HARNESS=1. \
    Legacy code archived at: docs/archive/backends/llvm-inkwell-legacy/"
);

// Stub exports for compilation compatibility
pub mod box_types;
pub mod compiler;
pub mod context;

pub fn compile_and_execute(_program: &str) -> Result<(), String> {
    Err("LLVM Legacy backend deprecated. Use Python LLVM harness.".to_string())
}

pub fn compile_to_object(_program: &str) -> Result<Vec<u8>, String> {
    Err("LLVM Legacy backend deprecated. Use Python LLVM harness.".to_string())
}