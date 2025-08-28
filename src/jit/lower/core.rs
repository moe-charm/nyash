use crate::mir::{MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, ValueId};
use super::builder::{IRBuilder, BinOpKind, CmpKind};

/// Lower(Core-1): Minimal lowering skeleton for Const/Move/BinOp/Cmp/Branch/Ret
/// This does not emit real CLIF yet; it only walks MIR and validates coverage.
pub struct LowerCore {
    pub unsupported: usize,
    pub covered: usize,
    /// Minimal constant propagation for i64 to feed host-call args
    known_i64: std::collections::HashMap<ValueId, i64>,
    /// Parameter index mapping for ValueId
    param_index: std::collections::HashMap<ValueId, usize>,
    /// Track values produced by Phi (for minimal PHI path)
    phi_values: std::collections::HashSet<ValueId>,
    /// Map (block, phi dst) -> param index in that block (for multi-PHI)
    phi_param_index: std::collections::HashMap<(crate::mir::BasicBlockId, ValueId), usize>,
    /// Track values that are boolean (b1) results, e.g., Compare destinations
    bool_values: std::collections::HashSet<ValueId>,
    /// Track PHI destinations that are boolean (all inputs derived from bool_values)
    bool_phi_values: std::collections::HashSet<ValueId>,
    // Per-function statistics (last lowered)
    last_phi_total: u64,
    last_phi_b1: u64,
    last_ret_bool_hint_used: bool,
    // Minimal local slot mapping for Load/Store (ptr ValueId -> slot index)
    local_index: std::collections::HashMap<ValueId, usize>,
    next_local: usize,
}

impl LowerCore {
    pub fn new() -> Self { Self { unsupported: 0, covered: 0, known_i64: std::collections::HashMap::new(), param_index: std::collections::HashMap::new(), phi_values: std::collections::HashSet::new(), phi_param_index: std::collections::HashMap::new(), bool_values: std::collections::HashSet::new(), bool_phi_values: std::collections::HashSet::new(), last_phi_total: 0, last_phi_b1: 0, last_ret_bool_hint_used: false, local_index: std::collections::HashMap::new(), next_local: 0 } }

    /// Get statistics for the last lowered function
    pub fn last_stats(&self) -> (u64, u64, bool) { (self.last_phi_total, self.last_phi_b1, self.last_ret_bool_hint_used) }

    /// Walk the MIR function and count supported/unsupported instructions.
    /// In the future, this will build CLIF via Cranelift builders.
    pub fn lower_function(&mut self, func: &MirFunction, builder: &mut dyn IRBuilder) -> Result<(), String> {
        // Prepare a simple i64 ABI based on param count; always assume i64 return for now
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
        if use_typed || ret_is_f64 {
            builder.prepare_signature_typed(&kinds, ret_is_f64);
        } else {
            builder.prepare_signature_i64(func.params.len(), true);
        }
        builder.begin_function(&func.signature.name);
        // Iterate blocks in the sorted order to keep indices stable
        self.phi_values.clear();
        self.phi_param_index.clear();
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
                self.try_emit(builder, instr, *bb_id);
            }
            if let Some(term) = &bb.terminator {
                self.cover_if_supported(term);
                // Branch/Jump need block mapping: pass indices
                match term {
                    crate::mir::MirInstruction::Branch { condition, then_bb, else_bb } => {
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
                    _ => {
                        self.try_emit(builder, term, *bb_id);
                    }
                }
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
    fn push_value_if_known_or_param(&self, b: &mut dyn IRBuilder, id: &ValueId) {
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
        }
    }

    fn cover_if_supported(&mut self, instr: &MirInstruction) {
        use crate::mir::MirInstruction as I;
        let supported = matches!(
            instr,
            I::Const { .. }
                | I::Copy { .. }
                | I::Cast { .. }
                | I::BinOp { .. }
                | I::Compare { .. }
                | I::Jump { .. }
                | I::Branch { .. }
                | I::Return { .. }
                | I::ArrayGet { .. }
                | I::ArraySet { .. }
        );
        if supported { self.covered += 1; } else { self.unsupported += 1; }
    }

    fn try_emit(&mut self, b: &mut dyn IRBuilder, instr: &MirInstruction, cur_bb: crate::mir::BasicBlockId) {
        use crate::mir::MirInstruction as I;
        match instr {
            I::Cast { dst, value, target_type: _ } => {
                // Minimal cast footing: materialize source when param/known
                // Bool→Int: rely on producers (compare) and branch/b1 loaders; here we just reuse integer path
                self.push_value_if_known_or_param(b, value);
                // Track known i64 if source known
                if let Some(v) = self.known_i64.get(value).copied() { self.known_i64.insert(*dst, v); }
            }
            I::Const { dst, value } => match value {
                ConstValue::Integer(i) => { 
                    b.emit_const_i64(*i);
                    self.known_i64.insert(*dst, *i);
                }
                ConstValue::Float(f) => b.emit_const_f64(*f),
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
                // If source is a parameter, materialize it on the stack for downstream ops
                if let Some(pidx) = self.param_index.get(src).copied() {
                    b.emit_param_i64(pidx);
                }
                // Propagate boolean classification through Copy
                if self.bool_values.contains(src) { self.bool_values.insert(*dst); }
                // Otherwise no-op for codegen (stack-machine handles sources directly later)
            }
            I::BinOp { dst, op, lhs, rhs } => {
                // Ensure operands are on stack when available (param or known const)
                self.push_value_if_known_or_param(b, lhs);
                self.push_value_if_known_or_param(b, rhs);
                let kind = match op {
                    BinaryOp::Add => BinOpKind::Add,
                    BinaryOp::Sub => BinOpKind::Sub,
                    BinaryOp::Mul => BinOpKind::Mul,
                    BinaryOp::Div => BinOpKind::Div,
                    BinaryOp::Mod => BinOpKind::Mod,
                    // Not yet supported in Core-1
                    BinaryOp::And | BinaryOp::Or
                    | BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => { return; }
                };
                b.emit_binop(kind);
                if let (Some(a), Some(b)) = (self.known_i64.get(lhs), self.known_i64.get(rhs)) {
                    let res = match op {
                        BinaryOp::Add => a.wrapping_add(*b),
                        BinaryOp::Sub => a.wrapping_sub(*b),
                        BinaryOp::Mul => a.wrapping_mul(*b),
                        BinaryOp::Div => if *b != 0 { a.wrapping_div(*b) } else { 0 },
                        BinaryOp::Mod => if *b != 0 { a.wrapping_rem(*b) } else { 0 },
                        _ => 0,
                    };
                    self.known_i64.insert(*dst, res);
                }
            }
            I::Compare { op, lhs, rhs, .. } => {
                // Ensure operands are on stack when available (param or known const)
                self.push_value_if_known_or_param(b, lhs);
                self.push_value_if_known_or_param(b, rhs);
                let kind = match op {
                    CompareOp::Eq => CmpKind::Eq,
                    CompareOp::Ne => CmpKind::Ne,
                    CompareOp::Lt => CmpKind::Lt,
                    CompareOp::Le => CmpKind::Le,
                    CompareOp::Gt => CmpKind::Gt,
                    CompareOp::Ge => CmpKind::Ge,
                };
                b.emit_compare(kind);
                // Mark the last dst (compare produces a boolean)
                if let MirInstruction::Compare { dst, .. } = instr { self.bool_values.insert(*dst); }
            }
            I::Jump { .. } => b.emit_jump(),
            I::Branch { .. } => b.emit_branch(),
            I::Return { value } => {
                if let Some(v) = value { self.push_value_if_known_or_param(b, v); }
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
                // Minimal PHI: load current block param; b1 when classified boolean
                let pos = self.phi_param_index.get(&(cur_bb, *dst)).copied().unwrap_or(0);
                if self.bool_phi_values.contains(dst) {
                    b.push_block_param_b1_at(pos);
                } else {
                    b.push_block_param_i64_at(pos);
                }
            }
            I::ArrayGet { array, index, .. } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    let idx = self.known_i64.get(index).copied().unwrap_or(0);
                    if let Some(pidx) = self.param_index.get(array).copied() {
                        // Handle-based: push handle value from param, then index
                        b.emit_param_i64(pidx);
                        b.emit_const_i64(idx);
                        b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET_H, 2, true);
                    } else {
                        // Fallback to index-based (param index unknown)
                        let arr_idx = -1;
                        b.emit_const_i64(arr_idx);
                        b.emit_const_i64(idx);
                        b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET, 2, true);
                    }
                }
            }
            I::ArraySet { array, index, value } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    let idx = self.known_i64.get(index).copied().unwrap_or(0);
                    let val = self.known_i64.get(value).copied().unwrap_or(0);
                    if let Some(pidx) = self.param_index.get(array).copied() {
                        b.emit_param_i64(pidx);
                        b.emit_const_i64(idx);
                        b.emit_const_i64(val);
                        b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET_H, 3, false);
                    } else {
                        let arr_idx = -1;
                        b.emit_const_i64(arr_idx);
                        b.emit_const_i64(idx);
                        b.emit_const_i64(val);
                        b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET, 3, false);
                    }
                }
            }
            I::BoxCall { box_val: array, method, args, dst, .. } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    match method.as_str() {
                        "len" | "length" => {
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                // Handle-based generic length: supports ArrayBox and StringBox
                                b.emit_param_i64(pidx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, dst.is_some());
                            } else {
                                let arr_idx = -1;
                                b.emit_const_i64(arr_idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                            }
                        }
                        "isEmpty" | "empty" => {
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                b.emit_param_i64(pidx);
                                // returns i64 0/1
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_IS_EMPTY_H, 1, dst.is_some());
                            }
                        }
                        "push" => {
                            // argc=2: (array_handle, value)
                            let val = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(val);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H, 2, false);
                            } else {
                                let arr_idx = -1;
                                b.emit_const_i64(arr_idx);
                                b.emit_const_i64(val);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_PUSH, 2, false);
                            }
                        }
                        "size" => {
                            // MapBox.size(): argc=1 (map_handle)
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                b.emit_param_i64(pidx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE_H, 1, dst.is_some());
                            } else {
                                let map_idx = -1;
                                b.emit_const_i64(map_idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE, 1, dst.is_some());
                            }
                        }
                        "get" => {
                            // MapBox.get(key): (map_handle, key_i64)
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(key);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_GET_H, 2, dst.is_some());
                            }
                        }
                        "set" => {
                            // MapBox.set(key, value): (map_handle, key_i64, val_i64) — PoC: integer-only
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                let val = args.get(1).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(key);
                                b.emit_const_i64(val);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SET_H, 3, false);
                            }
                        }
                        "charCodeAt" => {
                            // String.charCodeAt(index)
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let idx = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(idx);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_CHARCODE_AT_H, 2, dst.is_some());
                            }
                        }
                        "has" => {
                            // MapBox.has(key_i64) -> 0/1
                            if let Some(pidx) = self.param_index.get(array).copied() {
                                let key = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                                b.emit_param_i64(pidx);
                                b.emit_const_i64(key);
                                b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_HAS_H, 2, dst.is_some());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

/// Emit a simple DOT graph for a MIR function's CFG.
/// Includes block ids, successor edges, and PHI counts per block when phi_min is enabled.
pub fn dump_cfg_dot(func: &crate::mir::MirFunction, path: &str, phi_min: bool) -> std::io::Result<()> {
    use std::io::Write;
    let mut out = String::new();
    out.push_str(&format!("digraph \"{}\" {{\n", func.signature.name));
    out.push_str("  node [shape=box, fontsize=10];\n");
    // Derive simple bool sets: compare dsts are bool; phi of all-bool inputs are bool
    let mut bool_values: std::collections::HashSet<crate::mir::ValueId> = std::collections::HashSet::new();
    for (_bb_id, bb) in func.blocks.iter() {
        for ins in bb.instructions.iter() {
            if let crate::mir::MirInstruction::Compare { dst, .. } = ins { bool_values.insert(*dst); }
        }
    }
    let mut bool_phi: std::collections::HashSet<crate::mir::ValueId> = std::collections::HashSet::new();
    if phi_min {
        for (_bb_id, bb) in func.blocks.iter() {
            for ins in bb.instructions.iter() {
                if let crate::mir::MirInstruction::Phi { dst, inputs } = ins {
                    if !inputs.is_empty() && inputs.iter().all(|(_, v)| bool_values.contains(v)) {
                        bool_phi.insert(*dst);
                    }
                }
            }
        }
    }
    // Sort blocks for deterministic output
    let mut bb_ids: Vec<_> = func.blocks.keys().copied().collect();
    bb_ids.sort_by_key(|b| b.0);
    // Emit nodes with labels
    for bb_id in bb_ids.iter() {
        let bb = func.blocks.get(bb_id).unwrap();
        let phi_count = bb.instructions.iter().filter(|ins| matches!(ins, crate::mir::MirInstruction::Phi { .. })).count();
        let phi_b1_count = bb.instructions.iter().filter(|ins| match ins { crate::mir::MirInstruction::Phi { dst, .. } => bool_phi.contains(dst), _ => false }).count();
        let mut label = format!("bb{}", bb_id.0);
        if phi_min && phi_count > 0 {
            if phi_b1_count > 0 { label = format!("{}\\nphi:{} (b1:{})", label, phi_count, phi_b1_count); }
            else { label = format!("{}\\nphi:{}", label, phi_count); }
        }
        if *bb_id == func.entry_block { label = format!("{}\\nENTRY", label); }
        out.push_str(&format!("  n{} [label=\"{}\"];\n", bb_id.0, label));
    }
    // Emit edges based on terminators
    for bb_id in bb_ids.iter() {
        let bb = func.blocks.get(bb_id).unwrap();
        if let Some(term) = &bb.terminator {
            match term {
                crate::mir::MirInstruction::Jump { target } => {
                    out.push_str(&format!("  n{} -> n{};\n", bb_id.0, target.0));
                }
                crate::mir::MirInstruction::Branch { then_bb, else_bb, .. } => {
                    // Branch condition is boolean (b1)
                    out.push_str(&format!("  n{} -> n{} [label=\"then cond:b1\"];\n", bb_id.0, then_bb.0));
                    out.push_str(&format!("  n{} -> n{} [label=\"else cond:b1\"];\n", bb_id.0, else_bb.0));
                }
                _ => {}
            }
        }
    }
    out.push_str("}\n");
    std::fs::write(path, out)
}
