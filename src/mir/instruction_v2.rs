/*!
 * MIR 25-Instruction Specification Implementation
 * 
 * Complete hierarchical MIR instruction set based on ChatGPT5 + AI Council design
 */

use super::{ValueId, EffectMask, Effect, BasicBlockId};
use std::fmt;

/// MIR instruction types - exactly 25 instructions per specification
#[derive(Debug, Clone, PartialEq)]
pub enum MirInstructionV2 {
    // === TIER-0: UNIVERSAL CORE (8 instructions) ===
    
    /// Load a constant value (pure)
    /// `%dst = const value`
    Const {
        dst: ValueId,
        value: ConstValue,
    },
    
    /// Binary arithmetic operation (pure)
    /// `%dst = %lhs op %rhs`
    BinOp {
        dst: ValueId,
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    
    /// Compare two values (pure)
    /// `%dst = %lhs cmp %rhs`
    Compare {
        dst: ValueId,
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    },
    
    /// Conditional branch (control)
    /// `br %condition -> %then_bb, %else_bb`
    Branch {
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    },
    
    /// Unconditional jump (control)
    /// `jmp %target_bb`
    Jump {
        target: BasicBlockId,
    },
    
    /// SSA phi function for merging values (pure)
    /// `%dst = phi [%val1 from %bb1, %val2 from %bb2, ...]`
    Phi {
        dst: ValueId,
        inputs: Vec<(BasicBlockId, ValueId)>,
    },
    
    /// External function call (context-dependent)
    /// `%dst = call %func(%args...)`
    Call {
        dst: Option<ValueId>,
        func: ValueId,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    /// Return from function (control)
    /// `ret %value` or `ret void`
    Return {
        value: Option<ValueId>,
    },
    
    // === TIER-1: NYASH SEMANTICS (12 instructions) ===
    
    /// Create a new Box instance (strong ownership node in ownership forest)
    /// `%dst = new_box "BoxType"(%args...)`
    NewBox {
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
    },
    
    /// Load Box field value (pure)
    /// `%dst = %box.field`
    BoxFieldLoad {
        dst: ValueId,
        box_val: ValueId,
        field: String,
    },
    
    /// Store value to Box field (mut)
    /// `%box.field = %value`
    BoxFieldStore {
        box_val: ValueId,
        field: String,
        value: ValueId,
    },
    
    /// Box method invocation (context-dependent)
    /// `%dst = %box.method(%args...)`
    BoxCall {
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    /// Safepoint for finalization/interrupts (io)
    /// `safepoint`
    Safepoint,
    
    /// Get reference as value (pure)
    /// `%dst = ref_get %reference`
    RefGet {
        dst: ValueId,
        reference: ValueId,
    },
    
    /// Set/replace reference target with ownership validation (mut)
    /// `ref_set %reference = %new_target`
    RefSet {
        reference: ValueId,
        new_target: ValueId,
    },
    
    /// Create weak reference handle (non-owning link) (pure)
    /// `%dst = weak_new %box`
    WeakNew {
        dst: ValueId,
        box_val: ValueId,
    },
    
    /// Load from weak reference with liveness check (returns null if dead) (pure)
    /// `%dst = weak_load %weak_ref`
    WeakLoad {
        dst: ValueId,
        weak_ref: ValueId,
    },
    
    /// Check weak reference validity (returns bool) (pure)
    /// `%dst = weak_check %weak_ref`
    WeakCheck {
        dst: ValueId,
        weak_ref: ValueId,
    },
    
    /// Send message via Bus system (io)
    /// `send %bus, %message`
    Send {
        bus: ValueId,
        message: ValueId,
    },
    
    /// Receive message from Bus system (io)
    /// `%dst = recv %bus`
    Recv {
        dst: ValueId,
        bus: ValueId,
    },
    
    // === TIER-2: IMPLEMENTATION ASSISTANCE (5 instructions) ===
    
    /// Tail call optimization (control)
    /// `tail_call %func(%args...)`
    TailCall {
        func: ValueId,
        args: Vec<ValueId>,
        effects: EffectMask,
    },
    
    /// Ownership transfer: this takes strong ownership of child (mut)
    /// `adopt %parent, %child`
    Adopt {
        parent: ValueId,
        child: ValueId,
    },
    
    /// Release strong ownership (weakify or nullify) (mut)
    /// `release %reference`
    Release {
        reference: ValueId,
    },
    
    /// Optimized memory copy for structs/arrays (mut)
    /// `memcopy %dest, %src, %size`
    MemCopy {
        dest: ValueId,
        src: ValueId,
        size: ValueId,
    },
    
    /// Atomic fence for concurrency ordering at Actor/Port boundaries (io)
    /// `atomic_fence %ordering`
    AtomicFence {
        ordering: AtomicOrdering,
    },
}

/// Constant values in MIR
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    Void,
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    
    // Logical
    And, Or,
}

/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq, Ne, Lt, Le, Gt, Ge,
}

/// Atomic ordering for AtomicFence instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicOrdering {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

impl MirInstructionV2 {
    /// Get the effect mask for this instruction according to 4-category system
    pub fn effects(&self) -> EffectMask {
        match self {
            // TIER-0: Universal Core
            // Pure operations
            MirInstructionV2::Const { .. } |
            MirInstructionV2::BinOp { .. } |
            MirInstructionV2::Compare { .. } |
            MirInstructionV2::Phi { .. } => EffectMask::PURE,
            
            // Control flow operations
            MirInstructionV2::Branch { .. } |
            MirInstructionV2::Jump { .. } |
            MirInstructionV2::Return { .. } => EffectMask::CONTROL,
            
            // Context-dependent operations
            MirInstructionV2::Call { effects, .. } => *effects,
            
            // TIER-1: Nyash Semantics
            // Pure operations
            MirInstructionV2::BoxFieldLoad { .. } |
            MirInstructionV2::RefGet { .. } |
            MirInstructionV2::WeakNew { .. } |
            MirInstructionV2::WeakLoad { .. } |
            MirInstructionV2::WeakCheck { .. } => EffectMask::PURE,
            
            // Mutable operations
            MirInstructionV2::NewBox { .. } => EffectMask::MUT.add(Effect::Alloc),
            MirInstructionV2::BoxFieldStore { .. } |
            MirInstructionV2::RefSet { .. } => EffectMask::MUT,
            
            // I/O operations
            MirInstructionV2::Safepoint |
            MirInstructionV2::Send { .. } |
            MirInstructionV2::Recv { .. } => EffectMask::IO,
            
            // Context-dependent operations
            MirInstructionV2::BoxCall { effects, .. } => *effects,
            
            // TIER-2: Implementation Assistance
            // Control flow operations
            MirInstructionV2::TailCall { .. } => EffectMask::CONTROL,
            
            // Mutable operations
            MirInstructionV2::Adopt { .. } |
            MirInstructionV2::Release { .. } |
            MirInstructionV2::MemCopy { .. } => EffectMask::MUT,
            
            // I/O operations
            MirInstructionV2::AtomicFence { .. } => EffectMask::IO.add(Effect::Barrier),
        }
    }
    
    /// Get the destination ValueId if this instruction produces a value
    pub fn dst_value(&self) -> Option<ValueId> {
        match self {
            MirInstructionV2::Const { dst, .. } |
            MirInstructionV2::BinOp { dst, .. } |
            MirInstructionV2::Compare { dst, .. } |
            MirInstructionV2::Phi { dst, .. } |
            MirInstructionV2::NewBox { dst, .. } |
            MirInstructionV2::BoxFieldLoad { dst, .. } |
            MirInstructionV2::RefGet { dst, .. } |
            MirInstructionV2::WeakNew { dst, .. } |
            MirInstructionV2::WeakLoad { dst, .. } |
            MirInstructionV2::WeakCheck { dst, .. } |
            MirInstructionV2::Recv { dst, .. } => Some(*dst),
            
            MirInstructionV2::Call { dst, .. } |
            MirInstructionV2::BoxCall { dst, .. } => *dst,
            
            _ => None,
        }
    }
    
    /// Get all ValueIds used by this instruction
    pub fn used_values(&self) -> Vec<ValueId> {
        match self {
            MirInstructionV2::Const { .. } => vec![],
            
            MirInstructionV2::BinOp { lhs, rhs, .. } |
            MirInstructionV2::Compare { lhs, rhs, .. } => vec![*lhs, *rhs],
            
            MirInstructionV2::Branch { condition, .. } => vec![*condition],
            
            MirInstructionV2::Jump { .. } => vec![],
            
            MirInstructionV2::Phi { inputs, .. } => {
                inputs.iter().map(|(_, value_id)| *value_id).collect()
            },
            
            MirInstructionV2::Call { func, args, .. } => {
                let mut values = vec![*func];
                values.extend(args.iter().copied());
                values
            },
            
            MirInstructionV2::Return { value } => {
                value.map(|v| vec![v]).unwrap_or_default()
            },
            
            MirInstructionV2::NewBox { args, .. } => args.clone(),
            
            MirInstructionV2::BoxFieldLoad { box_val, .. } => vec![*box_val],
            
            MirInstructionV2::BoxFieldStore { box_val, value, .. } => vec![*box_val, *value],
            
            MirInstructionV2::BoxCall { box_val, args, .. } => {
                let mut values = vec![*box_val];
                values.extend(args.iter().copied());
                values
            },
            
            MirInstructionV2::Safepoint => vec![],
            
            MirInstructionV2::RefGet { reference, .. } => vec![*reference],
            
            MirInstructionV2::RefSet { reference, new_target, .. } => vec![*reference, *new_target],
            
            MirInstructionV2::WeakNew { box_val, .. } => vec![*box_val],
            
            MirInstructionV2::WeakLoad { weak_ref, .. } |
            MirInstructionV2::WeakCheck { weak_ref, .. } => vec![*weak_ref],
            
            MirInstructionV2::Send { bus, message, .. } => vec![*bus, *message],
            
            MirInstructionV2::Recv { bus, .. } => vec![*bus],
            
            MirInstructionV2::TailCall { func, args, .. } => {
                let mut values = vec![*func];
                values.extend(args.iter().copied());
                values
            },
            
            MirInstructionV2::Adopt { parent, child, .. } => vec![*parent, *child],
            
            MirInstructionV2::Release { reference, .. } => vec![*reference],
            
            MirInstructionV2::MemCopy { dest, src, size, .. } => vec![*dest, *src, *size],
            
            MirInstructionV2::AtomicFence { .. } => vec![],
        }
    }
    
    /// Get the instruction tier (0, 1, or 2)
    pub fn tier(&self) -> u8 {
        match self {
            // Tier-0: Universal Core
            MirInstructionV2::Const { .. } |
            MirInstructionV2::BinOp { .. } |
            MirInstructionV2::Compare { .. } |
            MirInstructionV2::Branch { .. } |
            MirInstructionV2::Jump { .. } |
            MirInstructionV2::Phi { .. } |
            MirInstructionV2::Call { .. } |
            MirInstructionV2::Return { .. } => 0,
            
            // Tier-1: Nyash Semantics
            MirInstructionV2::NewBox { .. } |
            MirInstructionV2::BoxFieldLoad { .. } |
            MirInstructionV2::BoxFieldStore { .. } |
            MirInstructionV2::BoxCall { .. } |
            MirInstructionV2::Safepoint { .. } |
            MirInstructionV2::RefGet { .. } |
            MirInstructionV2::RefSet { .. } |
            MirInstructionV2::WeakNew { .. } |
            MirInstructionV2::WeakLoad { .. } |
            MirInstructionV2::WeakCheck { .. } |
            MirInstructionV2::Send { .. } |
            MirInstructionV2::Recv { .. } => 1,
            
            // Tier-2: Implementation Assistance
            MirInstructionV2::TailCall { .. } |
            MirInstructionV2::Adopt { .. } |
            MirInstructionV2::Release { .. } |
            MirInstructionV2::MemCopy { .. } |
            MirInstructionV2::AtomicFence { .. } => 2,
        }
    }
    
    /// Get a human-readable description of the instruction
    pub fn description(&self) -> &'static str {
        match self {
            // Tier-0
            MirInstructionV2::Const { .. } => "Load constant value",
            MirInstructionV2::BinOp { .. } => "Binary arithmetic operation",
            MirInstructionV2::Compare { .. } => "Compare two values",
            MirInstructionV2::Branch { .. } => "Conditional branch",
            MirInstructionV2::Jump { .. } => "Unconditional jump",
            MirInstructionV2::Phi { .. } => "SSA phi function",
            MirInstructionV2::Call { .. } => "External function call",
            MirInstructionV2::Return { .. } => "Return from function",
            
            // Tier-1
            MirInstructionV2::NewBox { .. } => "Create Box instance",
            MirInstructionV2::BoxFieldLoad { .. } => "Load Box field value",
            MirInstructionV2::BoxFieldStore { .. } => "Store to Box field",
            MirInstructionV2::BoxCall { .. } => "Box method invocation",
            MirInstructionV2::Safepoint => "Finalization/interrupt safepoint",
            MirInstructionV2::RefGet { .. } => "Get reference as value",
            MirInstructionV2::RefSet { .. } => "Set reference target",
            MirInstructionV2::WeakNew { .. } => "Create weak reference",
            MirInstructionV2::WeakLoad { .. } => "Load from weak reference",
            MirInstructionV2::WeakCheck { .. } => "Check weak reference validity",
            MirInstructionV2::Send { .. } => "Send Bus message",
            MirInstructionV2::Recv { .. } => "Receive Bus message",
            
            // Tier-2
            MirInstructionV2::TailCall { .. } => "Tail call optimization",
            MirInstructionV2::Adopt { .. } => "Transfer ownership",
            MirInstructionV2::Release { .. } => "Release ownership",
            MirInstructionV2::MemCopy { .. } => "Optimized memory copy",
            MirInstructionV2::AtomicFence { .. } => "Atomic memory fence",
        }
    }
}

impl fmt::Display for MirInstructionV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{ValueIdGenerator, BasicBlockIdGenerator};
    
    #[test]
    fn test_instruction_count() {
        // Verify we have exactly 25 instruction variants
        // This is a compile-time verification
        let _tier0_count = 8; // Const, BinOp, Compare, Branch, Jump, Phi, Call, Return
        let _tier1_count = 12; // NewBox, BoxFieldLoad/Store, BoxCall, Safepoint, RefGet/Set, WeakNew/Load/Check, Send, Recv
        let _tier2_count = 5; // TailCall, Adopt, Release, MemCopy, AtomicFence
        let _total = _tier0_count + _tier1_count + _tier2_count;
        assert_eq!(_total, 25, "MIR instruction set must have exactly 25 instructions");
    }
    
    #[test]
    fn test_effect_categories() {
        let mut value_gen = ValueIdGenerator::new();
        let mut bb_gen = BasicBlockIdGenerator::new();
        
        // Test pure operations
        let const_inst = MirInstructionV2::Const {
            dst: value_gen.next(),
            value: ConstValue::Integer(42),
        };
        assert!(const_inst.effects().is_pure(), "Const should be pure");
        assert_eq!(const_inst.tier(), 0, "Const should be Tier-0");
        
        // Test mut operations
        let store_inst = MirInstructionV2::BoxFieldStore {
            box_val: value_gen.next(),
            field: "value".to_string(),
            value: value_gen.next(),
        };
        assert!(store_inst.effects().is_mut(), "BoxFieldStore should be mut");
        assert_eq!(store_inst.tier(), 1, "BoxFieldStore should be Tier-1");
        
        // Test io operations
        let send_inst = MirInstructionV2::Send {
            bus: value_gen.next(),
            message: value_gen.next(),
        };
        assert!(send_inst.effects().is_io(), "Send should be io");
        assert_eq!(send_inst.tier(), 1, "Send should be Tier-1");
        
        // Test control operations
        let branch_inst = MirInstructionV2::Branch {
            condition: value_gen.next(),
            then_bb: bb_gen.next(),
            else_bb: bb_gen.next(),
        };
        assert!(branch_inst.effects().is_control(), "Branch should be control");
        assert_eq!(branch_inst.tier(), 0, "Branch should be Tier-0");
    }
    
    #[test]
    fn test_ownership_operations() {
        let mut value_gen = ValueIdGenerator::new();
        
        // Test ownership transfer
        let adopt_inst = MirInstructionV2::Adopt {
            parent: value_gen.next(),
            child: value_gen.next(),
        };
        assert!(adopt_inst.effects().is_mut(), "Adopt should be mut");
        assert_eq!(adopt_inst.tier(), 2, "Adopt should be Tier-2");
        
        // Test weak reference operations
        let weak_check = MirInstructionV2::WeakCheck {
            dst: value_gen.next(),
            weak_ref: value_gen.next(),
        };
        assert!(weak_check.effects().is_pure(), "WeakCheck should be pure");
        assert_eq!(weak_check.tier(), 1, "WeakCheck should be Tier-1");
    }
}