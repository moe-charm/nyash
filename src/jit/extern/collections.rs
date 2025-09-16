use std::sync::Arc;

use crate::backend::vm::VMValue;
use crate::box_trait::{IntegerBox, NyashBox, StringBox};

/// Symbol names for host externs (stable ABI for JIT)
pub const SYM_ARRAY_LEN: &str = "nyash.array.len";
pub const SYM_ARRAY_GET: &str = "nyash.array.get";
pub const SYM_ARRAY_SET: &str = "nyash.array.set";
pub const SYM_ARRAY_PUSH: &str = "nyash.array.push";

pub const SYM_MAP_GET: &str = "nyash.map.get";
pub const SYM_MAP_SET: &str = "nyash.map.set";
pub const SYM_MAP_SIZE: &str = "nyash.map.size";

// Handle-based variants for direct JIT bridging
pub const SYM_ARRAY_LEN_H: &str = "nyash.array.len_h";
pub const SYM_ARRAY_GET_H: &str = "nyash.array.get_h";
pub const SYM_ARRAY_SET_H: &str = "nyash.array.set_h";
pub const SYM_ARRAY_SET_HH: &str = "nyash.array.set_hh";
pub const SYM_ARRAY_PUSH_H: &str = "nyash.array.push_h";
pub const SYM_ARRAY_LAST_H: &str = "nyash.array.last_h";
pub const SYM_MAP_SIZE_H: &str = "nyash.map.size_h";
pub const SYM_MAP_GET_H: &str = "nyash.map.get_h";
pub const SYM_MAP_GET_HH: &str = "nyash.map.get_hh";
pub const SYM_MAP_SET_H: &str = "nyash.map.set_h";
pub const SYM_MAP_HAS_H: &str = "nyash.map.has_h";
// Generic read-only helper
pub const SYM_ANY_LEN_H: &str = "nyash.any.length_h";
pub const SYM_ANY_IS_EMPTY_H: &str = "nyash.any.is_empty_h";
pub const SYM_STRING_CHARCODE_AT_H: &str = "nyash.string.charCodeAt_h";
pub const SYM_STRING_LEN_H: &str = "nyash.string.len_h";
pub const SYM_STRING_BIRTH_H: &str = "nyash.string.birth_h";
pub const SYM_STRING_FROM_U64X2: &str = "nyash.string.from_u64x2";
pub const SYM_INTEGER_BIRTH_H: &str = "nyash.integer.birth_h";
pub const SYM_CONSOLE_BIRTH_H: &str = "nyash.console.birth_h";
// String-like operations (handle, handle)
pub const SYM_STRING_CONCAT_HH: &str = "nyash.string.concat_hh";
pub const SYM_STRING_EQ_HH: &str = "nyash.string.eq_hh";
pub const SYM_STRING_LT_HH: &str = "nyash.string.lt_hh";
// Unified semantics: addition for dynamic boxes (handle,handle)
pub const SYM_SEMANTICS_ADD_HH: &str = "nyash.semantics.add_hh";

fn as_array(args: &[VMValue]) -> Option<&crate::boxes::array::ArrayBox> {
    match args.get(0) {
        Some(VMValue::BoxRef(b)) => b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>(),
        _ => None,
    }
}

fn as_map(args: &[VMValue]) -> Option<&crate::boxes::map_box::MapBox> {
    match args.get(0) {
        Some(VMValue::BoxRef(b)) => b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>(),
        _ => None,
    }
}

pub fn array_len(args: &[VMValue]) -> VMValue {
    if let Some(arr) = as_array(args) {
        if let Some(len_box) = arr.length().as_any().downcast_ref::<IntegerBox>() {
            return VMValue::Integer(len_box.value);
        }
    }
    VMValue::Integer(0)
}

pub fn array_get(args: &[VMValue]) -> VMValue {
    if let (Some(arr), Some(VMValue::Integer(idx))) = (as_array(args), args.get(1)) {
        // ArrayBox.get expects a NyashBox index
        let val = arr.get(Box::new(IntegerBox::new(*idx)));
        return VMValue::from_nyash_box(val);
    }
    VMValue::Void
}

pub fn array_set(args: &[VMValue]) -> VMValue {
    // Enforce policy for mutating operation
    if crate::jit::policy::current().read_only
        && !crate::jit::policy::current()
            .hostcall_whitelist
            .iter()
            .any(|s| s == SYM_ARRAY_SET)
    {
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_ARRAY_SET, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return VMValue::Integer(0);
    }
    if let (Some(arr), Some(VMValue::Integer(idx)), Some(value)) =
        (as_array(args), args.get(1), args.get(2))
    {
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        let res = arr.set(Box::new(IntegerBox::new(*idx)), val_box);
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_ARRAY_SET, "decision":"allow", "argc":3, "arg_types":["Handle","I64","Handle"]}),
            "hostcall",
            "<jit>",
        );
        return VMValue::from_nyash_box(res);
    }
    VMValue::BoxRef(Arc::new(StringBox::new(
        "Error: array.set expects (ArrayBox, i64, value)",
    )))
}

pub fn array_push(args: &[VMValue]) -> VMValue {
    if crate::jit::policy::current().read_only
        && !crate::jit::policy::current()
            .hostcall_whitelist
            .iter()
            .any(|s| s == SYM_ARRAY_PUSH)
    {
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_ARRAY_PUSH, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return VMValue::Integer(0);
    }
    if let (Some(arr), Some(value)) = (as_array(args), args.get(1)) {
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        let res = arr.push(val_box);
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_ARRAY_PUSH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
            "hostcall",
            "<jit>",
        );
        return VMValue::from_nyash_box(res);
    }
    VMValue::BoxRef(Arc::new(StringBox::new(
        "Error: array.push expects (ArrayBox, value)",
    )))
}

pub fn map_get(args: &[VMValue]) -> VMValue {
    if let (Some(map), Some(key)) = (as_map(args), args.get(1)) {
        let key_box: Box<dyn NyashBox> = key.to_nyash_box();
        return VMValue::from_nyash_box(map.get(key_box));
    }
    VMValue::Void
}

pub fn map_set(args: &[VMValue]) -> VMValue {
    if crate::jit::policy::current().read_only
        && !crate::jit::policy::current()
            .hostcall_whitelist
            .iter()
            .any(|s| s == SYM_MAP_SET)
    {
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_MAP_SET, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return VMValue::Integer(0);
    }
    if let (Some(map), Some(key), Some(value)) = (as_map(args), args.get(1), args.get(2)) {
        let key_box: Box<dyn NyashBox> = key.to_nyash_box();
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        let out = map.set(key_box, val_box);
        crate::jit::events::emit_runtime(
            serde_json::json!({"id": SYM_MAP_SET, "decision":"allow", "argc":3, "arg_types":["Handle","Handle","Handle"]}),
            "hostcall",
            "<jit>",
        );
        return VMValue::from_nyash_box(out);
    }
    VMValue::BoxRef(Arc::new(StringBox::new(
        "Error: map.set expects (MapBox, key, value)",
    )))
}

pub fn map_size(args: &[VMValue]) -> VMValue {
    if let Some(map) = as_map(args) {
        if let Some(sz) = map.size().as_any().downcast_ref::<IntegerBox>() {
            return VMValue::Integer(sz.value);
        }
    }
    VMValue::Integer(0)
}
