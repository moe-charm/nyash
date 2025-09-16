use inkwell::values::{IntValue, PointerValue};

/// Lightweight newtypes for string representations used in lowering.
/// StrHandle crosses basic blocks; StrPtr is created at call sites within the same block.
pub struct StrHandle<'ctx>(pub IntValue<'ctx>);
pub struct StrPtr<'ctx>(pub PointerValue<'ctx>);

impl<'ctx> StrHandle<'ctx> {
    #[inline]
    pub fn as_i64(&self) -> IntValue<'ctx> {
        self.0
    }
}

impl<'ctx> From<PointerValue<'ctx>> for StrPtr<'ctx> {
    fn from(p: PointerValue<'ctx>) -> Self {
        Self(p)
    }
}
