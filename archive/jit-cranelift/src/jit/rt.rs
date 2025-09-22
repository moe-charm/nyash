use std::cell::RefCell;

use crate::backend::vm::VMValue;
use crate::jit::abi::JitValue;

// Legacy TLS for hostcalls that still expect VMValue — keep for compatibility
// Legacy VM args TLS — disabled in jit-direct-only (no-op/empty)
#[cfg(not(feature = "jit-direct-only"))]
thread_local! { static LEGACY_VM_ARGS: RefCell<Vec<VMValue>> = RefCell::new(Vec::new()); }

#[cfg(not(feature = "jit-direct-only"))]
pub fn set_legacy_vm_args(args: &[VMValue]) {
    LEGACY_VM_ARGS.with(|cell| {
        let mut v = cell.borrow_mut();
        v.clear();
        v.extend_from_slice(args);
    });
}

#[cfg(feature = "jit-direct-only")]
pub fn set_legacy_vm_args(_args: &[VMValue]) { /* no-op in jit-direct-only */
}

#[cfg(not(feature = "jit-direct-only"))]
pub fn with_legacy_vm_args<F, R>(f: F) -> R
where
    F: FnOnce(&[VMValue]) -> R,
{
    LEGACY_VM_ARGS.with(|cell| {
        let v = cell.borrow();
        f(&v)
    })
}

#[cfg(feature = "jit-direct-only")]
pub fn with_legacy_vm_args<F, R>(f: F) -> R
where
    F: FnOnce(&[VMValue]) -> R,
{
    f(&[])
}

// New TLS for independent JIT ABI values
thread_local! {
    static CURRENT_JIT_ARGS: RefCell<Vec<JitValue>> = RefCell::new(Vec::new());
}

pub fn set_current_jit_args(args: &[JitValue]) {
    CURRENT_JIT_ARGS.with(|cell| {
        let mut v = cell.borrow_mut();
        v.clear();
        v.extend_from_slice(args);
    });
}

pub fn with_jit_args<F, R>(f: F) -> R
where
    F: FnOnce(&[JitValue]) -> R,
{
    CURRENT_JIT_ARGS.with(|cell| {
        let v = cell.borrow();
        f(&v)
    })
}

// === JIT runtime counters (minimal) ===
use std::sync::atomic::{AtomicU64, Ordering};
static B1_NORM_COUNT: AtomicU64 = AtomicU64::new(0);
static RET_BOOL_HINT_COUNT: AtomicU64 = AtomicU64::new(0);
static PHI_TOTAL_SLOTS: AtomicU64 = AtomicU64::new(0);
static PHI_B1_SLOTS: AtomicU64 = AtomicU64::new(0);

pub fn b1_norm_inc(delta: u64) {
    B1_NORM_COUNT.fetch_add(delta, Ordering::Relaxed);
}
pub fn b1_norm_get() -> u64 {
    B1_NORM_COUNT.load(Ordering::Relaxed)
}

pub fn ret_bool_hint_inc(delta: u64) {
    RET_BOOL_HINT_COUNT.fetch_add(delta, Ordering::Relaxed);
}
pub fn ret_bool_hint_get() -> u64 {
    RET_BOOL_HINT_COUNT.load(Ordering::Relaxed)
}

pub fn phi_total_inc(delta: u64) {
    PHI_TOTAL_SLOTS.fetch_add(delta, Ordering::Relaxed);
}
pub fn phi_total_get() -> u64 {
    PHI_TOTAL_SLOTS.load(Ordering::Relaxed)
}
pub fn phi_b1_inc(delta: u64) {
    PHI_B1_SLOTS.fetch_add(delta, Ordering::Relaxed);
}
pub fn phi_b1_get() -> u64 {
    PHI_B1_SLOTS.load(Ordering::Relaxed)
}

// === 10.7c PoC: JIT Handle Registry (thread-local) ===
use std::collections::HashMap;
use std::sync::Arc;

pub mod handles {
    use super::*;

    thread_local! {
        static REG: RefCell<HandleRegistry> = RefCell::new(HandleRegistry::new());
        static CREATED: RefCell<Vec<u64>> = RefCell::new(Vec::new());
        static SCOPES: RefCell<Vec<usize>> = RefCell::new(Vec::new());
    }

    struct HandleRegistry {
        next: u64,
        map: HashMap<u64, Arc<dyn crate::box_trait::NyashBox>>, // BoxRef-compatible
    }

    impl HandleRegistry {
        fn new() -> Self {
            Self {
                next: 1,
                map: HashMap::new(),
            }
        }
        fn to_handle(&mut self, obj: Arc<dyn crate::box_trait::NyashBox>) -> u64 {
            // Reuse existing handle if already present (pointer equality check)
            // For PoC simplicity, always assign new handle
            let h = self.next;
            self.next = self.next.saturating_add(1);
            self.map.insert(h, obj);
            if std::env::var("NYASH_JIT_HANDLE_DEBUG").ok().as_deref() == Some("1") {
                eprintln!("[JIT][handle] new h={}", h);
            }
            h
        }
        fn get(&self, h: u64) -> Option<Arc<dyn crate::box_trait::NyashBox>> {
            self.map.get(&h).cloned()
        }
        #[allow(dead_code)]
        fn drop_handle(&mut self, h: u64) {
            self.map.remove(&h);
        }
        #[allow(dead_code)]
        fn clear(&mut self) {
            self.map.clear();
            self.next = 1;
        }
    }

    pub fn to_handle(obj: Arc<dyn crate::box_trait::NyashBox>) -> u64 {
        let h = REG.with(|cell| cell.borrow_mut().to_handle(obj));
        CREATED.with(|c| c.borrow_mut().push(h));
        h
    }
    pub fn get(h: u64) -> Option<Arc<dyn crate::box_trait::NyashBox>> {
        REG.with(|cell| cell.borrow().get(h))
    }
    #[allow(dead_code)]
    pub fn clear() {
        REG.with(|cell| cell.borrow_mut().clear());
    }
    pub fn len() -> usize {
        REG.with(|cell| cell.borrow().map.len())
    }
    /// Tally handles by NyashBox type name (best-effort)
    pub fn type_tally() -> Vec<(String, usize)> {
        use std::collections::HashMap;
        REG.with(|cell| {
            let reg = cell.borrow();
            let mut map: HashMap<String, usize> = HashMap::new();
            for (_h, obj) in reg.map.iter() {
                let tn = obj.type_name().to_string();
                *map.entry(tn).or_insert(0) += 1;
            }
            let mut v: Vec<(String, usize)> = map.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            v
        })
    }
    /// Snapshot current handle objects (Arc clones)
    pub fn snapshot_arcs() -> Vec<Arc<dyn crate::box_trait::NyashBox>> {
        REG.with(|cell| {
            let reg = cell.borrow();
            reg.map.values().cloned().collect()
        })
    }

    // Scope management: track and clear handles created within a JIT call
    pub fn begin_scope() {
        CREATED.with(|c| {
            let cur_len = c.borrow().len();
            SCOPES.with(|s| s.borrow_mut().push(cur_len));
        });
    }
    pub fn end_scope_clear() {
        let start = SCOPES.with(|s| s.borrow_mut().pop()).unwrap_or(0);
        let to_drop: Vec<u64> = CREATED.with(|c| {
            let mut v = c.borrow_mut();
            let slice = v[start..].to_vec();
            v.truncate(start);
            slice
        });
        REG.with(|cell| {
            let mut reg = cell.borrow_mut();
            for h in to_drop {
                reg.map.remove(&h);
            }
        });
    }
}
