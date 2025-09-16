//! Global cache version map for vtable/PIC invalidation

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static CACHE_VERSIONS: Lazy<Mutex<HashMap<String, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Get current version for a cache label (default 0)
pub fn get_version(label: &str) -> u32 {
    let map = CACHE_VERSIONS.lock().unwrap();
    *map.get(label).unwrap_or(&0)
}

/// Bump version for a cache label
pub fn bump_version(label: &str) {
    let mut map = CACHE_VERSIONS.lock().unwrap();
    let e = map.entry(label.to_string()).or_insert(0);
    *e = e.saturating_add(1);
}

/// Convenience: bump for multiple labels
pub fn bump_many(labels: &[String]) {
    let mut map = CACHE_VERSIONS.lock().unwrap();
    for l in labels {
        let e = map.entry(l.clone()).or_insert(0);
        *e = e.saturating_add(1);
    }
}
