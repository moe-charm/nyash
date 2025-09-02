use std::collections::HashMap;

use crate::mir::{BasicBlockId, MirFunction, MirInstruction};
use super::super::builder::IRBuilder;
use super::LowerCore;

impl LowerCore {
    pub(crate) fn build_phi_succords(
        &mut self,
        func: &MirFunction,
        bb_ids: &Vec<BasicBlockId>,
        builder: &mut dyn IRBuilder,
        enable_phi_min: bool,
    ) -> HashMap<BasicBlockId, Vec<crate::mir::ValueId>> {
        let mut succ_phi_order: HashMap<BasicBlockId, Vec<crate::mir::ValueId>> = HashMap::new();
        if !enable_phi_min { return succ_phi_order; }
        for (bb_id, bb) in func.blocks.iter() {
            let mut order: Vec<crate::mir::ValueId> = Vec::new();
            for ins in bb.instructions.iter() {
                if let MirInstruction::Phi { dst, .. } = ins { order.push(*dst); }
            }
            if !order.is_empty() { succ_phi_order.insert(*bb_id, order); }
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
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() != Some("1") { return; }
        let succs = succ_phi_order.len();
        eprintln!("[JIT] cfg: blocks={} phi_succ={} (phi_min={})", blocks_len, succs, enable_phi_min);
        if enable_phi_min {
            let mut total_phi_slots: usize = 0;
            let mut total_phi_b1_slots: usize = 0;
            for (succ, order) in succ_phi_order.iter() {
                let mut preds_set: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
                let mut phi_lines: Vec<String> = Vec::new();
                if let Some(bb_succ) = func.blocks.get(succ) {
                    for ins in bb_succ.instructions.iter() {
                        if let MirInstruction::Phi { dst, inputs } = ins {
                            for (pred, _) in inputs.iter() { preds_set.insert(pred.0 as i64); }
                            let mut pairs: Vec<String> = Vec::new();
                            for (pred, val) in inputs.iter() { pairs.push(format!("{}:{}", pred.0, val.0)); }
                            let used_as_branch = func.blocks.values().any(|bbx| {
                                if let Some(MirInstruction::Branch { condition, .. }) = &bbx.terminator { condition == dst } else { false }
                            });
                            let is_b1 = self.bool_phi_values.contains(dst)
                                || inputs.iter().all(|(_, v)| {
                                    self.bool_values.contains(v) || self.known_i64.get(v).map(|&iv| iv == 0 || iv == 1).unwrap_or(false)
                                })
                                || used_as_branch;
                            if is_b1 { total_phi_b1_slots += 1; }
                            total_phi_slots += 1;
                            phi_lines.push(format!(" phi: bb={} dst={} inputs=[{}] (b1={})",
                                succ.0, dst.0, pairs.join(","), is_b1));
                        }
                    }
                }
                let preds_list: Vec<String> = preds_set.into_iter().map(|p| p.to_string()).collect();
                eprintln!("[JIT] phi: bb={} slots={} preds={}", succ.0, order.len(), preds_list.join("|"));
                for ln in phi_lines { eprintln!("[JIT]{}", ln); }
            }
            eprintln!("[JIT] phi_summary: total_slots={} b1_slots={}", total_phi_slots, total_phi_b1_slots);
        }
    }
}

