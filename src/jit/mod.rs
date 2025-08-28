//! JIT subsystem: Cranelift-based JIT manager and lowering stubs

pub mod manager;
pub mod engine;
pub mod lower;
pub mod r#extern;
pub mod rt;
pub mod abi;
pub mod config;
pub mod policy;
pub mod events;
pub mod hostcall_registry;
pub mod boundary;
