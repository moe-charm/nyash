//! Handle-based extern thunks used by the JIT runtime path.
//! Moved out of builder.rs to keep files small and responsibilities clear.

#[cfg(feature = "cranelift-jit")]
use crate::jit::events;

#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::collections as c;

// ---- Math (native f64) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_sin_f64(x: f64) -> f64 { x.sin() }
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_cos_f64(x: f64) -> f64 { x.cos() }
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_abs_f64(x: f64) -> f64 { x.abs() }
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_min_f64(a: f64, b: f64) -> f64 { a.min(b) }
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_max_f64(a: f64, b: f64) -> f64 { a.max(b) }

// ---- Array (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_len_h(handle: u64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_ARRAY_LEN_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_get_h(handle: u64, idx: i64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_ARRAY_GET_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let val = arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
            if let Some(ib) = val.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_last_h(handle: u64) -> i64 {
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Ok(items) = arr.items.read() {
                if let Some(last) = items.last() {
                    if let Some(ib) = last.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
                }
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_set_h(handle: u64, idx: i64, val: i64) -> i64 {
    use crate::jit::hostcall_registry::{classify, HostcallKind};
    let sym = c::SYM_ARRAY_SET_H;
    let pol = crate::jit::policy::current();
    let wh = pol.hostcall_whitelist;
    if classify(sym) == HostcallKind::Mutating && pol.read_only && !wh.iter().any(|s| s == sym) {
        events::emit_runtime(serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}), "hostcall", "<jit>");
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let _ = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), Box::new(crate::box_trait::IntegerBox::new(val)));
            events::emit_runtime(serde_json::json!({"id": sym, "decision":"allow", "argc":3, "arg_types":["Handle","I64","I64"]}), "hostcall", "<jit>");
            return 0;
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_push_h(handle: u64, val: i64) -> i64 {
    use crate::jit::hostcall_registry::{classify, HostcallKind};
    let sym = c::SYM_ARRAY_PUSH_H;
    let pol = crate::jit::policy::current();
    let wh = pol.hostcall_whitelist;
    if classify(sym) == HostcallKind::Mutating && pol.read_only && !wh.iter().any(|s| s == sym) {
        events::emit_runtime(serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}), "hostcall", "<jit>");
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let ib = crate::box_trait::IntegerBox::new(val);
            let _ = arr.push(Box::new(ib));
            events::emit_runtime(serde_json::json!({"id": sym, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}), "hostcall", "<jit>");
            return 0;
        }
    }
    0
}

// ---- Map (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_size_h(handle: u64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_MAP_SIZE_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            if let Some(ib) = map.size().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_get_h(handle: u64, key: i64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_MAP_GET_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box = Box::new(crate::box_trait::IntegerBox::new(key));
            let val = map.get(key_box);
            if let Some(ib) = val.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_get_hh(map_h: u64, key_h: u64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_MAP_GET_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}), "hostcall", "<jit>");
    let map_arc = crate::jit::rt::handles::get(map_h);
    let key_arc = crate::jit::rt::handles::get(key_h);
    if let (Some(mobj), Some(kobj)) = (map_arc, key_arc) {
        if let Some(map) = mobj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box: Box<dyn crate::box_trait::NyashBox> = kobj.share_box();
            let val = map.get(key_box);
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(val);
            let h = crate::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_set_h(handle: u64, key: i64, val: i64) -> i64 {
    use crate::jit::hostcall_registry::{classify, HostcallKind};
    let sym = c::SYM_MAP_SET_H;
    let pol = crate::jit::policy::current();
    let wh = pol.hostcall_whitelist;
    if classify(sym) == HostcallKind::Mutating && pol.read_only && !wh.iter().any(|s| s == sym) {
        events::emit_runtime(serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}), "hostcall", "<jit>");
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box = Box::new(crate::box_trait::IntegerBox::new(key));
            let val_box = Box::new(crate::box_trait::IntegerBox::new(val));
            let _ = map.set(key_box, val_box);
            events::emit_runtime(serde_json::json!({"id": sym, "decision":"allow", "argc":3, "arg_types":["Handle","I64","I64"]}), "hostcall", "<jit>");
            return 0;
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_has_h(handle: u64, key: i64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_MAP_HAS_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box = Box::new(crate::box_trait::IntegerBox::new(key));
            let val = map.get(key_box);
            let is_present = !val.as_any().is::<crate::box_trait::VoidBox>();
            return if is_present { 1 } else { 0 };
        }
    }
    0
}

// ---- Any helpers ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_any_length_h(handle: u64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_ANY_LEN_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
        }
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            return sb.value.len() as i64;
        }
    } else {
        // Fallback: some call sites may still pass a parameter index instead of a handle (legacy path)
        // Try to interpret small values as param index and read from legacy VM args
        if handle <= 16 {
            let idx = handle as usize;
            let val = crate::jit::rt::with_legacy_vm_args(|args| args.get(idx).cloned());
            if let Some(v) = val {
                match v {
                    crate::backend::vm::VMValue::BoxRef(b) => {
                        if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                            if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
                        }
                        if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                            return sb.value.len() as i64;
                        }
                    }
                    crate::backend::vm::VMValue::String(s) => { return s.len() as i64; }
                    _ => {}
                }
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_any_is_empty_h(handle: u64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_ANY_IS_EMPTY_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}), "hostcall", "<jit>");
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Ok(items) = arr.items.read() { return if items.is_empty() { 1 } else { 0 }; }
        }
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            return if sb.value.is_empty() { 1 } else { 0 };
        }
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            if let Some(ib) = map.size().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return if ib.value == 0 { 1 } else { 0 }; }
        }
    }
    0
}

// ---- String ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_charcode_at_h(handle: u64, idx: i64) -> i64 {
    events::emit_runtime(serde_json::json!({"id": c::SYM_STRING_CHARCODE_AT_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}), "hostcall", "<jit>");
    if idx < 0 { return -1; }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            let s = &sb.value;
            let i = idx as usize;
            if i < s.len() { return s.as_bytes()[i] as i64; } else { return -1; }
        }
    }
    -1
}

// ---- Birth (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_birth_h() -> i64 {
    // Create a new StringBox via unified plugin host (or builtin fallback), store as handle
    if let Ok(host_g) = crate::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("StringBox", &[]) {
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = crate::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_integer_birth_h() -> i64 {
    if let Ok(host_g) = crate::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("IntegerBox", &[]) {
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = crate::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}
