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
}

impl LowerCore {
    pub fn new() -> Self { Self { unsupported: 0, covered: 0, known_i64: std::collections::HashMap::new(), param_index: std::collections::HashMap::new() } }

    /// Walk the MIR function and count supported/unsupported instructions.
    /// In the future, this will build CLIF via Cranelift builders.
    pub fn lower_function(&mut self, func: &MirFunction, builder: &mut dyn IRBuilder) -> Result<(), String> {
        // Prepare a simple i64 ABI based on param count; always assume i64 return for now
        // Build param index map
        self.param_index.clear();
        for (i, v) in func.params.iter().copied().enumerate() {
            self.param_index.insert(v, i);
        }
        // Prepare block mapping (Phase 10.7): deterministic ordering by sorted keys
        let mut bb_ids: Vec<_> = func.blocks.keys().copied().collect();
        bb_ids.sort_by_key(|b| b.0);
        builder.prepare_blocks(bb_ids.len());
        // Optional: collect single-PHI targets for minimal PHI path
        let enable_phi_min = std::env::var("NYASH_JIT_PHI_MIN").ok().as_deref() == Some("1");
        let mut phi_targets: std::collections::HashMap<crate::mir::BasicBlockId, std::collections::HashMap<crate::mir::BasicBlockId, crate::mir::ValueId>> = std::collections::HashMap::new();
        if enable_phi_min {
            for (bb_id, bb) in func.blocks.iter() {
                // gather Phi instructions in this block
                let mut phis: Vec<&crate::mir::MirInstruction> = Vec::new();
                for ins in bb.instructions.iter() { if let crate::mir::MirInstruction::Phi { .. } = ins { phis.push(ins); } }
                if phis.len() == 1 {
                    if let crate::mir::MirInstruction::Phi { inputs, .. } = phis[0] {
                        let mut map: std::collections::HashMap<crate::mir::BasicBlockId, crate::mir::ValueId> = std::collections::HashMap::new();
                        for (pred, val) in inputs.iter() { map.insert(*pred, *val); }
                        phi_targets.insert(*bb_id, map);
                    }
                }
            }
        }
        builder.prepare_signature_i64(func.params.len(), true);
        builder.begin_function(&func.signature.name);
        // Iterate blocks in the sorted order to keep indices stable
        for (idx, bb_id) in bb_ids.iter().enumerate() {
            let bb = func.blocks.get(bb_id).unwrap();
            builder.switch_to_block(idx);
            for instr in bb.instructions.iter() {
                self.cover_if_supported(instr);
                self.try_emit(builder, instr);
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
                            // For minimal PHI, pass one i64 arg if successor defines a single PHI with this block as pred
                            let mut then_n = 0usize;
                            let mut else_n = 0usize;
                            if let Some(pred_map) = phi_targets.get(then_bb) {
                                if let Some(v) = pred_map.get(bb_id) { self.push_value_if_known_or_param(builder, v); then_n = 1; builder.ensure_block_param_i64(then_index); }
                            }
                            if let Some(pred_map) = phi_targets.get(else_bb) {
                                if let Some(v) = pred_map.get(bb_id) { self.push_value_if_known_or_param(builder, v); else_n = 1; builder.ensure_block_param_i64(else_index); }
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
                            if let Some(pred_map) = phi_targets.get(target) {
                                if let Some(v) = pred_map.get(bb_id) { self.push_value_if_known_or_param(builder, v); n = 1; builder.ensure_block_param_i64(target_index); }
                            }
                            builder.jump_with_args(target_index, n);
                        } else {
                            builder.jump_to(target_index);
                        }
                        builder.seal_block(target_index);
                    }
                    _ => {
                        self.try_emit(builder, term);
                    }
                }
            }
        }
        builder.end_function();
        Ok(())
    }

    /// Push a value onto the builder stack if it is a known i64 const or a parameter.
    fn push_value_if_known_or_param(&self, b: &mut dyn IRBuilder, id: &ValueId) {
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

    fn try_emit(&mut self, b: &mut dyn IRBuilder, instr: &MirInstruction) {
        use crate::mir::MirInstruction as I;
        match instr {
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
            }
            I::Jump { .. } => b.emit_jump(),
            I::Branch { .. } => b.emit_branch(),
            I::Return { value } => {
                if let Some(v) = value { self.push_value_if_known_or_param(b, v); }
                b.emit_return()
            }
            I::Phi { .. } => {
                // Minimal PHI: load current block param as value (i64)
                b.push_block_param_i64();
            }
            I::ArrayGet { array, index, .. } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    // Push args: array param index (or -1), index (known or 0)
                    let idx = self.known_i64.get(index).copied().unwrap_or(0);
                    let arr_idx = self.param_index.get(array).copied().map(|x| x as i64).unwrap_or(-1);
                    b.emit_const_i64(arr_idx);
                    b.emit_const_i64(idx);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_GET, 2, true);
                }
            }
            I::ArraySet { array, index, value } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    let idx = self.known_i64.get(index).copied().unwrap_or(0);
                    let val = self.known_i64.get(value).copied().unwrap_or(0);
                    let arr_idx = self.param_index.get(array).copied().map(|x| x as i64).unwrap_or(-1);
                    b.emit_const_i64(arr_idx);
                    b.emit_const_i64(idx);
                    b.emit_const_i64(val);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_SET, 3, false);
                }
            }
            I::BoxCall { box_val: array, method, args, dst, .. } => {
                if std::env::var("NYASH_JIT_HOSTCALL").ok().as_deref() == Some("1") {
                    match method.as_str() {
                        "len" | "length" => {
                            // argc=1: (array_param_index)
                            let arr_idx = self.param_index.get(array).copied().map(|x| x as i64).unwrap_or(-1);
                            b.emit_const_i64(arr_idx);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                        }
                        "push" => {
                            // argc=2: (array, value)
                            let val = args.get(0).and_then(|v| self.known_i64.get(v)).copied().unwrap_or(0);
                            let arr_idx = self.param_index.get(array).copied().map(|x| x as i64).unwrap_or(-1);
                            b.emit_const_i64(arr_idx);
                            b.emit_const_i64(val);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_PUSH, 2, false);
                        }
                        "size" => {
                            // MapBox.size(): argc=1 (map_param_index)
                            let map_idx = self.param_index.get(array).copied().map(|x| x as i64).unwrap_or(-1);
                            b.emit_const_i64(map_idx);
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE, 1, dst.is_some());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
