//! Instance Manager Macros for Plugins
//!
//! Provides standardized instance storage and access patterns for all plugins.
//! Eliminates ~150 lines of boilerplate across 15 plugins.
//!
//! ## Usage
//!
//! ```rust
//! use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};
//!
//! // Define your instance type
//! struct MyPluginInstance {
//!     data: Vec<String>,
//! }
//!
//! // Generate storage (replaces 5 lines of boilerplate)
//! define_instance_storage!(MyPluginInstance);
//!
//! // Safe instance access (replaces 5-7 lines each time)
//! pub extern "C" fn my_method(instance_id: u32) -> i32 {
//!     with_instance!(instance_id, |inst| {
//!         inst.data.len() as i32
//!     }).unwrap_or(-1)
//! }
//!
//! pub extern "C" fn my_mutating_method(instance_id: u32, value: i64) -> i32 {
//!     with_instance_mut!(instance_id, |inst| {
//!         inst.data.push(value.to_string());
//!         0
//!     }).unwrap_or(-1)
//! }
//! ```
//!
//! ## Benefits
//!
//! - **Deadlock prevention**: Automatic scope management
//! - **Error handling**: Unified error codes (HAKO_E_INVALID_HANDLE, HAKO_E_PLUGIN_ERROR)
//! - **Code reduction**: ~10 lines → 1-2 lines per instance access
//! - **Type safety**: Compile-time instance type checking

/// Define instance storage for a plugin
///
/// Generates:
/// - `INSTANCES`: Global instance registry (Lazy<Mutex<HashMap<u32, T>>>)
/// - `INSTANCE_COUNTER`: Atomic counter for instance IDs
///
/// # Example
///
/// ```rust
/// struct ArrayInstance { data: Vec<ArrayValue> }
/// define_instance_storage!(ArrayInstance);
/// ```
#[macro_export]
macro_rules! define_instance_storage {
    ($inst_type:ty) => {
        use once_cell::sync::Lazy;
        use std::collections::HashMap;
        use std::sync::{
            atomic::{AtomicU32, Ordering},
            Mutex,
        };

        static INSTANCES: Lazy<Mutex<HashMap<u32, $inst_type>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));
        static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

        /// Allocate a new instance ID
        #[allow(dead_code)]
        fn allocate_instance_id() -> u32 {
            INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
        }

        /// Store an instance with the given ID
        #[allow(dead_code)]
        fn store_instance(id: u32, instance: $inst_type) -> Result<(), i32> {
            match INSTANCES.lock() {
                Ok(mut map) => {
                    map.insert(id, instance);
                    Ok(())
                }
                Err(_) => Err(-5), // E_PLUGIN_ERROR
            }
        }

        /// Remove an instance by ID
        #[allow(dead_code)]
        fn remove_instance(id: u32) -> Option<$inst_type> {
            match INSTANCES.lock() {
                Ok(mut map) => map.remove(&id),
                Err(_) => None,
            }
        }

        /// Clear all instances (for testing)
        #[allow(dead_code)]
        fn clear_instances() {
            if let Ok(mut map) = INSTANCES.lock() {
                map.clear();
            }
        }
    };
}

/// Safe immutable instance access
///
/// Automatically handles:
/// - Lock acquisition and release
/// - Instance existence check
/// - Error code mapping (lock error → HAKO_E_PLUGIN_ERROR, not found → HAKO_E_INVALID_HANDLE)
///
/// # Example
///
/// ```rust
/// with_instance!(instance_id, |inst| {
///     inst.data.len() as i32
/// })
/// ```
///
/// Returns: `Result<T, i32>` where T is the closure's return type
#[macro_export]
macro_rules! with_instance {
    ($id:expr, $f:expr) => {{
        match INSTANCES.lock() {
            Ok(map) => match map.get(&$id) {
                Some(inst) => Ok($f(inst)),
                None => Err(-8), // E_INVALID_HANDLE
            },
            Err(_) => Err(-5), // E_PLUGIN_ERROR
        }
    }};
}

/// Safe mutable instance access
///
/// Same as `with_instance!` but provides `&mut` access to the instance.
///
/// # Example
///
/// ```rust
/// with_instance_mut!(instance_id, |inst| {
///     inst.data.push(value);
///     0 // success
/// })
/// ```
///
/// Returns: `Result<T, i32>` where T is the closure's return type
#[macro_export]
macro_rules! with_instance_mut {
    ($id:expr, $f:expr) => {{
        match INSTANCES.lock() {
            Ok(mut map) => match map.get_mut(&$id) {
                Some(inst) => Ok($f(inst)),
                None => Err(-8), // E_INVALID_HANDLE
            },
            Err(_) => Err(-5), // E_PLUGIN_ERROR
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_instance_manager_basic() {
        struct TestInstance {
            value: i32,
        }

        define_instance_storage!(TestInstance);

        // Allocate and store
        let id = allocate_instance_id();
        assert!(store_instance(id, TestInstance { value: 42 }).is_ok());

        // Read access
        let result = with_instance!(id, |inst| inst.value);
        assert_eq!(result, Ok(42));

        // Write access
        let result = with_instance_mut!(id, |inst| {
            inst.value = 100;
            inst.value
        });
        assert_eq!(result, Ok(100));

        // Verify mutation
        let result = with_instance!(id, |inst| inst.value);
        assert_eq!(result, Ok(100));

        // Remove
        let removed = remove_instance(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 100);

        // Access after remove should fail
        let result = with_instance!(id, |inst| inst.value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), -8); // E_INVALID_HANDLE
    }
}
