//! Legacy LLVM codegen placeholder.
//!
//! The original inkwell-based implementation was removed during the Phase-15
//! refactor. We keep a stub module so tools like `cargo fmt` can resolve the
//! module tree even when the legacy feature is gated off.

#[allow(dead_code)]
pub(crate) fn lower_module() {
    // Intentionally left blank.
}
