/*!
 * MIR Builder - AST to MIR conversion
 * 
 * This is the main entry point for the MIR builder.
 * Most implementation has been moved to submodules:
 * - core.rs: Core builder functionality
 * - expressions.rs: Expression conversion
 * - statements.rs: Statement conversion  
 * - control_flow.rs: Control flow conversion
 * - box_handlers.rs: Box-related conversions
 */

pub mod core;
pub mod expressions;
pub mod statements;
pub mod control_flow;
pub mod box_handlers;
pub mod loop_builder;

pub use self::core::MirBuilder;

use crate::ast::ASTNode;
use crate::mir::{MirModule, MirFunction, MirInstruction, FunctionSignature, MirType, EffectMask, ValueId};

impl MirBuilder {
    /// Build a MIR module from an AST
    pub fn build_module(&mut self, ast: ASTNode) -> Result<MirModule, String> {
        // Create a new module
        let module = MirModule::new("main".to_string());
        
        // Create a main function to contain the AST
        let main_signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let entry_block = self.block_gen.next();
        let mut main_function = MirFunction::new(main_signature, entry_block);
        main_function.metadata.is_entry_point = true;
        
        // Set up building context
        self.current_module = Some(module);
        self.current_function = Some(main_function);
        self.current_block = Some(entry_block);
        
        // Optional: Add safepoint at function entry
        if std::env::var("NYASH_BUILDER_SAFEPOINT_ENTRY").ok().as_deref() == Some("1") {
            self.emit_instruction(MirInstruction::Safepoint)?;
        }
        
        // Convert AST to MIR
        let result_value = self.build_expression(ast)?;
        
        // Add return instruction if needed
        if let Some(block_id) = self.current_block {
            if let Some(ref mut function) = self.current_function {
                if let Some(block) = function.get_block_mut(block_id) {
                    if !block.is_terminated() {
                        block.add_instruction(MirInstruction::Return {
                            value: Some(result_value),
                        });
                    }
                }
            }
        }
        
        // Finalize and return module
        let mut module = self.current_module.take().unwrap();
        let function = self.current_function.take().unwrap();
        module.add_function(function);
        
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, LiteralValue, Span};
    
    #[test]
    fn test_build_literal() {
        let mut builder = MirBuilder::new();
        let ast = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert!(module.functions.contains_key("main"));
    }
    
    #[test]
    fn test_build_simple_program() {
        let mut builder = MirBuilder::new();
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::Literal {
                    value: LiteralValue::Integer(1),
                    span: Span::unknown(),
                },
                ASTNode::Literal {
                    value: LiteralValue::Integer(2),
                    span: Span::unknown(),
                },
            ],
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
    }
}
