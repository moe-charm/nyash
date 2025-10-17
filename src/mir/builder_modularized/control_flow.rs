/*!
 * MIR Builder Control Flow - Control flow AST node conversion
 * 
 * Handles conversion of control flow AST nodes (if, loop, try-catch) to MIR instructions
 */

use super::*;
use crate::ast::ASTNode;

impl MirBuilder {
    /// Build if statement with conditional branches
    pub(super) fn build_if_statement(&mut self, condition: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Result<ValueId, String> {
        self.lower_if_form(condition, then_branch, else_branch)
    }

    // Assigned variable extraction is centralized in phi_core::if_phi now.
    
    /// Build a loop statement: loop(condition) { body }
    ///
    /// Force structured Loop-Form lowering (preheader → header(φ) → body → latch → header|exit)
    /// to ensure PHI correctness for loop-carried values.
    pub(super) fn build_loop_statement(&mut self, condition: ASTNode, body: Vec<ASTNode>) -> Result<ValueId, String> {
        // Delegate to the unified control-flow entry which uses LoopBuilder
        self.cf_loop(condition, body)
    }
    
    /// Build a try/catch statement
    pub(super) fn build_try_catch_statement(&mut self, try_body: Vec<ASTNode>, catch_clauses: Vec<crate::ast::CatchClause>, finally_body: Option<Vec<ASTNode>>) -> Result<ValueId, String> {
        let try_block = self.block_gen.next();
        let catch_block = self.block_gen.next();
        let finally_block = if finally_body.is_some() { Some(self.block_gen.next()) } else { None };
        let exit_block = self.block_gen.next();
        
        // Set up exception handler for the try block (before we enter it)
        if std::env::var("NYASH_BUILDER_DISABLE_TRYCATCH").ok().as_deref() == Some("1") {
            // Fallback: build try body only
        } else if let Some(catch_clause) = catch_clauses.first() {
            let exception_value = self.safe_next_value();
            
            // Register catch handler for exceptions that may occur in try block
            self.emit_instruction(MirInstruction::Catch {
                exception_type: catch_clause.exception_type.clone(),
                exception_value,
                handler_bb: catch_block,
            })?;
        }
        
        // Jump to try block
        self.emit_instruction(MirInstruction::Jump { target: try_block })?;
        
        // Build try block
        self.start_new_block(try_block)?;
        
        let try_ast = ASTNode::Program {
            statements: try_body,
            span: crate::ast::Span::unknown(),
        };
        let _try_result = self.build_expression(try_ast)?;
        
        // Normal completion of try block - jump to finally or exit (if not already terminated)
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            self.emit_instruction(MirInstruction::Jump { target: next_target })?;
        }
        
        // Build catch block (reachable via exception handling)
        self.start_new_block(catch_block)?;
        
        // Handle catch clause
        if std::env::var("NYASH_BUILDER_DISABLE_TRYCATCH").ok().as_deref() != Some("1") {
        if let Some(catch_clause) = catch_clauses.first() {
            // Build catch body
            let catch_ast = ASTNode::Program {
                statements: catch_clause.body.clone(),
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(catch_ast)?;
        }}
        
        // Catch completion - jump to finally or exit (if not already terminated)
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            self.emit_instruction(MirInstruction::Jump { target: next_target })?;
        }
        
        // Build finally block if present
        if let (Some(finally_block_id), Some(finally_statements)) = (finally_block, finally_body) {
            self.start_new_block(finally_block_id)?;
            
            let finally_ast = ASTNode::Program {
                statements: finally_statements,
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(finally_ast)?;
            
            self.emit_instruction(MirInstruction::Jump { target: exit_block })?;
        }
        
        // Create exit block
        self.start_new_block(exit_block)?;
        
        // Return void for now (in a complete implementation, would use phi for try/catch values)
        let result = self.safe_next_value();
        self.emit_instruction(MirInstruction::Const {
            dst: result,
            value: ConstValue::Void,
        })?;
        
        Ok(result)
    }

    /// Check if the current basic block is terminated
    pub(super) fn is_current_block_terminated(&self) -> bool {
        if let (Some(block_id), Some(ref function)) = (self.current_block, &self.current_function) {
            if let Some(block) = function.get_block(block_id) {
                return block.is_terminated();
            }
        }
        false
    }
}
