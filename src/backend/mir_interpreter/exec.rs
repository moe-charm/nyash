use super::*;
use crate::mir::basic_block::BasicBlock;
use std::mem;

impl MirInterpreter {
    pub(super) fn exec_function_inner(
        &mut self,
        func: &MirFunction,
        arg_vals: Option<&[VMValue]>,
    ) -> Result<VMValue, VMError> {
        let saved_regs = mem::take(&mut self.regs);
        let saved_fn = self.cur_fn.clone();
        self.cur_fn = Some(func.signature.name.clone());

        if let Some(args) = arg_vals {
            for (i, pid) in func.params.iter().enumerate() {
                let v = args.get(i).cloned().unwrap_or(VMValue::Void);
                self.regs.insert(*pid, v);
            }
        }

        let mut cur = func.entry_block;
        let mut last_pred: Option<BasicBlockId> = None;

        loop {
            let block = func
                .blocks
                .get(&cur)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("bb {:?} not found", cur)))?;

            self.apply_phi_nodes(block, last_pred)?;
            self.execute_block_instructions(block)?;

            match self.handle_terminator(block)? {
                BlockOutcome::Return(result) => {
                    self.cur_fn = saved_fn;
                    self.regs = saved_regs;
                    return Ok(result);
                }
                BlockOutcome::Next {
                    target,
                    predecessor,
                } => {
                    last_pred = Some(predecessor);
                    cur = target;
                }
            }
        }
    }

    fn apply_phi_nodes(
        &mut self,
        block: &BasicBlock,
        last_pred: Option<BasicBlockId>,
    ) -> Result<(), VMError> {
        for inst in block.phi_instructions() {
            if let MirInstruction::Phi { dst, inputs } = inst {
                let dst_id = *dst;
                if let Some(pred) = last_pred {
                    if let Some((_, val)) = inputs.iter().find(|(bb, _)| *bb == pred) {
                        let v = self.reg_load(*val)?;
                        self.regs.insert(dst_id, v);
                    }
                } else if let Some((_, val)) = inputs.first() {
                    let v = self.reg_load(*val)?;
                    self.regs.insert(dst_id, v);
                }
            }
        }
        Ok(())
    }

    fn execute_block_instructions(&mut self, block: &BasicBlock) -> Result<(), VMError> {
        for inst in block.non_phi_instructions() {
            self.execute_instruction(inst)?;
        }
        Ok(())
    }

    fn handle_terminator(&mut self, block: &BasicBlock) -> Result<BlockOutcome, VMError> {
        match &block.terminator {
            Some(MirInstruction::Return { value }) => {
                let result = if let Some(v) = value {
                    self.reg_load(*v)?
                } else {
                    VMValue::Void
                };
                Ok(BlockOutcome::Return(result))
            }
            Some(MirInstruction::Jump { target }) => Ok(BlockOutcome::Next {
                target: *target,
                predecessor: block.id,
            }),
            Some(MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            }) => {
                let cond = self.reg_load(*condition)?;
                let branch = to_bool_vm(&cond).map_err(VMError::TypeError)?;
                let target = if branch { *then_bb } else { *else_bb };
                Ok(BlockOutcome::Next {
                    target,
                    predecessor: block.id,
                })
            }
            None => Err(VMError::InvalidBasicBlock(format!(
                "unterminated block {:?}",
                block.id
            ))),
            Some(other) => Err(VMError::InvalidInstruction(format!(
                "invalid terminator in MIR interp: {:?}",
                other
            ))),
        }
    }
}

enum BlockOutcome {
    Return(VMValue),
    Next {
        target: BasicBlockId,
        predecessor: BasicBlockId,
    },
}
