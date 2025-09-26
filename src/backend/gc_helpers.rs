//! GC-related small helpers for VM-side use

use crate::backend::vm::VMValue;

/// Return true if the BoxCall is a known mutating builtin call (e.g., Array/Map set/push)
pub fn is_mutating_builtin_call(recv: &VMValue, method: &str) -> bool {
    // Lightweight table of mutating methods by builtin box type
    // Array: set, push
    // Map: set, put, insert, remove (superset to future-proof)
    const ARRAY_METHODS: &[&str] = &["set", "push"];
    const MAP_METHODS: &[&str] = &["set", "put", "insert", "remove"]; // tolerate aliases

    match recv {
        VMValue::BoxRef(b) => {
            if b.as_any()
                .downcast_ref::<crate::boxes::array::ArrayBox>()
                .is_some()
            {
                return ARRAY_METHODS.iter().any(|m| *m == method);
            }
            if b.as_any()
                .downcast_ref::<crate::boxes::map_box::MapBox>()
                .is_some()
            {
                return MAP_METHODS.iter().any(|m| *m == method);
            }
            false
        }
        _ => false,
    }
}

/// Unified trigger for GC Write-Barrier with site logging
pub fn gc_write_barrier_site(runtime: &crate::runtime::NyashRuntime, site: &str) {
    let trace = crate::config::env::gc_trace();
    let strict = crate::config::env::gc_barrier_strict();
    let before = if strict {
        runtime.gc.snapshot_counters()
    } else {
        None
    };
    if trace {
        eprintln!("[GC] barrier: Write @{}", site);
    }
    runtime.gc.barrier(crate::runtime::gc::BarrierKind::Write);
    if strict {
        let after = runtime.gc.snapshot_counters();
        match (before, after) {
            (Some((_, _, bw)), Some((_, _, aw))) if aw > bw => {}
            (Some(_), Some(_)) => {
                panic!(
                    "[GC][STRICT] write barrier did not increment at site='{}'",
                    site
                );
            }
            _ => {
                panic!(
                    "[GC][STRICT] CountingGc required for strict verification at site='{}'",
                    site
                );
            }
        }
    }
}
