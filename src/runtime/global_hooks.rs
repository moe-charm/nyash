//! Lightweight global hooks for JIT/extern to reach GC/scheduler without owning NyashRuntime.

use once_cell::sync::OnceCell;
use std::sync::{Arc, RwLock};

use super::{gc::GcHooks, scheduler::Scheduler};

static GLOBAL_GC: OnceCell<RwLock<Option<Arc<dyn GcHooks>>>> = OnceCell::new();
static GLOBAL_SCHED: OnceCell<RwLock<Option<Arc<dyn Scheduler>>>> = OnceCell::new();

fn gc_cell() -> &'static RwLock<Option<Arc<dyn GcHooks>>> { GLOBAL_GC.get_or_init(|| RwLock::new(None)) }
fn sched_cell() -> &'static RwLock<Option<Arc<dyn Scheduler>>> { GLOBAL_SCHED.get_or_init(|| RwLock::new(None)) }

pub fn set_from_runtime(rt: &crate::runtime::nyash_runtime::NyashRuntime) {
    if let Ok(mut g) = gc_cell().write() { *g = Some(rt.gc.clone()); }
    if let Ok(mut s) = sched_cell().write() { *s = rt.scheduler.as_ref().cloned(); }
}

pub fn set_gc(gc: Arc<dyn GcHooks>) { if let Ok(mut g) = gc_cell().write() { *g = Some(gc); } }
pub fn set_scheduler(s: Arc<dyn Scheduler>) { if let Ok(mut w) = sched_cell().write() { *w = Some(s); } }

/// Perform a runtime safepoint and poll the scheduler if available.
pub fn safepoint_and_poll() {
    if let Ok(g) = gc_cell().read() {
        if let Some(gc) = g.as_ref() { gc.safepoint(); }
    }
    if let Ok(s) = sched_cell().read() {
        if let Some(sched) = s.as_ref() { sched.poll(); }
    }
}

/// Try to schedule a task on the global scheduler. Returns true if scheduled.
pub fn spawn_task(_name: &str, f: Box<dyn FnOnce() + 'static>) -> bool {
    // Minimal inline execution to avoid Send bounds; upgrade to true scheduling later
    f();
    true
}
