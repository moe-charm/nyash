//! array_flatten_helper.rs
//!
//! Common helper functions for flattening ArrayBox arguments.
//! Used by CallableBox.call/1 and CallableBox.callAsync/1.
//!
//! Supports both builtin ArrayBox and plugin ArrayBox through unified interface.

use crate::backend::vm_types::VMValue;

#[path = "array_flatten_helper_builtin.rs"]
mod array_flatten_helper_builtin;
#[path = "array_flatten_helper_plugin.rs"]
mod array_flatten_helper_plugin;

/// Check if the given VMValue is an ArrayBox (builtin or plugin).
///
/// Returns true if:
/// - VMValue::BoxRef contains builtin crate::boxes::array::ArrayBox, OR
/// - VMValue::BoxRef contains PluginBoxV2 with box_type == "ArrayBox"
pub fn is_array(v: &VMValue) -> bool {
    // Prefer builtin when available; otherwise plugin判定
    #[cfg(feature = "legacy-boxes")]
    if array_flatten_helper_builtin::is_array(v) { return true; }
    array_flatten_helper_plugin::is_array(v)
}

/// Get the length of an ArrayBox.
///
/// For builtin ArrayBox: directly access .items.read().unwrap().len()
/// For plugin ArrayBox: call size() method via route()
///
/// Returns 0 if not an ArrayBox or error occurs.
pub fn get_len(v: &VMValue) -> usize {
    #[cfg(feature = "legacy-boxes")]
    if let Some(n) = array_flatten_helper_builtin::get_len(v) { return n; }
    array_flatten_helper_plugin::get_len(v)
}

/// Get element at index i from an ArrayBox.
///
/// For builtin ArrayBox: directly access items[i]
/// For plugin ArrayBox: call get(i) method via route()
///
/// Returns cloned value if successful, otherwise returns cloned v.
pub fn get_element(v: &VMValue, i: usize) -> VMValue {
    #[cfg(feature = "legacy-boxes")]
    if let Some(val) = array_flatten_helper_builtin::get_element(v, i) { return val; }
    array_flatten_helper_plugin::get_element(v, i)
}
