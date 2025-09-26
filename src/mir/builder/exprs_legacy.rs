// Legacy expression lowering kept in a dedicated module to slim down builder.rs
use super::ValueId;
use crate::ast::{ASTNode, Span};

impl super::MirBuilder {
    pub(super) fn build_expression_impl_legacy(
        &mut self,
        ast: ASTNode,
    ) -> Result<ValueId, String> {
        match ast {
            ASTNode::Program { statements, .. } => {
                // Sequentially lower statements and return last value (or Void)
                self.cf_block(statements)
            }
            ASTNode::Print { expression, .. } => {
                self.build_print_statement(*expression)
            }
            ASTNode::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let then_node = ASTNode::Program {
                    statements: then_body,
                    span: Span::unknown(),
                };
                let else_node = else_body.map(|b| ASTNode::Program {
                    statements: b,
                    span: Span::unknown(),
                });
                self.cf_if(*condition, then_node, else_node)
            }
            ASTNode::Loop { condition, body, .. } => {
                self.cf_loop(*condition, body)
            }
            ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                ..
            } => self.cf_try_catch(try_body, catch_clauses, finally_body),

            ASTNode::Throw { expression, .. } => self.cf_throw(*expression),

            other => Err(format!(
                "Unsupported AST in legacy dispatcher: {:?}",
                other
            )),
        }
    }
}
