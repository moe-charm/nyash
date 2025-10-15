pub mod compare;
pub mod boxcall;
pub mod call;

// Re-export minimal types for convenience within vm_ops if needed
pub use crate::backend::vm_types::{VMValue, VMError};
