//! GC hook abstractions for switchable runtime (Phase 10.4 preparation)
//!
//! Minimal, no-alloc, no-type-coupling interfaces that VM can call.
//! Default implementation is a no-op. Real collectors can plug later.

#[derive(Debug, Clone, Copy)]
pub enum BarrierKind { Read, Write }

/// GC hooks that execution engines may call at key points.
/// Implementations must be Send + Sync for multi-thread preparation.
pub trait GcHooks: Send + Sync {
    /// Safe point for cooperative GC (e.g., poll or yield).
    fn safepoint(&self) {}
    /// Memory barrier hint for loads/stores.
    fn barrier(&self, _kind: BarrierKind) {}
    /// Optional counters snapshot for diagnostics. Default: None.
    fn snapshot_counters(&self) -> Option<(u64, u64, u64)> { None }
}

/// Default no-op hooks.
pub struct NullGc;

impl GcHooks for NullGc {}

use std::sync::atomic::{AtomicU64, Ordering};

/// Simple counting GC (PoC): counts safepoints and barriers.
/// Useful to validate hook frequency without affecting semantics.
pub struct CountingGc {
    pub safepoints: AtomicU64,
    pub barrier_reads: AtomicU64,
    pub barrier_writes: AtomicU64,
}

impl CountingGc {
    pub fn new() -> Self {
        Self {
            safepoints: AtomicU64::new(0),
            barrier_reads: AtomicU64::new(0),
            barrier_writes: AtomicU64::new(0),
        }
    }
    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.safepoints.load(Ordering::Relaxed),
            self.barrier_reads.load(Ordering::Relaxed),
            self.barrier_writes.load(Ordering::Relaxed),
        )
    }
}

impl GcHooks for CountingGc {
    fn safepoint(&self) {
        self.safepoints.fetch_add(1, Ordering::Relaxed);
    }
    fn barrier(&self, kind: BarrierKind) {
        match kind {
            BarrierKind::Read => { self.barrier_reads.fetch_add(1, Ordering::Relaxed); }
            BarrierKind::Write => { self.barrier_writes.fetch_add(1, Ordering::Relaxed); }
        }
    }
    fn snapshot_counters(&self) -> Option<(u64, u64, u64)> {
        Some(self.snapshot())
    }
}
