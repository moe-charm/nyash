use super::*;
use crate::mir::basic_block::BasicBlock;
use std::mem;

impl MirInterpreter {
    fn trace_enabled() -> bool {
        std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_TRACE_EXEC").ok().as_deref() == Some("1")
    }

    pub(super) fn exec_function_inner(
        &mut self,
        func: &MirFunction,
        arg_vals: Option<&[VMValue]>,
    ) -> Result<VMValue, VMError> {
        // Phase 1: delegate cross-class reroute / narrow fallbacks to method_router
        if let Some(r) = super::method_router::pre_exec_reroute(self, func, arg_vals) { return r; }
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

            if Self::trace_enabled() {
                eprintln!(
                    "[vm-trace] enter bb={:?} pred={:?} fn={}",
                    cur,
                    last_pred,
                    self.cur_fn.as_deref().unwrap_or("")
                );
            }

            self.apply_phi_nodes(block, last_pred)?;
            if let Err(e) = self.execute_block_instructions(block) {
                if Self::trace_enabled() {
                    eprintln!(
                        "[vm-trace] error in bb={:?}: {:?}\n  last_inst={:?}",
                        cur, e, self.last_inst
                    );
                }
                return Err(e);
            }

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
                        let v = match self.reg_load(*val) {
                            Ok(v) => v,
                            Err(e) => {
                                // Dev safety valve: tolerate undefined phi inputs by substituting Void
                                if std::env::var("NYASH_VM_PHI_TOLERATE_UNDEFINED").ok().as_deref() == Some("1") {
                                    if Self::trace_enabled() {
                                        eprintln!("[vm-trace] phi tolerate undefined input {:?} -> Void (err={:?})", val, e);
                                    }
                                    VMValue::Void
                                } else {
                                    return Err(e);
                                }
                            }
                        };
                        self.regs.insert(dst_id, v);
                        if Self::trace_enabled() {
                            eprintln!(
                                "[vm-trace] phi dst={:?} take pred={:?} val={:?}",
                                dst_id, pred, val
                            );
                        }
                    }
                } else if let Some((_, val)) = inputs.first() {
                    let v = match self.reg_load(*val) {
                        Ok(v) => v,
                        Err(e) => {
                            if std::env::var("NYASH_VM_PHI_TOLERATE_UNDEFINED").ok().as_deref() == Some("1") {
                                if Self::trace_enabled() {
                                    eprintln!("[vm-trace] phi tolerate undefined default input {:?} -> Void (err={:?})", val, e);
                                }
                                VMValue::Void
                            } else {
                                return Err(e);
                            }
                        }
                    };
                    self.regs.insert(dst_id, v);
                    if Self::trace_enabled() {
                        eprintln!(
                            "[vm-trace] phi dst={:?} take default val={:?}",
                            dst_id, val
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_block_instructions(&mut self, block: &BasicBlock) -> Result<(), VMError> {
        for inst in block.non_phi_instructions() {
            self.last_block = Some(block.id);
            self.last_inst = Some(inst.clone());
            if Self::trace_enabled() {
                eprintln!("[vm-trace] inst bb={:?} {:?}", block.id, inst);
            }
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
