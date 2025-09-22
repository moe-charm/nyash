//! JIT subsystem: Cranelift-based JIT manager and lowering stubs

pub mod abi;
pub mod boundary;
pub mod config;
pub mod engine;
pub mod events;
pub mod r#extern;
pub mod hostcall_registry;
pub mod lower;
pub mod manager;
pub mod observe;
pub mod policy;
pub mod rt;
pub mod semantics;
pub mod shim_trace;
