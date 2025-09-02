use super::{MirInstruction, EffectMask, Effect, ConstValue, ValueId};
use crate::ast::ASTNode;

impl super::MirBuilder {
    pub(super) fn build_print_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        self.build_print_statement_legacy(expression)
    }
    pub(super) fn build_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        self.build_block_legacy(statements)
    }
    pub(super) fn build_if_statement(&mut self, condition: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Result<ValueId, String> {
        self.build_if_statement_legacy(condition, then_branch, else_branch)
    }
    pub(super) fn build_loop_statement(&mut self, condition: ASTNode, body: Vec<ASTNode>) -> Result<ValueId, String> {
        self.build_loop_statement_legacy(condition, body)
    }
    pub(super) fn build_try_catch_statement(&mut self, try_body: Vec<ASTNode>, catch_clauses: Vec<crate::ast::CatchClause>, finally_body: Option<Vec<ASTNode>>) -> Result<ValueId, String> {
        self.build_try_catch_statement_legacy(try_body, catch_clauses, finally_body)
    }
    pub(super) fn build_throw_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        self.build_throw_statement_legacy(expression)
    }
    pub(super) fn build_local_statement(&mut self, variables: Vec<String>, initial_values: Vec<Option<Box<ASTNode>>>) -> Result<ValueId, String> {
        self.build_local_statement_legacy(variables, initial_values)
    }
    pub(super) fn build_return_statement(&mut self, value: Option<Box<ASTNode>>) -> Result<ValueId, String> {
        self.build_return_statement_legacy(value)
    }
    pub(super) fn build_nowait_statement(&mut self, variable: String, expression: ASTNode) -> Result<ValueId, String> {
        self.build_nowait_statement_legacy(variable, expression)
    }
    pub(super) fn build_await_expression(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        self.build_await_expression_legacy(expression)
    }
    pub(super) fn build_me_expression(&mut self) -> Result<ValueId, String> {
        self.build_me_expression_legacy()
    }
}

