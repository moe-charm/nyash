//! Lightweight global hooks for JIT/extern to reach GC/scheduler without owning NyashRuntime.

use once_cell::sync::OnceCell;
use std::sync::{Arc, RwLock};

use super::scheduler::CancellationToken;
use super::{gc::BarrierKind, gc::GcHooks, scheduler::Scheduler};
use crate::runtime::meta::future::future_box::{FutureBox, FutureWeak};

// Unified global runtime hooks state (single lock for consistency)
struct GlobalHooksState {
    gc: Option<Arc<dyn GcHooks>>,
    sched: Option<Arc<dyn Scheduler>>,
    cur_token: Option<CancellationToken>,
    futures: Vec<FutureWeak>,
    strong: Vec<FutureBox>,
    scope_depth: usize,
    #[cfg(feature = "legacy-boxes")]
    group_stack: Vec<std::sync::Arc<crate::boxes::task_group_box::TaskGroupInner>>,
}

impl GlobalHooksState {
    fn new() -> Self {
        Self {
            gc: None,
            sched: None,
            cur_token: None,
            futures: Vec::new(),
            strong: Vec::new(),
            scope_depth: 0,
            #[cfg(feature = "legacy-boxes")]
            group_stack: Vec::new(),
        }
    }
}

static GLOBAL_STATE: OnceCell<RwLock<GlobalHooksState>> = OnceCell::new();

fn state() -> &'static RwLock<GlobalHooksState> {
    GLOBAL_STATE.get_or_init(|| RwLock::new(GlobalHooksState::new()))
}

pub fn set_from_runtime(rt: &crate::runtime::nyash_runtime::NyashRuntime) {
    if let Ok(mut st) = state().write() {
        st.gc = Some(rt.gc.clone());
        st.sched = rt.scheduler.as_ref().cloned();
        if st.cur_token.is_none() {
            st.cur_token = Some(CancellationToken::new());
        }
        { st.futures.clear(); st.strong.clear(); }
        st.scope_depth = 0;
        #[cfg(feature = "legacy-boxes")]
        { st.group_stack.clear(); }
    }
}

pub fn set_gc(gc: Arc<dyn GcHooks>) {
    if let Ok(mut st) = state().write() {
        st.gc = Some(gc);
    }
}
pub fn set_scheduler(s: Arc<dyn Scheduler>) {
    if let Ok(mut st) = state().write() {
        st.sched = Some(s);
    }
}
/// Set the current task group's cancellation token (scaffold).
pub fn set_current_group_token(tok: CancellationToken) {
    if let Ok(mut st) = state().write() {
        st.cur_token = Some(tok);
    }
}

/// Get the current task group's cancellation token (no-op default).
pub fn current_group_token() -> CancellationToken {
    if let Ok(st) = state().read() {
        if let Some(t) = st.cur_token.as_ref() {
            return t.clone();
        }
    }
    CancellationToken::new()
}

/// Register a Future into the current group's registry (best-effort; clones share state)
pub fn register_future_to_current_group(fut: &FutureBox) {
    if let Ok(mut st) = state().write() {
        // Prefer explicit current TaskGroup at top of stack
        #[cfg(feature = "legacy-boxes")]
        {
            if let Some(inner) = st.group_stack.last() {
                if let Ok(mut v) = inner.strong.lock() {
                    v.push(fut.clone());
                    return;
                }
            }
        }
        // Fallback to implicit global group
        st.futures.push(fut.downgrade());
        st.strong.push(fut.clone());
    }
}

/// Join all currently registered futures with a coarse timeout guard.
pub fn join_all_registered_futures(timeout_ms: u64) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let mut all_ready = true;
        // purge + readiness check under single state lock (short critical sections)
        if let Ok(mut st) = state().write() {
            st.futures.retain(|fw| fw.is_ready().is_some());
            st.strong.retain(|f| !f.ready());
            for fw in st.futures.iter() {
                if let Some(ready) = fw.is_ready() {
                    if !ready {
                        all_ready = false;
                        break;
                    }
                }
            }
        }
        if all_ready {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        safepoint_and_poll();
        std::thread::yield_now();
    }
    // Final sweep
    if let Ok(mut st) = state().write() {
        st.strong.retain(|f| !f.ready());
        st.futures.retain(|fw| matches!(fw.is_ready(), Some(false)));
    }
}

/// Push a task scope (footing). On pop of the outermost scope, perform a best-effort join.
pub fn push_task_scope() {
    if let Ok(mut st) = state().write() {
        st.scope_depth += 1;
        // Push a new explicit TaskGroup for this scope
        #[cfg(feature = "legacy-boxes")]
        st.group_stack.push(std::sync::Arc::new(
            crate::boxes::task_group_box::TaskGroupInner { strong: std::sync::Mutex::new(Vec::new()) }
        ));
    }
    // Set a fresh cancellation token for this scope (best-effort)
    set_current_group_token(CancellationToken::new());
}

/// Pop a task scope. When depth reaches 0, join outstanding futures.
pub fn pop_task_scope() {
    let mut do_join = false;
    #[cfg(feature = "legacy-boxes")]
    let mut popped: Option<std::sync::Arc<crate::boxes::task_group_box::TaskGroupInner>> = None;
    if let Ok(mut st) = state().write() {
        if st.scope_depth > 0 {
            st.scope_depth -= 1;
        }
        if st.scope_depth == 0 { do_join = true; }
        // Pop explicit group for this scope
        #[cfg(feature = "legacy-boxes")]
        { popped = st.group_stack.pop(); }
    }
    if do_join {
        let ms = crate::runtime::env_gate_box::string_alias_or(
            "NYASH_TASK_SCOPE_JOIN_MS",
            "HAKO_TASK_SCOPE_JOIN_MS",
            "1000",
        )
        .parse::<u64>()
        .unwrap_or(1000);
        #[cfg(feature = "legacy-boxes")]
        if let Some(ref inner) = popped {
            // Join this group's outstanding futures
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms);
            loop {
                let mut all_ready = true;
                if let Ok(mut list) = inner.strong.lock() {
                    list.retain(|f| !f.ready());
                    if !list.is_empty() {
                        all_ready = false;
                    }
                }
                if all_ready {
                    break;
                }
                if std::time::Instant::now() >= deadline {
                    break;
                }
                safepoint_and_poll();
                std::thread::yield_now();
            }
        }
        #[cfg(feature = "legacy-boxes")]
        {
            if popped.is_none() {
                join_all_registered_futures(ms);
            }
        }
        #[cfg(not(feature = "legacy-boxes"))]
        {
            // Fallback to implicit global group
            let _ = ms; // no-op in plugin-only
        }
    }
    // Reset token (best-effort)
    set_current_group_token(CancellationToken::new());
}

/// Perform a runtime safepoint and poll the scheduler if available.
pub fn safepoint_and_poll() {
    if let Ok(st) = state().read() {
        if let Some(gc) = st.gc.as_ref() {
            gc.safepoint();
        }
        if let Some(sched) = st.sched.as_ref() {
            sched.poll();
        }
    }
}

/// Try to schedule a task on the global scheduler. Returns true if scheduled.
pub fn spawn_task(name: &str, f: Box<dyn FnOnce() + Send + 'static>) -> bool {
    // If a scheduler is registered, enqueue the task; otherwise run inline.
    if let Ok(st) = state().read() {
        if let Some(sched) = st.sched.as_ref() {
            sched.spawn(name, f);
            return true;
        }
    }
    // Fallback inline execution
    f();
    false
}

/// Spawn a task bound to a cancellation token when available (skeleton).
pub fn spawn_task_with_token(
    name: &str,
    token: crate::runtime::scheduler::CancellationToken,
    f: Box<dyn FnOnce() + Send + 'static>,
) -> bool {
    if let Ok(st) = state().read() {
        if let Some(sched) = st.sched.as_ref() {
            sched.spawn_with_token(name, token, f);
            return true;
        }
    }
    f();
    false
}

/// Spawn a delayed task via scheduler if available; returns true if scheduled.
pub fn spawn_task_after(delay_ms: u64, name: &str, f: Box<dyn FnOnce() + Send + 'static>) -> bool {
    if let Ok(st) = state().read() {
        if let Some(sched) = st.sched.as_ref() {
            sched.spawn_after(delay_ms, name, f);
            return true;
        }
    }
    // Fallback: run inline after blocking sleep
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        f();
    });
    false
}

/// Forward a GC barrier event to the currently registered GC hooks (if any).
pub fn gc_barrier(kind: BarrierKind) {
    if let Ok(st) = state().read() {
        if let Some(gc) = st.gc.as_ref() {
            gc.barrier(kind);
        }
    }
}

/// Report an allocation to the current GC hooks (best-effort)
pub fn gc_alloc(bytes: u64) {
    if let Ok(st) = state().read() {
        if let Some(gc) = st.gc.as_ref() {
            gc.alloc(bytes);
        }
    }
}
