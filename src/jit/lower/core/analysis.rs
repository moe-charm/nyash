use std::collections::HashSet;

use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};

// removed unused imports
use super::LowerCore;

impl LowerCore {
    pub(crate) fn analyze(&mut self, func: &MirFunction, bb_ids: &Vec<BasicBlockId>) {
        // Seed boolean lattice with boolean parameters from MIR signature
        if !func.signature.params.is_empty() {
            for (idx, vid) in func.params.iter().copied().enumerate() {
                if let Some(mt) = func.signature.params.get(idx) {
                    if matches!(mt, crate::mir::MirType::Bool) {
                        self.bool_values.insert(vid);
                    }
                }
            }
        }
        // Pre-scan to classify boolean-producing values and propagate via Copy/Phi/Load-Store heuristics.
        self.bool_values.clear();
        let mut copy_edges: Vec<(ValueId, ValueId)> = Vec::new();
        let mut phi_defs: Vec<(ValueId, Vec<ValueId>)> = Vec::new();
        let mut stores: Vec<(ValueId, ValueId)> = Vec::new(); // (ptr, value)
        let mut loads: Vec<(ValueId, ValueId)> = Vec::new(); // (dst, ptr)
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    match ins {
                        MirInstruction::Compare { dst, .. } => {
                            self.bool_values.insert(*dst);
                        }
                        MirInstruction::Const { dst, value } => {
                            if let crate::mir::ConstValue::Bool(_) = value {
                                self.bool_values.insert(*dst);
                            }
                        }
                        MirInstruction::Copy { dst, src } => {
                            copy_edges.push((*dst, *src));
                        }
                        MirInstruction::Phi { dst, inputs } => {
                            self.phi_values.insert(*dst);
                            let ins: Vec<ValueId> = inputs.iter().map(|(_, v)| *v).collect();
                            phi_defs.push((*dst, ins));
                        }
                        MirInstruction::Store { ptr, value } => {
                            stores.push((*ptr, *value));
                        }
                        MirInstruction::Load { dst, ptr } => {
                            loads.push((*dst, *ptr));
                        }
                        _ => {}
                    }
                }
            }
        }
        // Fixed-point propagation
        let mut store_bool_ptrs: HashSet<ValueId> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            // Copy propagation
            for (dst, src) in copy_edges.iter().copied() {
                if self.bool_values.contains(&src) && !self.bool_values.contains(&dst) {
                    self.bool_values.insert(dst);
                    changed = true;
                }
                if store_bool_ptrs.contains(&src) && !store_bool_ptrs.contains(&dst) {
                    store_bool_ptrs.insert(dst);
                    changed = true;
                }
            }
            // Store marking
            for (ptr, val) in stores.iter().copied() {
                if self.bool_values.contains(&val) && !store_bool_ptrs.contains(&ptr) {
                    store_bool_ptrs.insert(ptr);
                    changed = true;
                }
            }
            // Load propagation
            for (dst, ptr) in loads.iter().copied() {
                if store_bool_ptrs.contains(&ptr) && !self.bool_values.contains(&dst) {
                    self.bool_values.insert(dst);
                    changed = true;
                }
            }
            // PHI closure for value booleans
            for (dst, inputs) in phi_defs.iter() {
                if inputs.iter().all(|v| self.bool_values.contains(v))
                    && !self.bool_values.contains(dst)
                {
                    self.bool_values.insert(*dst);
                    self.bool_phi_values.insert(*dst);
                    changed = true;
                }
            }
            // PHI closure for pointer aliases
            for (dst, inputs) in phi_defs.iter() {
                if inputs.iter().all(|v| store_bool_ptrs.contains(v))
                    && !store_bool_ptrs.contains(dst)
                {
                    store_bool_ptrs.insert(*dst);
                    changed = true;
                }
            }
        }
        // PHI statistics
        let mut total_phi_slots: usize = 0;
        let mut total_phi_b1_slots: usize = 0;
        for (dst, inputs) in phi_defs.iter() {
            total_phi_slots += 1;
            let used_as_branch = func.blocks.values().any(|bbx| {
                if let Some(MirInstruction::Branch { condition, .. }) = &bbx.terminator {
                    condition == dst
                } else {
                    false
                }
            });
            let is_b1 = self.bool_phi_values.contains(dst)
                || inputs.iter().all(|v| {
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
        }
        if total_phi_slots > 0 {
            crate::jit::rt::phi_total_inc(total_phi_slots as u64);
            crate::jit::rt::phi_b1_inc(total_phi_b1_slots as u64);
            self.last_phi_total = total_phi_slots as u64;
            self.last_phi_b1 = total_phi_b1_slots as u64;
        }
    }
}
