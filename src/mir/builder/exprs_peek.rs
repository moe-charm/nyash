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
        // Evaluate scrutinee in the current block
        let scr_val = self.build_expression_impl(scrutinee)?;

        // Prepare merge and result
        let merge_block: BasicBlockId = self.block_gen.next();
        let result_val = self.safe_next_value();
        let mut phi_inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();

        // Create dispatch block where we start comparing arms
        let dispatch_block = self.block_gen.next();
        // Jump from current block to dispatch (ensure terminator exists)
        let need_jump = {
            let cur = self.current_block;
            if let (Some(cb), Some(ref func)) = (cur, &self.current_function) {
                if let Some(bb) = func.blocks.get(&cb) {
                    !bb.is_terminated()
                } else {
                    true
                }
            } else {
                true
            }
        };
        if need_jump {
            crate::mir::builder::emission::branch::emit_jump(self, dispatch_block)?;
        }
        self.start_new_block(dispatch_block)?;

        // If there are no arms, fall through to else directly
        if arms.is_empty() {
            let else_block = self.block_gen.next();
            crate::mir::builder::emission::branch::emit_jump(self, else_block)?;
            self.start_new_block(else_block)?;
            let else_val = self.build_expression_impl(else_expr)?;
            phi_inputs.push((else_block, else_val));
            crate::mir::builder::emission::branch::emit_jump(self, merge_block)?;
            self.start_new_block(merge_block)?;
            // フェーズM: 常にPHI命令を使用（no_phi_mode撤廃）
            self.emit_instruction(super::MirInstruction::Phi {
                dst: result_val,
                inputs: phi_inputs,
            })?;
            return Ok(result_val);
        }

        // Else block to handle default case
        let else_block = self.block_gen.next();

        // Chain dispatch blocks for each arm
        let mut cur_dispatch = dispatch_block;
        for (i, (label, arm_expr)) in arms.iter().cloned().enumerate() {
            let then_block = self.block_gen.next();
            // Next dispatch (only for non-last arm)
            let next_dispatch = if i + 1 < arms.len() {
                Some(self.block_gen.next())
            } else {
                None
            };
            let else_target = next_dispatch.unwrap_or(else_block);

            // In current dispatch block, compare and branch
            self.start_new_block(cur_dispatch)?;
            let lit_id = match label {
                LiteralValue::String(s) => crate::mir::builder::emission::constant::emit_string(self, s),
                LiteralValue::Integer(i) => crate::mir::builder::emission::constant::emit_integer(self, i),
                LiteralValue::Bool(b) => crate::mir::builder::emission::constant::emit_bool(self, b),
                LiteralValue::Float(f) => crate::mir::builder::emission::constant::emit_float(self, f),
                LiteralValue::Null => crate::mir::builder::emission::constant::emit_null(self),
                LiteralValue::Void => crate::mir::builder::emission::constant::emit_void(self),
            };
            let cond_id = self.safe_next_value();
            crate::mir::builder::emission::compare::emit_to(self, cond_id, super::CompareOp::Eq, scr_val, lit_id)?;
            crate::mir::builder::emission::branch::emit_conditional(self, cond_id, then_block, else_target)?;

            // then arm
            self.start_new_block(then_block)?;
            let cur_then = self.current_block.expect("then block must be current");
            let then_val = self.build_expression_impl(arm_expr)?;
            // If arm terminates (Return/Throw), do not contribute to PHI and do not emit jump.
            let terminated = self.is_block_terminated(cur_then);
            if !terminated {
                phi_inputs.push((then_block, then_val));
                crate::mir::builder::emission::branch::emit_jump(self, merge_block)?;
            }

            // Move to next dispatch or else block
            cur_dispatch = else_target;
        }

        // Lower else expression in else_block
        self.start_new_block(else_block)?;
        let else_val = self.build_expression_impl(else_expr)?;
        phi_inputs.push((else_block, else_val));
        crate::mir::builder::emission::branch::emit_jump(self, merge_block)?;

        // Merge and yield result
        self.start_new_block(merge_block)?;
        // フェーズM: 常にPHI命令を使用（no_phi_mode撤廃）
        self.emit_instruction(super::MirInstruction::Phi {
            dst: result_val,
            inputs: phi_inputs,
        })?;
        Ok(result_val)
    }
}
