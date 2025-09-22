use super::Semantics;
use crate::jit::lower::builder::{BinOpKind, CmpKind, IRBuilder};
use crate::mir::{BasicBlockId, ValueId};

/// Adapter that translates Semantics operations into IRBuilder calls (Cranelift path)
pub struct ClifSemanticsAdapter<'a> {
    pub builder: &'a mut dyn IRBuilder,
}

impl<'a> ClifSemanticsAdapter<'a> {
    pub fn new(builder: &'a mut dyn IRBuilder) -> Self {
        Self { builder }
    }
}

impl<'a> Semantics for ClifSemanticsAdapter<'a> {
    type Val = ();
    type Ptr = ValueId;
    type BB = BasicBlockId;

    fn const_i64(&mut self, v: i64) -> Self::Val {
        self.builder.emit_const_i64(v);
    }
    fn const_f64(&mut self, v: f64) -> Self::Val {
        self.builder.emit_const_f64(v);
    }
    fn const_bool(&mut self, v: bool) -> Self::Val {
        self.builder.emit_const_i64(if v { 1 } else { 0 });
    }
    fn const_null(&mut self) -> Self::Val {
        self.builder.emit_const_i64(0);
    }
    fn const_str(&mut self, _s: &str) -> Self::Val {
        self.builder.emit_const_i64(0);
    }

    fn neg(&mut self, _x: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Sub);
    }
    fn not(&mut self, _x: Self::Val) -> Self::Val { /* handled via compare/select in LowerCore */
    }
    fn bit_not(&mut self, _x: Self::Val) -> Self::Val { /* not used here */
    }
    fn add(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Add);
    }
    fn sub(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Sub);
    }
    fn mul(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Mul);
    }
    fn div(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Div);
    }
    fn modulo(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_binop(BinOpKind::Mod);
    }
    fn cmp_eq(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Eq);
    }
    fn cmp_ne(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Ne);
    }
    fn cmp_lt(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Lt);
    }
    fn cmp_le(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Le);
    }
    fn cmp_gt(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Gt);
    }
    fn cmp_ge(&mut self, _a: Self::Val, _b: Self::Val) -> Self::Val {
        self.builder.emit_compare(CmpKind::Ge);
    }

    fn alloca_ptr(&mut self, _vid: ValueId) -> Self::Ptr {
        _vid
    }
    fn load(&mut self, _ptr: &Self::Ptr) -> Self::Val {
        self.builder.load_local_i64(_ptr.as_u32() as usize);
    }
    fn store(&mut self, _ptr: &Self::Ptr, _v: Self::Val) {
        self.builder.store_local_i64(_ptr.as_u32() as usize);
    }
    fn jump(&mut self, _target: BasicBlockId) { /* handled by LowerCore */
    }
    fn branch(&mut self, _cond: Self::Val, _then_bb: BasicBlockId, _else_bb: BasicBlockId) {
        /* handled by LowerCore */
    }
    fn phi_select(&mut self, _incoming: &[(BasicBlockId, Self::Val)]) -> Self::Val {
        ()
    }
    fn ret(&mut self, _v: Option<Self::Val>) {
        self.builder.emit_return();
    }

    fn new_box(&mut self, _type_id: i64, _args: &[Self::Val]) -> Self::Val {
        ()
    }
    fn box_call_tagged(
        &mut self,
        _type_id: i64,
        _method_id: i64,
        _recv: Self::Val,
        _argv: &[Self::Val],
        _tags: &[i64],
    ) -> Self::Val {
        ()
    }
    fn extern_call(&mut self, _iface: &str, _method: &str, _args: &[Self::Val]) -> Self::Val {
        ()
    }

    fn barrier_read(&mut self, v: Self::Val) -> Self::Val {
        v
    }
    fn barrier_write(&mut self, _ptr: &Self::Ptr, v: Self::Val) -> Self::Val {
        v
    }
    fn safepoint(&mut self) { /* Lowered via explicit hostcall in LowerCore path */
    }
}
