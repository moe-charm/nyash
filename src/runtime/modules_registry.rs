//! Minimal global registry for env.modules (Phase 15 P0b)

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::box_trait::NyashBox;

static REGISTRY: Lazy<Mutex<HashMap<String, Box<dyn NyashBox>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn set(name: String, value: Box<dyn NyashBox>) {
    if let Ok(mut map) = REGISTRY.lock() {
        map.insert(name, value);
    }
}

pub fn get(name: &str) -> Option<Box<dyn NyashBox>> {
    if let Ok(mut map) = REGISTRY.lock() {
        if let Some(b) = map.get_mut(name) {
            // clone_box to hand out an owned copy
            return Some(b.clone_box());
        }
    }
    None
}

/// Snapshot names and their stringified values (best‑effort).
/// Intended for diagnostics; values are obtained via to_string_box().value.
pub fn snapshot_names_and_strings() -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Ok(mut map) = REGISTRY.lock() {
        for (k, v) in map.iter_mut() {
            // Best-effort stringify
            let s = v.to_string_box().value;
            out.push((k.clone(), s));
        }
    }
    out
}

/// Snapshot all Box values as GC roots (Arc<dyn NyashBox>), best‑effort.
/// Uses clone_box() to obtain owned copies and wraps them into Arc for traversal.
pub fn snapshot_boxes() -> Vec<std::sync::Arc<dyn NyashBox>> {
    let mut out = Vec::new();
    if let Ok(mut map) = REGISTRY.lock() {
        for (_k, v) in map.iter_mut() {
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(v.clone_box());
            out.push(arc);
        }
    }
    out
}
