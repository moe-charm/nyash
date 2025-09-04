use super::{BasicBlockId, ValueId};
use crate::ast::{ASTNode, LiteralValue};

impl super::MirBuilder {
    // Peek expression lowering
    pub(super) fn build_peek_expression(
        &mut self,
        scrutinee: ASTNode,
        arms: Vec<(LiteralValue, ASTNode)>,
        else_expr: ASTNode,
    ) -> Result<ValueId, String> {
        let scr_val = self.build_expression_impl(scrutinee)?;
        let merge_block: BasicBlockId = self.block_gen.next();
        let result_val = self.value_gen.next();
        let mut phi_inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
        let mut next_block = self.block_gen.next();
        self.start_new_block(next_block)?;

        for (i, (label, arm_expr)) in arms.iter().cloned().enumerate() {
            let then_block = self.block_gen.next();
            // Only string labels handled here (behavior unchanged)
            if let LiteralValue::String(s) = label {
                let lit_id = self.value_gen.next();
                self.emit_instruction(super::MirInstruction::Const { dst: lit_id, value: super::ConstValue::String(s) })?;
                let cond_id = self.value_gen.next();
                self.emit_instruction(super::MirInstruction::Compare { dst: cond_id, op: super::CompareOp::Eq, lhs: scr_val, rhs: lit_id })?;
                self.emit_instruction(super::MirInstruction::Branch { condition: cond_id, then_bb: then_block, else_bb: next_block })?;
                self.start_new_block(then_block)?;
                let then_val = self.build_expression_impl(arm_expr)?;
                phi_inputs.push((then_block, then_val));
                self.emit_instruction(super::MirInstruction::Jump { target: merge_block })?;
                if i < arms.len() - 1 {
                    let b = self.block_gen.next();
                    self.start_new_block(b)?;
                    next_block = b;
                }
            }
        }

        let else_block_id = next_block;
        self.start_new_block(else_block_id)?;
        let else_val = self.build_expression_impl(else_expr)?;
        phi_inputs.push((else_block_id, else_val));
        self.emit_instruction(super::MirInstruction::Jump { target: merge_block })?;
        self.start_new_block(merge_block)?;
        self.emit_instruction(super::MirInstruction::Phi { dst: result_val, inputs: phi_inputs })?;
        Ok(result_val)
    }
}

