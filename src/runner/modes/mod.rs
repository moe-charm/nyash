#[cfg(feature = "vm-legacy")]
pub mod bench;
#[cfg(feature = "interpreter-legacy")]
pub mod interpreter;
pub mod llvm;
pub mod mir;
#[cfg(feature = "vm-legacy")]
pub mod vm;
pub mod pyvm;
pub mod macro_child;

// Shared helpers extracted from common.rs (in progress)
pub mod common_util;

#[cfg(feature = "cranelift-jit")]
pub mod aot;
