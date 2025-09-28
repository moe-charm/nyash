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

        // Phase 3.1: Use unified call with CallTarget::Value for indirect calls
        let use_unified = super::calls::call_unified::is_unified_call_enabled();

        if use_unified {
            // New unified path - use emit_unified_call with Value target
            let dst = self.value_gen.next();
            self.emit_unified_call(
                Some(dst),
                super::builder_calls::CallTarget::Value(callee_id),
                arg_ids,
            )?;
            Ok(dst)
        } else {
            // Legacy path - keep for compatibility
            let dst = self.value_gen.next();
            self.emit_instruction(super::MirInstruction::Call {
                dst: Some(dst),
                func: callee_id,
                callee: None, // Legacy call expression - use old resolution
                args: arg_ids,
                effects: super::EffectMask::PURE,
            })?;
            Ok(dst)
        }
    }
}
