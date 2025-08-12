/*!
 * Nyash MIR (Mid-level Intermediate Representation) - Stage 1 Implementation
 * 
 * ChatGPT5-designed MIR infrastructure for native compilation support
 * Based on SSA form with effect tracking and Box-aware optimizations
 */

pub mod instruction;
pub mod basic_block;
pub mod function;
pub mod builder;
pub mod verification;
pub mod printer;
pub mod value_id;
pub mod effect;

// Re-export main types for easy access
pub use instruction::{MirInstruction, BinaryOp, CompareOp, UnaryOp, ConstValue, MirType};
pub use basic_block::{BasicBlock, BasicBlockId, BasicBlockIdGenerator};
pub use function::{MirFunction, MirModule, FunctionSignature};
pub use builder::MirBuilder;
pub use verification::{MirVerifier, VerificationError};
pub use printer::MirPrinter;
pub use value_id::{ValueId, LocalId, ValueIdGenerator};
pub use effect::{EffectMask, Effect};

/// MIR compilation result
#[derive(Debug, Clone)]
pub struct MirCompileResult {
    pub module: MirModule,
    pub verification_result: Result<(), Vec<VerificationError>>,
}

/// MIR compiler - converts AST to MIR/SSA form
pub struct MirCompiler {
    builder: MirBuilder,
    verifier: MirVerifier,
}

impl MirCompiler {
    /// Create a new MIR compiler
    pub fn new() -> Self {
        Self {
            builder: MirBuilder::new(),
            verifier: MirVerifier::new(),
        }
    }
    
    /// Compile AST to MIR module with verification
    pub fn compile(&mut self, ast: crate::ast::ASTNode) -> Result<MirCompileResult, String> {
        // Convert AST to MIR using builder
        let module = self.builder.build_module(ast)?;
        
        // Verify the generated MIR
        let verification_result = self.verifier.verify_module(&module);
        
        Ok(MirCompileResult {
            module,
            verification_result,
        })
    }
    
    /// Dump MIR to string for debugging
    pub fn dump_mir(&self, module: &MirModule) -> String {
        MirPrinter::new().print_module(module)
    }
}

impl Default for MirCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, LiteralValue};
    
    #[test]
    fn test_basic_mir_compilation() {
        let mut compiler = MirCompiler::new();
        
        // Create a simple literal AST node
        let ast = ASTNode::Literal { 
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown()
        };
        
        // Compile to MIR
        let result = compiler.compile(ast);
        assert!(result.is_ok(), "Basic MIR compilation should succeed");
        
        let compile_result = result.unwrap();
        assert!(!compile_result.module.functions.is_empty(), "Module should contain at least one function");
    }
    
    #[test]
    fn test_mir_dump() {
        let mut compiler = MirCompiler::new();
        
        let ast = ASTNode::Literal { 
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown()
        };
        
        let result = compiler.compile(ast).unwrap();
        let mir_dump = compiler.dump_mir(&result.module);
        
        assert!(!mir_dump.is_empty(), "MIR dump should not be empty");
        assert!(mir_dump.contains("function"), "MIR dump should contain function information");
    }
}