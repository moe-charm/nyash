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
        let condition_val = self.build_expression(condition)?;
        
        // Create basic blocks for then/else/merge
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();
        
        // Emit branch instruction in current block
        self.emit_instruction(MirInstruction::Branch {
            condition: condition_val,
            then_bb: then_block,
            else_bb: else_block,
        })?;
        
        // Build then branch
        self.current_block = Some(then_block);
        self.ensure_block_exists(then_block)?;
        // Keep a copy of AST for analysis (phi for variable reassignment)
        let then_ast_for_analysis = then_branch.clone();
        let then_value = self.build_expression(then_branch)?;
        if !self.is_current_block_terminated() {
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }
        
        // Build else branch
        self.current_block = Some(else_block);
        self.ensure_block_exists(else_block)?;
        let (else_value, else_ast_for_analysis) = if let Some(else_ast) = else_branch {
            let val = self.build_expression(else_ast.clone())?;
            (val, Some(else_ast))
        } else {
            // No else branch, use void
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            })?;
            (void_val, None)
        };
        if !self.is_current_block_terminated() {
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }
        
        // Create merge block with phi function
        self.current_block = Some(merge_block);
        self.ensure_block_exists(merge_block)?;
        let result_val = self.value_gen.next();
        
        self.emit_instruction(MirInstruction::Phi {
            dst: result_val,
            inputs: vec![
                (then_block, then_value),
                (else_block, else_value),
            ],
        })?;

        // Heuristic: If both branches assign the same variable name, bind that variable to the phi result
        let assigned_var_then = Self::extract_assigned_var(&then_ast_for_analysis);
        let assigned_var_else = else_ast_for_analysis.as_ref().and_then(|a| Self::extract_assigned_var(a));
        if let (Some(a), Some(b)) = (assigned_var_then, assigned_var_else) {
            if a == b {
                self.variable_map.insert(a, result_val);
            }
        }

        Ok(result_val)
    }

    /// Extract assigned variable name from an AST node if it represents an assignment to a variable.
    /// Handles direct Assignment and Program with trailing single-statement Assignment.
    fn extract_assigned_var(ast: &ASTNode) -> Option<String> {
        match ast {
            ASTNode::Assignment { target, .. } => {
                if let ASTNode::Variable { name, .. } = target.as_ref() { Some(name.clone()) } else { None }
            }
            ASTNode::Program { statements, .. } => {
                // Inspect the last statement as the resulting value of the block
                statements.last().and_then(|st| Self::extract_assigned_var(st))
            }
            _ => None,
        }
    }
    
    /// Build a loop statement: loop(condition) { body }
    ///
    /// Uses the shared LoopBuilder facade to avoid tight coupling.
    pub(super) fn build_loop_statement(&mut self, condition: ASTNode, body: Vec<ASTNode>) -> Result<ValueId, String> {
        // Evaluate condition first (boolean-ish value)
        let cond_val = self.build_expression(condition)?;

        // Use loop_api helper with a closure that builds the loop body
        let mut body_builder = |lb: &mut Self| -> Result<(), String> {
            for stmt in &body {
                let _ = lb.build_expression(stmt.clone())?;
            }
            Ok(())
        };

        crate::mir::loop_api::build_simple_loop(self, cond_val, &mut body_builder)
    }
    
    /// Build a try/catch statement
    pub(super) fn build_try_catch_statement(&mut self, try_body: Vec<ASTNode>, catch_clauses: Vec<crate::ast::CatchClause>, finally_body: Option<Vec<ASTNode>>) -> Result<ValueId, String> {
        let try_block = self.block_gen.next();
        let catch_block = self.block_gen.next();
        let finally_block = if finally_body.is_some() { Some(self.block_gen.next()) } else { None };
        let exit_block = self.block_gen.next();
        
        // Set up exception handler for the try block (before we enter it)
        if let Some(catch_clause) = catch_clauses.first() {
            let exception_value = self.value_gen.next();
            
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
        if let Some(catch_clause) = catch_clauses.first() {
            // Build catch body
            let catch_ast = ASTNode::Program {
                statements: catch_clause.body.clone(),
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(catch_ast)?;
        }
        
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
        let result = self.value_gen.next();
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
