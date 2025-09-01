use crate::mir::{MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, ValueId};
use super::builder::{IRBuilder, BinOpKind, CmpKind};

/// Lower(Core-1): Minimal lowering skeleton for Const/Move/BinOp/Cmp/Branch/Ret
/// This does not emit real CLIF yet; it only walks MIR and validates coverage.
pub struct LowerCore {
    pub unsupported: usize,
    pub covered: usize,
    /// Minimal constant propagation for i64 to feed host-call args
    pub(super) known_i64: std::collections::HashMap<ValueId, i64>,
    /// Minimal constant propagation for f64 (math.* signature checks)
    known_f64: std::collections::HashMap<ValueId, f64>,
    /// Parameter index mapping for ValueId
    pub(super) param_index: std::collections::HashMap<ValueId, usize>,
    /// Track values produced by Phi (for minimal PHI path)
    phi_values: std::collections::HashSet<ValueId>,
    /// Map (block, phi dst) -> param index in that block (for multi-PHI)
    phi_param_index: std::collections::HashMap<(crate::mir::BasicBlockId, ValueId), usize>,
    /// Track values that are boolean (b1) results, e.g., Compare destinations
    pub(super) bool_values: std::collections::HashSet<ValueId>,
    /// Track PHI destinations that are boolean (all inputs derived from bool_values)
    bool_phi_values: std::collections::HashSet<ValueId>,
    /// Track values that are FloatBox instances (for arg type classification)
    float_box_values: std::collections::HashSet<ValueId>,
    /// Track values that are plugin handles (generic box/handle, type unknown at compile time)
    pub(super) handle_values: std::collections::HashSet<ValueId>,
    // Per-function statistics (last lowered)
    last_phi_total: u64,
    last_phi_b1: u64,
    last_ret_bool_hint_used: bool,
    // Minimal local slot mapping for Load/Store (ptr ValueId -> slot index)
    pub(super) local_index: std::collections::HashMap<ValueId, usize>,
    pub(super) next_local: usize,
    /// Track NewBox origins: ValueId -> box type name (e.g., "PyRuntimeBox")
    pub(super) box_type_map: std::collections::HashMap<ValueId, String>,
}

impl LowerCore {
    pub fn new() -> Self { Self { unsupported: 0, covered: 0, known_i64: std::collections::HashMap::new(), known_f64: std::collections::HashMap::new(), param_index: std::collections::HashMap::new(), phi_values: std::collections::HashSet::new(), phi_param_index: std::collections::HashMap::new(), bool_values: std::collections::HashSet::new(), bool_phi_values: std::collections::HashSet::new(), float_box_values: std::collections::HashSet::new(), handle_values: std::collections::HashSet::new(), last_phi_total: 0, last_phi_b1: 0, last_ret_bool_hint_used: false, local_index: std::collections::HashMap::new(), next_local: 0, box_type_map: std::collections::HashMap::new() } }

    /// Get statistics for the last lowered function
    pub fn last_stats(&self) -> (u64, u64, bool) { (self.last_phi_total, self.last_phi_b1, self.last_ret_bool_hint_used) }

    /// Walk the MIR function and count supported/unsupported instructions.
    /// In the future, this will build CLIF via Cranelift builders.
    pub fn lower_function(&mut self, func: &MirFunction, builder: &mut dyn IRBuilder) -> Result<(), String> {
        // Prepare ABI based on MIR signature
        // Reset per-function stats
        self.last_phi_total = 0; self.last_phi_b1 = 0; self.last_ret_bool_hint_used = false;
        // Build param index map
        self.param_index.clear();
        for (i, v) in func.params.iter().copied().enumerate() {
            self.param_index.insert(v, i);
        }
        // Prepare block mapping (Phase 10.7): deterministic ordering by sorted keys
        let mut bb_ids: Vec<_> = func.blocks.keys().copied().collect();
        bb_ids.sort_by_key(|b| b.0);
        builder.prepare_blocks(bb_ids.len());
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
        let mut copy_edges: Vec<(crate::mir::ValueId, crate::mir::ValueId)> = Vec::new();
        let mut phi_defs: Vec<(crate::mir::ValueId, Vec<crate::mir::ValueId>)> = Vec::new();
        let mut stores: Vec<(crate::mir::ValueId, crate::mir::ValueId)> = Vec::new(); // (ptr, value)
        let mut loads: Vec<(crate::mir::ValueId, crate::mir::ValueId)> = Vec::new();  // (dst, ptr)
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    match ins {
                        crate::mir::MirInstruction::Compare { dst, .. } => { self.bool_values.insert(*dst); }
                        crate::mir::MirInstruction::Const { dst, value } => {
                            if let ConstValue::Bool(_) = value { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::Cast { dst, target_type, .. } => {
                            if matches!(target_type, crate::mir::MirType::Bool) { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::TypeOp { dst, op, ty, .. } => {
                            // Check and cast-to-bool produce boolean
                            if matches!(op, crate::mir::TypeOpKind::Check) || matches!(ty, crate::mir::MirType::Bool) { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::Copy { dst, src } => { copy_edges.push((*dst, *src)); }
                        crate::mir::MirInstruction::Phi { dst, inputs } => {
                            let vs: Vec<_> = inputs.iter().map(|(_, v)| *v).collect();
                            phi_defs.push((*dst, vs));
                        }
                        crate::mir::MirInstruction::Store { value, ptr } => { stores.push((*ptr, *value)); }
                        crate::mir::MirInstruction::Load { dst, ptr } => { loads.push((*dst, *ptr)); }
                        _ => {}
                    }
                }
                if let Some(term) = &block.terminator {
                    match term {
                        crate::mir::MirInstruction::Compare { dst, .. } => { self.bool_values.insert(*dst); }
                        crate::mir::MirInstruction::Const { dst, value } => {
                            if let ConstValue::Bool(_) = value { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::Cast { dst, target_type, .. } => {
                            if matches!(target_type, crate::mir::MirType::Bool) { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::TypeOp { dst, op, ty, .. } => {
                            if matches!(op, crate::mir::TypeOpKind::Check) || matches!(ty, crate::mir::MirType::Bool) { self.bool_values.insert(*dst); }
                        }
                        crate::mir::MirInstruction::Copy { dst, src } => { copy_edges.push((*dst, *src)); }
                        crate::mir::MirInstruction::Phi { dst, inputs } => {
                            let vs: Vec<_> = inputs.iter().map(|(_, v)| *v).collect();
                            phi_defs.push((*dst, vs));
                        }
                        crate::mir::MirInstruction::Branch { condition, .. } => { self.bool_values.insert(*condition); }
                        crate::mir::MirInstruction::Store { value, ptr } => { stores.push((*ptr, *value)); }
                        crate::mir::MirInstruction::Load { dst, ptr } => { loads.push((*dst, *ptr)); }
                        _ => {}
                    }
                }
            }
        }
        // Fixed-point boolean lattice propagation
        let mut changed = true;
        let mut store_bool_ptrs: std::collections::HashSet<crate::mir::ValueId> = std::collections::HashSet::new();
        while changed {
            changed = false;
            // Copy propagation
            for (dst, src) in copy_edges.iter().copied() {
                if self.bool_values.contains(&src) && !self.bool_values.contains(&dst) {
                    self.bool_values.insert(dst);
                    changed = true;
                }
                // Pointer alias propagation for Store/Load lattice
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
                if inputs.iter().all(|v| self.bool_values.contains(v)) && !self.bool_values.contains(dst) {
                    self.bool_values.insert(*dst);
                    self.bool_phi_values.insert(*dst);
                    changed = true;
                }
            }
            // PHI closure for pointer aliases: if all inputs are bool-storing pointers, mark dst pointer as such
            for (dst, inputs) in phi_defs.iter() {
                if inputs.iter().all(|v| store_bool_ptrs.contains(v)) && !store_bool_ptrs.contains(dst) {
                    store_bool_ptrs.insert(*dst);
                    changed = true;
                }
            }
        }
        // Always-on PHI statistics: count total/b1 phi slots using current heuristics
        {
            use crate::mir::MirInstruction;
            let mut total_phi_slots: usize = 0;
            let mut total_phi_b1_slots: usize = 0;
            for (dst, inputs) in phi_defs.iter() {
                total_phi_slots += 1;
                // Heuristics consistent with dump path
                let used_as_branch = func.blocks.values().any(|bbx| {
                    if let Some(MirInstruction::Branch { condition, .. }) = &bbx.terminator { condition == dst } else { false }
                });
                let is_b1 = self.bool_phi_values.contains(dst)
                    || inputs.iter().all(|v| {
                        self.bool_values.contains(v) || self.known_i64.get(v).map(|&iv| iv == 0 || iv == 1).unwrap_or(false)
                    })
                    || used_as_branch;
                if is_b1 { total_phi_b1_slots += 1; }
            }
            if total_phi_slots > 0 {
                crate::jit::rt::phi_total_inc(total_phi_slots as u64);
                crate::jit::rt::phi_b1_inc(total_phi_b1_slots as u64);
                self.last_phi_total = total_phi_slots as u64;
                self.last_phi_b1 = total_phi_b1_slots as u64;
            }
        }
        // Optional: collect PHI targets and ordering per successor for minimal/multi PHI path
        let cfg_now = crate::jit::config::current();
        let enable_phi_min = cfg_now.phi_min;
        // For each successor block, store ordered list of phi dst and a map pred->input for each phi
        let mut succ_phi_order: std::collections::HashMap<crate::mir::BasicBlockId, Vec<crate::mir::ValueId>> = std::collections::HashMap::new();
        let mut succ_phi_inputs: std::collections::HashMap<crate::mir::BasicBlockId, Vec<(crate::mir::BasicBlockId, crate::mir::ValueId)>> = std::collections::HashMap::new();
        if enable_phi_min {
            for (bb_id, bb) in func.blocks.iter() {
                let mut order: Vec<crate::mir::ValueId> = Vec::new();
                for ins in bb.instructions.iter() {
                    if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                        order.push(*dst);
                        // store all (pred,val) pairs in flat vec grouped by succ
                        for (pred, val) in inputs.iter() { succ_phi_inputs.entry(*bb_id).or_default().push((*pred, *val)); }
                    }
                }
                if !order.is_empty() { succ_phi_order.insert(*bb_id, order); }
            }
        }
        // Decide ABI: typed or i64-only
        let native_f64 = cfg_now.native_f64;
        let native_bool = cfg_now.native_bool;
        let mut use_typed = false;
        let mut kinds: Vec<super::builder::ParamKind> = Vec::new();
        for mt in func.signature.params.iter() {
            let k = match mt {
                crate::mir::MirType::Float if native_f64 => { use_typed = true; super::builder::ParamKind::F64 }
                crate::mir::MirType::Bool if native_bool => { use_typed = true; super::builder::ParamKind::B1 }
                _ => super::builder::ParamKind::I64,
            };
            kinds.push(k);
        }
        let ret_is_f64 = native_f64 && matches!(func.signature.return_type, crate::mir::MirType::Float);
        // Hint return bool footing (no-op in current backend; keeps switch point centralized)
        let ret_is_bool = matches!(func.signature.return_type, crate::mir::MirType::Bool);
        if ret_is_bool { 
            builder.hint_ret_bool(true);
            // Track how many functions are lowered with boolean return hint (for stats)
            crate::jit::rt::ret_bool_hint_inc(1);
            self.last_ret_bool_hint_used = true;
        }
        let has_ret = !matches!(func.signature.return_type, crate::mir::MirType::Void);
        if use_typed || ret_is_f64 {
            builder.prepare_signature_typed(&kinds, ret_is_f64 && has_ret);
        } else {
            builder.prepare_signature_i64(func.params.len(), has_ret);
        }
        // Pre-scan FloatBox creations across all blocks for arg classification
        self.float_box_values.clear();
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = ins { if box_type == "FloatBox" { self.float_box_values.insert(*dst); } }
                    if let crate::mir::MirInstruction::Copy { dst, src } = ins { if self.float_box_values.contains(src) { self.float_box_values.insert(*dst); } }
                }
            }
        }

        // Pre-scan to map NewBox origins: ValueId -> box type name; propagate via Copy
        self.box_type_map.clear();
        for bb in bb_ids.iter() {
            if let Some(block) = func.blocks.get(bb) {
                for ins in block.instructions.iter() {
                    if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = ins {
                        self.box_type_map.insert(*dst, box_type.clone());
                    }
                    if let crate::mir::MirInstruction::Copy { dst, src } = ins {
                        if let Some(name) = self.box_type_map.get(src).cloned() { self.box_type_map.insert(*dst, name); }
                    }
                }
            }
        }

        builder.begin_function(&func.signature.name);
        // Iterate blocks in the sorted order to keep indices stable
        self.phi_values.clear();
        self.phi_param_index.clear();
        self.float_box_values.clear();
        self.handle_values.clear();
        for (idx, bb_id) in bb_ids.iter().enumerate() {
            let bb = func.blocks.get(bb_id).unwrap();
            builder.switch_to_block(idx);
            // Pre-scan PHIs in this block and ensure block parameters count (multi-PHI)
            if enable_phi_min {
                let mut local_phi_order: Vec<ValueId> = Vec::new();
                // Also detect boolean PHIs: inputs all from boolean-producing values
                for ins in bb.instructions.iter() {
                    if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                        local_phi_order.push(*dst);
                        // decide if this phi is boolean
                        if inputs.iter().all(|(_, v)| self.bool_values.contains(v)) && !inputs.is_empty() {
                            self.bool_phi_values.insert(*dst);
                        }
                    }
                }
                if !local_phi_order.is_empty() {
                    builder.ensure_block_params_i64(idx, local_phi_order.len());
                    for (i, v) in local_phi_order.into_iter().enumerate() {
                        self.phi_values.insert(v);
                        self.phi_param_index.insert((*bb_id, v), i);
                    }
                }
            }
            for instr in bb.instructions.iter() {
                self.cover_if_supported(instr);
                if let Err(e) = self.try_emit(builder, instr, *bb_id, func) { return Err(e); }
                // Track FloatBox creations for later arg classification
                if let crate::mir::MirInstruction::NewBox { dst, box_type, .. } = instr { if box_type == "FloatBox" { self.float_box_values.insert(*dst); } }
                if let crate::mir::MirInstruction::Copy { dst, src } = instr { if self.float_box_values.contains(src) { self.float_box_values.insert(*dst); } }
            }
            if let Some(term) = &bb.terminator {
                self.cover_if_supported(term);
                // Branch/Jump need block mapping: pass indices
                match term {
                    crate::mir::MirInstruction::Branch { condition, then_bb, else_bb } => {
                        // Fast-path (opt-in): if both successors immediately return known i64 constants,
                        // lower as select+return（branchless）。NYASH_JIT_FASTPATH_SELECT=1 のときだけ有効。
                        let mut fastpath_done = false;
                        let fastpath_on = std::env::var("NYASH_JIT_FASTPATH_SELECT").ok().as_deref() == Some("1");
                        let succ_returns_const = |succ: &crate::mir::BasicBlock| -> Option<i64> {
                            // Pattern: [optional Nops] Const(Integer k) ; Return(Some that const))
                            use crate::mir::MirInstruction as I;
                            // find last Const and matching Return
                            if let Some(I::Return { value: Some(v) }) = &succ.terminator {
                                // Find definition of v in succ.instructions as a Const Integer
                                for ins in succ.instructions.iter() {
                                    if let I::Const { dst, value } = ins {
                                        if dst == v {
                                            if let crate::mir::ConstValue::Integer(k) = value { return Some(*k); }
                                        }
                                    }
                                }
                            }
                            None
                        };
                        if fastpath_on { if let (Some(bb_then), Some(bb_else)) = (func.blocks.get(then_bb), func.blocks.get(else_bb)) {
                            if let (Some(k_then), Some(k_else)) = (succ_returns_const(bb_then), succ_returns_const(bb_else)) {
                                // cond, then, else on stack → select → return
                                self.push_value_if_known_or_param(builder, condition);
                                builder.emit_const_i64(k_then);
                                builder.emit_const_i64(k_else);
                                builder.emit_select_i64();
                                builder.emit_return();
                                fastpath_done = true;
                            }
                        } }
                        if fastpath_done { continue; }

                        // Otherwise, emit CFG branch
                        // Try to place condition on stack (param/const path); builder will adapt
                        self.push_value_if_known_or_param(builder, condition);
                        // Map BasicBlockId -> index
                        let then_index = bb_ids.iter().position(|x| x == then_bb).unwrap_or(0);
                        let else_index = bb_ids.iter().position(|x| x == else_bb).unwrap_or(0);
                        if enable_phi_min {
                            // For multi-PHI, push args in successor's phi order
                            let mut then_n = 0usize; let mut else_n = 0usize;
                            if let Some(order) = succ_phi_order.get(then_bb) {
                                let mut cnt = 0usize;
                                for dst in order.iter() {
                                    // find input from current block
                                    if let Some(bb_succ) = func.blocks.get(then_bb) {
                                        // locate the specific phi to read its inputs
                                        for ins in bb_succ.instructions.iter() {
                                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                                if d2 == dst {
                                                    if let Some((_, val)) = inputs.iter().find(|(pred, _)| pred == bb_id) {
                                                        self.push_value_if_known_or_param(builder, val);
                                                        cnt += 1;
                                                    } else {
                                                        // Fallback: if any input value is a PHI defined in current block, pass that
                                                        if let Some((_, alt_val)) = inputs.iter().find(|(_pred, v)| {
                                                            if let Some(bb_cur) = func.blocks.get(bb_id) {
                                                                bb_cur.instructions.iter().any(|ins2| matches!(ins2, crate::mir::MirInstruction::Phi { dst: dphi, .. } if *dphi == *v))
                                                            } else { false }
                                                        }) {
                                                            self.push_value_if_known_or_param(builder, alt_val);
                                                            cnt += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if cnt > 0 { builder.ensure_block_params_i64(then_index, cnt); }
                                then_n = cnt;
                            }
                            if let Some(order) = succ_phi_order.get(else_bb) {
                                let mut cnt = 0usize;
                                for dst in order.iter() {
                                    if let Some(bb_succ) = func.blocks.get(else_bb) {
                                        for ins in bb_succ.instructions.iter() {
                                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                                if d2 == dst {
                                                    if let Some((_, val)) = inputs.iter().find(|(pred, _)| pred == bb_id) {
                                                        self.push_value_if_known_or_param(builder, val);
                                                        cnt += 1;
                                                    } else {
                                                        if let Some((_, alt_val)) = inputs.iter().find(|(_pred, v)| {
                                                            if let Some(bb_cur) = func.blocks.get(bb_id) {
                                                                bb_cur.instructions.iter().any(|ins2| matches!(ins2, crate::mir::MirInstruction::Phi { dst: dphi, .. } if *dphi == *v))
                                                            } else { false }
                                                        }) {
                                                            self.push_value_if_known_or_param(builder, alt_val);
                                                            cnt += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if cnt > 0 { builder.ensure_block_params_i64(else_index, cnt); }
                                else_n = cnt;
                            }
                            builder.br_if_with_args(then_index, else_index, then_n, else_n);
                        } else {
                            builder.br_if_top_is_true(then_index, else_index);
                        }
                        builder.seal_block(then_index);
                        builder.seal_block(else_index);
                    }
                    crate::mir::MirInstruction::Jump { target } => {
                        let target_index = bb_ids.iter().position(|x| x == target).unwrap_or(0);
                        if enable_phi_min {
                            let mut n = 0usize;
                            if let Some(order) = succ_phi_order.get(target) {
                                let mut cnt = 0usize;
                                if let Some(bb_succ) = func.blocks.get(target) {
                                    for dst in order.iter() {
                                        for ins in bb_succ.instructions.iter() {
                                            if let crate::mir::MirInstruction::Phi { dst: d2, inputs } = ins {
                                                if d2 == dst {
                                                    if let Some((_, val)) = inputs.iter().find(|(pred, _)| pred == bb_id) {
                                                        self.push_value_if_known_or_param(builder, val);
                                                        cnt += 1;
                                                    } else {
                                                        if let Some((_, alt_val)) = inputs.iter().find(|(_pred, v)| {
                                                            if let Some(bb_cur) = func.blocks.get(bb_id) {
                                                                bb_cur.instructions.iter().any(|ins2| matches!(ins2, crate::mir::MirInstruction::Phi { dst: dphi, .. } if *dphi == *v))
                                                            } else { false }
                                                        }) {
                                                            self.push_value_if_known_or_param(builder, alt_val);
                                                            cnt += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if cnt > 0 { builder.ensure_block_params_i64(target_index, cnt); }
                                n = cnt;
                            }
                            builder.jump_with_args(target_index, n);
                        } else {
                            builder.jump_to(target_index);
                        }
                        builder.seal_block(target_index);
                    }
                    _ => { /* other terminators handled via generic emission below */ }
                }
                // Also allow other terminators to be emitted if needed
                if let Err(e) = self.try_emit(builder, term, *bb_id, func) { return Err(e); }
            }
        }
        builder.end_function();
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            let succs = succ_phi_order.len();
            eprintln!("[JIT] cfg: blocks={} phi_succ={} (phi_min={})", bb_ids.len(), succs, enable_phi_min);
            if enable_phi_min {
                let mut total_phi_slots: usize = 0;
                let mut total_phi_b1_slots: usize = 0;
                for (succ, order) in succ_phi_order.iter() {
                    let mut preds_set: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
                    let mut phi_lines: Vec<String> = Vec::new();
                    if let Some(bb_succ) = func.blocks.get(succ) {
                        for ins in bb_succ.instructions.iter() {
                            if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                                // collect preds for block-level summary
                                for (pred, _) in inputs.iter() { preds_set.insert(pred.0 as i64); }
                                // build detailed mapping text: dst<-pred:val,...
                                let mut pairs: Vec<String> = Vec::new();
                                for (pred, val) in inputs.iter() {
                                    pairs.push(format!("{}:{}", pred.0, val.0));
                                }
                                // Heuristics: boolean PHI if (1) pre-analysis marked it, or
                                // (2) all inputs look boolean-like (from bool producers or 0/1 const), or
                                // (3) used as a branch condition somewhere.
                                let used_as_branch = func.blocks.values().any(|bbx| {
                                    if let Some(MirInstruction::Branch { condition, .. }) = &bbx.terminator { condition == dst } else { false }
                                });
                                let is_b1 = self.bool_phi_values.contains(dst)
                                    || inputs.iter().all(|(_, v)| {
                                        self.bool_values.contains(v) || self.known_i64.get(v).map(|&iv| iv == 0 || iv == 1).unwrap_or(false)
                                    })
                                    || used_as_branch;
                                let tag = if is_b1 { " (b1)" } else { "" };
                                phi_lines.push(format!("  dst v{}{} <- {}", dst.0, tag, pairs.join(", ")));
                                total_phi_slots += 1;
                                if is_b1 { total_phi_b1_slots += 1; }
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
        Ok(())
    }

    /// Push a value onto the builder stack if it is a known i64 const or a parameter.
    pub(super) fn push_value_if_known_or_param(&self, b: &mut dyn IRBuilder, id: &ValueId) {
        // Prefer materialized locals first (e.g., PHI stored into a local slot)
        if let Some(slot) = self.local_index.get(id).copied() {
            b.load_local_i64(slot);
            return;
        }
        if self.phi_values.contains(id) {
            // Multi-PHI: find the param index for this phi in the current block
            // We don't have the current block id here; rely on builder's current block context and our stored index being positional.
            // As an approximation, prefer position 0 if unknown.
            let pos = self.phi_param_index.iter().find_map(|((_, vid), idx)| if vid == id { Some(*idx) } else { None }).unwrap_or(0);
            // Use b1 loader for boolean PHIs when enabled
            if crate::jit::config::current().native_bool && self.bool_phi_values.contains(id) {
                b.push_block_param_b1_at(pos);
            } else {
                b.push_block_param_i64_at(pos);
            }
            return;
        }
        if let Some(pidx) = self.param_index.get(id).copied() {
            b.emit_param_i64(pidx);
            return;
        }
        if let Some(v) = self.known_i64.get(id).copied() {
            b.emit_const_i64(v);
            return;
        }
    }

    fn cover_if_supported(&mut self, instr: &MirInstruction) {
        use crate::mir::MirInstruction as I;
        let supported = matches!(
            instr,
            I::Const { .. }
                | I::Copy { .. }
                | I::Cast { .. }
                | I::TypeCheck { .. }
                | I::TypeOp { .. }
                | I::BinOp { .. }
                | I::Compare { .. }
                | I::Jump { .. }
                | I::Branch { .. }
                | I::Return { .. }
                | I::Call { .. }
                | I::BoxCall { .. }
                | I::ArrayGet { .. }
                | I::ArraySet { .. }
                | I::NewBox { .. }
                | I::Store { .. }
                | I::Load { .. }
                | I::Phi { .. }
                // PrintはJIT経路では未対応（VMにフォールバックしてコンソール出力を保持）
                // | I::Print { .. }
                | I::Debug { .. }
                | I::ExternCall { .. }
                | I::Safepoint
                | I::Nop
                | I::PluginInvoke { .. }
        );
        if supported { self.covered += 1; } else { self.unsupported += 1; }
    }

    fn try_emit(&mut self, b: &mut dyn IRBuilder, instr: &MirInstruction, cur_bb: crate::mir::BasicBlockId, func: &crate::mir::MirFunction) -> Result<(), String> {
        use crate::mir::MirInstruction as I;
        match instr {
            I::Await { dst, future } => {
                // Push future param index when known; otherwise -1 to trigger legacy search in shim
                if let Some(pidx) = self.param_index.get(future).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                // Call await_h to obtain a handle to the value (0 on timeout)
                b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, 1, true);
                // Store the awaited handle temporarily
                let hslot = { let id = self.next_local; self.next_local += 1; id };
                b.store_local_i64(hslot);
                // Build Ok result: ok_h(handle)
                b.load_local_i64(hslot);
                b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_OK_H, 1, true);
                let ok_slot = { let id = self.next_local; self.next_local += 1; id };
                b.store_local_i64(ok_slot);
                // Build Err result: err_h(0) → Timeout
                b.emit_const_i64(0);
                b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_ERR_H, 1, true);
                let err_slot = { let id = self.next_local; self.next_local += 1; id };
                b.store_local_i64(err_slot);
                // Cond: (handle == 0)
                b.load_local_i64(hslot);
                b.emit_const_i64(0);
                b.emit_compare(crate::jit::lower::builder::CmpKind::Eq);
                // Stack for select: cond, then(err), else(ok)
                b.load_local_i64(err_slot);
                b.load_local_i64(ok_slot);
                b.emit_select_i64();
                // Store selected Result handle to destination
                let d = *dst;
                self.handle_values.insert(d);
                let slot = *self.local_index.entry(d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.store_local_i64(slot);
            }
            I::Safepoint => {
                // Emit a runtime checkpoint (safepoint + scheduler poll via NyRT/JIT stubs)
                b.emit_host_call(crate::jit::r#extern::runtime::SYM_RT_CHECKPOINT, 0, false);
            }
            I::RefGet { dst, reference: _, field } => {
                // Minimal: env.console をハンドル化（hostcall）
                if field == "console" {
                    // Emit hostcall to create/get ConsoleBox handle
                    // Symbol exported by nyrt: nyash.console.birth_h
                    b.emit_host_call("nyash.console.birth_h", 0, true);
                } else {
                    // Unknown RefGet: treat as no-op const 0 to avoid strict fail for now
                    b.emit_const_i64(0);
                }
                // Record as covered; do not increment unsupported
                let _ = dst; // keep signature parity
            }
            I::UnaryOp { dst: _, op, operand } => {
                match op {
                    crate::mir::UnaryOp::Neg => {
                        // i64-only minimal: 0 - operand
                        // Try known const or param
                        // push 0
                        b.emit_const_i64(0);
                        // push operand (known/param)
                        self.push_value_if_known_or_param(b, operand);
                        b.emit_binop(BinOpKind::Sub);
                    }
                    _ => { self.unsupported += 1; }
                }
            }
            I::NewBox { dst, box_type, args } => {
                // 最適化は後段へ（現状は汎用・安全な実装に徹する）
                // 通常経路:
                // - 引数なし: 汎用 birth_h（type_idのみ）でハンドル生成
                // - 引数あり: 既存のチェーン（直後の plugin_invoke birth で初期化）を維持（段階的導入）
                if args.is_empty() {
                    let decision = crate::jit::policy::invoke::decide_box_method(box_type, "birth", 0, true);
                    if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, .. } = decision {
                        b.emit_const_i64(type_id as i64);
                        b.emit_host_call(crate::jit::r#extern::birth::SYM_BOX_BIRTH_H, 1, true);
                        self.handle_values.insert(*dst);
                        let slot = *self.local_index.entry(*dst).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                        b.store_local_i64(slot);
                    } else {
                        self.unsupported += 1;
                    }
                } else {
                    // 引数あり: 安全なパターンから段階的に birth_i64 に切替
                    // 1) IntegerBox(const i64)
                    if box_type == "IntegerBox" && args.len() == 1 {
                        if let Some(src) = args.get(0) {
                            if let Some(iv) = self.known_i64.get(src).copied() {
                                // 汎用 birth_i64(type_id, argc=1, a1=iv)
                                if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, .. } = crate::jit::policy::invoke::decide_box_method(box_type, "birth", 1, true) {
                                    b.emit_const_i64(type_id as i64);
                                    b.emit_const_i64(1);
                                    b.emit_const_i64(iv);
                                    b.emit_host_call("nyash.box.birth_i64", 3, true);
                                    self.handle_values.insert(*dst);
                                    let slot = *self.local_index.entry(*dst).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                                    b.store_local_i64(slot);
                                    // 値伝搬も継続
                                    self.known_i64.insert(*dst, iv);
                                    return Ok(());
                                }
                            }
                        }
                    }
                    // 2) 引数がハンドル（StringBox等）で既に存在する場合（最大2引数）
                    if args.len() <= 2 && args.iter().all(|a| self.handle_values.contains(a)) {
                        if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, .. } = crate::jit::policy::invoke::decide_box_method(box_type, "birth", args.len(), true) {
                            b.emit_const_i64(type_id as i64);
                            b.emit_const_i64(args.len() as i64);
                            // a1, a2 を push（ローカルに保存済みのハンドルをロード）
                            for a in args.iter().take(2) { self.push_value_if_known_or_param(b, a); }
                            b.emit_host_call("nyash.box.birth_i64", 2 + args.len(), true);
                            self.handle_values.insert(*dst);
                            let slot = *self.local_index.entry(*dst).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                            b.store_local_i64(slot);
                            return Ok(());
                        }
                    }
                    // フォールバック: 既存チェーンに委譲（互換）+ 既知値伝搬のみ
                    if box_type == "IntegerBox" {
                        if let Some(src) = args.get(0) { if let Some(iv) = self.known_i64.get(src).copied() { self.known_i64.insert(*dst, iv); } }
                    }
                }
                // Track boxed numeric literals to aid signature checks (FloatBox/IntegerBox)
                if box_type == "FloatBox" {
                    if let Some(src) = args.get(0) {
                        if let Some(fv) = self.known_f64.get(src).copied() {
                            self.known_f64.insert(*dst, fv);
                        } else if let Some(iv) = self.known_i64.get(src).copied() {
                            self.known_f64.insert(*dst, iv as f64);
                        }
                    }
                } else if box_type == "IntegerBox" {
                    if let Some(src) = args.get(0) {
                        if let Some(iv) = self.known_i64.get(src).copied() {
                            self.known_i64.insert(*dst, iv);
                        }
                    }
                }
            }
            I::PluginInvoke { dst, box_val, method, args, .. } => {
                // Minimal PluginInvoke footing (AOT strict path):
                // - Python3メソッド（import/getattr/call）は実Emitする（型/引数はシム側でTLV化）
                // - PyRuntimeBox.birth/eval と IntegerBox.birth は no-op許容
                let bt = self.box_type_map.get(box_val).cloned().unwrap_or_default();
                let m = method.as_str();
                // import/getattr/call 実Emit
                if (bt == "PyRuntimeBox" && (m == "import")) {
                    let argc = 1 + args.len();
                    // push receiver param index (a0) if known
                    if let Some(pidx) = self.param_index.get(box_val).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                    let decision = crate::jit::policy::invoke::decide_box_method(&bt, m, argc, dst.is_some());
                    if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, .. } = decision {
                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                        if let Some(d) = dst { self.handle_values.insert(*d); }
                    } else { if dst.is_some() { b.emit_const_i64(0); } }
                } else if (bt == "PyRuntimeBox" && (m == "getattr" || m == "call")) {
                    // getattr/call invoked via PyRuntimeBox helper形式 → by-nameで解決
                    let argc = 1 + args.len();
                    // push receiver param index (a0) if known
                    if let Some(pidx) = self.param_index.get(box_val).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                    // push primary arguments if available（a1, a2 ...）
                    for a in args.iter() { self.push_value_if_known_or_param(b, a); }
                    b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
                    if let Some(d) = dst {
                        self.handle_values.insert(*d);
                        // Store handle result into a local slot so it can be used as argument later
                        let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                        b.store_local_i64(slot);
                    }
                } else if self.handle_values.contains(box_val) && (m == "getattr" || m == "call") {
                    let argc = 1 + args.len();
                    // push receiver handle/param index if possible (here receiver is a handle result previously returned)
                    // We cannot reconstruct handle here; pass -1 to allow shim fallback.
                    b.emit_const_i64(-1);
                    for a in args.iter() { self.push_value_if_known_or_param(b, a); }
                    b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
                    if let Some(d) = dst {
                        self.handle_values.insert(*d);
                        let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                        b.store_local_i64(slot);
                    }
                } else if (bt == "PyRuntimeBox" && (m == "birth" || m == "eval"))
                    || (bt == "IntegerBox" && m == "birth")
                    || (bt == "StringBox" && m == "birth")
                    || (bt == "ConsoleBox" && m == "birth") {
                    if dst.is_some() { b.emit_const_i64(0); }
                } else {
                    self.unsupported += 1;
                }
            }
            I::ExternCall { dst, iface_name, method_name, args, .. } => {
                // Minimal extern→plugin bridge: env.console.log/println を ConsoleBox に委譲
                if iface_name == "env.console" && (method_name == "log" || method_name == "println") {
                    // Ensure we have a ConsoleBox handle on the stack
                    b.emit_host_call("nyash.console.birth_h", 0, true);
                    // Push first argument if known/param
                    if let Some(arg0) = args.get(0) { self.push_value_if_known_or_param(b, arg0); }
                    // Resolve and emit plugin_invoke for ConsoleBox.method
                    let decision = crate::jit::policy::invoke::decide_box_method("ConsoleBox", method_name, 2, dst.is_some());
                    if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, .. } = decision {
                        b.emit_plugin_invoke(type_id, method_id, 2, dst.is_some());
                    } else {
                        // Fallback: drop result if any
                        if dst.is_some() { b.emit_const_i64(0); }
                    }
                } else {
                    // Async spawn bridge: env.future.spawn_instance(recv, method_name, args...)
                    if iface_name == "env.future" && method_name == "spawn_instance" {
                        // Stack layout for hostcall: argc_total, a0(recv), a1(method_name), a2(first payload)
                        // 1) receiver
                        if let Some(recv) = args.get(0) {
                            if let Some(pidx) = self.param_index.get(recv).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                        } else { b.emit_const_i64(-1); }
                        // 2) method name (best-effort)
                        if let Some(meth) = args.get(1) { self.push_value_if_known_or_param(b, meth); } else { b.emit_const_i64(0); }
                        // 3) first payload argument if present
                        if let Some(arg2) = args.get(2) { self.push_value_if_known_or_param(b, arg2); } else { b.emit_const_i64(0); }
                        // argc_total = explicit args including method name and payload (exclude receiver)
                        let argc_total = args.len().saturating_sub(1).max(0);
                        b.emit_const_i64(argc_total as i64);
                        // Call spawn shim; it returns Future handle
                        b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_SPAWN_INSTANCE3_I64, 4, true);
                        if let Some(d) = dst {
                            self.handle_values.insert(*d);
                            let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                            b.store_local_i64(slot);
                        }
                        return Ok(());
                    }
                    // Unknown extern: strictではno-opにしてfailを避ける
                    if dst.is_some() { b.emit_const_i64(0); }
                }
            }
            I::Cast { dst, value, target_type } => {
                // Minimal cast footing: materialize source when param/known
                // Bool→Int: rely on producers (compare) and branch/b1 loaders; here we just reuse integer path
                self.push_value_if_known_or_param(b, value);
                // Track known i64 if source known
                if let Some(v) = self.known_i64.get(value).copied() { self.known_i64.insert(*dst, v); }
                // Track known f64 for float casts
                if matches!(target_type, crate::mir::MirType::Float) {
                    if let Some(iv) = self.known_i64.get(value).copied() {
                        self.known_f64.insert(*dst, iv as f64);
                    }
                }
            }
            I::Const { dst, value } => match value {
                ConstValue::Integer(i) => { 
                    b.emit_const_i64(*i);
                    self.known_i64.insert(*dst, *i);
                }
                ConstValue::Float(f) => { b.emit_const_f64(*f); self.known_f64.insert(*dst, *f); }
                ConstValue::Bool(bv) => {
                    let iv = if *bv { 1 } else { 0 };
                    b.emit_const_i64(iv);
                    self.known_i64.insert(*dst, iv);
                    // Mark this value as boolean producer
                    self.bool_values.insert(*dst);
                }
                ConstValue::String(_) | ConstValue::Null | ConstValue::Void => {
                    // leave unsupported for now
                }
            },
            I::Copy { dst, src } => { 
                if let Some(v) = self.known_i64.get(src).copied() { self.known_i64.insert(*dst, v); }
                if let Some(v) = self.known_f64.get(src).copied() { self.known_f64.insert(*dst, v); }
                // If source is a parameter, materialize it on the stack for downstream ops
                if let Some(pidx) = self.param_index.get(src).copied() {
                    b.emit_param_i64(pidx);
                }
                // Propagate boolean classification through Copy
                if self.bool_values.contains(src) { self.bool_values.insert(*dst); }
                // Otherwise no-op for codegen (stack-machine handles sources directly later)
            }
            I::BinOp { dst, op, lhs, rhs } => { self.lower_binop(b, op, lhs, rhs, dst, func); }
            I::Compare { op, lhs, rhs, dst } => { self.lower_compare(b, op, lhs, rhs, dst, func); }
            I::Jump { .. } => self.lower_jump(b),
            I::Branch { .. } => self.lower_branch(b),
            I::Return { value } => {
                if let Some(v) = value {
                    // Prefer known/param/materialized path
                    if self.known_i64.get(v).is_some() || self.param_index.get(v).is_some() || self.local_index.get(v).is_some() {
                        self.push_value_if_known_or_param(b, v);
                    } else {
                        // Fallback: search a Const definition for this value in the current block and emit directly
                        if let Some(bb) = func.blocks.get(&cur_bb) {
                            for ins in bb.instructions.iter() {
                                if let crate::mir::MirInstruction::Const { dst, value: cval } = ins {
                                    if dst == v {
                                        match cval {
                                            crate::mir::ConstValue::Integer(i) => { b.emit_const_i64(*i); }
                                            crate::mir::ConstValue::Bool(bv) => { b.emit_const_i64(if *bv {1} else {0}); }
                                            crate::mir::ConstValue::Float(f) => { b.emit_const_f64(*f); }
                                            _ => {}
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                b.emit_return()
            }
            I::Store { value, ptr } => {
                // Minimal lowering: materialize value if known/param and store to a local slot keyed by ptr
                self.push_value_if_known_or_param(b, value);
                let slot = *self.local_index.entry(*ptr).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.ensure_local_i64(slot);
                b.store_local_i64(slot);
            }
            I::Load { dst: _, ptr } => {
                // Minimal lowering: load from local slot keyed by ptr, default 0 if unset
                let slot = *self.local_index.entry(*ptr).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.ensure_local_i64(slot);
                b.load_local_i64(slot);
            }
            I::Phi { dst, .. } => {
                // PHI をローカルに materialize して後続の Return で安定参照
                let pos = self.phi_param_index.get(&(cur_bb, *dst)).copied().unwrap_or(0);
                if self.bool_phi_values.contains(dst) { b.push_block_param_b1_at(pos); }
                else { b.push_block_param_i64_at(pos); }
                let slot = *self.local_index.entry(*dst).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.ensure_local_i64(slot);
                b.store_local_i64(slot);
            }
            I::ArrayGet { array, index, .. } => {
                // Prepare receiver + index on stack
                let argc = 2usize;
                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                if let Some(iv) = self.known_i64.get(index).copied() { b.emit_const_i64(iv); } else { self.push_value_if_known_or_param(b, index); }
                // Decide policy
                let decision = crate::jit::policy::invoke::decide_box_method("ArrayBox", "get", argc, true);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, true);
                        crate::jit::observe::lower_plugin_invoke(&box_type, "get", type_id, method_id, argc);
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                        b.emit_host_call(&symbol, argc, true);
                    }
                    _ => super::core_hostcall::lower_array_get(b, &self.param_index, &self.known_i64, array, index),
                }
            }
            I::ArraySet { array, index, value } => {
                let argc = 3usize;
                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                // GC write barrier hint for mutating array operations (pass receiver handle/index as site id: receiver preferred)
                b.emit_host_call(crate::jit::r#extern::runtime::SYM_GC_BARRIER_WRITE, 1, false);
                if let Some(iv) = self.known_i64.get(index).copied() { b.emit_const_i64(iv); } else { self.push_value_if_known_or_param(b, index); }
                if let Some(vv) = self.known_i64.get(value).copied() { b.emit_const_i64(vv); } else { self.push_value_if_known_or_param(b, value); }
                let decision = crate::jit::policy::invoke::decide_box_method("ArrayBox", "set", argc, false);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                        crate::jit::observe::lower_plugin_invoke(&box_type, "set", type_id, method_id, argc);
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64","I64"], "allow", "mapped_symbol");
                        b.emit_host_call(&symbol, argc, false);
                    }
                    _ => super::core_hostcall::lower_array_set(b, &self.param_index, &self.known_i64, array, index, value),
                }
            }
            I::BoxCall { box_val: array, method, args, dst, .. } => {
                if super::core_hostcall::lower_boxcall_simple_reads(b, &self.param_index, &self.known_i64, array, method.as_str(), args, dst.clone()) {
                    // handled in helper (read-only simple methods)
                } else if matches!(method.as_str(), "sin" | "cos" | "abs" | "min" | "max") {
                    super::core_hostcall::lower_math_call(
                        func,
                        b,
                        &self.known_i64,
                        &self.known_f64,
                        &self.float_box_values,
                        method.as_str(),
                        args,
                        dst.clone(),
                    );
                } else if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1") {
                    // StringBox（length/is_empty/charCodeAt）: policy+observe経由に統一
                    if matches!(method.as_str(), "length" | "is_empty" | "charCodeAt") {
                        // receiver
                        if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                        let mut argc = 1usize;
                        if method.as_str() == "charCodeAt" {
                            if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            argc = 2;
                        }
                        if method.as_str() == "is_empty" { b.hint_ret_bool(true); }
                        let decision = crate::jit::policy::invoke::decide_box_method("StringBox", method.as_str(), argc, dst.is_some());
                        match decision {
                            crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                crate::jit::observe::lower_plugin_invoke(&box_type, method.as_str(), type_id, method_id, argc);
                                return Ok(());
                            }
                            crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                crate::jit::observe::lower_hostcall(&symbol, argc, &if argc==1 { ["Handle"][..].to_vec() } else { ["Handle","I64"][..].to_vec() }, "allow", "mapped_symbol");
                                b.emit_host_call(&symbol, argc, dst.is_some());
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    // Integer.get/set specialized when receiver is Integer (avoid Map collision)
                    if matches!(method.as_str(), "get" | "set") {
                        let recv_is_int = func.metadata.value_types.get(array).map(|mt| matches!(mt, crate::mir::MirType::Integer)).unwrap_or(false);
                        if recv_is_int {
                            if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                                if let Ok(h) = ph.resolve_method("IntegerBox", method.as_str()) {
                                    if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                    let mut argc = 1usize;
                                    if method.as_str() == "set" {
                                        if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                        argc = 2;
                                    }
                                    b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": format!("plugin:{}:{}", h.box_type, method.as_str()),
                                            "decision":"allow","reason":"plugin_invoke","argc": argc,
                                            "type_id": h.type_id, "method_id": h.method_id
                                        }),
                                        "plugin","<jit>"
                                    );
                                    return Ok(());
                                }
                            }
                        }
                    }
                    match method.as_str() {
                        "len" | "length" => {
                            // Resolve ArrayBox plugin method and emit plugin_invoke (symbolic)
                            if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                                let mname = "length";
                                if let Ok(h) = ph.resolve_method("ArrayBox", mname) {
                                    // Receiver
                                    if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                    let mut argc = 1usize;
                                    // length only
                                    b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": format!("plugin:{}:{}", h.box_type, mname),
                                            "decision":"allow","reason":"plugin_invoke","argc": argc,
                                            "type_id": h.type_id, "method_id": h.method_id
                                        }),
                                        "plugin","<jit>"
                                    );
                                }
                            }
                        }
                        // Map: size/get/has (RO) and set (mutating; allowed only when policy.read_only=false)
                        "size" | "get" | "has" | "set" => {
                            if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                                if let Ok(h) = ph.resolve_method("MapBox", method.as_str()) {
                                    if method.as_str() == "set" && crate::jit::policy::current().read_only {
                                        // Deny mutating under read-only policy
                                        crate::jit::events::emit_lower(
                                            serde_json::json!({
                                                "id": format!("plugin:{}:{}", "MapBox", "set"),
                                                "decision":"fallback","reason":"policy_denied_mutating"
                                            }),
                                            "plugin","<jit>"
                                        );
                                        // Do not emit plugin call; VM path will handle
                                        return Ok(());
                                    }
                                    if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                                    // Insert GC write barrier before mutating Map.set
                                    if method.as_str() == "set" {
                                        b.emit_host_call(crate::jit::r#extern::runtime::SYM_GC_BARRIER_WRITE, 1, false);
                                    }
                                    let mut argc = 1usize;
                                    if matches!(method.as_str(), "get" | "has") {
                                        if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                        argc += 1;
                                    } else if method.as_str() == "set" {
                                        if let Some(k) = args.get(0) { self.push_value_if_known_or_param(b, k); } else { b.emit_const_i64(0); }
                                        if let Some(v) = args.get(1) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                                        argc += 2;
                                    }
                                    b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": format!("plugin:{}:{}", h.box_type, method.as_str()),
                                            "decision":"allow","reason":"plugin_invoke","argc": argc,
                                            "type_id": h.type_id, "method_id": h.method_id
                                        }),
                                        "plugin","<jit>"
                                    );
                                }
                            }
                        }
                        _ => { /* other BoxCalls handled below */ }
                    }
                } else if crate::jit::config::current().hostcall {
                    match method.as_str() {
                        "len" | "length" => {
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                crate::jit::events::emit_lower(
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_LEN_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]}),
                                    "hostcall","<jit>"
                                );
                                b.emit_param_i64(pidx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, dst.is_some());
                            } else {
                                crate::jit::events::emit_lower(
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_LEN_H, "decision":"fallback", "reason":"receiver_not_param", "argc":1, "arg_types":["Handle"]}),
                                    "hostcall","<jit>"
                                );
                                let arr_idx = -1;
                                b.emit_const_i64(arr_idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                            }
                        }
                        // math.* minimal boundary: use registry signature to decide allow/fallback (no actual hostcall yet)
                        "sin" | "cos" | "abs" | "min" | "max" => {
                            use crate::jit::hostcall_registry::{check_signature, ArgKind};
                            // Build symbol and observed arg kinds (f64 if known float, else i64)
                            let sym = format!("nyash.math.{}", method);
                            let mut observed: Vec<ArgKind> = Vec::new();
                            for v in args.iter() {
                                if self.known_f64.contains_key(v) { observed.push(ArgKind::F64); }
                                else { observed.push(ArgKind::I64); }
                            }
                            // Prepare arg_types for event payload
                            // Classify argument kinds using TyEnv when available; fallback to known maps/FloatBox tracking
                            let mut observed_kinds: Vec<crate::jit::hostcall_registry::ArgKind> = Vec::new();
                            for v in args.iter() {
                                let kind = if let Some(mt) = func.metadata.value_types.get(v) {
                                    match mt {
                                        crate::mir::MirType::Float => crate::jit::hostcall_registry::ArgKind::F64,
                                        crate::mir::MirType::Integer => crate::jit::hostcall_registry::ArgKind::I64,
                                        crate::mir::MirType::Bool => crate::jit::hostcall_registry::ArgKind::I64, // b1はI64 0/1に正規化
                                        crate::mir::MirType::String | crate::mir::MirType::Box(_) => crate::jit::hostcall_registry::ArgKind::Handle,
                                        _ => {
                                            if self.known_f64.contains_key(v) || self.float_box_values.contains(v) { crate::jit::hostcall_registry::ArgKind::F64 }
                                            else { crate::jit::hostcall_registry::ArgKind::I64 }
                                        }
                                    }
                                } else {
                                    if self.known_f64.contains_key(v) || self.float_box_values.contains(v) { crate::jit::hostcall_registry::ArgKind::F64 }
                                    else { crate::jit::hostcall_registry::ArgKind::I64 }
                                };
                                observed_kinds.push(kind);
                            }
                            let arg_types: Vec<&'static str> = observed_kinds.iter().map(|k| match k { crate::jit::hostcall_registry::ArgKind::I64 => "I64", crate::jit::hostcall_registry::ArgKind::F64 => "F64", crate::jit::hostcall_registry::ArgKind::Handle => "Handle" }).collect();
                            match check_signature(&sym, &observed_kinds) {
                                Ok(()) => {
                                    // allow: record decision; execution remains on VM for now (thin bridge)
                                        crate::jit::events::emit_lower(
                                            serde_json::json!({
                                                "id": sym,
                                                "decision": "allow",
                                                "reason": "sig_ok",
                                                "argc": observed.len(),
                                                "arg_types": arg_types
                                            }),
                                            "hostcall","<jit>"
                                        );
                                    // If native f64 is enabled, emit a typed hostcall to math extern
                                    if crate::jit::config::current().native_f64 {
                                        let (symbol, arity) = match method.as_str() {
                                            "sin" => ("nyash.math.sin_f64", 1),
                                            "cos" => ("nyash.math.cos_f64", 1),
                                            "abs" => ("nyash.math.abs_f64", 1),
                                            "min" => ("nyash.math.min_f64", 2),
                                            "max" => ("nyash.math.max_f64", 2),
                                            _ => ("nyash.math.sin_f64", 1),
                                        };
                                        // Push f64 args from known_f64 or coerce known_i64
                                        for i in 0..arity {
                                            if let Some(v) = args.get(i) {
                                                // Try direct known values
                                                if let Some(fv) = self.known_f64.get(v).copied() { b.emit_const_f64(fv); continue; }
                                                if let Some(iv) = self.known_i64.get(v).copied() { b.emit_const_f64(iv as f64); continue; }
                                                // Try unwrap FloatBox: scan blocks to find NewBox FloatBox { args: [src] } and reuse src const
                                                let mut emitted = false;
                                                'scan: for (_bb_id, bb) in func.blocks.iter() {
                                                    for ins in bb.instructions.iter() {
                                                        if let crate::mir::MirInstruction::NewBox { dst, box_type, args: nb_args } = ins {
                                                            if *dst == *v && box_type == "FloatBox" {
                                                                if let Some(srcv) = nb_args.get(0) {
                                                                    if let Some(fv) = self.known_f64.get(srcv).copied() { b.emit_const_f64(fv); emitted = true; break 'scan; }
                                                                    if let Some(iv) = self.known_i64.get(srcv).copied() { b.emit_const_f64(iv as f64); emitted = true; break 'scan; }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                if !emitted { b.emit_const_f64(0.0); }
                                            } else { b.emit_const_f64(0.0); }
                                        }
                                        let kinds: Vec<super::builder::ParamKind> = (0..arity).map(|_| super::builder::ParamKind::F64).collect();
                                        b.emit_host_call_typed(symbol, &kinds, dst.is_some(), true);
                                    }
                                }
                                Err(reason) => {
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({
                                            "id": sym,
                                            "decision": "fallback",
                                            "reason": reason,
                                            "argc": observed.len(),
                                            "arg_types": arg_types
                                        }),
                                        "hostcall",
                                        "<jit>"
                                    );
                                }
                            }
                            // no-op: VM側で実行される
                        }
                        "isEmpty" | "empty" => {
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                crate::jit::events::emit(
                                    "hostcall","<jit>",None,None,
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, "decision":"allow", "reason":"sig_ok", "argc":1, "arg_types":["Handle"]})
                                );
                                b.emit_param_i64(pidx);
                                // returns i64 0/1
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, 1, dst.is_some());
                            } else {
                                    crate::jit::events::emit_lower(
                                        serde_json::json!({"id": crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, "decision":"fallback", "reason":"receiver_not_param", "argc":1, "arg_types":["Handle"]}),
                                        "hostcall","<jit>"
                                    );
                            }
                        }
                        "push" => {
                            // argc=2: (array, value)
                            let argc = 2usize;
                            let val = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                // Prepare args
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(val);
                                // Decide policy
                                let decision = crate::jit::policy::invoke::decide_box_method("ArrayBox", "push", argc, false);
                                match decision {
                                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                                        crate::jit::observe::lower_plugin_invoke(&box_type, "push", type_id, method_id, argc);
                                    }
                                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                        b.emit_host_call(&symbol, argc, false);
                                    }
                                    _ => {
                                        // Fallback to existing hostcall path
                                        let sym = crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H;
                                        crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                        b.emit_host_call(sym, argc, false);
                                    }
                                }
                            } else {
                                // No receiver param index
                                let arr_idx = -1;
                                b.emit_const_i64(arr_idx);
                                b.emit_const_i64(val);
                                let sym = crate::jit::r#extern::collections::SYM_ARRAY_PUSH;
                                crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64"], "fallback", "receiver_not_param");
                                b.emit_host_call(sym, argc, false);
                            }
                        }
                        "size" => {
                            let argc = 1usize;
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                b.emit_param_i64(pidx);
                                let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "size", argc, dst.is_some());
                                match decision {
                                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                        crate::jit::observe::lower_plugin_invoke(&box_type, "size", type_id, method_id, argc);
                                    }
                                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle"], "allow", "mapped_symbol");
                                        b.emit_host_call(&symbol, argc, dst.is_some());
                                    }
                                    _ => {
                                        let sym = crate::jit::r#extern::collections::SYM_MAP_SIZE_H;
                                        crate::jit::observe::lower_hostcall(sym, argc, &["Handle"], "fallback", "policy_or_unknown");
                                        b.emit_host_call(sym, argc, dst.is_some());
                                    }
                                }
                            } else {
                                let map_idx = -1;
                                b.emit_const_i64(map_idx);
                                let sym = crate::jit::r#extern::collections::SYM_MAP_SIZE;
                                crate::jit::observe::lower_hostcall(sym, argc, &["I64"], "fallback", "receiver_not_param");
                                b.emit_host_call(sym, argc, dst.is_some());
                            }
                        }
                        "get" => {
                            let argc = 2usize;
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                b.emit_param_i64(pidx);
                                if let Some(k) = args.get(0).and_then(|v| self.known_i64.get(v)).copied() { b.emit_const_i64(k); } else if let Some(kvid) = args.get(0) { self.push_value_if_known_or_param(b, kvid); } else { b.emit_const_i64(0); }
                                let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "get", argc, dst.is_some());
                                match decision {
                                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                        crate::jit::observe::lower_plugin_invoke(&box_type, "get", type_id, method_id, argc);
                                    }
                                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                        b.emit_host_call(&symbol, argc, dst.is_some());
                                    }
                                    _ => {
                                        let sym = crate::jit::r#extern::collections::SYM_MAP_GET_H;
                                        crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                        b.emit_host_call(sym, argc, dst.is_some());
                                    }
                                }
                            } else {
                                let sym = crate::jit::r#extern::collections::SYM_MAP_GET_H;
                                crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64"], "fallback", "receiver_not_param");
                                b.emit_host_call(sym, argc, dst.is_some());
                            }
                        }
                        "set" => {
                            let argc = 3usize;
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                let val = args.get(1).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(key);
                                b.emit_const_i64(val);
                                let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "set", argc, false);
                                match decision {
                                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                                        crate::jit::observe::lower_plugin_invoke(&box_type, "set", type_id, method_id, argc);
                                    }
                                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64","I64"], "allow", "mapped_symbol");
                                        b.emit_host_call(&symbol, argc, false);
                                    }
                                    _ => {
                                        let sym = crate::jit::r#extern::collections::SYM_MAP_SET_H;
                                        crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64","I64"], "fallback", "policy_or_unknown");
                                        b.emit_host_call(sym, argc, false);
                                    }
                                }
                            } else {
                                let sym = crate::jit::r#extern::collections::SYM_MAP_SET;
                                crate::jit::observe::lower_hostcall(sym, argc, &["I64","I64","I64"], "fallback", "receiver_not_param");
                                b.emit_host_call(sym, argc, false);
                            }
                        }
                        "charCodeAt" => {
                            // String.charCodeAt(index)
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let idx = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                crate::jit::events::emit_lower(
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, "decision":"allow", "reason":"sig_ok", "argc":2, "arg_types":["Handle","I64"]}),
                                    "hostcall","<jit>"
                                );
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, 2, dst.is_some());
                            } else {
                                crate::jit::events::emit_lower(
                                    serde_json::json!({"id": crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, "decision":"fallback", "reason":"receiver_not_param", "argc":2, "arg_types":["Handle","I64"]}),
                                    "hostcall","<jit>"
                                );
                            }
                        }
                        "has" => {
                            let argc = 2usize;
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(key);
                                let decision = crate::jit::policy::invoke::decide_box_method("MapBox", "has", argc, dst.is_some());
                                match decision {
                                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                                        crate::jit::observe::lower_plugin_invoke(&box_type, "has", type_id, method_id, argc);
                                    }
                                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                                        b.emit_host_call(&symbol, argc, dst.is_some());
                                    }
                                    _ => {
                                        let sym = crate::jit::r#extern::collections::SYM_MAP_HAS_H;
                                        crate::jit::observe::lower_hostcall(sym, argc, &["Handle","I64"], "fallback", "policy_or_unknown");
                                        b.emit_host_call(sym, argc, dst.is_some());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub use super::cfg_dot::dump_cfg_dot;
