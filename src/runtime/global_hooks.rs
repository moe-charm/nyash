//! Lightweight global hooks for JIT/extern to reach GC/scheduler without owning NyashRuntime.

use once_cell::sync::OnceCell;
use std::sync::{Arc, RwLock};

use super::{gc::GcHooks, scheduler::Scheduler};
use super::scheduler::CancellationToken;

static GLOBAL_GC: OnceCell<RwLock<Option<Arc<dyn GcHooks>>>> = OnceCell::new();
static GLOBAL_SCHED: OnceCell<RwLock<Option<Arc<dyn Scheduler>>>> = OnceCell::new();
// Phase 2 scaffold: current task group's cancellation token (no-op default)
static GLOBAL_CUR_TOKEN: OnceCell<RwLock<Option<CancellationToken>>> = OnceCell::new();
// Phase 2 scaffold: current group's child futures registry (best-effort)
static GLOBAL_GROUP_FUTURES: OnceCell<RwLock<Vec<crate::boxes::future::FutureWeak>>> = OnceCell::new();
// Strong ownership list for implicit group (pre-TaskGroup actualization)
static GLOBAL_GROUP_STRONG: OnceCell<RwLock<Vec<crate::boxes::future::FutureBox>>> = OnceCell::new();
// Simple scope depth counter for implicit group (join-at-scope-exit footing)
static TASK_SCOPE_DEPTH: OnceCell<RwLock<usize>> = OnceCell::new();
// TaskGroup scope stack (explicit group ownership per function scope)
static TASK_GROUP_STACK: OnceCell<RwLock<Vec<std::sync::Arc<crate::boxes::task_group_box::TaskGroupInner>>>> = OnceCell::new();

fn gc_cell() -> &'static RwLock<Option<Arc<dyn GcHooks>>> { GLOBAL_GC.get_or_init(|| RwLock::new(None)) }
fn sched_cell() -> &'static RwLock<Option<Arc<dyn Scheduler>>> { GLOBAL_SCHED.get_or_init(|| RwLock::new(None)) }
fn token_cell() -> &'static RwLock<Option<CancellationToken>> { GLOBAL_CUR_TOKEN.get_or_init(|| RwLock::new(None)) }
fn futures_cell() -> &'static RwLock<Vec<crate::boxes::future::FutureWeak>> { GLOBAL_GROUP_FUTURES.get_or_init(|| RwLock::new(Vec::new())) }
fn strong_cell() -> &'static RwLock<Vec<crate::boxes::future::FutureBox>> { GLOBAL_GROUP_STRONG.get_or_init(|| RwLock::new(Vec::new())) }
fn scope_depth_cell() -> &'static RwLock<usize> { TASK_SCOPE_DEPTH.get_or_init(|| RwLock::new(0)) }
fn group_stack_cell() -> &'static RwLock<Vec<std::sync::Arc<crate::boxes::task_group_box::TaskGroupInner>>> { TASK_GROUP_STACK.get_or_init(|| RwLock::new(Vec::new())) }

pub fn set_from_runtime(rt: &crate::runtime::nyash_runtime::NyashRuntime) {
    if let Ok(mut g) = gc_cell().write() { *g = Some(rt.gc.clone()); }
    if let Ok(mut s) = sched_cell().write() { *s = rt.scheduler.as_ref().cloned(); }
    // Optional: initialize a fresh token for the runtime's root group (Phase 2 wiring)
    if let Ok(mut t) = token_cell().write() { if t.is_none() { *t = Some(CancellationToken::new()); } }
    // Reset group futures registry on new runtime
    if let Ok(mut f) = futures_cell().write() { f.clear(); }
    if let Ok(mut s) = strong_cell().write() { s.clear(); }
    if let Ok(mut d) = scope_depth_cell().write() { *d = 0; }
    if let Ok(mut st) = group_stack_cell().write() { st.clear(); }
}

pub fn set_gc(gc: Arc<dyn GcHooks>) { if let Ok(mut g) = gc_cell().write() { *g = Some(gc); } }
pub fn set_scheduler(s: Arc<dyn Scheduler>) { if let Ok(mut w) = sched_cell().write() { *w = Some(s); } }
/// Set the current task group's cancellation token (scaffold).
pub fn set_current_group_token(tok: CancellationToken) { if let Ok(mut w) = token_cell().write() { *w = Some(tok); } }

/// Get the current task group's cancellation token (no-op default).
pub fn current_group_token() -> CancellationToken {
    if let Ok(r) = token_cell().read() {
        if let Some(t) = r.as_ref() { return t.clone(); }
    }
    CancellationToken::new()
}

/// Register a Future into the current group's registry (best-effort; clones share state)
pub fn register_future_to_current_group(fut: &crate::boxes::future::FutureBox) {
    // Prefer explicit current TaskGroup at top of stack
    if let Ok(st) = group_stack_cell().read() {
        if let Some(inner) = st.last() {
            if let Ok(mut v) = inner.strong.lock() { v.push(fut.clone()); return; }
        }
    }
    // Fallback to implicit global group
    if let Ok(mut list) = futures_cell().write() { list.push(fut.downgrade()); }
    if let Ok(mut s) = strong_cell().write() { s.push(fut.clone()); }
}

/// Join all currently registered futures with a coarse timeout guard.
pub fn join_all_registered_futures(timeout_ms: u64) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let mut all_ready = true;
        // purge list of dropped or completed futures opportunistically
        {
            // purge weak list: keep only upgradeable futures
            if let Ok(mut list) = futures_cell().write() { list.retain(|fw| fw.is_ready().is_some()); }
            // purge strong list: remove completed futures to reduce retention
            if let Ok(mut s) = strong_cell().write() { s.retain(|f| !f.ready()); }
        }
        // check readiness
        {
            if let Ok(list) = futures_cell().read() {
                for fw in list.iter() {
                    if let Some(ready) = fw.is_ready() {
                        if !ready { all_ready = false; break; }
                    }
                }
            }
        }
        if all_ready { break; }
        if Instant::now() >= deadline { break; }
        safepoint_and_poll();
        std::thread::yield_now();
    }
    // Final sweep
    if let Ok(mut s) = strong_cell().write() { s.retain(|f| !f.ready()); }
    if let Ok(mut list) = futures_cell().write() { list.retain(|fw| matches!(fw.is_ready(), Some(false))); }
}

/// Push a task scope (footing). On pop of the outermost scope, perform a best-effort join.
pub fn push_task_scope() {
    if let Ok(mut d) = scope_depth_cell().write() { *d += 1; }
    // Push a new explicit TaskGroup for this scope
    if let Ok(mut st) = group_stack_cell().write() {
        st.push(std::sync::Arc::new(crate::boxes::task_group_box::TaskGroupInner { strong: std::sync::Mutex::new(Vec::new()) }));
    }
}

/// Pop a task scope. When depth reaches 0, join outstanding futures.
pub fn pop_task_scope() {
    let mut do_join = false;
    let mut popped: Option<std::sync::Arc<crate::boxes::task_group_box::TaskGroupInner>> = None;
    {
        if let Ok(mut d) = scope_depth_cell().write() {
            if *d > 0 { *d -= 1; }
            if *d == 0 { do_join = true; }
        }
    }
    // Pop explicit group for this scope
    if let Ok(mut st) = group_stack_cell().write() { popped = st.pop(); }
    if do_join {
        let ms: u64 = std::env::var("NYASH_TASK_SCOPE_JOIN_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(1000);
        if let Some(inner) = popped {
            // Join this group's outstanding futures
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms);
            loop {
                let mut all_ready = true;
                if let Ok(mut list) = inner.strong.lock() { list.retain(|f| !f.ready()); if !list.is_empty() { all_ready = false; } }
                if all_ready { break; }
                if std::time::Instant::now() >= deadline { break; }
                safepoint_and_poll();
                std::thread::yield_now();
            }
        } else {
            // Fallback to implicit global group
            join_all_registered_futures(ms);
        }
    }
}

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
pub fn spawn_task(name: &str, f: Box<dyn FnOnce() + Send + 'static>) -> bool {
    // If a scheduler is registered, enqueue the task; otherwise run inline.
    if let Ok(s) = sched_cell().read() {
        if let Some(sched) = s.as_ref() {
            sched.spawn(name, f);
            return true;
        }
    }
    // Fallback inline execution
    f();
    false
}

/// Spawn a task bound to a cancellation token when available (skeleton).
pub fn spawn_task_with_token(name: &str, token: crate::runtime::scheduler::CancellationToken, f: Box<dyn FnOnce() + Send + 'static>) -> bool {
    if let Ok(s) = sched_cell().read() {
        if let Some(sched) = s.as_ref() {
            sched.spawn_with_token(name, token, f);
            return true;
        }
    }
    f();
    false
}
