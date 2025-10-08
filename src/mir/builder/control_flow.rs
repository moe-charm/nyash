//! Control-flow entrypoints (if/loop/try/throw) centralized here.
use super::{Effect, EffectMask, MirInstruction, ValueId};
use crate::ast::ASTNode;

impl super::MirBuilder {
    /// Control-flow: block
    pub(super) fn cf_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        // identical to build_block; kept here for future policy hooks
        self.build_block(statements)
    }

    /// Control-flow: if
    pub(super) fn cf_if(
        &mut self,
        condition: ASTNode,
        then_branch: ASTNode,
        else_branch: Option<ASTNode>,
    ) -> Result<ValueId, String> {
        // current impl is a simple forward to canonical lowering
        self.lower_if_form(condition, then_branch, else_branch)
    }

    /// Control-flow: loop
    pub(super) fn cf_loop(
        &mut self,
        condition: ASTNode,
        body: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Delegate to LoopBuilder for consistent handling
        let mut loop_builder = crate::mir::loop_builder::LoopBuilder::new(self);
        loop_builder.build_loop(condition, body)
    }

    /// Control-flow: try/catch/finally
    pub(super) fn cf_try_catch(
        &mut self,
        try_body: Vec<ASTNode>,
        catch_clauses: Vec<crate::ast::CatchClause>,
        finally_body: Option<Vec<ASTNode>>,
    ) -> Result<ValueId, String> {
        if std::env::var("NYASH_BUILDER_DISABLE_TRYCATCH")
            .ok()
            .as_deref()
            == Some("1")
        {
            let try_ast = ASTNode::Program {
                statements: try_body,
                span: crate::ast::Span::unknown(),
            };
            let result = self.build_expression(try_ast)?;
            return Ok(result);
        }

        let try_block = self.block_gen.next();
        let catch_block = self.block_gen.next();
        let finally_block = if finally_body.is_some() {
            Some(self.block_gen.next())
        } else {
            None
        };
        let exit_block = self.block_gen.next();

        // Snapshot deferred-return state
        let saved_defer_active = self.return_defer_active;
        let saved_defer_slot = self.return_defer_slot;
        let saved_defer_target = self.return_defer_target;
        let saved_deferred_flag = self.return_deferred_emitted;
        let saved_in_cleanup = self.in_cleanup_block;
        let saved_allow_ret = self.cleanup_allow_return;
        let saved_allow_throw = self.cleanup_allow_throw;

        let ret_slot = self.value_gen.next();
        self.return_defer_active = true;
        self.return_defer_slot = Some(ret_slot);
        self.return_deferred_emitted = false;
        self.return_defer_target = Some(finally_block.unwrap_or(exit_block));

        if let Some(catch_clause) = catch_clauses.first() {
            if std::env::var("NYASH_DEBUG_TRYCATCH").ok().as_deref() == Some("1") {
                eprintln!(
                    "[BUILDER] Emitting catch handler for {:?}",
                    catch_clause.exception_type
                );
            }
            let exception_value = self.value_gen.next();
            self.emit_instruction(MirInstruction::Catch {
                exception_type: catch_clause.exception_type.clone(),
                exception_value,
                handler_bb: catch_block,
            })?;
        }

        // Enter try block
        crate::mir::builder::emission::branch::emit_jump(self, try_block)?;
        self.start_new_block(try_block)?;
        let try_ast = ASTNode::Program {
            statements: try_body,
            span: crate::ast::Span::unknown(),
        };
        let _try_result = self.build_expression(try_ast)?;
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            crate::mir::builder::emission::branch::emit_jump(self, next_target)?;
        }

        // Catch block
        self.start_new_block(catch_block)?;
        if std::env::var("NYASH_DEBUG_TRYCATCH").ok().as_deref() == Some("1") {
            eprintln!("[BUILDER] Enter catch block {:?}", catch_block);
        }
        if let Some(catch_clause) = catch_clauses.first() {
            let catch_ast = ASTNode::Program {
                statements: catch_clause.body.clone(),
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(catch_ast)?;
        }
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            crate::mir::builder::emission::branch::emit_jump(self, next_target)?;
        }

        // Finally
        let mut cleanup_terminated = false;
        if let (Some(finally_block_id), Some(finally_statements)) = (finally_block, finally_body) {
            self.start_new_block(finally_block_id)?;
            self.in_cleanup_block = true;
            self.cleanup_allow_return = crate::config::env::cleanup_allow_return();
            self.cleanup_allow_throw = crate::config::env::cleanup_allow_throw();
            self.return_defer_active = false; // do not defer inside cleanup

            let finally_ast = ASTNode::Program {
                statements: finally_statements,
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(finally_ast)?;
            cleanup_terminated = self.is_current_block_terminated();
            if !cleanup_terminated {
                crate::mir::builder::emission::branch::emit_jump(self, exit_block)?;
            }
            self.in_cleanup_block = false;
        }

        // Exit block
        self.start_new_block(exit_block)?;
        let result = if self.return_deferred_emitted && !cleanup_terminated {
            self.emit_instruction(MirInstruction::Return { value: Some(ret_slot) })?;
            crate::mir::builder::emission::constant::emit_void(self)
        } else {
            crate::mir::builder::emission::constant::emit_void(self)
        };

        // Restore context
        self.return_defer_active = saved_defer_active;
        self.return_defer_slot = saved_defer_slot;
        self.return_defer_target = saved_defer_target;
        self.return_deferred_emitted = saved_deferred_flag;
        self.in_cleanup_block = saved_in_cleanup;
        self.cleanup_allow_return = saved_allow_ret;
        self.cleanup_allow_throw = saved_allow_throw;

        Ok(result)
    }

    /// Control-flow: throw
    pub(super) fn cf_throw(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        if self.in_cleanup_block && !self.cleanup_allow_throw {
            return Err("throw is not allowed inside cleanup block (enable NYASH_CLEANUP_ALLOW_THROW=1 to permit)".to_string());
        }
        if std::env::var("NYASH_BUILDER_DISABLE_THROW").ok().as_deref() == Some("1") {
            let v = self.build_expression(expression)?;
            #[allow(deprecated)]
            // Route to unified global print for debug traces as well
            self.emit_unified_call(
                None,
                super::builder_calls::CallTarget::Global("print".to_string()),
                vec![v],
            )?;
            return Ok(v);
        }
        let exception_value = self.build_expression(expression)?;
        self.emit_instruction(MirInstruction::Throw {
            exception: exception_value,
            effects: EffectMask::PANIC,
        })?;
        Ok(exception_value)
    }
}
