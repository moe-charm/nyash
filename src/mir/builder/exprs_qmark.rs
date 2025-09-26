use super::ValueId;
use crate::ast::ASTNode;

impl super::MirBuilder {
    // QMarkPropagate: result?.value (Result-like)
    pub(super) fn build_qmark_propagate_expression(
        &mut self,
        expression: ASTNode,
    ) -> Result<ValueId, String> {
        let res_val = self.build_expression_impl(expression)?;
        let ok_id = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::BoxCall {
            dst: Some(ok_id),
            box_val: res_val,
            method: "isOk".to_string(),
            args: vec![],
            method_id: None,
            effects: super::EffectMask::PURE,
        })?;
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        self.emit_instruction(super::MirInstruction::Branch {
            condition: ok_id,
            then_bb: then_block,
            else_bb: else_block,
        })?;
        self.start_new_block(then_block)?;
        self.emit_instruction(super::MirInstruction::Return {
            value: Some(res_val),
        })?;
        self.start_new_block(else_block)?;
        let val_id = self.value_gen.next();
        self.emit_instruction(super::MirInstruction::BoxCall {
            dst: Some(val_id),
            box_val: res_val,
            method: "getValue".to_string(),
            args: vec![],
            method_id: None,
            effects: super::EffectMask::PURE,
        })?;
        Ok(val_id)
    }
}
