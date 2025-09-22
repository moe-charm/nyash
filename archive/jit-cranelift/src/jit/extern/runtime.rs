//! Runtime/GC related hostcall symbol names reserved for JIT/AOT.

/// Runtime safepoint checkpoint (no-op stub for now)
pub const SYM_RT_CHECKPOINT: &str = "nyash.rt.checkpoint";

/// Write barrier hint for GC (no-op stub for now)
pub const SYM_GC_BARRIER_WRITE: &str = "nyash.gc.barrier_write";
