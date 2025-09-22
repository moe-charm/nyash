use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static TRACE_ENABLED: AtomicBool = AtomicBool::new(false);
static EVENTS: Lazy<Mutex<VecDeque<String>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(256)));
const MAX_EVENTS: usize = 256;

pub fn set_enabled(on: bool) {
    TRACE_ENABLED.store(on, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    if TRACE_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
    std::env::var("NYASH_JIT_SHIM_TRACE").ok().as_deref() == Some("1")
}

pub fn push(event: String) {
    if !is_enabled() {
        return;
    }
    if let Ok(mut q) = EVENTS.lock() {
        if q.len() >= MAX_EVENTS {
            q.pop_front();
        }
        q.push_back(event);
    }
}

pub fn snapshot_joined() -> String {
    if let Ok(q) = EVENTS.lock() {
        let mut out = String::new();
        for (i, e) in q.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(e);
        }
        out
    } else {
        String::new()
    }
}

pub fn clear() {
    if let Ok(mut q) = EVENTS.lock() {
        q.clear();
    }
}
