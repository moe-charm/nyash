//! Kind-specific instruction metadata (PoC) used to gradually
//! move large enum matches to small, testable structs.
//!
//! Non-functional: only mirrors data for selected instructions and
//! provides introspection (effects/dst/used). Core behavior remains
//! in `MirInstruction`.

use super::{BasicBlockId, ConstValue, Effect, EffectMask, ValueId};
use crate::mir::instruction::MirInstruction;
use crate::mir::types::{
    BarrierOp as MirBarrierOp, BinaryOp as MirBinOp, MirType, TypeOpKind as MirTypeOpKind,
    WeakRefOp as MirWeakRefOp,
};

// Local macro utilities for generating InstructionMeta boilerplate.
// This macro is intentionally scoped to this module to avoid polluting the crate namespace.
macro_rules! inst_meta {
    (
        $(
            pub struct $name:ident { $($field:ident : $fty:ty),* $(,)? }
            => {
                from_mir = |$i:ident| $from_expr:expr;
                effects = $effects:expr;
                dst = $dst:expr;
                used = $used:expr;
            }
        )+
    ) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $name { $(pub $field: $fty),* }

            impl $name {
                pub fn from_mir($i: &MirInstruction) -> Option<Self> { $from_expr }
            }

            impl InstructionMeta for $name {
                fn effects(&self) -> EffectMask { ($effects)(self) }
                fn dst(&self) -> Option<ValueId> { ($dst)(self) }
                fn used(&self) -> Vec<ValueId> { ($used)(self) }
            }
        )+
    };
}

pub trait InstructionMeta {
    fn effects(&self) -> EffectMask;
    fn dst(&self) -> Option<ValueId>;
    fn used(&self) -> Vec<ValueId>;
}

// ---- Const ----
#[derive(Debug, Clone)]
pub struct ConstInst {
    pub dst: ValueId,
    pub value: ConstValue,
}

impl ConstInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Const { dst, value } => Some(ConstInst { dst: *dst, value: value.clone() }),
            _ => None,
        }
    }
}

impl InstructionMeta for ConstInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { Vec::new() }
}

// ---- BinOp ----
#[derive(Debug, Clone)]
pub struct BinOpInst {
    pub dst: ValueId,
    pub op: MirBinOp,
    pub lhs: ValueId,
    pub rhs: ValueId,
}

impl BinOpInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::BinOp { dst, op, lhs, rhs } => Some(BinOpInst { dst: *dst, op: *op, lhs: *lhs, rhs: *rhs }),
            _ => None,
        }
    }
}

impl InstructionMeta for BinOpInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.lhs, self.rhs] }
}

// ---- Helper delegation for MirInstruction methods ----

pub fn effects_via_meta(i: &MirInstruction) -> Option<EffectMask> {
    if let Some(k) = ConstInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = BinOpInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = UnaryOpInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = CompareInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = LoadInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = CastInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = TypeOpInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = ArrayGetInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = PhiInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = NewBoxInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = StoreInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = ArraySetInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = ReturnInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = BranchInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = JumpInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = PrintInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = DebugInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = TypeCheckInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = CopyInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = NopInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = ThrowInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = CatchInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = SafepointInst::from_mir(i) { return Some(k.effects()); }
    if let Some(k) = FunctionNewInst::from_mir(i) { return Some(k.effects()); }
    None
}

pub fn dst_via_meta(i: &MirInstruction) -> Option<ValueId> {
    if let Some(k) = ConstInst::from_mir(i) { return k.dst(); }
    if let Some(k) = BinOpInst::from_mir(i) { return k.dst(); }
    if let Some(k) = UnaryOpInst::from_mir(i) { return k.dst(); }
    if let Some(k) = CompareInst::from_mir(i) { return k.dst(); }
    if let Some(k) = LoadInst::from_mir(i) { return k.dst(); }
    if let Some(k) = CastInst::from_mir(i) { return k.dst(); }
    if let Some(k) = TypeOpInst::from_mir(i) { return k.dst(); }
    if let Some(k) = ArrayGetInst::from_mir(i) { return k.dst(); }
    if let Some(k) = PhiInst::from_mir(i) { return k.dst(); }
    if let Some(k) = NewBoxInst::from_mir(i) { return k.dst(); }
    if let Some(_k) = StoreInst::from_mir(i) { return None; }
    if let Some(_k) = ArraySetInst::from_mir(i) { return None; }
    if let Some(_k) = ReturnInst::from_mir(i) { return None; }
    if let Some(_k) = BranchInst::from_mir(i) { return None; }
    if let Some(_k) = JumpInst::from_mir(i) { return None; }
    if let Some(_k) = PrintInst::from_mir(i) { return None; }
    if let Some(_k) = DebugInst::from_mir(i) { return None; }
    if let Some(k) = CallLikeInst::from_mir(i) { return k.dst(); }
    if let Some(k) = TypeCheckInst::from_mir(i) { return k.dst(); }
    if let Some(k) = CopyInst::from_mir(i) { return k.dst(); }
    if let Some(_k) = NopInst::from_mir(i) { return None; }
    if let Some(_k) = ThrowInst::from_mir(i) { return None; }
    if let Some(k) = CatchInst::from_mir(i) { return k.dst(); }
    if let Some(_k) = SafepointInst::from_mir(i) { return None; }
    if let Some(k) = FunctionNewInst::from_mir(i) { return k.dst(); }
    None
}

pub fn used_via_meta(i: &MirInstruction) -> Option<Vec<ValueId>> {
    if let Some(k) = ConstInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = BinOpInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = UnaryOpInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = CompareInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = LoadInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = CastInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = TypeOpInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = ArrayGetInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = PhiInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = NewBoxInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = StoreInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = ArraySetInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = ReturnInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = BranchInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = JumpInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = PrintInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = DebugInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = CallLikeInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = TypeCheckInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = CopyInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = NopInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = ThrowInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = CatchInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = SafepointInst::from_mir(i) { return Some(k.used()); }
    if let Some(k) = FunctionNewInst::from_mir(i) { return Some(k.used()); }
    None
}

// ---- BarrierRead ---- (macro-generated)
inst_meta! {
    pub struct BarrierReadInst { ptr: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::BarrierRead { ptr } => Some(BarrierReadInst { ptr: *ptr }), _ => None };
        effects = |_: &Self| EffectMask::READ.add(Effect::Barrier);
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.ptr];
    }
}

// ---- BarrierWrite ---- (macro-generated)
inst_meta! {
    pub struct BarrierWriteInst { ptr: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::BarrierWrite { ptr } => Some(BarrierWriteInst { ptr: *ptr }), _ => None };
        effects = |_: &Self| EffectMask::WRITE.add(Effect::Barrier);
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.ptr];
    }
}

// ---- Barrier (unified) ---- (macro-generated)
inst_meta! {
    pub struct BarrierInst { op: MirBarrierOp, ptr: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Barrier { op, ptr } => Some(BarrierInst { op: *op, ptr: *ptr }), _ => None };
        effects = |s: &Self| match s.op { MirBarrierOp::Read => EffectMask::READ.add(Effect::Barrier), MirBarrierOp::Write => EffectMask::WRITE.add(Effect::Barrier) };
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.ptr];
    }
}

// ---- Ref ops ---- (macro-generated)
inst_meta! {
    pub struct RefNewInst { dst: ValueId, box_val: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::RefNew { dst, box_val } => Some(RefNewInst { dst: *dst, box_val: *box_val }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.box_val];
    }
}
inst_meta! {
    pub struct RefGetInst { dst: ValueId, reference: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::RefGet { dst, reference, .. } => Some(RefGetInst { dst: *dst, reference: *reference }), _ => None };
        effects = |_: &Self| EffectMask::READ;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.reference];
    }
}
inst_meta! {
    pub struct RefSetInst { reference: ValueId, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::RefSet { reference, value, .. } => Some(RefSetInst { reference: *reference, value: *value }), _ => None };
        effects = |_: &Self| EffectMask::WRITE;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.reference, s.value];
    }
}

// ---- Weak ops ---- (macro-generated)
inst_meta! {
    pub struct WeakNewInst { dst: ValueId, box_val: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::WeakNew { dst, box_val } => Some(WeakNewInst { dst: *dst, box_val: *box_val }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.box_val];
    }
}
inst_meta! {
    pub struct WeakLoadInst { dst: ValueId, weak_ref: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::WeakLoad { dst, weak_ref } => Some(WeakLoadInst { dst: *dst, weak_ref: *weak_ref }), _ => None };
        effects = |_: &Self| EffectMask::READ;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.weak_ref];
    }
}
inst_meta! {
    pub struct WeakRefInst { dst: ValueId, op: MirWeakRefOp, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::WeakRef { dst, op, value } => Some(WeakRefInst { dst: *dst, op: *op, value: *value }), _ => None };
        effects = |s: &Self| match s.op { MirWeakRefOp::New => EffectMask::PURE, MirWeakRefOp::Load => EffectMask::READ };
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.value];
    }
}

// ---- Future ops ---- (macro-generated)
inst_meta! {
    pub struct FutureNewInst { dst: ValueId, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::FutureNew { dst, value } => Some(FutureNewInst { dst: *dst, value: *value }), _ => None };
        effects = |_: &Self| EffectMask::PURE.add(Effect::Alloc);
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.value];
    }
}
inst_meta! {
    pub struct FutureSetInst { future: ValueId, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::FutureSet { future, value } => Some(FutureSetInst { future: *future, value: *value }), _ => None };
        effects = |_: &Self| EffectMask::WRITE;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.future, s.value];
    }
}
inst_meta! {
    pub struct AwaitInst { dst: ValueId, future: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Await { dst, future } => Some(AwaitInst { dst: *dst, future: *future }), _ => None };
        effects = |_: &Self| EffectMask::READ.add(Effect::Async);
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.future];
    }
}

// ---- UnaryOp ---- (macro-generated)
inst_meta! {
    pub struct UnaryOpInst { dst: ValueId, operand: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::UnaryOp { dst, operand, .. } => Some(UnaryOpInst { dst: *dst, operand: *operand }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.operand];
    }
}

// ---- Compare ---- (macro-generated)
inst_meta! {
    pub struct CompareInst { dst: ValueId, lhs: ValueId, rhs: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Compare { dst, lhs, rhs, .. } => Some(CompareInst { dst: *dst, lhs: *lhs, rhs: *rhs }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.lhs, s.rhs];
    }
}

// ---- Load ---- (macro-generated)
inst_meta! {
    pub struct LoadInst { dst: ValueId, ptr: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Load { dst, ptr } => Some(LoadInst { dst: *dst, ptr: *ptr }), _ => None };
        effects = |_: &Self| EffectMask::READ;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.ptr];
    }
}

// ---- Cast ---- (macro-generated)
inst_meta! {
    pub struct CastInst { dst: ValueId, value: ValueId, target_type: MirType }
    => {
        from_mir = |i| match i { MirInstruction::Cast { dst, value, target_type } => Some(CastInst { dst: *dst, value: *value, target_type: target_type.clone() }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.value];
    }
}

// ---- TypeOp ---- (macro-generated)
inst_meta! {
    pub struct TypeOpInst { dst: ValueId, op: MirTypeOpKind, value: ValueId, ty: MirType }
    => {
        from_mir = |i| match i { MirInstruction::TypeOp { dst, op, value, ty } => Some(TypeOpInst { dst: *dst, op: *op, value: *value, ty: ty.clone() }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.value];
    }
}

// ---- ArrayGet ---- (macro-generated)
inst_meta! {
    pub struct ArrayGetInst { dst: ValueId, array: ValueId, index: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::ArrayGet { dst, array, index } => Some(ArrayGetInst { dst: *dst, array: *array, index: *index }), _ => None };
        effects = |_: &Self| EffectMask::READ;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.array, s.index];
    }
}

// ---- Phi ---- (macro-generated)
inst_meta! {
    pub struct PhiInst { dst: ValueId, inputs: Vec<(BasicBlockId, ValueId)> }
    => {
        from_mir = |i| match i { MirInstruction::Phi { dst, inputs } => Some(PhiInst { dst: *dst, inputs: inputs.clone() }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| s.inputs.iter().map(|(_, v)| *v).collect();
    }
}

// ---- NewBox ---- (macro-generated)
inst_meta! {
    pub struct NewBoxInst { dst: ValueId, args: Vec<ValueId> }
    => {
        from_mir = |i| match i { MirInstruction::NewBox { dst, args, .. } => Some(NewBoxInst { dst: *dst, args: args.clone() }), _ => None };
        effects = |_: &Self| EffectMask::PURE.add(Effect::Alloc);
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| s.args.clone();
    }
}

// ---- FunctionNew ---- (macro-generated)
inst_meta! {
    pub struct FunctionNewInst { dst: ValueId, captures: Vec<(String, ValueId)>, me: Option<ValueId> }
    => {
        from_mir = |i| match i { MirInstruction::FunctionNew { dst, captures, me, .. } => Some(FunctionNewInst { dst: *dst, captures: captures.clone(), me: *me }), _ => None };
        effects = |_: &Self| EffectMask::PURE.add(Effect::Alloc);
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| { let mut v: Vec<ValueId> = s.captures.iter().map(|(_, id)| *id).collect(); if let Some(m) = s.me { v.push(m); } v };
    }
}

// ---- Store ---- (macro-generated)
inst_meta! {
    pub struct StoreInst { value: ValueId, ptr: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Store { value, ptr } => Some(StoreInst { value: *value, ptr: *ptr }), _ => None };
        effects = |_: &Self| EffectMask::WRITE;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.value, s.ptr];
    }
}

// ---- ArraySet ---- (macro-generated)
inst_meta! {
    pub struct ArraySetInst { array: ValueId, index: ValueId, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::ArraySet { array, index, value } => Some(ArraySetInst { array: *array, index: *index, value: *value }), _ => None };
        effects = |_: &Self| EffectMask::WRITE;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.array, s.index, s.value];
    }
}

// ---- Return ---- (macro-generated)
inst_meta! {
    pub struct ReturnInst { value: Option<ValueId> }
    => {
        from_mir = |i| match i { MirInstruction::Return { value } => Some(ReturnInst { value: *value }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |_: &Self| None;
        used = |s: &Self| s.value.map(|v| vec![v]).unwrap_or_default();
    }
}

// ---- Branch ---- (macro-generated)
inst_meta! {
    pub struct BranchInst { condition: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Branch { condition, .. } => Some(BranchInst { condition: *condition }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.condition];
    }
}

// ---- Jump ---- (macro-generated)
inst_meta! {
    pub struct JumpInst { }
    => {
        from_mir = |i| match i { MirInstruction::Jump { .. } => Some(JumpInst {}), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |_: &Self| None;
        used = |_: &Self| Vec::new();
    }
}

// ---- Print ---- (macro-generated)
inst_meta! {
    pub struct PrintInst { value: ValueId, effects_mask: EffectMask }
    => {
        from_mir = |i| match i { MirInstruction::Print { value, effects } => Some(PrintInst { value: *value, effects_mask: *effects }), _ => None };
        effects = |s: &Self| s.effects_mask;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.value];
    }
}

// ---- Debug ---- (macro-generated)
inst_meta! {
    pub struct DebugInst { value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Debug { value, .. } => Some(DebugInst { value: *value }), _ => None };
        effects = |_: &Self| EffectMask::PURE.add(Effect::Debug);
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.value];
    }
}

// ---- TypeCheck ---- (macro-generated)
inst_meta! {
    pub struct TypeCheckInst { dst: ValueId, value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::TypeCheck { dst, value, .. } => Some(TypeCheckInst { dst: *dst, value: *value }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.value];
    }
}

// ---- Copy ---- (macro-generated)
inst_meta! {
    pub struct CopyInst { dst: ValueId, src: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Copy { dst, src } => Some(CopyInst { dst: *dst, src: *src }), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |s: &Self| Some(s.dst);
        used = |s: &Self| vec![s.src];
    }
}

// ---- Nop ---- (macro-generated)
inst_meta! {
    pub struct NopInst { }
    => {
        from_mir = |i| match i { MirInstruction::Nop => Some(NopInst {}), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |_: &Self| None;
        used = |_: &Self| Vec::new();
    }
}

// ---- Throw ---- (macro-generated)
inst_meta! {
    pub struct ThrowInst { exception: ValueId, effects_mask: EffectMask }
    => {
        from_mir = |i| match i { MirInstruction::Throw { exception, effects } => Some(ThrowInst { exception: *exception, effects_mask: *effects }), _ => None };
        effects = |s: &Self| s.effects_mask;
        dst = |_: &Self| None;
        used = |s: &Self| vec![s.exception];
    }
}

// ---- Catch ---- (macro-generated)
inst_meta! {
    pub struct CatchInst { exception_value: ValueId }
    => {
        from_mir = |i| match i { MirInstruction::Catch { exception_value, .. } => Some(CatchInst { exception_value: *exception_value }), _ => None };
        effects = |_: &Self| EffectMask::CONTROL;
        dst = |s: &Self| Some(s.exception_value);
        used = |_: &Self| Vec::new();
    }
}

// ---- Safepoint ---- (macro-generated)
inst_meta! {
    pub struct SafepointInst { }
    => {
        from_mir = |i| match i { MirInstruction::Safepoint => Some(SafepointInst {}), _ => None };
        effects = |_: &Self| EffectMask::PURE;
        dst = |_: &Self| None;
        used = |_: &Self| Vec::new();
    }
}

// ---- Call-like (dst/used only; effects fallback in MirInstruction) ----
#[derive(Debug, Clone)]
pub enum CallLikeInst {
    Call { dst: Option<ValueId>, func: ValueId, args: Vec<ValueId> },
    BoxCall { dst: Option<ValueId>, box_val: ValueId, args: Vec<ValueId> },
    PluginInvoke { dst: Option<ValueId>, box_val: ValueId, args: Vec<ValueId> },
    ExternCall { dst: Option<ValueId>, args: Vec<ValueId> },
}

impl CallLikeInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Call { dst, func, args, .. } =>
                Some(CallLikeInst::Call { dst: *dst, func: *func, args: args.clone() }),
            MirInstruction::BoxCall { dst, box_val, args, .. } =>
                Some(CallLikeInst::BoxCall { dst: *dst, box_val: *box_val, args: args.clone() }),
            MirInstruction::PluginInvoke { dst, box_val, args, .. } =>
                Some(CallLikeInst::PluginInvoke { dst: *dst, box_val: *box_val, args: args.clone() }),
            MirInstruction::ExternCall { dst, args, .. } =>
                Some(CallLikeInst::ExternCall { dst: *dst, args: args.clone() }),
            _ => None,
        }
    }

    pub fn dst(&self) -> Option<ValueId> {
        match self {
            CallLikeInst::Call { dst, .. }
            | CallLikeInst::BoxCall { dst, .. }
            | CallLikeInst::PluginInvoke { dst, .. }
            | CallLikeInst::ExternCall { dst, .. } => *dst,
        }
    }

    pub fn used(&self) -> Vec<ValueId> {
        match self {
            CallLikeInst::Call { func, args, .. } => {
                let mut v = vec![*func]; v.extend(args.iter().copied()); v
            }
            CallLikeInst::BoxCall { box_val, args, .. }
            | CallLikeInst::PluginInvoke { box_val, args, .. } => {
                let mut v = vec![*box_val]; v.extend(args.iter().copied()); v
            }
            CallLikeInst::ExternCall { args, .. } => args.clone(),
        }
    }
}
