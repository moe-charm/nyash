//! Deprecated LLVM Legacy Compiler
//! Archived at: docs/archive/backends/llvm-inkwell-legacy/

#[cfg(feature = "llvm-inkwell-legacy")]
compile_error!("LLVM Inkwell Legacy backend deprecated. Use Python LLVM harness.");

// Stub exports for compatibility
pub struct LegacyCompiler;
pub fn compile_mir(_mir: &str) -> Result<(), String> {
    Err("LLVM Legacy compiler deprecated. Use Python LLVM harness.".to_string())
}