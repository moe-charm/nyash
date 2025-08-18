/*!
 * LLVM Backend Module - Compile MIR to LLVM IR for AOT execution
 * 
 * This module provides LLVM-based compilation of Nyash MIR to native code.
 * Phase 9.78 PoC implementation focused on minimal "return 42" support.
 */

pub mod context;
pub mod compiler;

use crate::mir::function::MirModule;
use crate::box_trait::{NyashBox, IntegerBox};

/// Compile MIR module to object file and execute
pub fn compile_and_execute(
    mir_module: &MirModule,
    output_path: &str,
) -> Result<Box<dyn NyashBox>, String> {
    let compiler = compiler::LLVMCompiler::new()?;
    compiler.compile_and_execute(mir_module, output_path)
}

/// Compile MIR module to object file only
pub fn compile_to_object(
    mir_module: &MirModule,
    output_path: &str,
) -> Result<(), String> {
    let compiler = compiler::LLVMCompiler::new()?;
    compiler.compile_module(mir_module, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_llvm_module_creation() {
        // Basic test to ensure the module can be loaded
        // Actual compilation tests require full MIR infrastructure
        assert!(true);
    }
}