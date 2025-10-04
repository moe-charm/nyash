//! Method implementations for MIR Instructions
//!
//! Contains utility methods for MIR instruction analysis including:
//! - Effect tracking (effects())
//! - Destination value extraction (dst_value())
//! - Used values collection (used_values())

use super::super::{Effect, EffectMask, ValueId};
use crate::mir::instruction::MirInstruction;
use crate::mir::instruction_kinds as inst_meta;
use crate::mir::types::{BarrierOp, ConstValue, WeakRefOp};

impl MirInstruction {
    /// Get the effect mask for this instruction
    pub fn effects(&self) -> EffectMask {
        if let Some(eff) = inst_meta::effects_via_meta(self) {
            return eff;
        }
        match self {
            // Pure operations (with refined exceptions below)
            MirInstruction::Const { .. } => EffectMask::PURE,
            // Binary operations are generally pure, but some variants may panic at runtime
            // (e.g., division/modulo by zero). Mark such ops as having Panic effect so that
            // passes like DCE do not eliminate them even if their result is unused.
            MirInstruction::BinOp { op, .. } => {
                use crate::mir::types::BinaryOp;
                match op {
                    BinaryOp::Div | BinaryOp::Mod => EffectMask::PURE.add(Effect::Panic),
                    _ => EffectMask::PURE,
                }
            }
            MirInstruction::UnaryOp { .. }
            | MirInstruction::Compare { .. }
            | MirInstruction::Cast { .. }
            | MirInstruction::TypeOp { .. }
            | MirInstruction::Copy { .. }
            | MirInstruction::Phi { .. }
            | MirInstruction::TypeCheck { .. }
            | MirInstruction::Nop => EffectMask::PURE,

            // Memory operations
            MirInstruction::Load { .. } => EffectMask::READ,
            MirInstruction::Store { .. } | MirInstruction::ArraySet { .. } => EffectMask::WRITE,
            MirInstruction::ArrayGet { .. } => EffectMask::READ,

            // Function calls use provided effect mask
            MirInstruction::Call { effects, .. }
            | MirInstruction::BoxCall { effects, .. }
            | MirInstruction::PluginInvoke { effects, .. } => *effects,

            // Control flow (pure but affects execution)
            MirInstruction::Branch { .. }
            | MirInstruction::Jump { .. }
            | MirInstruction::Return { .. } => EffectMask::PURE,

            // Box creation may allocate
            MirInstruction::NewBox { .. } => EffectMask::PURE.add(Effect::Alloc),

            // Debug has debug effect
            MirInstruction::Debug { .. } => EffectMask::PURE.add(Effect::Debug),

            // Print has external write effect
            MirInstruction::Print { effects, .. } => *effects,

            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { effects, .. } => *effects,
            MirInstruction::Catch { .. } => EffectMask::CONTROL, // Handler setup affects control handling
            MirInstruction::Safepoint => EffectMask::PURE,       // No-op for now

            // Phase 6: Box reference operations
            MirInstruction::RefNew { .. } => EffectMask::PURE, // Creating reference is pure
            MirInstruction::RefGet { .. } => EffectMask::READ, // Reading field has read effects
            MirInstruction::RefSet { .. } => EffectMask::WRITE, // Writing field has write effects
            MirInstruction::WeakNew { .. } => EffectMask::PURE, // Creating weak ref is pure
            MirInstruction::WeakLoad { .. } => EffectMask::READ, // Loading weak ref has read effects
            MirInstruction::BarrierRead { .. } => EffectMask::READ.add(Effect::Barrier), // Memory barrier with read
            MirInstruction::BarrierWrite { .. } => EffectMask::WRITE.add(Effect::Barrier), // Memory barrier with write
            // PoC unified ops mirror legacy effects
            MirInstruction::WeakRef { op, .. } => match op {
                WeakRefOp::New => EffectMask::PURE,
                WeakRefOp::Load => EffectMask::READ,
            },
            MirInstruction::Barrier { op, .. } => match op {
                BarrierOp::Read => EffectMask::READ.add(Effect::Barrier),
                BarrierOp::Write => EffectMask::WRITE.add(Effect::Barrier),
            },

            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { .. } => EffectMask::PURE.add(Effect::Alloc), // Creating future may allocate
            MirInstruction::FutureSet { .. } => EffectMask::WRITE, // Setting future has write effects
            MirInstruction::Await { .. } => EffectMask::READ.add(Effect::Async), // Await blocks and reads

            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { effects, .. } => *effects, // Use provided effect mask
            // Function value construction: treat as pure with allocation
            MirInstruction::NewClosure { .. } => EffectMask::PURE.add(Effect::Alloc),
        }
    }

    /// Get the destination ValueId if this instruction produces a value
    pub fn dst_value(&self) -> Option<ValueId> {
        if let Some(dst) = inst_meta::dst_via_meta(self) {
            return Some(dst);
        }
        match self {
            MirInstruction::Const { dst, .. }
            | MirInstruction::BinOp { dst, .. }
            | MirInstruction::UnaryOp { dst, .. }
            | MirInstruction::Compare { dst, .. }
            | MirInstruction::Load { dst, .. }
            | MirInstruction::Phi { dst, .. }
            | MirInstruction::NewBox { dst, .. }
            | MirInstruction::TypeCheck { dst, .. }
            | MirInstruction::Cast { dst, .. }
            | MirInstruction::TypeOp { dst, .. }
            | MirInstruction::ArrayGet { dst, .. }
            | MirInstruction::Copy { dst, .. }
            | MirInstruction::RefNew { dst, .. }
            | MirInstruction::RefGet { dst, .. }
            | MirInstruction::WeakNew { dst, .. }
            | MirInstruction::WeakLoad { dst, .. }
            | MirInstruction::WeakRef { dst, .. }
            | MirInstruction::FutureNew { dst, .. }
            | MirInstruction::Await { dst, .. } => Some(*dst),
            MirInstruction::NewClosure { dst, .. } => Some(*dst),

            MirInstruction::Call { dst, .. }
            | MirInstruction::BoxCall { dst, .. }
            | MirInstruction::PluginInvoke { dst, .. }
            | MirInstruction::ExternCall { dst, .. } => *dst,

            MirInstruction::Store { .. }
            | MirInstruction::Branch { .. }
            | MirInstruction::Jump { .. }
            | MirInstruction::Return { .. }
            | MirInstruction::ArraySet { .. }
            | MirInstruction::Debug { .. }
            | MirInstruction::Print { .. }
            | MirInstruction::Throw { .. }
            | MirInstruction::RefSet { .. }
            | MirInstruction::BarrierRead { .. }
            | MirInstruction::BarrierWrite { .. }
            | MirInstruction::Barrier { .. }
            | MirInstruction::FutureSet { .. }
            | MirInstruction::Safepoint
            | MirInstruction::Nop => None,

            MirInstruction::Catch {
                exception_value, ..
            } => Some(*exception_value),
        }
    }

    /// Get all ValueIds used by this instruction
    pub fn used_values(&self) -> Vec<ValueId> {
        if let Some(used) = inst_meta::used_via_meta(self) {
            return used;
        }
        match self {
            MirInstruction::Const { .. } | MirInstruction::Jump { .. } | MirInstruction::Nop => {
                Vec::new()
            }

            MirInstruction::UnaryOp { operand, .. }
            | MirInstruction::Load { ptr: operand, .. }
            | MirInstruction::TypeCheck { value: operand, .. }
            | MirInstruction::Cast { value: operand, .. }
            | MirInstruction::TypeOp { value: operand, .. }
            | MirInstruction::Copy { src: operand, .. }
            | MirInstruction::Debug { value: operand, .. }
            | MirInstruction::Print { value: operand, .. } => vec![*operand],

            MirInstruction::BinOp { lhs, rhs, .. }
            | MirInstruction::Compare { lhs, rhs, .. }
            | MirInstruction::Store {
                value: lhs,
                ptr: rhs,
                ..
            } => vec![*lhs, *rhs],

            MirInstruction::ArrayGet { array, index, .. } => vec![*array, *index],

            MirInstruction::ArraySet {
                array,
                index,
                value,
            } => vec![*array, *index, *value],

            MirInstruction::Branch { condition, .. } => vec![*condition],

            MirInstruction::Return { value } => value.map(|v| vec![v]).unwrap_or_default(),

            MirInstruction::Call { func, args, .. } => {
                let mut used = vec![*func];
                used.extend(args);
                used
            }
            MirInstruction::NewClosure { captures, me, .. } => {
                let mut used: Vec<ValueId> = Vec::new();
                used.extend(captures.iter().map(|(_, v)| *v));
                if let Some(m) = me {
                    used.push(*m);
                }
                used
            }

            MirInstruction::BoxCall { box_val, args, .. }
            | MirInstruction::PluginInvoke { box_val, args, .. } => {
                let mut used = vec![*box_val];
                used.extend(args);
                used
            }

            MirInstruction::NewBox { args, .. } => args.clone(),

            MirInstruction::Phi { inputs, .. } => inputs.iter().map(|(_, value)| *value).collect(),

            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { exception, .. } => vec![*exception],
            MirInstruction::Catch { .. } => Vec::new(), // Handler setup doesn't use values
            MirInstruction::Safepoint => Vec::new(),

            // Phase 6: Box reference operations
            MirInstruction::RefNew { box_val, .. } => vec![*box_val],
            MirInstruction::RefGet { reference, .. } => vec![*reference],
            MirInstruction::RefSet {
                reference, value, ..
            } => vec![*reference, *value],
            MirInstruction::WeakNew { box_val, .. } => vec![*box_val],
            MirInstruction::WeakLoad { weak_ref, .. } => vec![*weak_ref],
            MirInstruction::BarrierRead { ptr } => vec![*ptr],
            MirInstruction::BarrierWrite { ptr } => vec![*ptr],
            MirInstruction::WeakRef { value, .. } => vec![*value],
            MirInstruction::Barrier { ptr, .. } => vec![*ptr],

            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { value, .. } => vec![*value],
            MirInstruction::FutureSet { future, value } => vec![*future, *value],
            MirInstruction::Await { future, .. } => vec![*future],

            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { args, .. } => args.clone(),
        }
    }
}


impl ConstValue {
    /*
    /// Convert to NyashValue
    pub fn to_nyash_value(&self) -> NyashValue {
        match self {
            ConstValue::Integer(n) => NyashValue::new_integer(*n),
            ConstValue::Float(f) => NyashValue::new_float(*f),
            ConstValue::Bool(b) => NyashValue::new_bool(*b),
            ConstValue::String(s) => NyashValue::new_string(s.clone()),
            ConstValue::Null => NyashValue::new_null(),
            ConstValue::Void => NyashValue::new_void(),
        }
    }

    /// Create from NyashValue
    pub fn from_nyash_value(value: &NyashValue) -> Option<Self> {
        match value {
            NyashValue::Integer(n) => Some(ConstValue::Integer(*n)),
            NyashValue::Float(f) => Some(ConstValue::Float(*f)),
            NyashValue::Bool(b) => Some(ConstValue::Bool(*b)),
            NyashValue::String(s) => Some(ConstValue::String(s.clone())),
            NyashValue::Null => Some(ConstValue::Null),
            NyashValue::Void => Some(ConstValue::Void),
            _ => None, // Collections and Boxes can't be constants
        }
    }
    */
}
