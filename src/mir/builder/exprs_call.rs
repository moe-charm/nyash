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
        let dst = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::Call {
            dst: Some(dst),
            func: callee_id,
            args: arg_ids,
            effects: super::EffectMask::PURE,
        })?;
        Ok(dst)
    }
}
