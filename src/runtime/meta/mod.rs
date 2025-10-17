//! Runtime meta layer — host-owned meta boxes (Callable/Future)
//!
//! Responsibility
//! - Provide language-level meta boxes owned by the host runtime.
//! - No external I/O; no direct plugin dependency. May depend on GC/Scheduler.
//! - Keep a minimal, stable surface for router/scheduler integrations.

pub mod callable;
pub mod future;

