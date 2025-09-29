// bench module removed with vm-legacy
pub mod llvm;
pub mod mir;
pub mod vm_fallback;
pub mod pyvm;
pub mod macro_child;
pub mod common_util;

// Unified VM interface (Phase‑A)
pub mod super_iface {
    pub use crate::runner::vm_iface::{vm_engine_from_env, VmEngine};
}

#[cfg(feature = "cranelift-jit")]
pub mod aot;
