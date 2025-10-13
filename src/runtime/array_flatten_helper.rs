//! array_flatten_helper.rs
//!
//! Common helper functions for flattening ArrayBox arguments.
//! Used by CallableBox.call/1 and CallableBox.callAsync/1.
//!
//! Supports both builtin ArrayBox and plugin ArrayBox through unified interface.

use crate::backend::vm_types::VMValue;

/// Check if the given VMValue is an ArrayBox (builtin or plugin).
///
/// Returns true if:
/// - VMValue::BoxRef contains builtin crate::boxes::array::ArrayBox, OR
/// - VMValue::BoxRef contains PluginBoxV2 with box_type == "ArrayBox"
pub fn is_array(v: &VMValue) -> bool {
    if let VMValue::BoxRef(bx) = v {
        // Check builtin ArrayBox
        if bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() {
            return true;
        }
        // Check plugin ArrayBox
        if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            if p.box_type == "ArrayBox" {
                return true;
            }
        }
    }
    false
}

/// Get the length of an ArrayBox.
///
/// For builtin ArrayBox: directly access .items.read().unwrap().len()
/// For plugin ArrayBox: call size() method via route()
///
/// Returns 0 if not an ArrayBox or error occurs.
pub fn get_len(v: &VMValue) -> usize {
    if let VMValue::BoxRef(bx) = v {
        // Builtin ArrayBox
        if let Some(a) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            return a.items.read().unwrap().len();
        }
        // Plugin ArrayBox: call size() via route
        if let Some(_p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            let mut tmp_interp = crate::backend::mir_interpreter::MirInterpreter::new();
            if let Ok(VMValue::Integer(sz)) = crate::runtime::method_router_box::route(&mut tmp_interp, v, "size", &[]) {
                return sz as usize;
            }
        }
    }
    0
}

/// Get element at index i from an ArrayBox.
///
/// For builtin ArrayBox: directly access items[i]
/// For plugin ArrayBox: call get(i) method via route()
///
/// Returns cloned value if successful, otherwise returns cloned v.
pub fn get_element(v: &VMValue, i: usize) -> VMValue {
    if let VMValue::BoxRef(bx) = v {
        // Builtin ArrayBox
        if let Some(a) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let guard = a.items.read().unwrap();
            return VMValue::from_nyash_box(guard[i].clone_box());
        }
        // Plugin ArrayBox: call get(i) via route
        if let Some(_p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            let mut tmp_interp = crate::backend::mir_interpreter::MirInterpreter::new();
            if let Ok(val) = crate::runtime::method_router_box::route(&mut tmp_interp, v, "get", &[VMValue::Integer(i as i64)]) {
                return val;
            }
        }
    }
    v.clone()
}
