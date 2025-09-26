//! State management for FileBox plugin

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

// ============ FileBox Instance ============
pub struct FileBoxInstance {
    pub file: Option<File>,
    pub path: String,
    pub buffer: Option<Vec<u8>>, // プラグインが管理するバッファ
}

impl FileBoxInstance {
    pub fn new() -> Self {
        Self {
            file: None,
            path: String::new(),
            buffer: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_path(path: String) -> Self {
        Self {
            file: None,
            path,
            buffer: None,
        }
    }
}

// グローバルインスタンス管理
pub static INSTANCES: Lazy<Mutex<HashMap<u32, FileBoxInstance>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// インスタンスIDカウンタ（1開始）
pub static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Allocate a new instance ID
#[allow(dead_code)]
pub fn allocate_instance_id() -> u32 {
    INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Store an instance with the given ID
#[allow(dead_code)]
pub fn store_instance(id: u32, instance: FileBoxInstance) -> Result<(), &'static str> {
    match INSTANCES.lock() {
        Ok(mut map) => {
            map.insert(id, instance);
            Ok(())
        }
        Err(_) => Err("Failed to lock instances map"),
    }
}

/// Remove an instance by ID
#[allow(dead_code)]
pub fn remove_instance(id: u32) -> Option<FileBoxInstance> {
    match INSTANCES.lock() {
        Ok(mut map) => map.remove(&id),
        Err(_) => None,
    }
}

/// Get mutable access to an instance
#[allow(dead_code)]
pub fn with_instance_mut<F, R>(id: u32, f: F) -> Result<R, &'static str>
where
    F: FnOnce(&mut FileBoxInstance) -> R,
{
    match INSTANCES.lock() {
        Ok(mut map) => match map.get_mut(&id) {
            Some(instance) => Ok(f(instance)),
            None => Err("Instance not found"),
        },
        Err(_) => Err("Failed to lock instances map"),
    }
}

/// Get access to an instance
#[allow(dead_code)]
pub fn with_instance<F, R>(id: u32, f: F) -> Result<R, &'static str>
where
    F: FnOnce(&FileBoxInstance) -> R,
{
    match INSTANCES.lock() {
        Ok(map) => match map.get(&id) {
            Some(instance) => Ok(f(instance)),
            None => Err("Instance not found"),
        },
        Err(_) => Err("Failed to lock instances map"),
    }
}

/// Clear all instances from the registry
pub fn clear_instances() {
    if let Ok(mut map) = INSTANCES.lock() {
        map.clear();
    }
}
