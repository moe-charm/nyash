/*!
 * Semantics Layer (skeleton)
 *
 * Single source of truth for MIR semantics. Backends implement this trait
 * to realize the same MIR behavior in different targets (VM/Cranelift/LLVM/WASM).
 *
 * Phase 11.7 PoC: interface only — no wiring yet.
 */

#![allow(dead_code)]

use crate::mir::{BasicBlockId, ValueId};

/// The unified semantics interface for MIR evaluation/lowering.
pub trait Semantics {
    type Val: Clone;
    type Ptr: Clone;
    type BB: Copy + Clone;

    // Debug (optional)
    fn debug_location(&mut self, _line: u32, _col: u32) {}
    fn debug_value(&mut self, _name: &str, _val: &Self::Val) {}

    // Constants
    fn const_i64(&mut self, v: i64) -> Self::Val;
    fn const_f64(&mut self, v: f64) -> Self::Val;
    fn const_bool(&mut self, v: bool) -> Self::Val;
    fn const_null(&mut self) -> Self::Val;
    fn const_str(&mut self, s: &str) -> Self::Val;

    // Unary/Binary/Compare
    fn neg(&mut self, x: Self::Val) -> Self::Val;
    fn not(&mut self, x: Self::Val) -> Self::Val;
    fn bit_not(&mut self, x: Self::Val) -> Self::Val;
    fn add(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn sub(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn mul(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn div(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn modulo(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_eq(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_ne(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_lt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_le(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_gt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;
    fn cmp_ge(&mut self, a: Self::Val, b: Self::Val) -> Self::Val;

    // Memory & control
    fn alloca_ptr(&mut self, _vid: ValueId) -> Self::Ptr;
    fn load(&mut self, ptr: &Self::Ptr) -> Self::Val;
    fn store(&mut self, ptr: &Self::Ptr, v: Self::Val);
    fn jump(&mut self, _target: BasicBlockId);
    fn branch(&mut self, _cond: Self::Val, _then_bb: BasicBlockId, _else_bb: BasicBlockId);
    fn phi_select(&mut self, _incoming: &[(BasicBlockId, Self::Val)]) -> Self::Val;
    fn ret(&mut self, v: Option<Self::Val>);

    // Host/Box calls
    fn new_box(&mut self, type_id: i64, args: &[Self::Val]) -> Self::Val;
    fn box_call_tagged(
        &mut self,
        type_id: i64,
        method_id: i64,
        recv: Self::Val,
        argv: &[Self::Val],
        tags: &[i64],
    ) -> Self::Val;
    fn extern_call(&mut self, iface: &str, method: &str, args: &[Self::Val]) -> Self::Val;

    // GC hooks
    fn barrier_read(&mut self, v: Self::Val) -> Self::Val {
        v
    }
    fn barrier_write(&mut self, _ptr: &Self::Ptr, v: Self::Val) -> Self::Val {
        v
    }
    fn safepoint(&mut self) {}
}

/// Optional helpers extension — default blanket impl with conveniences.
pub trait SemanticsExt: Semantics {
    fn to_bool_hint(&mut self, v: Self::Val) -> Self::Val {
        v
    }
}

impl<T: Semantics> SemanticsExt for T {}

// pub mod clif_adapter; // ARCHIVED: moved to archive/jit-cranelift/
pub mod vm_impl;
