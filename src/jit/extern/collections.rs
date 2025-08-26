use std::sync::Arc;

use crate::backend::vm::VMValue;
use crate::box_trait::{NyashBox, IntegerBox, StringBox};

/// Symbol names for host externs (stable ABI for JIT)
pub const SYM_ARRAY_LEN: &str = "nyash.array.len";
pub const SYM_ARRAY_GET: &str = "nyash.array.get";
pub const SYM_ARRAY_SET: &str = "nyash.array.set";
pub const SYM_ARRAY_PUSH: &str = "nyash.array.push";

pub const SYM_MAP_GET: &str = "nyash.map.get";
pub const SYM_MAP_SET: &str = "nyash.map.set";
pub const SYM_MAP_SIZE: &str = "nyash.map.size";

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
    if let (Some(arr), Some(VMValue::Integer(idx)), Some(value)) = (as_array(args), args.get(1), args.get(2)) {
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        let res = arr.set(Box::new(IntegerBox::new(*idx)), val_box);
        return VMValue::from_nyash_box(res);
    }
    VMValue::BoxRef(Arc::new(StringBox::new("Error: array.set expects (ArrayBox, i64, value)")))
}

pub fn array_push(args: &[VMValue]) -> VMValue {
    if let (Some(arr), Some(value)) = (as_array(args), args.get(1)) {
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        let res = arr.push(val_box);
        return VMValue::from_nyash_box(res);
    }
    VMValue::BoxRef(Arc::new(StringBox::new("Error: array.push expects (ArrayBox, value)")))
}

pub fn map_get(args: &[VMValue]) -> VMValue {
    if let (Some(map), Some(key)) = (as_map(args), args.get(1)) {
        let key_box: Box<dyn NyashBox> = key.to_nyash_box();
        return VMValue::from_nyash_box(map.get(key_box));
    }
    VMValue::Void
}

pub fn map_set(args: &[VMValue]) -> VMValue {
    if let (Some(map), Some(key), Some(value)) = (as_map(args), args.get(1), args.get(2)) {
        let key_box: Box<dyn NyashBox> = key.to_nyash_box();
        let val_box: Box<dyn NyashBox> = value.to_nyash_box();
        return VMValue::from_nyash_box(map.set(key_box, val_box));
    }
    VMValue::BoxRef(Arc::new(StringBox::new("Error: map.set expects (MapBox, key, value)")))
}

pub fn map_size(args: &[VMValue]) -> VMValue {
    if let Some(map) = as_map(args) {
        if let Some(sz) = map.size().as_any().downcast_ref::<IntegerBox>() {
            return VMValue::Integer(sz.value);
        }
    }
    VMValue::Integer(0)
}

