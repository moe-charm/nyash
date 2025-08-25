/*!
 * MIR Builder Box Handlers - Box-related AST node conversion
 * 
 * Handles conversion of Box-related AST nodes (new expressions, box declarations) to MIR instructions
 */

use super::*;
use crate::ast::ASTNode;
use std::collections::{HashMap, HashSet};

impl MirBuilder {
    /// Build static box Main - extracts main() method body and converts to Program
    pub(super) fn build_static_main_box(&mut self, methods: HashMap<String, ASTNode>) -> Result<ValueId, String> {
        // Look for the main() method
        if let Some(main_method) = methods.get("main") {
            if let ASTNode::FunctionDeclaration { body, .. } = main_method {
                // Convert the method body to a Program AST node and lower it
                let program_ast = ASTNode::Program {
                    statements: body.clone(),
                    span: crate::ast::Span::unknown(),
                };
                
                // Use existing Program lowering logic
                self.build_expression(program_ast)
            } else {
                Err("main method in static box Main is not a FunctionDeclaration".to_string())
            }
        } else {
            Err("static box Main must contain a main() method".to_string())
        }
    }
    
    /// Build box declaration - register type metadata
    pub(super) fn build_box_declaration(&mut self, name: String, methods: HashMap<String, ASTNode>, fields: Vec<String>, weak_fields: Vec<String>) -> Result<(), String> {
        // For Phase 8.4, we'll emit metadata instructions to register the box type
        // In a full implementation, this would register type information for later use
        
        // Create a type registration constant
        let type_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: type_id,
            value: ConstValue::String(format!("__box_type_{}", name)),
        })?;
        
        // For each field, emit metadata about the field
        for field in fields {
            let field_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: field_id,
                value: ConstValue::String(format!("__field_{}_{}", name, field)),
            })?;
        }

        // Record weak fields for this box
        if !weak_fields.is_empty() {
            let set: HashSet<String> = weak_fields.into_iter().collect();
            self.weak_fields_by_box.insert(name.clone(), set);
        }
        
        // Process methods - now methods is a HashMap
        for (method_name, method_ast) in methods {
            if let ASTNode::FunctionDeclaration { .. } = method_ast {
                let method_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: method_id,
                    value: ConstValue::String(format!("__method_{}_{}", name, method_name)),
                })?;
            }
        }
        
        Ok(())
    }
}