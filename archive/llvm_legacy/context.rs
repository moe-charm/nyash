//! Deprecated LLVM Legacy Context
//! Archived at: docs/archive/backends/llvm-inkwell-legacy/

#[cfg(feature = "llvm-inkwell-legacy")]
compile_error!("LLVM Inkwell Legacy backend deprecated. Use Python LLVM harness.");

// Stub exports for compatibility
pub struct LegacyContext;
pub struct LegacyModule;