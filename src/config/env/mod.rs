//! Environment configuration module
//!
//! Split from monolithic env.rs (861 lines) into focused submodules
//! following Single Responsibility Principle.
//!
//! Organization (110 functions across 10 modules):
//! - core: Shared infrastructure + bootstrap (3 functions)
//! - vm: VM runtime settings (16 functions)
//! - gc: Garbage collector settings (11 functions)
//! - using: Using/namespace system (10 functions)
//! - mir: MIR compiler settings (7 functions)
//! - compiler: Ny compiler settings (9 functions)
//! - parser: Parser settings (3 functions)
//! - plugin: Plugin system settings (5 functions)
//! - runtime: Runtime features (13 functions)
//! - features: Misc feature flags (33 functions)

mod core;
mod vm;
mod gc;
mod using;
mod mir;
mod compiler;
mod parser;
mod plugin;
mod runtime;
mod features;

// Re-export all public functions
pub use core::*;
pub use vm::*;
pub use gc::*;
pub use using::*;
pub use mir::*;
pub use compiler::*;
pub use parser::*;
pub use plugin::*;
pub use runtime::*;
pub use features::*;
