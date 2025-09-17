//! Kind-specific instruction metadata (PoC) used to gradually
//! move large enum matches to small, testable structs.
//!
//! Non-functional: only mirrors data for selected instructions and
//! provides introspection (effects/dst/used). Core behavior remains
//! in `MirInstruction`.

use super::{BasicBlockId, ConstValue, Effect, EffectMask, ValueId};
use crate::mir::instruction::{
    BarrierOp as MirBarrierOp, BinaryOp as MirBinOp, MirInstruction, MirType,
    TypeOpKind as MirTypeOpKind, WeakRefOp as MirWeakRefOp,
};

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
    None
}

// ---- BarrierRead ----
#[derive(Debug, Clone, Copy)]
pub struct BarrierReadInst { pub ptr: ValueId }

impl BarrierReadInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::BarrierRead { ptr } => Some(BarrierReadInst { ptr: *ptr }), _ => None }
    }
}

impl InstructionMeta for BarrierReadInst {
    fn effects(&self) -> EffectMask { EffectMask::READ.add(Effect::Barrier) }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.ptr] }
}

// ---- BarrierWrite ----
#[derive(Debug, Clone, Copy)]
pub struct BarrierWriteInst { pub ptr: ValueId }

impl BarrierWriteInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::BarrierWrite { ptr } => Some(BarrierWriteInst { ptr: *ptr }), _ => None }
    }
}

impl InstructionMeta for BarrierWriteInst {
    fn effects(&self) -> EffectMask { EffectMask::WRITE.add(Effect::Barrier) }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.ptr] }
}

// ---- Barrier (unified) ----
#[derive(Debug, Clone, Copy)]
pub struct BarrierInst { pub op: MirBarrierOp, pub ptr: ValueId }

impl BarrierInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::Barrier { op, ptr } => Some(BarrierInst { op: *op, ptr: *ptr }), _ => None }
    }
}

impl InstructionMeta for BarrierInst {
    fn effects(&self) -> EffectMask {
        match self.op {
            MirBarrierOp::Read => EffectMask::READ.add(Effect::Barrier),
            MirBarrierOp::Write => EffectMask::WRITE.add(Effect::Barrier),
        }
    }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.ptr] }
}

// ---- Ref ops ----
#[derive(Debug, Clone, Copy)]
pub struct RefNewInst { pub dst: ValueId, pub box_val: ValueId }
impl RefNewInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::RefNew { dst, box_val } => Some(RefNewInst { dst: *dst, box_val: *box_val }), _ => None }
    }
}
impl InstructionMeta for RefNewInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.box_val] }
}

#[derive(Debug, Clone, Copy)]
pub struct RefGetInst { pub dst: ValueId, pub reference: ValueId }
impl RefGetInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::RefGet { dst, reference, .. } => Some(RefGetInst { dst: *dst, reference: *reference }), _ => None }
    }
}
impl InstructionMeta for RefGetInst {
    fn effects(&self) -> EffectMask { EffectMask::READ }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.reference] }
}

#[derive(Debug, Clone, Copy)]
pub struct RefSetInst { pub reference: ValueId, pub value: ValueId }
impl RefSetInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::RefSet { reference, value, .. } => Some(RefSetInst { reference: *reference, value: *value }), _ => None }
    }
}
impl InstructionMeta for RefSetInst {
    fn effects(&self) -> EffectMask { EffectMask::WRITE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.reference, self.value] }
}

// ---- Weak ops ----
#[derive(Debug, Clone, Copy)]
pub struct WeakNewInst { pub dst: ValueId, pub box_val: ValueId }
impl WeakNewInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::WeakNew { dst, box_val } => Some(WeakNewInst { dst: *dst, box_val: *box_val }), _ => None }
    }
}
impl InstructionMeta for WeakNewInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.box_val] }
}

#[derive(Debug, Clone, Copy)]
pub struct WeakLoadInst { pub dst: ValueId, pub weak_ref: ValueId }
impl WeakLoadInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::WeakLoad { dst, weak_ref } => Some(WeakLoadInst { dst: *dst, weak_ref: *weak_ref }), _ => None }
    }
}
impl InstructionMeta for WeakLoadInst {
    fn effects(&self) -> EffectMask { EffectMask::READ }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.weak_ref] }
}

#[derive(Debug, Clone, Copy)]
pub struct WeakRefInst { pub dst: ValueId, pub op: MirWeakRefOp, pub value: ValueId }
impl WeakRefInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::WeakRef { dst, op, value } => Some(WeakRefInst { dst: *dst, op: *op, value: *value }), _ => None }
    }
}
impl InstructionMeta for WeakRefInst {
    fn effects(&self) -> EffectMask {
        match self.op {
            MirWeakRefOp::New => EffectMask::PURE,
            MirWeakRefOp::Load => EffectMask::READ,
        }
    }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
}

// ---- Future ops ----
#[derive(Debug, Clone, Copy)]
pub struct FutureNewInst { pub dst: ValueId, pub value: ValueId }
impl FutureNewInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::FutureNew { dst, value } => Some(FutureNewInst { dst: *dst, value: *value }), _ => None }
    }
}
impl InstructionMeta for FutureNewInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE.add(Effect::Alloc) }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
}

#[derive(Debug, Clone, Copy)]
pub struct FutureSetInst { pub future: ValueId, pub value: ValueId }
impl FutureSetInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::FutureSet { future, value } => Some(FutureSetInst { future: *future, value: *value }), _ => None }
    }
}
impl InstructionMeta for FutureSetInst {
    fn effects(&self) -> EffectMask { EffectMask::WRITE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.future, self.value] }
}

#[derive(Debug, Clone, Copy)]
pub struct AwaitInst { pub dst: ValueId, pub future: ValueId }
impl AwaitInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::Await { dst, future } => Some(AwaitInst { dst: *dst, future: *future }), _ => None }
    }
}
impl InstructionMeta for AwaitInst {
    fn effects(&self) -> EffectMask { EffectMask::READ.add(Effect::Async) }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.future] }
}

// ---- UnaryOp ----
#[derive(Debug, Clone, Copy)]
pub struct UnaryOpInst {
    pub dst: ValueId,
    pub operand: ValueId,
}

impl UnaryOpInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::UnaryOp { dst, operand, .. } => Some(UnaryOpInst { dst: *dst, operand: *operand }),
            _ => None,
        }
    }
}

impl InstructionMeta for UnaryOpInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.operand] }
}

// ---- Compare ----
#[derive(Debug, Clone, Copy)]
pub struct CompareInst {
    pub dst: ValueId,
    pub lhs: ValueId,
    pub rhs: ValueId,
}

impl CompareInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Compare { dst, lhs, rhs, .. } => Some(CompareInst { dst: *dst, lhs: *lhs, rhs: *rhs }),
            _ => None,
        }
    }
}

impl InstructionMeta for CompareInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.lhs, self.rhs] }
}

// ---- Load ----
#[derive(Debug, Clone, Copy)]
pub struct LoadInst {
    pub dst: ValueId,
    pub ptr: ValueId,
}

impl LoadInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Load { dst, ptr } => Some(LoadInst { dst: *dst, ptr: *ptr }),
            _ => None,
        }
    }
}

impl InstructionMeta for LoadInst {
    fn effects(&self) -> EffectMask { EffectMask::READ }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.ptr] }
}

// ---- Cast ----
#[derive(Debug, Clone)]
pub struct CastInst {
    pub dst: ValueId,
    pub value: ValueId,
    pub target_type: MirType,
}

impl CastInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Cast { dst, value, target_type } =>
                Some(CastInst { dst: *dst, value: *value, target_type: target_type.clone() }),
            _ => None,
        }
    }
}

impl InstructionMeta for CastInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
}

// ---- TypeOp ----
#[derive(Debug, Clone)]
pub struct TypeOpInst {
    pub dst: ValueId,
    pub op: MirTypeOpKind,
    pub value: ValueId,
    pub ty: MirType,
}

impl TypeOpInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::TypeOp { dst, op, value, ty } =>
                Some(TypeOpInst { dst: *dst, op: *op, value: *value, ty: ty.clone() }),
            _ => None,
        }
    }
}

impl InstructionMeta for TypeOpInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
}

// ---- ArrayGet ----
#[derive(Debug, Clone, Copy)]
pub struct ArrayGetInst {
    pub dst: ValueId,
    pub array: ValueId,
    pub index: ValueId,
}

impl ArrayGetInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::ArrayGet { dst, array, index } =>
                Some(ArrayGetInst { dst: *dst, array: *array, index: *index }),
            _ => None,
        }
    }
}

impl InstructionMeta for ArrayGetInst {
    fn effects(&self) -> EffectMask { EffectMask::READ }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { vec![self.array, self.index] }
}

// ---- Phi ----
#[derive(Debug, Clone)]
pub struct PhiInst { pub dst: ValueId, pub inputs: Vec<(BasicBlockId, ValueId)> }

impl PhiInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Phi { dst, inputs } => Some(PhiInst { dst: *dst, inputs: inputs.clone() }),
            _ => None,
        }
    }
}

impl InstructionMeta for PhiInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { self.inputs.iter().map(|(_, v)| *v).collect() }
}

// ---- NewBox ----
#[derive(Debug, Clone)]
pub struct NewBoxInst {
    pub dst: ValueId,
    pub args: Vec<ValueId>,
}

impl NewBoxInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::NewBox { dst, args, .. } =>
                Some(NewBoxInst { dst: *dst, args: args.clone() }),
            _ => None,
        }
    }
}

impl InstructionMeta for NewBoxInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE.add(Effect::Alloc) }
    fn dst(&self) -> Option<ValueId> { Some(self.dst) }
    fn used(&self) -> Vec<ValueId> { self.args.clone() }
}

// ---- Store ----
#[derive(Debug, Clone, Copy)]
pub struct StoreInst {
    pub value: ValueId,
    pub ptr: ValueId,
}

impl StoreInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Store { value, ptr } => Some(StoreInst { value: *value, ptr: *ptr }),
            _ => None,
        }
    }
}

impl InstructionMeta for StoreInst {
    fn effects(&self) -> EffectMask { EffectMask::WRITE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.value, self.ptr] }
}

// ---- ArraySet ----
#[derive(Debug, Clone, Copy)]
pub struct ArraySetInst {
    pub array: ValueId,
    pub index: ValueId,
    pub value: ValueId,
}

impl ArraySetInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::ArraySet { array, index, value } =>
                Some(ArraySetInst { array: *array, index: *index, value: *value }),
            _ => None,
        }
    }
}

impl InstructionMeta for ArraySetInst {
    fn effects(&self) -> EffectMask { EffectMask::WRITE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.array, self.index, self.value] }
}

// ---- Return ----
#[derive(Debug, Clone, Copy)]
pub struct ReturnInst { pub value: Option<ValueId> }

impl ReturnInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Return { value } => Some(ReturnInst { value: *value }),
            _ => None,
        }
    }
}

impl InstructionMeta for ReturnInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { self.value.map(|v| vec![v]).unwrap_or_default() }
}

// ---- Branch ----
#[derive(Debug, Clone, Copy)]
pub struct BranchInst { pub condition: ValueId }

impl BranchInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Branch { condition, .. } => Some(BranchInst { condition: *condition }),
            _ => None,
        }
    }
}

impl InstructionMeta for BranchInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.condition] }
}

// ---- Jump ----
#[derive(Debug, Clone, Copy)]
pub struct JumpInst;

impl JumpInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::Jump { .. } => Some(JumpInst), _ => None }
    }
}

impl InstructionMeta for JumpInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { Vec::new() }
}

// ---- Print ----
#[derive(Debug, Clone, Copy)]
pub struct PrintInst { pub value: ValueId, pub effects_mask: EffectMask }

impl PrintInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i {
            MirInstruction::Print { value, effects } => Some(PrintInst { value: *value, effects_mask: *effects }),
            _ => None,
        }
    }
}

impl InstructionMeta for PrintInst {
    fn effects(&self) -> EffectMask { self.effects_mask }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
}

// ---- Debug ----
#[derive(Debug, Clone, Copy)]
pub struct DebugInst { pub value: ValueId }

impl DebugInst {
    pub fn from_mir(i: &MirInstruction) -> Option<Self> {
        match i { MirInstruction::Debug { value, .. } => Some(DebugInst { value: *value }), _ => None }
    }
}

impl InstructionMeta for DebugInst {
    fn effects(&self) -> EffectMask { EffectMask::PURE.add(Effect::Debug) }
    fn dst(&self) -> Option<ValueId> { None }
    fn used(&self) -> Vec<ValueId> { vec![self.value] }
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
