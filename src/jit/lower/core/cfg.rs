use super::super::builder::IRBuilder;
use super::LowerCore;
use crate::mir::{BasicBlockId, MirFunction, MirInstruction};
use std::collections::HashMap;

impl LowerCore {
    pub(crate) fn build_phi_succords(
        &mut self,
        func: &MirFunction,
        bb_ids: &Vec<BasicBlockId>,
        builder: &mut dyn IRBuilder,
        enable_phi_min: bool,
    ) -> HashMap<BasicBlockId, Vec<crate::mir::ValueId>> {
        let mut succ_phi_order: HashMap<BasicBlockId, Vec<crate::mir::ValueId>> = HashMap::new();
        if !enable_phi_min {
            return succ_phi_order;
        }
        for (bb_id, bb) in func.blocks.iter() {
            let mut order: Vec<crate::mir::ValueId> = Vec::new();
            for ins in bb.instructions.iter() {
                if let MirInstruction::Phi { dst, .. } = ins {
                    order.push(*dst);
                }
            }
            if !order.is_empty() {
                succ_phi_order.insert(*bb_id, order);
            }
        }
        // Pre-declare block parameter counts per successor to avoid late appends
        for (succ, order) in succ_phi_order.iter() {
            if let Some(idx) = bb_ids.iter().position(|x| x == succ) {
                builder.ensure_block_params_i64(idx, order.len());
            }
        }
        succ_phi_order
    }

    pub(crate) fn dump_phi_cfg(
        &self,
        succ_phi_order: &HashMap<BasicBlockId, Vec<crate::mir::ValueId>>,
        func: &MirFunction,
        blocks_len: usize,
        enable_phi_min: bool,
    ) {
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() != Some("1") {
            return;
        }
        let succs = succ_phi_order.len();
        eprintln!(
            "[JIT] cfg: blocks={} phi_succ={} (phi_min={})",
            blocks_len, succs, enable_phi_min
        );
        if enable_phi_min {
            let mut total_phi_slots: usize = 0;
            let mut total_phi_b1_slots: usize = 0;
            for (succ, order) in succ_phi_order.iter() {
                let mut preds_set: std::collections::BTreeSet<i64> =
                    std::collections::BTreeSet::new();
                let mut phi_lines: Vec<String> = Vec::new();
                if let Some(bb_succ) = func.blocks.get(succ) {
                    for ins in bb_succ.instructions.iter() {
                        if let MirInstruction::Phi { dst, inputs } = ins {
                            for (pred, _) in inputs.iter() {
                                preds_set.insert(pred.0 as i64);
                            }
                            let mut pairs: Vec<String> = Vec::new();
                            for (pred, val) in inputs.iter() {
                                pairs.push(format!("{}:{}", pred.0, val.0));
                            }
                            let used_as_branch = func.blocks.values().any(|bbx| {
                                if let Some(MirInstruction::Branch { condition, .. }) =
                                    &bbx.terminator
                                {
                                    condition == dst
                                } else {
                                    false
                                }
                            });
                            let is_b1 = self.bool_phi_values.contains(dst)
                                || inputs.iter().all(|(_, v)| {
                                    self.bool_values.contains(v)
                                        || self
                                            .known_i64
                                            .get(v)
                                            .map(|&iv| iv == 0 || iv == 1)
                                            .unwrap_or(false)
                                })
                                || used_as_branch;
                            if is_b1 {
                                total_phi_b1_slots += 1;
                            }
                            total_phi_slots += 1;
                            phi_lines.push(format!(
                                " phi: bb={} dst={} inputs=[{}] (b1={})",
                                succ.0,
                                dst.0,
                                pairs.join(","),
                                is_b1
                            ));
                        }
                    }
                }
                let preds_list: Vec<String> =
                    preds_set.into_iter().map(|p| p.to_string()).collect();
                eprintln!(
                    "[JIT] phi: bb={} slots={} preds={}",
                    succ.0,
                    order.len(),
                    preds_list.join("|")
                );
                for ln in phi_lines {
                    eprintln!("[JIT]{}", ln);
                }
            }
            eprintln!(
                "[JIT] phi_summary: total_slots={} b1_slots={}",
                total_phi_slots, total_phi_b1_slots
            );
        }
    }
}

impl LowerCore {
    /// Lower a Branch terminator, including fast-path select+return and PHI(min) argument wiring.
    pub(crate) fn lower_branch_terminator(
        &mut self,
        builder: &mut dyn IRBuilder,
        func: &MirFunction,
        bb_ids: &Vec<BasicBlockId>,
        bb_id: BasicBlockId,
        condition: &crate::mir::ValueId,
        then_bb: &BasicBlockId,
        else_bb: &BasicBlockId,
        succ_phi_order: &HashMap<BasicBlockId, Vec<crate::mir::ValueId>>,
        enable_phi_min: bool,
    ) {
        // Fast-path: if both successors immediately return known i64 constants, lower as select+return
        let mut fastpath_done = false;
        let succ_returns_const = |succ: &crate::mir::BasicBlock| -> Option<i64> {
            use crate::mir::MirInstruction as I;
            if let Some(I::Return { value: Some(v) }) = &succ.terminator {
                for ins in succ.instructions.iter() {
                    if let I::Const { dst, value } = ins {
                        if dst == v {
                            if let crate::mir::ConstValue::Integer(k) = value {
                                return Some(*k);
                            }
                        }
                    }
                }
            }
            None
        };
        if let (Some(bb_then), Some(bb_else)) = (func.blocks.get(then_bb), func.blocks.get(else_bb))
        {
            if let (Some(k_then), Some(k_else)) =
                (succ_returns_const(bb_then), succ_returns_const(bb_else))
            {
                self.push_value_if_known_or_param(builder, condition);
                builder.emit_const_i64(k_then);
                builder.emit_const_i64(k_else);
                builder.emit_select_i64();
                builder.emit_return();
                fastpath_done = true;
            }
        }
        if fastpath_done {
            return;
        }

        // Otherwise, emit CFG branch with optional PHI(min) argument wiring
        self.push_value_if_known_or_param(builder, condition);
        let then_index = bb_ids.iter().position(|x| x == then_bb).unwrap_or(0);
        let else_index = bb_ids.iter().position(|x| x == else_bb).unwrap_or(0);
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            eprintln!(
                "[LowerCore] br_if: cur_bb={} then_idx={} else_idx={}",
                bb_id.0, then_index, else_index
            );
        }
        if enable_phi_min {
            let mut then_n = 0usize;
            let mut else_n = 0usize;
            if let Some(order) = succ_phi_order.get(then_bb) {
                let mut cnt = 0usize;
                if let Some(bb_succ) = func.blocks.get(then_bb) {
                    for dst in order.iter() {
                        for ins in bb_succ.instructions.iter() {
                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                if d2 == dst {
                                    if let Some((_, val)) =
                                        inputs.iter().find(|(pred, _)| pred == &bb_id)
                                    {
                                        self.push_value_if_known_or_param(builder, val);
                                        cnt += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if cnt > 0 {
                    builder.ensure_block_params_i64(then_index, cnt);
                }
                then_n = cnt;
            }
            if let Some(order) = succ_phi_order.get(else_bb) {
                let mut cnt = 0usize;
                if let Some(bb_succ) = func.blocks.get(else_bb) {
                    for dst in order.iter() {
                        for ins in bb_succ.instructions.iter() {
                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                if d2 == dst {
                                    if let Some((_, val)) =
                                        inputs.iter().find(|(pred, _)| pred == &bb_id)
                                    {
                                        self.push_value_if_known_or_param(builder, val);
                                        cnt += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if cnt > 0 {
                    builder.ensure_block_params_i64(else_index, cnt);
                }
                else_n = cnt;
            }
            builder.br_if_with_args(then_index, else_index, then_n, else_n);
        } else {
            builder.br_if_top_is_true(then_index, else_index);
        }
    }

    /// Lower a Jump terminator with optional PHI(min) argument wiring.
    pub(crate) fn lower_jump_terminator(
        &mut self,
        builder: &mut dyn IRBuilder,
        func: &MirFunction,
        bb_ids: &Vec<BasicBlockId>,
        bb_id: BasicBlockId,
        target: &BasicBlockId,
        succ_phi_order: &HashMap<BasicBlockId, Vec<crate::mir::ValueId>>,
        enable_phi_min: bool,
    ) {
        let target_index = bb_ids.iter().position(|x| x == target).unwrap_or(0);
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            eprintln!(
                "[LowerCore] jump: cur_bb={} target_idx={}",
                bb_id.0, target_index
            );
        }
        if enable_phi_min {
            let mut n = 0usize;
            if let Some(order) = succ_phi_order.get(target) {
                let mut cnt = 0usize;
                if let Some(bb_succ) = func.blocks.get(target) {
                    for dst in order.iter() {
                        for ins in bb_succ.instructions.iter() {
                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                if d2 == dst {
                                    if let Some((_, val)) =
                                        inputs.iter().find(|(pred, _)| pred == &bb_id)
                                    {
                                        self.push_value_if_known_or_param(builder, val);
                                        cnt += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if cnt > 0 {
                    builder.ensure_block_params_i64(target_index, cnt);
                }
                n = cnt;
            }
            builder.jump_with_args(target_index, n);
        } else {
            builder.jump_to(target_index);
        }
    }
}
