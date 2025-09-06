//! Minimal global registry for env.modules (Phase 15 P0b)

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::box_trait::NyashBox;

static REGISTRY: Lazy<Mutex<HashMap<String, Box<dyn NyashBox>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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

