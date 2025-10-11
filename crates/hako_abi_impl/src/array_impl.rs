//! Array ABI implementation (shared by all consumers)

use hako_abi::{ArrayAbi, HakoHandle};
use hako_core_array::{classify_set_index, SetIndex};
use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicU64, Ordering}};

/// Array element value (plugin-side, no NyashBox dependency)
#[derive(Clone, Debug)]
pub enum ArrayValue {
    I64(i64),
    // TODO: Add String, Handle variants
}

/// Single array instance
struct ArrayInstance {
    data: Vec<ArrayValue>,
}

/// Thread-safe registry of all array instances
pub struct ArrayRegistry {
    next_id: AtomicU64,
    instances: Mutex<HashMap<u64, ArrayInstance>>,
}

impl ArrayRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            instances: Mutex::new(HashMap::new()),
        }
    }

    fn alloc(&self) -> HakoHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut map = self.instances.lock().unwrap();
        map.insert(id, ArrayInstance { data: Vec::new() });
        id
    }

    fn with_instance<F, R>(&self, handle: HakoHandle, f: F) -> Option<R>
    where
        F: FnOnce(&ArrayInstance) -> R,
    {
        let map = self.instances.lock().unwrap();
        map.get(&handle).map(f)
    }

    fn with_instance_mut<F, R>(&self, handle: HakoHandle, f: F) -> Option<R>
    where
        F: FnOnce(&mut ArrayInstance) -> R,
    {
        let mut map = self.instances.lock().unwrap();
        map.get_mut(&handle).map(f)
    }
}

impl ArrayAbi for ArrayRegistry {
    fn array_new() -> HakoHandle {
        REGISTRY.alloc()
    }

    fn array_get(handle: HakoHandle, idx: i64) -> i64 {
        REGISTRY
            .with_instance(handle, |inst| {
                if let Some(i) = hako_core_array::safe_get_index(inst.data.len(), idx) {
                    match &inst.data[i] {
                        ArrayValue::I64(v) => *v,
                    }
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64 {
        REGISTRY
            .with_instance_mut(handle, |inst| {
                match classify_set_index(inst.data.len(), idx) {
                    SetIndex::Replace(i) => {
                        inst.data[i] = ArrayValue::I64(val);
                        0
                    }
                    SetIndex::Append => {
                        inst.data.push(ArrayValue::I64(val));
                        0
                    }
                    SetIndex::Oob => -1,
                }
            })
            .unwrap_or(-1)
    }

    fn array_push(handle: HakoHandle, val: i64) -> i64 {
        REGISTRY
            .with_instance_mut(handle, |inst| {
                inst.data.push(ArrayValue::I64(val));
                inst.data.len() as i64
            })
            .unwrap_or(0)
    }

    fn array_len(handle: HakoHandle) -> i64 {
        REGISTRY
            .with_instance(handle, |inst| hako_core_array::length(inst.data.len()))
            .unwrap_or(0)
    }
}

static REGISTRY: once_cell::sync::Lazy<ArrayRegistry> =
    once_cell::sync::Lazy::new(ArrayRegistry::new);

#[cfg(test)]
mod tests {
    use super::*;
    use hako_abi::HAKO_INVALID_HANDLE;

    #[test]
    fn test_array_basic() {
        let h = ArrayRegistry::array_new();
        assert_ne!(h, HAKO_INVALID_HANDLE);

        // Push
        let len = ArrayRegistry::array_push(h, 42);
        assert_eq!(len, 1);

        // Get
        let val = ArrayRegistry::array_get(h, 0);
        assert_eq!(val, 42);

        // Len
        let len = ArrayRegistry::array_len(h);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_array_set() {
        let h = ArrayRegistry::array_new();
        ArrayRegistry::array_push(h, 10);

        // Replace
        let result = ArrayRegistry::array_set(h, 0, 20);
        assert_eq!(result, 0);
        assert_eq!(ArrayRegistry::array_get(h, 0), 20);

        // Append
        let result = ArrayRegistry::array_set(h, 1, 30);
        assert_eq!(result, 0);
        assert_eq!(ArrayRegistry::array_len(h), 2);

        // Out of bounds
        let result = ArrayRegistry::array_set(h, 10, 40);
        assert_eq!(result, -1);
    }
}
