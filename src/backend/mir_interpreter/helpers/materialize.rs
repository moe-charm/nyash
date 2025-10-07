use super::super::*;

impl MirInterpreter {
    /// LocalSSA-style materialization at runtime: if the given `src` value
    /// has been copied to a new SSA id earlier in the current basic block,
    /// return that latest in-block dst id. Otherwise, return `src` unchanged.
    ///
    /// Contract:
    /// - Only considers non-PHI instructions inside the current block
    /// - Scans up to (but not including) the current executing instruction (`last_inst`)
    /// - Picks the latest Copy(dst := src) if multiple are present
    ///
    /// This mirrors the builder's LocalSSA contract and makes the VM tolerant
    /// to occasional upstream misses without changing semantics.
    pub(in crate::backend::mir_interpreter) fn materialize_value_in_current_block(&self, src: ValueId) -> ValueId {
        let (fn_name, bb_id, last_inst) = match (self.cur_fn.as_ref(), self.last_block, self.last_inst.as_ref()) {
            (Some(f), Some(bb), Some(inst)) => (f.clone(), bb, inst.clone()),
            _ => return src,
        };
        let func = match self.functions.get(&fn_name) { Some(f) => f, None => return src };
        let bb = match func.blocks.get(&bb_id) { Some(b) => b, None => return src };
        // Work on non-PHI instructions in program order and stop at current instruction.
        let instrs: Vec<_> = bb.non_phi_instructions().collect();
        let stop = instrs
            .iter()
            .position(|i| **i == last_inst)
            .unwrap_or(instrs.len());
        let mut latest: Option<ValueId> = None;
        for inst in instrs.iter().take(stop) {
            if let crate::mir::MirInstruction::Copy { dst, src: s } = &**inst {
                if *s == src {
                    latest = Some(*dst);
                }
            }
        }
        latest.unwrap_or(src)
    }

    /// Materialize a slice of argument ValueIds within the current block.
    /// Debug builds assert that any changed id is Copy-defined in the block before current inst.
    pub(in crate::backend::mir_interpreter) fn materialize_args_in_current_block(&self, args: &[ValueId]) -> Vec<ValueId> {
        let out: Vec<ValueId> = args
            .iter()
            .map(|a| self.materialize_value_in_current_block(*a))
            .collect();
        #[cfg(debug_assertions)]
        {
            for (orig, got) in args.iter().zip(out.iter()) {
                self.debug_assert_materialized_in_block(*orig, *got);
            }
        }
        out
    }

    /// Materialize a receiver id within the current block and (in debug) validate its provenance.
    pub(in crate::backend::mir_interpreter) fn materialize_recv_in_current_block(&self, recv: ValueId) -> ValueId {
        let v = self.materialize_value_in_current_block(recv);
        #[cfg(debug_assertions)]
        self.debug_assert_materialized_in_block(recv, v);
        v
    }

    #[cfg(debug_assertions)]
    pub(in crate::backend::mir_interpreter) fn debug_assert_materialized_in_block(&self, original: ValueId, got: ValueId) {
        if original == got { return; }
        let (fn_name, bb_id, last_inst) = match (self.cur_fn.as_ref(), self.last_block, self.last_inst.as_ref()) {
            (Some(f), Some(bb), Some(inst)) => (f.clone(), bb, inst.clone()),
            _ => return,
        };
        if let Some(func) = self.functions.get(&fn_name) {
            if let Some(bb) = func.blocks.get(&bb_id) {
                let instrs: Vec<_> = bb.non_phi_instructions().collect();
                let stop = instrs.iter().position(|i| **i == last_inst).unwrap_or(instrs.len());
                let mut ok = false;
                for inst in instrs.iter().take(stop) {
                    if let crate::mir::MirInstruction::Copy { dst, .. } = &**inst {
                        if *dst == got { ok = true; break; }
                    }
                }
                assert!(ok, "materialized id {:?} must be Copy-defined in current block before call", got);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, BasicBlockId, FunctionSignature, MirFunction, MirInstruction, MirType, ValueId};
    use crate::mir::EffectMask;

    fn make_func_with_block(instrs: Vec<MirInstruction>) -> (MirFunction, BasicBlockId, MirInstruction) {
        let sig = FunctionSignature { name: "F.test".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
        let entry = BasicBlockId::new(0);
        let mut f = MirFunction::new(sig, entry);
        let mut b = BasicBlock::new(entry);
        for i in instrs.iter().cloned() { b.add_instruction(i); }
        let last = b.instructions.last().cloned().unwrap_or(MirInstruction::Nop);
        f.blocks.insert(entry, b);
        (f, entry, last)
    }

    #[test]
    fn materialize_picks_latest_copy_before_current_inst() {
        let s = ValueId::new(1);
        let v1 = ValueId::new(2);
        let v2 = ValueId::new(3);
        let instrs = vec![
            MirInstruction::Copy { dst: v1, src: s },
            MirInstruction::Copy { dst: v2, src: s },
            MirInstruction::Nop,
        ];
        let (func, bb, last_inst) = make_func_with_block(instrs);
        let mut interp = MirInterpreter::new();
        interp.functions.insert("F.test".into(), func);
        interp.cur_fn = Some("F.test".into());
        interp.last_block = Some(bb);
        interp.last_inst = Some(last_inst);
        let got = interp.materialize_value_in_current_block(s);
        assert_eq!(got, v2);
    }

    #[test]
    fn materialize_stops_at_current_inst() {
        let s = ValueId::new(10);
        let v1 = ValueId::new(11);
        let v2 = ValueId::new(12);
        let entry = BasicBlockId::new(0);
        let sig = FunctionSignature { name: "F.test".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
        let mut func = MirFunction::new(sig, entry);
        let mut bb = BasicBlock::new(entry);
        bb.add_instruction(MirInstruction::Copy { dst: v1, src: s });
        let nop = MirInstruction::Nop;
        bb.add_instruction(nop.clone());
        let last_inst = bb.instructions.last().cloned().unwrap();
        bb.add_instruction(MirInstruction::Copy { dst: v2, src: s });
        func.blocks.insert(entry, bb);

        let mut interp = MirInterpreter::new();
        interp.functions.insert("F.test".into(), func);
        interp.cur_fn = Some("F.test".into());
        interp.last_block = Some(entry);
        interp.last_inst = Some(last_inst);
        let got = interp.materialize_value_in_current_block(s);
        assert_eq!(got, v1);
    }
}
