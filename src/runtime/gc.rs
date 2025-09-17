//! GC hook abstractions for switchable runtime (Phase 10.4 preparation)
//!
//! Minimal, no-alloc, no-type-coupling interfaces that VM can call.
//! Default implementation is a no-op. Real collectors can plug later.

#[derive(Debug, Clone, Copy)]
pub enum BarrierKind {
    Read,
    Write,
}

/// GC hooks that execution engines may call at key points.
/// Implementations must be Send + Sync for multi-thread preparation.
pub trait GcHooks: Send + Sync + std::any::Any {
    /// Safe point for cooperative GC (e.g., poll or yield).
    fn safepoint(&self) {}
    /// Memory barrier hint for loads/stores.
    fn barrier(&self, _kind: BarrierKind) {}
    /// Allocation accounting (bytes are best-effort; may be 0 when unknown)
    fn alloc(&self, _bytes: u64) {}
    /// Optional counters snapshot for diagnostics. Default: None.
    fn snapshot_counters(&self) -> Option<(u64, u64, u64)> {
        None
    }
}

/// Default no-op hooks.
pub struct NullGc;

impl GcHooks for NullGc {}

/// CountingGc is now a thin wrapper around the unified GcController.
pub struct CountingGc {
    inner: crate::runtime::gc_controller::GcController,
}

impl CountingGc {
    pub fn new() -> Self {
        // Default to rc+cycle mode for development metrics
        let mode = crate::runtime::gc_mode::GcMode::RcCycle;
        Self {
            inner: crate::runtime::gc_controller::GcController::new(mode),
        }
    }
    pub fn snapshot(&self) -> (u64, u64, u64) {
        self.inner.snapshot()
    }
}

impl GcHooks for CountingGc {
    fn safepoint(&self) {
        self.inner.safepoint();
    }
    fn barrier(&self, kind: BarrierKind) {
        self.inner.barrier(kind);
    }
    fn alloc(&self, bytes: u64) {
        self.inner.alloc(bytes);
    }
    fn snapshot_counters(&self) -> Option<(u64, u64, u64)> {
        Some(self.inner.snapshot())
    }
}
