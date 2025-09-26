//! Core ops lowering (non-hostcall): BinOp, Compare, Branch, Jump
use super::builder::{BinOpKind, CmpKind, IRBuilder};
use crate::mir::{BinaryOp, CompareOp, MirFunction, MirType, ValueId};

use super::core::LowerCore;

impl LowerCore {
    fn is_string_like(&self, func: &MirFunction, v: &ValueId) -> bool {
        // Check per-value type metadata
        if let Some(mt) = func.metadata.value_types.get(v) {
            if matches!(mt, MirType::String) {
                return true;
            }
            if let MirType::Box(ref name) = mt {
                if name == "StringBox" {
                    return true;
                }
            }
        }
        // Check if this value is a parameter with String or StringBox type
        if let Some(pidx) = self.param_index.get(v).copied() {
            if let Some(pt) = func.signature.params.get(pidx) {
                if matches!(pt, MirType::String) {
                    return true;
                }
                if let MirType::Box(ref name) = pt {
                    if name == "StringBox" {
                        return true;
                    }
                }
            }
        }
        // Check if it originates from a StringBox NewBox
        if let Some(name) = self.box_type_map.get(v) {
            if name == "StringBox" {
                return true;
            }
        }
        false
    }

    pub fn lower_binop(
        &mut self,
        b: &mut dyn IRBuilder,
        op: &BinaryOp,
        lhs: &ValueId,
        rhs: &ValueId,
        dst: &ValueId,
        func: &MirFunction,
    ) {
        // Optional: consult unified grammar for operator strategy (non-invasive logging)
        if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
            match op {
                BinaryOp::Add => {
                    let strat = crate::grammar::engine::get().add_coercion_strategy();
                    crate::jit::events::emit(
                        "grammar",
                        "add",
                        None,
                        None,
                        serde_json::json!({"coercion": strat}),
                    );
                }
                BinaryOp::Sub => {
                    let strat = crate::grammar::engine::get().sub_coercion_strategy();
                    crate::jit::events::emit(
                        "grammar",
                        "sub",
                        None,
                        None,
                        serde_json::json!({"coercion": strat}),
                    );
                }
                BinaryOp::Mul => {
                    let strat = crate::grammar::engine::get().mul_coercion_strategy();
                    crate::jit::events::emit(
                        "grammar",
                        "mul",
                        None,
                        None,
                        serde_json::json!({"coercion": strat}),
                    );
                }
                BinaryOp::Div => {
                    let strat = crate::grammar::engine::get().div_coercion_strategy();
                    crate::jit::events::emit(
                        "grammar",
                        "div",
                        None,
                        None,
                        serde_json::json!({"coercion": strat}),
                    );
                }
                _ => {}
            }
        }
        // Route string-like addition to hostcall (handle,handle)
        if crate::jit::config::current().hostcall {
            if matches!(op, BinaryOp::Add) {
                if self.is_string_like(func, lhs) || self.is_string_like(func, rhs) {
                    self.push_value_if_known_or_param(b, lhs);
                    self.push_value_if_known_or_param(b, rhs);
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_STRING_CONCAT_HH,
                        2,
                        true,
                    );
                    // Track handle result for downstream usages
                    self.handle_values.insert(*dst);
                    let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.store_local_i64(slot);
                    return;
                }
                // If dynamic Box/Unknown types, route to unified semantics add (handle,handle)
                let is_dynamic = match (
                    func.metadata.value_types.get(lhs),
                    func.metadata.value_types.get(rhs),
                ) {
                    (Some(MirType::Box(_)) | Some(MirType::Unknown) | None, _)
                    | (_, Some(MirType::Box(_)) | Some(MirType::Unknown) | None) => true,
                    _ => false,
                };
                if is_dynamic {
                    self.push_value_if_known_or_param(b, lhs);
                    self.push_value_if_known_or_param(b, rhs);
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_SEMANTICS_ADD_HH,
                        2,
                        true,
                    );
                    self.handle_values.insert(*dst);
                    let slot = *self.local_index.entry(*dst).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.store_local_i64(slot);
                    return;
                }
            }
        }
        self.push_value_if_known_or_param(b, lhs);
        self.push_value_if_known_or_param(b, rhs);
        let kind = match op {
            BinaryOp::Add => BinOpKind::Add,
            BinaryOp::Sub => BinOpKind::Sub,
            BinaryOp::Mul => BinOpKind::Mul,
            BinaryOp::Div => BinOpKind::Div,
            BinaryOp::Mod => BinOpKind::Mod,
            _ => {
                return;
            }
        };
        b.emit_binop(kind);
        if let (Some(a), Some(bv)) = (self.known_i64.get(lhs), self.known_i64.get(rhs)) {
            let res = match op {
                BinaryOp::Add => a.wrapping_add(*bv),
                BinaryOp::Sub => a.wrapping_sub(*bv),
                BinaryOp::Mul => a.wrapping_mul(*bv),
                BinaryOp::Div => {
                    if *bv != 0 {
                        a.wrapping_div(*bv)
                    } else {
                        0
                    }
                }
                BinaryOp::Mod => {
                    if *bv != 0 {
                        a.wrapping_rem(*bv)
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            self.known_i64.insert(*dst, res);
        }
    }

    pub fn lower_compare(
        &mut self,
        b: &mut dyn IRBuilder,
        op: &CompareOp,
        lhs: &ValueId,
        rhs: &ValueId,
        dst: &ValueId,
        func: &MirFunction,
    ) {
        // Route string-like comparisons (Eq/Lt) to hostcalls (i64 0/1)
        if crate::jit::config::current().hostcall {
            if matches!(op, CompareOp::Eq | CompareOp::Lt) {
                if self.is_string_like(func, lhs) || self.is_string_like(func, rhs) {
                    self.push_value_if_known_or_param(b, lhs);
                    self.push_value_if_known_or_param(b, rhs);
                    let sym = match op {
                        CompareOp::Eq => crate::jit::r#extern::collections::SYM_STRING_EQ_HH,
                        CompareOp::Lt => crate::jit::r#extern::collections::SYM_STRING_LT_HH,
                        _ => unreachable!(),
                    };
                    b.emit_host_call(sym, 2, true);
                    self.bool_values.insert(*dst);
                    return;
                }
            }
        }
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
        // Persist compare result in a local slot so terminators (Branch) can reload it reliably
        self.bool_values.insert(*dst);
        let slot = *self.local_index.entry(*dst).or_insert_with(|| {
            let id = self.next_local;
            self.next_local += 1;
            id
        });
        b.store_local_i64(slot);
    }

    pub fn lower_jump(&mut self, b: &mut dyn IRBuilder) {
        b.emit_jump();
    }
    pub fn lower_branch(&mut self, b: &mut dyn IRBuilder) {
        b.emit_branch();
    }
}

// Methods moved from core.rs to reduce file size and centralize op helpers
impl LowerCore {
    // Push a value if known or param/local/phi
    pub(super) fn push_value_if_known_or_param(&self, b: &mut dyn IRBuilder, id: &ValueId) {
        // Prefer compile-time known constants to avoid stale local slots overshadowing folded values
        if let Some(v) = self.known_i64.get(id).copied() {
            b.emit_const_i64(v);
            return;
        }
        if let Some(slot) = self.local_index.get(id).copied() {
            b.load_local_i64(slot);
            return;
        }
        if self.phi_values.contains(id) {
            let pos = self
                .phi_param_index
                .iter()
                .find_map(|((_, vid), idx)| if vid == id { Some(*idx) } else { None })
                .unwrap_or(0);
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
    }

    // Coverage helper: increments covered/unsupported counts
    pub(super) fn cover_if_supported(&mut self, instr: &crate::mir::MirInstruction) {
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
                | I::Debug { .. }
                | I::ExternCall { .. }
                | I::Safepoint
                | I::Nop
                | I::PluginInvoke { .. }
        );
        if supported {
            self.covered += 1;
        } else {
            self.unsupported += 1;
        }
    }
}
