//! Handle-based extern thunks used by the JIT runtime path.
//! Moved out of builder.rs to keep files small and responsibilities clear.

#[cfg(feature = "cranelift-jit")]
use crate::jit::events;

#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::collections as c;
#[cfg(feature = "cranelift-jit")]
use crate::runtime::plugin_loader_unified;
#[cfg(feature = "cranelift-jit")]
use crate::runtime::plugin_loader_v2::PluginBoxV2;

// ---- Generic Birth (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_box_birth_h(type_id: i64) -> i64 {
    // Map type_id -> type name and create via plugin host; return runtime handle
    if type_id <= 0 { return 0; }
    let tid = type_id as u32;
    let name_opt = crate::runtime::plugin_loader_unified::get_global_plugin_host()
        .read().ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| m.into_iter().find(|(_k,v)| *v == tid).map(|(k,_v)| k));
    if let Some(box_type) = name_opt {
        if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host.create_box(&box_type, &[]) {
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
                let h = crate::jit::rt::handles::to_handle(arc);
                events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_h", "box_type": box_type, "type_id": tid, "handle": h}), "hostcall", "<jit>");
                return h as i64;
            } else {
                events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_h", "error": "create_failed", "box_type": box_type, "type_id": tid}), "hostcall", "<jit>");
            }
        }
    } else {
        events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_h", "error": "type_map_failed", "type_id": tid}), "hostcall", "<jit>");
    }
    0
}
// Generic birth with args on JIT side: (type_id, argc, a1, a2) -> handle
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_box_birth_i64(type_id: i64, argc: i64, a1: i64, a2: i64) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    if type_id <= 0 { return 0; }
    // Resolve invoke for the type by creating a temp instance
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    let mut box_type = String::new();
    if let Some(name) = crate::runtime::plugin_loader_unified::get_global_plugin_host()
        .read().ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| m.into_iter().find(|(_k,v)| *v == (type_id as u32)).map(|(k,_v)| k))
    {
        box_type = name;
        if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host.create_box(&box_type, &[]) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() { invoke = Some(p.inner.invoke_fn); }
            }
        }
    }
    if invoke.is_none() {
        events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_i64", "error": "no_invoke", "type_id": type_id}), "hostcall", "<jit>");
        return 0;
    }
    let method_id: u32 = 0; let instance_id: u32 = 0;
    // Build TLV from a1/a2
    let nargs = argc.max(0) as usize;
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut encode_val = |h: i64| {
        if h > 0 {
            if let Some(obj) = crate::jit::rt::handles::get(h as u64) {
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = crate::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                if let Some(s) = sb.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); return; }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                if let Some(i) = ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); return; }
                            }
                        }
                    }
                    crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                    return;
                }
            }
        }
        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, h);
    };
    if nargs >= 1 { encode_val(a1); }
    if nargs >= 2 { encode_val(a2); }
    // Invoke
    let mut out = vec![0u8; 1024]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_i64", "error": "invoke_failed", "type_id": type_id}), "hostcall", "<jit>"); return 0; }
    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        if tag == 8 && payload.len()==8 {
            let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]); let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]);
            let r_type = u32::from_le_bytes(t); let r_inst = u32::from_le_bytes(i);
            let pb = crate::runtime::plugin_loader_v2::make_plugin_box_v2(box_type.clone(), r_type, r_inst, invoke.unwrap());
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::new(pb);
            let h = crate::jit::rt::handles::to_handle(arc);
            events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_i64", "box_type": box_type, "type_id": type_id, "argc": nargs, "handle": h}), "hostcall", "<jit>");
            return h as i64;
        }
    }
    events::emit_runtime(serde_json::json!({"id": "nyash.box.birth_i64", "error": "decode_failed", "type_id": type_id}), "hostcall", "<jit>");
    0
}

// ---- Handle helpers ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_handle_of(v: i64) -> i64 {
    // If already a positive handle, pass through
    if v > 0 { return v; }
    // Otherwise interpret as legacy param index and convert BoxRef -> handle
    if v >= 0 {
        let idx = v as usize;
        let mut out: i64 = 0;
        crate::jit::rt::with_legacy_vm_args(|args| {
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b.clone());
                out = crate::jit::rt::handles::to_handle(arc) as i64;
            }
        });
        return out;
    }
    0
}

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

// ---- By-name plugin invoke (generic receiver; resolves method_id at runtime) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_plugin_invoke_name_getattr_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("getattr", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_plugin_invoke_name_call_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("call", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
fn nyash_plugin_invoke_name_common_i64(method: &str, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    // Resolve receiver
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0.max(0) as usize;
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let crate::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                        invoke = Some(p.inner.invoke_fn); break;
                    }
                }
            }
        });
    }
    if invoke.is_none() { events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "error": "no_invoke"}), "hostcall", "<jit>"); return 0; }
    let box_type = box_type.unwrap_or_default();
    // Resolve method_id via PluginHost
    let mh = if let Ok(host) = plugin_loader_unified::get_global_plugin_host().read() { host.resolve_method(&box_type, method) } else { events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "host_read_failed"}), "hostcall", "<jit>"); return 0 };
    let method_id = match mh { Ok(h) => h.method_id, Err(_) => { events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "resolve_failed"}), "hostcall", "<jit>"); return 0 } } as u32;
    // Build TLV args from a1/a2 preferring handles; fallback to legacy (skip receiver=pos0)
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(argc.saturating_sub(1).max(0) as u16);
    let mut encode_arg = |val: i64, pos: usize| {
        let mut appended = false;
        if val > 0 {
            if let Some(obj) = crate::jit::rt::handles::get(val as u64) {
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = crate::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                if let Some(s) = sb.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); appended = true; }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                if let Some(i) = ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); appended = true; }
                            }
                        }
                    }
                    if !appended { crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id()); appended = true; }
                }
            }
        }
        if !appended {
            // Fallback: encode from legacy VM args at position
            crate::jit::rt::with_legacy_vm_args(|args| {
                if let Some(v) = args.get(pos) {
                    use crate::backend::vm::VMValue as V;
                    match v {
                        V::String(s) => crate::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                        V::Integer(i) => crate::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                        V::Float(f) => crate::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                        V::Bool(b) => crate::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                        V::BoxRef(b) => {
                            if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                                let host = crate::runtime::get_global_plugin_host();
                                if let Ok(hg) = host.read() {
                                    if p.box_type == "StringBox" {
                                        if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                            if let Some(s) = sb.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); return; }
                                        }
                                    } else if p.box_type == "IntegerBox" {
                                        if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                            if let Some(i) = ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); return; }
                                        }
                                    }
                                }
                                crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                            } else {
                                let s = b.to_string_box().value; crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                            }
                        }
                        _ => {}
                    }
                } else {
                    // No legacy arg: encode raw i64 as last resort
                    crate::runtime::plugin_ffi_common::encode::i64(&mut buf, val);
                }
            });
        }
    };
    if argc >= 2 { encode_arg(a1, 1); }
    if argc >= 3 { encode_arg(a2, 2); }
    let mut out = vec![0u8; 4096]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "invoke_failed"}), "hostcall", "<jit>"); return 0; }
    let out_slice = &out[..out_len];
    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice) {
        match tag {
            3 => { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); } }
            1 => { return if crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => { events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "warn": "first_tlv_not_primitive_or_handle", "tag": tag}), "hostcall", "<jit>"); }
        }
    }
    events::emit_runtime(serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "decode_failed"}), "hostcall", "<jit>");
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
