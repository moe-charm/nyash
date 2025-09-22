//! Unified GC controller (skeleton)
//! Implements GcHooks and centralizes mode selection and metrics.

use std::sync::atomic::{AtomicU64, Ordering};

use super::gc::{BarrierKind, GcHooks};
use super::gc_mode::GcMode;
use crate::config::env;
use crate::runtime::gc_trace;
use std::collections::{HashSet, VecDeque};

pub struct GcController {
    mode: GcMode,
    safepoints: AtomicU64,
    barrier_reads: AtomicU64,
    barrier_writes: AtomicU64,
    alloc_bytes: AtomicU64,
    alloc_count: AtomicU64,
    sp_since_last: AtomicU64,
    bytes_since_last: AtomicU64,
    collect_sp_interval: Option<u64>,
    collect_alloc_bytes: Option<u64>,
    // Diagnostics: last trial reachability counters
    trial_nodes_last: AtomicU64,
    trial_edges_last: AtomicU64,
    // Diagnostics: collection counters and last duration/flags
    collect_count_total: AtomicU64,
    collect_by_sp: AtomicU64,
    collect_by_alloc: AtomicU64,
    trial_duration_last_ms: AtomicU64,
    trial_reason_last: AtomicU64, // bitflags: 1=sp, 2=alloc
}

impl GcController {
    pub fn new(mode: GcMode) -> Self {
        Self {
            mode,
            safepoints: AtomicU64::new(0),
            barrier_reads: AtomicU64::new(0),
            barrier_writes: AtomicU64::new(0),
            alloc_bytes: AtomicU64::new(0),
            alloc_count: AtomicU64::new(0),
            sp_since_last: AtomicU64::new(0),
            bytes_since_last: AtomicU64::new(0),
            collect_sp_interval: env::gc_collect_sp_interval(),
            collect_alloc_bytes: env::gc_collect_alloc_bytes(),
            trial_nodes_last: AtomicU64::new(0),
            trial_edges_last: AtomicU64::new(0),
            collect_count_total: AtomicU64::new(0),
            collect_by_sp: AtomicU64::new(0),
            collect_by_alloc: AtomicU64::new(0),
            trial_duration_last_ms: AtomicU64::new(0),
            trial_reason_last: AtomicU64::new(0),
        }
    }
    pub fn mode(&self) -> GcMode {
        self.mode
    }
    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.safepoints.load(Ordering::Relaxed),
            self.barrier_reads.load(Ordering::Relaxed),
            self.barrier_writes.load(Ordering::Relaxed),
        )
    }
}

impl GcHooks for GcController {
    fn safepoint(&self) {
        // Off mode: minimal overhead but still callable
        if self.mode != GcMode::Off {
            self.safepoints.fetch_add(1, Ordering::Relaxed);
            let sp = self.sp_since_last.fetch_add(1, Ordering::Relaxed) + 1;
            // Opportunistic collection trigger
            let sp_hit = self
                .collect_sp_interval
                .map(|n| n > 0 && sp >= n)
                .unwrap_or(false);
            let bytes = self.bytes_since_last.load(Ordering::Relaxed);
            let bytes_hit = self
                .collect_alloc_bytes
                .map(|n| n > 0 && bytes >= n)
                .unwrap_or(false);
            if sp_hit || bytes_hit {
                // Record reason flags for diagnostics
                let mut flags: u64 = 0;
                if sp_hit { flags |= 1; self.collect_by_sp.fetch_add(1, Ordering::Relaxed); }
                if bytes_hit { flags |= 2; self.collect_by_alloc.fetch_add(1, Ordering::Relaxed); }
                self.trial_reason_last.store(flags, Ordering::Relaxed);
                self.run_trial_collection();
            }
        }
        // Future: per-mode collection/cooperation hooks
    }
    fn barrier(&self, kind: BarrierKind) {
        if self.mode == GcMode::Off {
            return;
        }
        match kind {
            BarrierKind::Read => {
                self.barrier_reads.fetch_add(1, Ordering::Relaxed);
            }
            BarrierKind::Write => {
                self.barrier_writes.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    fn snapshot_counters(&self) -> Option<(u64, u64, u64)> {
        Some(self.snapshot())
    }
    fn alloc(&self, bytes: u64) {
        if self.mode == GcMode::Off {
            return;
        }
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
        self.alloc_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.bytes_since_last
            .fetch_add(bytes, Ordering::Relaxed);
    }
}

impl GcController {
    pub fn alloc_totals(&self) -> (u64, u64) {
        (
            self.alloc_count.load(Ordering::Relaxed),
            self.alloc_bytes.load(Ordering::Relaxed),
        )
    }
}

impl GcController {
    fn run_trial_collection(&self) {
        // Reset windows
        self.sp_since_last.store(0, Ordering::Relaxed);
        self.bytes_since_last.store(0, Ordering::Relaxed);
        // PoC: no object graph; report current handles as leak candidates and return.
        if self.mode == GcMode::Off {
            return;
        }
        // Only run for rc/rc+cycle/stw; rc+cycle is default.
        match self.mode {
            GcMode::Rc | GcMode::RcCycle | GcMode::STW => {
                let started = std::time::Instant::now();
                // Roots: Runtime handle registry snapshot
                // ARCHIVED: JIT handle implementation moved to archive/jit-cranelift/ during Phase 15
                let roots: Vec<std::sync::Arc<dyn crate::box_trait::NyashBox>> = Vec::new(); // TODO: Implement handle registry for Phase 15
                let mut visited: HashSet<u64> = HashSet::new();
                let mut q: VecDeque<std::sync::Arc<dyn crate::box_trait::NyashBox>> =
                    VecDeque::new();
                for r in roots.into_iter() {
                    let id = r.box_id();
                    if visited.insert(id) {
                        q.push_back(r);
                    }
                }
                let mut nodes: u64 = visited.len() as u64;
                let mut edges: u64 = 0;
                while let Some(cur) = q.pop_front() {
                    gc_trace::trace_children(&*cur, &mut |child| {
                        edges += 1;
                        let id = child.box_id();
                        if visited.insert(id) {
                            nodes += 1;
                            q.push_back(child);
                        }
                    });
                }
                // Store last diagnostics (available for JSON metrics)
                self.trial_nodes_last.store(nodes, Ordering::Relaxed);
                self.trial_edges_last.store(edges, Ordering::Relaxed);
                if (nodes + edges) > 0 && crate::config::env::gc_metrics() {
                    eprintln!(
                        "[GC] trial: reachable nodes={} edges={} (roots=jit_handles)",
                        nodes, edges
                    );
                }
                // Update counters
                let dur = started.elapsed();
                let ms = dur.as_millis() as u64;
                self.trial_duration_last_ms.store(ms, Ordering::Relaxed);
                self.collect_count_total.fetch_add(1, Ordering::Relaxed);
                // Reason flags derive from current env thresholds vs last windows reaching triggers
                // Note: we set flags in safepoint() where triggers were decided.
            }
            _ => {}
        }
    }
}

impl GcController {
    pub fn trial_reachability_last(&self) -> (u64, u64) {
        (
            self.trial_nodes_last.load(Ordering::Relaxed),
            self.trial_edges_last.load(Ordering::Relaxed),
        )
    }
    pub fn collection_totals(&self) -> (u64, u64, u64) {
        (
            self.collect_count_total.load(Ordering::Relaxed),
            self.collect_by_sp.load(Ordering::Relaxed),
            self.collect_by_alloc.load(Ordering::Relaxed),
        )
    }
    pub fn trial_duration_last_ms(&self) -> u64 {
        self.trial_duration_last_ms.load(Ordering::Relaxed)
    }
    pub fn trial_reason_last_bits(&self) -> u64 { self.trial_reason_last.load(Ordering::Relaxed) }
}
