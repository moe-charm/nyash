use super::ValueId;
use crate::ast::ASTNode;

impl super::MirBuilder {
    // Indirect call: (callee)(args...)
    pub(super) fn build_indirect_call_expression(
        &mut self,
        callee: ASTNode,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        let callee_id = self.build_expression_impl(callee)?;
        let mut arg_ids: Vec<ValueId> = Vec::new();
        for a in arguments {
            arg_ids.push(self.build_expression_impl(a)?);
        }

        // Always use unified call with Value target (legacy callee=None deprecated)
        let dst = self.safe_next_value();
        self.emit_unified_call(
            Some(dst),
            super::builder_calls::CallTarget::Value(callee_id),
            arg_ids,
        )?;
        Ok(dst)
    }
}
