/*!
 * MIR Instruction Set - 20 Core Instructions per ChatGPT5 Design
 *
 * SSA-form instructions with effect tracking for optimization
 */

use super::{EffectMask, ValueId};
use crate::mir::definitions::Callee;  // Import Callee from unified definitions
use crate::mir::types::{
    BarrierOp, BinaryOp, CompareOp, ConstValue, MirType, TypeOpKind, UnaryOp, WeakRefOp,
};


// Kind-specific metadata (non-functional refactor scaffolding)

/// MIR instruction types - limited to 20 core instructions
#[derive(Debug, Clone, PartialEq)]
pub enum MirInstruction {
    // === Constants and Values ===
    /// Load a constant value
    /// `%dst = const value`
    Const { dst: ValueId, value: ConstValue },

    // === Arithmetic Operations ===
    /// Binary arithmetic operation
    /// `%dst = %lhs op %rhs`
    BinOp {
        dst: ValueId,
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    },

    /// Unary operation
    /// `%dst = op %operand`
    UnaryOp {
        dst: ValueId,
        op: UnaryOp,
        operand: ValueId,
    },

    // === Comparison Operations ===
    /// Compare two values
    /// `%dst = %lhs cmp %rhs`
    Compare {
        dst: ValueId,
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    },

    // === Memory Operations ===
    /// Load from memory/variable
    /// `%dst = load %ptr`
    Load { dst: ValueId, ptr: ValueId },

    /// Store to memory/variable
    /// `store %value -> %ptr`
    Store { value: ValueId, ptr: ValueId },

    // === Function Calls ===
    /// Call a function with type-safe target resolution
    /// `%dst = call %func(%args...)` (legacy)
    /// `%dst = call Global("print")(%args...)` (new)
    ///
    /// Phase 1 Migration: Both func and callee fields present
    /// - callee: Some(_) -> Use new type-safe resolution (preferred)
    /// Note: `callee` is set for all builder-emitted calls. The deprecated legacy mode with `callee: None` is no longer used in the builder.
    Call {
        dst: Option<ValueId>,
        func: ValueId,              // Legacy: string-based resolution (deprecated)
        callee: Option<Callee>,     // New: type-safe resolution (preferred)
        args: Vec<ValueId>,
        effects: EffectMask,
    },

    /// Create a function value (FunctionBox) from params/body and optional captures
    /// `%dst = new_closure [params] {body} [captures...]`
    /// Minimal lowering support: captures may be empty; 'me' is optional.
    NewClosure {
        dst: ValueId,
        params: Vec<String>,
        body: Vec<crate::ast::ASTNode>,
        /// Pairs of (name, value) to capture by value
        captures: Vec<(String, ValueId)>,
        /// Optional 'me' value to capture weakly if it is a BoxRef at runtime
        me: Option<ValueId>,
    },

    /// Box method invocation
    /// `%dst = invoke %box.method(%args...)`
    /// method_id: Optional numeric slot id when resolved at build time
    BoxCall {
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        /// Optional numeric method slot id (Unified Registry). None = late bind.
        method_id: Option<u16>,
        args: Vec<ValueId>,
        effects: EffectMask,
    },

    /// Plugin invocation (forces plugin path; no builtin fallback)
    /// `%dst = plugin_invoke %box.method(%args...)`
    PluginInvoke {
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        args: Vec<ValueId>,
        effects: EffectMask,
    },

    // === Control Flow ===
    /// Conditional branch
    /// `br %condition -> %then_bb, %else_bb`
    Branch {
        condition: ValueId,
        then_bb: super::BasicBlockId,
        else_bb: super::BasicBlockId,
    },

    /// Unconditional jump
    /// `jmp %target_bb`
    Jump { target: super::BasicBlockId },

    /// Return from function
    /// `ret %value` or `ret void`
    Return { value: Option<ValueId> },

    // === SSA Phi Function ===
    /// SSA phi function for merging values from different paths
    /// `%dst = phi [%val1 from %bb1, %val2 from %bb2, ...]`
    Phi {
        dst: ValueId,
        inputs: Vec<(super::BasicBlockId, ValueId)>,
    },

    // === Box Operations ===
    /// Create a new Box instance
    /// `%dst = new_box "BoxType"(%args...)`
    NewBox {
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
        /// Optional fully-qualified birth name (e.g., "Class.birth/N").
        /// When present, backends may invoke birth immediately after allocation
        /// (C++-style constructor semantics).
        auto_birth: Option<String>,
    },

    /// Check Box type
    /// `%dst = type_check %box "BoxType"`
    TypeCheck {
        dst: ValueId,
        value: ValueId,
        expected_type: String,
    },

    // === Type Conversion ===
    /// Convert between types
    /// `%dst = cast %value as Type`
    Cast {
        dst: ValueId,
        value: ValueId,
        target_type: MirType,
    },

    // === Type Operations (Unified PoC) ===
    /// Unified type operation (PoC): Check or Cast
    /// `%dst = typeop(check|cast, %value, Type)`
    TypeOp {
        dst: ValueId,
        op: TypeOpKind,
        value: ValueId,
        ty: MirType,
    },

    // === Array Operations ===
    /// Get array element
    /// `%dst = %array[%index]`
    ArrayGet {
        dst: ValueId,
        array: ValueId,
        index: ValueId,
    },

    /// Set array element
    /// `%array[%index] = %value`
    ArraySet {
        array: ValueId,
        index: ValueId,
        value: ValueId,
    },

    // === Special Operations ===
    /// Copy a value (for optimization passes)
    /// `%dst = copy %src`
    Copy { dst: ValueId, src: ValueId },

    /// Debug/introspection instruction
    /// `debug %value "message"`
    Debug { value: ValueId, message: String },

    /// Print instruction for console output
    /// `print %value`
    Print { value: ValueId, effects: EffectMask },

    /// No-op instruction (for optimization placeholders)
    Nop,

    // === Control Flow & Exception Handling (Phase 5) ===
    /// Throw an exception
    /// `throw %exception_value`
    Throw {
        exception: ValueId,
        effects: EffectMask,
    },

    /// Catch handler setup (landing pad for exceptions)
    /// `catch %exception_type -> %handler_bb`
    Catch {
        exception_type: Option<String>, // None = catch-all
        exception_value: ValueId,       // Where to store caught exception
        handler_bb: super::BasicBlockId,
    },

    /// Safepoint instruction (no-op for now, can be used for GC/debugging)
    /// `safepoint`
    Safepoint,

    // === Phase 6: Box Reference Operations ===
    /// Create a new reference to a Box
    /// `%dst = ref_new %box`
    RefNew { dst: ValueId, box_val: ValueId },

    /// Get/dereference a Box field through reference
    /// `%dst = ref_get %ref.field`
    RefGet {
        dst: ValueId,
        reference: ValueId,
        field: String,
    },

    /// Set/assign Box field through reference
    /// `ref_set %ref.field = %value`
    RefSet {
        reference: ValueId,
        field: String,
        value: ValueId,
    },

    /// Create a weak reference to a Box
    /// `%dst = weak_new %box`
    WeakNew { dst: ValueId, box_val: ValueId },

    /// Load from weak reference (if still alive)
    /// `%dst = weak_load %weak_ref`
    WeakLoad { dst: ValueId, weak_ref: ValueId },

    /// Memory barrier read (no-op for now, proper effect annotation)
    /// `barrier_read %ptr`
    BarrierRead { ptr: ValueId },

    /// Memory barrier write (no-op for now, proper effect annotation)
    /// `barrier_write %ptr`
    BarrierWrite { ptr: ValueId },

    // === Unified PoC: WeakRef/Barrier (flags-only scaffolding) ===
    /// Unified weak reference op (PoC)
    /// `%dst = weakref new %box` or `%dst = weakref load %weak`
    WeakRef {
        dst: ValueId,
        op: WeakRefOp,
        value: ValueId,
    },

    /// Unified barrier op (PoC)
    /// `barrier read %ptr` or `barrier write %ptr`
    Barrier { op: BarrierOp, ptr: ValueId },

    // === Phase 7: Async/Future Operations ===
    /// Create a new Future with initial value
    /// `%dst = future_new %value`
    FutureNew { dst: ValueId, value: ValueId },

    /// Set Future value and mark as ready
    /// `future_set %future = %value`
    FutureSet { future: ValueId, value: ValueId },

    /// Wait for Future completion and get value
    /// `%dst = await %future`
    Await { dst: ValueId, future: ValueId },

    // === Phase 9.7: External Function Calls (Box FFI/ABI) ===
    /// External function call through Box FFI/ABI
    /// `%dst = extern_call interface.method(%args...)`
    ExternCall {
        dst: Option<ValueId>,
        iface_name: String,  // e.g., "env.console"
        method_name: String, // e.g., "log"
        args: Vec<ValueId>,
        effects: EffectMask,
    },
}



// Method implementations have been moved to src/mir/instruction/methods.rs
#[path = "instruction/methods.rs"]
mod methods;

// Display implementation has been moved to src/mir/instruction/display.rs
#[path = "instruction/display.rs"]
mod display;

// Tests have been moved to src/mir/instruction/tests.rs for better organization
#[cfg(test)]
#[path = "instruction/tests.rs"]
mod tests;
