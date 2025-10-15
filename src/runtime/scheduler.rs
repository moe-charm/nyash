//! Minimal scheduler abstraction (Phase 10.6b prep)
//!
//! Provides a pluggable interface to run tasks and yield cooperatively.

pub trait Scheduler: Send + Sync {
    /// Spawn a task/closure. Default impl may run inline.
    fn spawn(&self, _name: &str, f: Box<dyn FnOnce() + Send + 'static>);
    /// Spawn a task after given delay milliseconds.
    fn spawn_after(&self, _delay_ms: u64, _name: &str, _f: Box<dyn FnOnce() + Send + 'static>) {}
    /// Poll scheduler: run due tasks and a limited number of queued tasks.
    fn poll(&self) {}
    /// Cooperative yield point (no-op for single-thread).
    fn yield_now(&self) {}

    /// Optional: spawn with a cancellation token. Default delegates to spawn.
    fn spawn_with_token(
        &self,
        name: &str,
        _token: CancellationToken,
        f: Box<dyn FnOnce() + Send + 'static>,
    ) {
        self.spawn(name, f)
    }
}

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Single-thread scheduler with a simple queue and delayed tasks.
pub struct SingleThreadScheduler {
    queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>>,
    delayed: Arc<Mutex<Vec<(Instant, Box<dyn FnOnce() + Send + 'static>)>>>,
}

impl SingleThreadScheduler {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            delayed: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Scheduler for SingleThreadScheduler {
    fn spawn(&self, _name: &str, f: Box<dyn FnOnce() + Send + 'static>) {
        if let Ok(mut q) = self.queue.lock() {
            q.push_back(f);
        }
    }
    fn spawn_after(&self, delay_ms: u64, _name: &str, f: Box<dyn FnOnce() + Send + 'static>) {
        let when = Instant::now() + Duration::from_millis(delay_ms);
        if let Ok(mut d) = self.delayed.lock() {
            d.push((when, f));
        }
    }
    fn poll(&self) {
        // Move due delayed tasks to queue
        let trace = crate::runtime::env_gate_box::bool_any(&[
            "HAKO_SCHED_TRACE",
            "NYASH_SCHED_TRACE",
        ]);
        let now = Instant::now();
        let mut moved = 0usize;
        if let Ok(mut d) = self.delayed.lock() {
            let mut i = 0;
            while i < d.len() {
                if d[i].0 <= now {
                    let (_when, task) = d.remove(i);
                    if let Ok(mut q) = self.queue.lock() {
                        q.push_back(task);
                    }
                    moved += 1;
                } else {
                    i += 1;
                }
            }
        }
        // Run up to budget queued tasks
        let budget: usize = crate::runtime::env_gate_box::string_alias_or(
            "NYASH_SCHED_POLL_BUDGET",
            "HAKO_SCHED_POLL_BUDGET",
            "1",
        )
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
        .unwrap_or(1);
        let mut ran = 0usize;
        while ran < budget {
            let task_opt = {
                if let Ok(mut q) = self.queue.lock() {
                    q.pop_front()
                } else {
                    None
                }
            };
            if let Some(task) = task_opt {
                task();
                ran += 1;
            } else {
                break;
            }
        }
        if trace {
            eprintln!("[SCHED] poll moved={} ran={} budget={}", moved, ran, budget);
        }
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

/// Simple idempotent cancellation token for structured concurrency (skeleton)
#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
