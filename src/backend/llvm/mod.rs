//! Deprecated shim module for legacy Rust/inkwell backend
//! Please use `crate::backend::llvm_legacy` directly. This module re-exports
//! items to keep old paths working until full removal.

pub use crate::backend::llvm_legacy::{compile_and_execute, compile_to_object};

pub mod context {
    pub use crate::backend::llvm_legacy::context::*;
}
pub mod compiler {
    pub use crate::backend::llvm_legacy::compiler::*;
}
pub mod box_types {
    pub use crate::backend::llvm_legacy::box_types::*;
}
