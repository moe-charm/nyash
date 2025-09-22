//! Handle-based extern thunks used by the JIT runtime path.
//! Moved out of builder.rs to keep files small and responsibilities clear.

#[cfg(feature = "cranelift-jit")]
use crate::jit::events;

#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::collections as c;
#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::host_bridge as hb;
#[cfg(feature = "cranelift-jit")]
use crate::runtime::plugin_loader_v2::PluginBoxV2;

// ---- Generic Birth (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_box_birth_h(type_id: i64) -> i64 {
    // Map type_id -> type name and create via plugin host; return runtime handle
    if type_id <= 0 {
        return 0;
    }
    let tid = type_id as u32;
    if let Some(meta) = crate::runtime::plugin_loader_v2::metadata_for_type_id(tid) {
        if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host.create_box(&meta.box_type, &[]) {
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
                let h = crate::jit::rt::handles::to_handle(arc);
                events::emit_runtime(
                    serde_json::json!({"id": "nyash.box.birth_h", "box_type": meta.box_type, "type_id": meta.type_id, "handle": h}),
                    "hostcall",
                    "<jit>",
                );
                return h as i64;
            } else {
                events::emit_runtime(
                    serde_json::json!({"id": "nyash.box.birth_h", "error": "create_failed", "box_type": meta.box_type, "type_id": meta.type_id}),
                    "hostcall",
                    "<jit>",
                );
            }
        }
    } else {
        events::emit_runtime(
            serde_json::json!({"id": "nyash.box.birth_h", "error": "type_map_failed", "type_id": tid}),
            "hostcall",
            "<jit>",
        );
    }
    0
}
// Generic birth with args on JIT side: (type_id, argc, a1, a2) -> handle
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_box_birth_i64(type_id: i64, argc: i64, a1: i64, a2: i64) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    if type_id <= 0 {
        return 0;
    }
    // Resolve invoke for the type via loader metadata
    let Some(meta) = crate::runtime::plugin_loader_v2::metadata_for_type_id(type_id as u32) else {
        events::emit_runtime(
            serde_json::json!({"id": "nyash.box.birth_i64", "error": "type_map_failed", "type_id": type_id}),
            "hostcall",
            "<jit>",
        );
        return 0;
    };
    let invoke_fn = meta.invoke_fn;
    let box_type = meta.box_type.clone();
    let method_id: u32 = 0;
    let instance_id: u32 = 0;
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
                            if let Ok(Some(sb)) = hg.invoke_instance_method(
                                "StringBox",
                                "toUtf8",
                                p.instance_id(),
                                &[],
                            ) {
                                if let Some(s) =
                                    sb.as_any().downcast_ref::<crate::box_trait::StringBox>()
                                {
                                    crate::runtime::plugin_ffi_common::encode::string(
                                        &mut buf, &s.value,
                                    );
                                    return;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) =
                                hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                            {
                                if let Some(i) =
                                    ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                                {
                                    crate::runtime::plugin_ffi_common::encode::i64(
                                        &mut buf, i.value,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    crate::runtime::plugin_ffi_common::encode::plugin_handle(
                        &mut buf,
                        p.inner.type_id,
                        p.instance_id(),
                    );
                    return;
                }
            }
        }
        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, h);
    };
    if nargs >= 1 {
        encode_val(a1);
    }
    if nargs >= 2 {
        encode_val(a2);
    }
    // Invoke
    let mut out = vec![0u8; 1024];
    let mut out_len: usize = out.len();
    let rc = unsafe {
        invoke_fn(
            type_id as u32,
            method_id,
            instance_id,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if rc != 0 {
        events::emit_runtime(
            serde_json::json!({"id": "nyash.box.birth_i64", "error": "invoke_failed", "type_id": type_id}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    if let Some((tag, _sz, payload)) =
        crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        if tag == 8 && payload.len() == 8 {
            let mut t = [0u8; 4];
            t.copy_from_slice(&payload[0..4]);
            let mut i = [0u8; 4];
            i.copy_from_slice(&payload[4..8]);
            let r_type = u32::from_le_bytes(t);
            let r_inst = u32::from_le_bytes(i);
            let pb = crate::runtime::plugin_loader_v2::make_plugin_box_v2(
                box_type.clone(),
                r_type,
                r_inst,
                invoke_fn,
            );
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::new(pb);
            let h = crate::jit::rt::handles::to_handle(arc);
            events::emit_runtime(
                serde_json::json!({"id": "nyash.box.birth_i64", "box_type": box_type, "type_id": type_id, "argc": nargs, "handle": h}),
                "hostcall",
                "<jit>",
            );
            return h as i64;
        }
    }
    events::emit_runtime(
        serde_json::json!({"id": "nyash.box.birth_i64", "error": "decode_failed", "type_id": type_id}),
        "hostcall",
        "<jit>",
    );
    0
}

// ---- Handle helpers ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_handle_of(v: i64) -> i64 {
    // If already a positive handle, pass through
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        eprintln!("[JIT-HANDLE_OF] in v={}", v);
    }
    if v > 0 {
        return v;
    }
    // Otherwise interpret as legacy param index and convert BoxRef -> handle
    if v >= 0 {
        let idx = v as usize;
        let mut out: i64 = 0;
        crate::jit::rt::with_legacy_vm_args(|args| {
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                    std::sync::Arc::from(b.clone());
                out = crate::jit::rt::handles::to_handle(arc) as i64;
            }
        });
        if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
            eprintln!("[JIT-HANDLE_OF] param_idx={} out_handle={}", idx, out);
        }
        return out;
    }
    0
}

// ---- Math (native f64) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_sin_f64(x: f64) -> f64 {
    x.sin()
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_cos_f64(x: f64) -> f64 {
    x.cos()
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_abs_f64(x: f64) -> f64 {
    x.abs()
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_min_f64(a: f64, b: f64) -> f64 {
    a.min(b)
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_math_max_f64(a: f64, b: f64) -> f64 {
    a.max(b)
}

// ---- Console (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_console_birth_h() -> i64 {
    if let Ok(host_g) = crate::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("ConsoleBox", &[]) {
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = crate::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// ---- Runtime/GC stubs ----
// Minimal no-op checkpoints and barriers for reservation. They optionally trace when envs are set.
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_rt_checkpoint() -> i64 {
    if crate::config::env::runtime_checkpoint_trace() {
        eprintln!("[nyash.rt.checkpoint] reached");
    }
    // Bridge to GC/scheduler if configured
    crate::runtime::global_hooks::safepoint_and_poll();
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_gc_barrier_write(handle_or_ptr: u64) -> i64 {
    let _ = handle_or_ptr; // reserved; currently unused
    if crate::config::env::gc_barrier_trace() {
        eprintln!("[nyash.gc.barrier_write] h=0x{:x}", handle_or_ptr);
    }
    0
}

// ---- Array (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_len_h(handle: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_ARRAY_LEN_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Some(ib) = arr
                .length()
                .as_any()
                .downcast_ref::<crate::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_get_h(handle: u64, idx: i64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_ARRAY_GET_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let val = arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
            if let Some(ib) = val.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                return ib.value;
            }
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
                    if let Some(ib) = last.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                        return ib.value;
                    }
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
        events::emit_runtime(
            serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let _ = arr.set(
                Box::new(crate::box_trait::IntegerBox::new(idx)),
                Box::new(crate::box_trait::IntegerBox::new(val)),
            );
            events::emit_runtime(
                serde_json::json!({"id": sym, "decision":"allow", "argc":3, "arg_types":["Handle","I64","I64"]}),
                "hostcall",
                "<jit>",
            );
            return 0;
        }
    }
    0
}

// Array.set where value is a handle (StringBox, IntegerBox, etc.)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_array_set_hh(handle: u64, idx: i64, val_h: u64) -> i64 {
    use crate::jit::hostcall_registry::{classify, HostcallKind};
    let sym = c::SYM_ARRAY_SET_HH;
    let pol = crate::jit::policy::current();
    let wh = pol.hostcall_whitelist;
    if classify(sym) == HostcallKind::Mutating && pol.read_only && !wh.iter().any(|s| s == sym) {
        events::emit_runtime(
            serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            // Convert value handle to Box<dyn NyashBox>
            if let Some(v_arc) = crate::jit::rt::handles::get(val_h) {
                // Prefer share semantics for identity boxes
                let val_box: Box<dyn crate::box_trait::NyashBox> = v_arc.as_ref().clone_or_share();
                let _ = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), val_box);
                events::emit_runtime(
                    serde_json::json!({"id": sym, "decision":"allow", "argc":3, "arg_types":["Handle","I64","Handle"]}),
                    "hostcall",
                    "<jit>",
                );
                return 0;
            }
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
        events::emit_runtime(
            serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let ib = crate::box_trait::IntegerBox::new(val);
            let _ = arr.push(Box::new(ib));
            events::emit_runtime(
                serde_json::json!({"id": sym, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}),
                "hostcall",
                "<jit>",
            );
            return 0;
        }
    }
    0
}

// ---- Map (handle) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_size_h(handle: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_MAP_SIZE_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<crate::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_get_h(handle: u64, key: i64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_MAP_GET_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box = Box::new(crate::box_trait::IntegerBox::new(key));
            let val = map.get(key_box);
            if let Some(ib) = val.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                return ib.value;
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_get_hh(map_h: u64, key_h: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_MAP_GET_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
        "hostcall",
        "<jit>",
    );
    let map_arc = crate::jit::rt::handles::get(map_h);
    let key_arc = crate::jit::rt::handles::get(key_h);
    if let (Some(mobj), Some(kobj)) = (map_arc, key_arc) {
        if let Some(map) = mobj
            .as_any()
            .downcast_ref::<crate::boxes::map_box::MapBox>()
        {
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
        events::emit_runtime(
            serde_json::json!({"id": sym, "decision":"fallback", "reason":"policy_denied_mutating"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let key_box = Box::new(crate::box_trait::IntegerBox::new(key));
            let val_box = Box::new(crate::box_trait::IntegerBox::new(val));
            let _ = map.set(key_box, val_box);
            events::emit_runtime(
                serde_json::json!({"id": sym, "decision":"allow", "argc":3, "arg_types":["Handle","I64","I64"]}),
                "hostcall",
                "<jit>",
            );
            return 0;
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_map_has_h(handle: u64, key: i64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_MAP_HAS_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}),
        "hostcall",
        "<jit>",
    );
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
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_ANY_LEN_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Some(ib) = arr
                .length()
                .as_any()
                .downcast_ref::<crate::box_trait::IntegerBox>()
            {
                return ib.value;
            }
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
                        if let Some(arr) =
                            b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>()
                        {
                            if let Some(ib) = arr
                                .length()
                                .as_any()
                                .downcast_ref::<crate::box_trait::IntegerBox>()
                            {
                                return ib.value;
                            }
                        }
                        if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                            return sb.value.len() as i64;
                        }
                    }
                    crate::backend::vm::VMValue::String(s) => {
                        return s.len() as i64;
                    }
                    _ => {}
                }
            }
        }
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_any_is_empty_h(handle: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_ANY_IS_EMPTY_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}),
        "hostcall",
        "<jit>",
    );
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            if let Ok(items) = arr.items.read() {
                return if items.is_empty() { 1 } else { 0 };
            }
        }
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            return if sb.value.is_empty() { 1 } else { 0 };
        }
        if let Some(map) = obj.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<crate::box_trait::IntegerBox>()
            {
                return if ib.value == 0 { 1 } else { 0 };
            }
        }
    }
    0
}

// ---- By-name plugin invoke (generic receiver; resolves method_id at runtime) ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_plugin_invoke_name_getattr_i64(
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    nyash_plugin_invoke_name_common_i64("getattr", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_plugin_invoke_name_call_i64(
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    nyash_plugin_invoke_name_common_i64("call", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
fn nyash_plugin_invoke_name_common_i64(method: &str, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    // Resolve receiver
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                type_id = p.inner.type_id;
                box_type = Some(p.box_type.clone());
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1")
    {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0.max(0) as usize;
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    type_id = p.inner.type_id;
                    box_type = Some(p.box_type.clone());
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
                        instance_id = p.instance_id();
                        type_id = p.inner.type_id;
                        box_type = Some(p.box_type.clone());
                        invoke = Some(p.inner.invoke_fn);
                        break;
                    }
                }
            }
        });
    }
    if invoke.is_none() {
        events::emit_runtime(
            serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "error": "no_invoke"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    let box_type = box_type.unwrap_or_default();
    // Resolve method_id via PluginHost
    let mh = if let Ok(host) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
        host.resolve_method(&box_type, method)
    } else {
        events::emit_runtime(
            serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "host_read_failed"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    };
    let method_id = match mh {
        Ok(h) => h.method_id,
        Err(_) => {
            events::emit_runtime(
                serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "resolve_failed"}),
                "hostcall",
                "<jit>",
            );
            return 0;
        }
    } as u32;
    // Build TLV args from a1/a2 preferring handles; fallback to legacy (skip receiver=pos0)
    let mut buf =
        crate::runtime::plugin_ffi_common::encode_tlv_header(argc.saturating_sub(1).max(0) as u16);
    let mut encode_arg = |val: i64, pos: usize| {
        let mut appended = false;
        if val > 0 {
            if let Some(obj) = crate::jit::rt::handles::get(val as u64) {
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = crate::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method(
                                "StringBox",
                                "toUtf8",
                                p.instance_id(),
                                &[],
                            ) {
                                if let Some(s) =
                                    sb.as_any().downcast_ref::<crate::box_trait::StringBox>()
                                {
                                    crate::runtime::plugin_ffi_common::encode::string(
                                        &mut buf, &s.value,
                                    );
                                    appended = true;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) =
                                hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                            {
                                if let Some(i) =
                                    ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>()
                                {
                                    crate::runtime::plugin_ffi_common::encode::i64(
                                        &mut buf, i.value,
                                    );
                                    appended = true;
                                }
                            }
                        }
                    }
                    if !appended {
                        crate::runtime::plugin_ffi_common::encode::plugin_handle(
                            &mut buf,
                            p.inner.type_id,
                            p.instance_id(),
                        );
                        appended = true;
                    }
                } else {
                    // HostHandle for user/builtin boxes
                    let h = crate::runtime::host_handles::to_handle_arc(obj);
                    crate::runtime::plugin_ffi_common::encode::host_handle(&mut buf, h);
                    appended = true;
                }
            }
        }
        if !appended {
            // Fallback: encode from legacy VM args at position
            crate::jit::rt::with_legacy_vm_args(|args| {
                if let Some(v) = args.get(pos) {
                    use crate::backend::vm::VMValue as V;
                    match v {
                        V::String(s) => {
                            crate::runtime::plugin_ffi_common::encode::string(&mut buf, s)
                        }
                        V::Integer(i) => {
                            crate::runtime::plugin_ffi_common::encode::i64(&mut buf, *i)
                        }
                        V::Float(f) => crate::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                        V::Bool(b) => crate::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                        V::BoxRef(b) => {
                            if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                                let host = crate::runtime::get_global_plugin_host();
                                if let Ok(hg) = host.read() {
                                    if p.box_type == "StringBox" {
                                        if let Ok(Some(sb)) = hg.invoke_instance_method(
                                            "StringBox",
                                            "toUtf8",
                                            p.instance_id(),
                                            &[],
                                        ) {
                                            if let Some(s) = sb
                                                .as_any()
                                                .downcast_ref::<crate::box_trait::StringBox>()
                                            {
                                                crate::runtime::plugin_ffi_common::encode::string(
                                                    &mut buf, &s.value,
                                                );
                                                return;
                                            }
                                        }
                                    } else if p.box_type == "IntegerBox" {
                                        if let Ok(Some(ibx)) = hg.invoke_instance_method(
                                            "IntegerBox",
                                            "get",
                                            p.instance_id(),
                                            &[],
                                        ) {
                                            if let Some(i) =
                                                ibx.as_any()
                                                    .downcast_ref::<crate::box_trait::IntegerBox>()
                                            {
                                                crate::runtime::plugin_ffi_common::encode::i64(
                                                    &mut buf, i.value,
                                                );
                                                return;
                                            }
                                        }
                                    }
                                }
                                crate::runtime::plugin_ffi_common::encode::plugin_handle(
                                    &mut buf,
                                    p.inner.type_id,
                                    p.instance_id(),
                                );
                            } else {
                                // HostHandle fallback
                                let h = crate::runtime::host_handles::to_handle_arc(b.clone());
                                crate::runtime::plugin_ffi_common::encode::host_handle(&mut buf, h);
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
    if argc >= 2 {
        encode_arg(a1, 1);
    }
    if argc >= 3 {
        encode_arg(a2, 2);
    }
    let mut out = vec![0u8; 4096];
    let mut out_len: usize = out.len();
    let rc = unsafe {
        invoke.unwrap()(
            type_id as u32,
            method_id,
            instance_id,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if rc != 0 {
        events::emit_runtime(
            serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "invoke_failed"}),
            "hostcall",
            "<jit>",
        );
        return 0;
    }
    let out_slice = &out[..out_len];
    if let Some((tag, _sz, payload)) =
        crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice)
    {
        match tag {
            3 => {
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return i64::from_le_bytes(b);
                }
            }
            1 => {
                return if crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false)
                {
                    1
                } else {
                    0
                };
            }
            5 => {
                if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                    if payload.len() == 8 {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(payload);
                        let f = f64::from_le_bytes(b);
                        return f as i64;
                    }
                }
            }
            _ => {
                events::emit_runtime(
                    serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "warn": "first_tlv_not_primitive_or_handle", "tag": tag}),
                    "hostcall",
                    "<jit>",
                );
            }
        }
    }
    events::emit_runtime(
        serde_json::json!({"id": "plugin_invoke_by_name", "method": method, "box_type": box_type, "error": "decode_failed"}),
        "hostcall",
        "<jit>",
    );
    0
}

// ---- String ----
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_charcode_at_h(handle: u64, idx: i64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_STRING_CHARCODE_AT_H, "decision":"allow", "argc":2, "arg_types":["Handle","I64"]}),
        "hostcall",
        "<jit>",
    );
    if idx < 0 {
        return -1;
    }
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            let s = &sb.value;
            let i = idx as usize;
            if i < s.len() {
                return s.as_bytes()[i] as i64;
            } else {
                return -1;
            }
        }
    }
    -1
}

// String.len_h(handle) -> i64 with param-index fallback (JIT bridge)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_len_h(handle: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_STRING_LEN_H, "decision":"allow", "argc":1, "arg_types":["Handle"]}),
        "hostcall",
        "<jit>",
    );
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        eprintln!("[JIT-LEN_H] handle={}", handle);
    }
    if handle > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(handle) {
            if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                return sb.value.len() as i64;
            }
        }
        // Fallback to any.length_h for non-string handles
        let v = nyash_any_length_h(handle);
        if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
            eprintln!("[JIT-LEN_H] any.length_h(handle={}) -> {}", handle, v);
        }
        return v;
    }
    // Legacy param index fallback (0..16): disabled in jit-direct-only
    #[cfg(not(feature = "jit-direct-only"))]
    {
        if handle <= 16 {
            let idx = handle as usize;
            let val = crate::jit::rt::with_legacy_vm_args(|args| args.get(idx).cloned());
            if let Some(v) = val {
                match v {
                    crate::backend::vm::VMValue::BoxRef(b) => {
                        if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                            return sb.value.len() as i64;
                        }
                    }
                    crate::backend::vm::VMValue::String(s) => {
                        return s.len() as i64;
                    }
                    _ => {}
                }
            }
        }
    }
    0
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

// ---- String-like helpers and ops (handle, handle) ----
#[cfg(feature = "cranelift-jit")]
fn handle_to_string_like(handle: u64) -> Option<String> {
    // Prefer runtime handle registry
    if let Some(obj) = crate::jit::rt::handles::get(handle) {
        if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            return Some(sb.value.clone());
        }
        if let Some(pb) = obj.as_any().downcast_ref::<PluginBoxV2>() {
            if pb.box_type == "StringBox" {
                if let Ok(host) = crate::runtime::get_global_plugin_host().read() {
                    if let Ok(val_opt) =
                        host.invoke_instance_method("StringBox", "toUtf8", pb.instance_id(), &[])
                    {
                        if let Some(vb) = val_opt {
                            if let Some(sbb) =
                                vb.as_any().downcast_ref::<crate::box_trait::StringBox>()
                            {
                                return Some(sbb.value.clone());
                            }
                        }
                    }
                }
            }
        }
        // Fallback for any NyashBox
        return Some(obj.to_string_box().value);
    }
    // Legacy fallback: treat small values as VM arg index
    if handle <= 16 {
        let idx = handle as usize;
        let val = crate::jit::rt::with_legacy_vm_args(|args| args.get(idx).cloned());
        if let Some(v) = val {
            use crate::backend::vm::VMValue as V;
            return match v {
                V::String(s) => Some(s),
                V::BoxRef(b) => Some(b.to_string_box().value),
                V::Integer(i) => Some(i.to_string()),
                V::Float(f) => Some(f.to_string()),
                V::Bool(b) => Some(b.to_string()),
                _ => None,
            };
        }
    }
    None
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_concat_hh(a_h: u64, b_h: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_STRING_CONCAT_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
        "hostcall",
        "<jit>",
    );
    let a = handle_to_string_like(a_h).unwrap_or_default();
    let b = handle_to_string_like(b_h).unwrap_or_default();
    let s = format!("{}{}", a, b);
    let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
        std::sync::Arc::new(crate::box_trait::StringBox::new(s));
    let h = crate::jit::rt::handles::to_handle(arc);
    h as i64
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_eq_hh(a_h: u64, b_h: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_STRING_EQ_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
        "hostcall",
        "<jit>",
    );
    let a = handle_to_string_like(a_h).unwrap_or_default();
    let b = handle_to_string_like(b_h).unwrap_or_default();
    if a == b {
        1
    } else {
        0
    }
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_lt_hh(a_h: u64, b_h: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": c::SYM_STRING_LT_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
        "hostcall",
        "<jit>",
    );
    let a = handle_to_string_like(a_h).unwrap_or_default();
    let b = handle_to_string_like(b_h).unwrap_or_default();
    if a < b {
        1
    } else {
        0
    }
}

// Unified semantics: addition for dynamic boxes via shared coercions
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_semantics_add_hh(lhs_h: u64, rhs_h: u64) -> i64 {
    events::emit_runtime(
        serde_json::json!({"id": crate::jit::r#extern::collections::SYM_SEMANTICS_ADD_HH, "decision":"allow", "argc":2, "arg_types":["Handle","Handle"]}),
        "hostcall",
        "<jit>",
    );
    use crate::box_trait::{IntegerBox, StringBox};
    use crate::jit::rt::handles;
    use crate::runtime::semantics;
    let lhs = if let Some(o) = handles::get(lhs_h) {
        o
    } else {
        return 0;
    };
    let rhs = if let Some(o) = handles::get(rhs_h) {
        o
    } else {
        return 0;
    };
    let ls_opt = semantics::coerce_to_string(lhs.as_ref());
    let rs_opt = semantics::coerce_to_string(rhs.as_ref());
    if ls_opt.is_some() || rs_opt.is_some() {
        let ls = ls_opt.unwrap_or_else(|| lhs.to_string_box().value);
        let rs = rs_opt.unwrap_or_else(|| rhs.to_string_box().value);
        let s = format!("{}{}", ls, rs);
        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
            std::sync::Arc::new(StringBox::new(s));
        return handles::to_handle(arc) as i64;
    }
    if let (Some(li), Some(ri)) = (
        semantics::coerce_to_i64(lhs.as_ref()),
        semantics::coerce_to_i64(rhs.as_ref()),
    ) {
        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
            std::sync::Arc::new(IntegerBox::new(li + ri));
        return handles::to_handle(arc) as i64;
    }
    // Fallback stringify concat
    let s = format!("{}{}", lhs.to_string_box().value, rhs.to_string_box().value);
    let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
        std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// ==== Host-bridge (by-slot) external thunks ====
// These thunks adapt JIT runtime handles and primitive args into VMValue vectors
// and call the host-bridge helpers, which TLV-encode and invoke NyRT C-ABI by slot.

#[cfg(feature = "cranelift-jit")]
fn vmvalue_from_jit_arg_i64(v: i64) -> crate::backend::vm::VMValue {
    use crate::backend::vm::VMValue as V;
    if v <= 0 {
        return V::Integer(v);
    }
    if let Some(obj) = crate::jit::rt::handles::get(v as u64) {
        return V::BoxRef(obj);
    }
    // Legacy fallback: allow small indices to refer into legacy VM args for string/name lookups
    if (v as u64) <= 16 {
        return crate::jit::rt::with_legacy_vm_args(|args| args.get(v as usize).cloned())
            .unwrap_or(V::Integer(v));
    }
    V::Integer(v)
}

#[cfg(feature = "cranelift-jit")]
fn i64_from_vmvalue(v: crate::backend::vm::VMValue) -> i64 {
    use crate::backend::vm::VMValue as V;
    match v {
        V::Integer(i) => i,
        V::Bool(b) => {
            if b {
                1
            } else {
                0
            }
        }
        V::Float(f) => f as i64,
        V::String(s) => {
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                std::sync::Arc::new(crate::box_trait::StringBox::new(&s));
            crate::jit::rt::handles::to_handle(arc) as i64
        }
        V::BoxRef(b) => crate::jit::rt::handles::to_handle(b) as i64,
        V::Future(fu) => {
            let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::new(fu);
            crate::jit::rt::handles::to_handle(arc) as i64
        }
        V::Void => 0,
    }
}

// nyash.host.instance.getField(recv, name)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_instance_getfield(recv_h: u64, name_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let recv = match crate::jit::rt::handles::get(recv_h) {
        Some(a) => a,
        None => return 0,
    };
    let name_v = vmvalue_from_jit_arg_i64(name_i);
    if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") {
        eprintln!(
            "[HB|getField] name_i={} kind={}",
            name_i,
            match &name_v {
                V::String(_) => "String",
                V::BoxRef(b) =>
                    if b.as_any()
                        .downcast_ref::<crate::box_trait::StringBox>()
                        .is_some()
                    {
                        "StringBox"
                    } else {
                        "BoxRef"
                    },
                V::Integer(_) => "Integer",
                V::Bool(_) => "Bool",
                V::Float(_) => "Float",
                V::Void => "Void",
                V::Future(_) => "Future",
            }
        );
    }
    let out = hb::instance_getfield(&[V::BoxRef(recv), name_v]);
    i64_from_vmvalue(out)
}

// nyash.host.instance.setField(recv, name, value)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_instance_setfield(recv_h: u64, name_i: i64, val_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let recv = match crate::jit::rt::handles::get(recv_h) {
        Some(a) => a,
        None => return 0,
    };
    let name_v = vmvalue_from_jit_arg_i64(name_i);
    let val_v = vmvalue_from_jit_arg_i64(val_i);
    let out = hb::instance_setfield(&[V::BoxRef(recv), name_v, val_v]);
    i64_from_vmvalue(out)
}

// Unified instance field op: (recv, name, val_or_sentinel) → getField if val == -1, else setField
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_instance_field3(recv_h: u64, name_i: i64, val_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let recv = match crate::jit::rt::handles::get(recv_h) {
        Some(a) => a,
        None => return 0,
    };
    let name_v = vmvalue_from_jit_arg_i64(name_i);
    if val_i == -1 {
        // getField
        let out = hb::instance_getfield(&[V::BoxRef(recv), name_v]);
        return i64_from_vmvalue(out);
    }
    // setField
    let val_v = vmvalue_from_jit_arg_i64(val_i);
    let _ = hb::instance_setfield(&[V::BoxRef(recv), name_v, val_v]);
    0
}

// nyash.host.string.len(recv)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_string_len(recv_h: u64) -> i64 {
    use crate::backend::vm::VMValue as V;
    if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") {
        eprintln!("[HB|string.len] recv_h={}", recv_h);
    }
    let recv = match crate::jit::rt::handles::get(recv_h) {
        Some(a) => a,
        None => {
            if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") {
                eprintln!("[HB|string.len] recv handle not found");
            }
            return 0;
        }
    };
    let out = hb::string_len(&[V::BoxRef(recv)]);
    let ret = i64_from_vmvalue(out);
    if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") {
        eprintln!("[HB|string.len] ret_i64={}", ret);
    }
    ret
}

// nyash.host.console.log(value)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_console_log_i64(val_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let v = vmvalue_from_jit_arg_i64(val_i);
    let _ = crate::jit::r#extern::host_bridge::console_log(&[v]);
    0
}

// nyash.host.console.warn(value)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_console_warn_i64(val_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let v = vmvalue_from_jit_arg_i64(val_i);
    let _ = crate::jit::r#extern::host_bridge::console_warn(&[v]);
    0
}

// nyash.host.console.error(value)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_host_console_error_i64(val_i: i64) -> i64 {
    use crate::backend::vm::VMValue as V;
    let v = vmvalue_from_jit_arg_i64(val_i);
    let _ = crate::jit::r#extern::host_bridge::console_error(&[v]);
    0
}

// Build a StringBox handle from raw bytes pointer and length
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_from_ptr(ptr: u64, len: u64) -> i64 {
    if ptr == 0 || len == 0 {
        return 0;
    }
    unsafe {
        let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
        let s = match std::str::from_utf8(slice) {
            Ok(t) => t.to_string(),
            Err(_) => String::from_utf8_lossy(slice).to_string(),
        };
        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
            std::sync::Arc::new(crate::box_trait::StringBox::new(s));
        return crate::jit::rt::handles::to_handle(arc) as i64;
    }
}

// ===== FunctionBox call shims (by arity, up to 4) =====

#[cfg(feature = "cranelift-jit")]
fn fn_call_impl(func_h: u64, args: &[i64]) -> i64 {
    use crate::box_trait::NyashBox;
    let f_arc = match crate::jit::rt::handles::get(func_h) {
        Some(a) => a,
        None => return 0,
    };
    if let Some(fun) = f_arc
        .as_any()
        .downcast_ref::<crate::boxes::function_box::FunctionBox>()
    {
        let mut ny_args: Vec<Box<dyn NyashBox>> = Vec::new();
        for &ai in args {
            let v = vmvalue_from_jit_arg_i64(ai);
            ny_args.push(v.to_nyash_box());
        }
        match crate::interpreter::run_function_box(fun, ny_args) {
            Ok(out) => {
                let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                i64_from_vmvalue(vmv)
            }
            Err(_) => 0,
        }
    } else {
        0
    }
}

#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call0(func_h: u64) -> i64 {
    fn_call_impl(func_h, &[])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call1(func_h: u64, a0: i64) -> i64 {
    fn_call_impl(func_h, &[a0])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call2(func_h: u64, a0: i64, a1: i64) -> i64 {
    fn_call_impl(func_h, &[a0, a1])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call3(func_h: u64, a0: i64, a1: i64, a2: i64) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call4(func_h: u64, a0: i64, a1: i64, a2: i64, a3: i64) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2, a3])
}

// extended arities (5..8)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call5(
    func_h: u64,
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2, a3, a4])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call6(
    func_h: u64,
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: i64,
) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2, a3, a4, a5])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call7(
    func_h: u64,
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: i64,
    a6: i64,
) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2, a3, a4, a5, a6])
}
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_fn_call8(
    func_h: u64,
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: i64,
    a6: i64,
    a7: i64,
) -> i64 {
    fn_call_impl(func_h, &[a0, a1, a2, a3, a4, a5, a6, a7])
}

// Build a StringBox handle from two u64 chunks (little-endian) and length (<=16)
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_string_from_u64x2(lo: u64, hi: u64, len: i64) -> i64 {
    let n = if len <= 0 {
        0usize
    } else {
        core::cmp::min(len as usize, 16usize)
    };
    let mut buf = [0u8; 16];
    for i in 0..core::cmp::min(8, n) {
        buf[i] = ((lo >> (8 * i)) & 0xFF) as u8;
    }
    if n > 8 {
        for i in 0..(n - 8) {
            buf[8 + i] = ((hi >> (8 * i)) & 0xFF) as u8;
        }
    }
    let s = match std::str::from_utf8(&buf[..n]) {
        Ok(t) => t.to_string(),
        Err(_) => String::from_utf8_lossy(&buf[..n]).to_string(),
    };
    let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
        std::sync::Arc::new(crate::box_trait::StringBox::new(s));
    let h = crate::jit::rt::handles::to_handle(arc.clone()) as i64;
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        if let Some(sb) = arc.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            eprintln!(
                "[JIT-STR_H] new handle={} val='{}' len={}",
                h,
                sb.value,
                sb.value.len()
            );
        } else {
            eprintln!("[JIT-STR_H] new handle={} (non-StringBox)", h);
        }
    }
    h
}

// Create an instance by type name via global unified registry: birth(name) -> handle
#[cfg(feature = "cranelift-jit")]
pub(super) extern "C" fn nyash_instance_birth_name_u64x2(lo: u64, hi: u64, len: i64) -> i64 {
    let n = if len <= 0 {
        0usize
    } else {
        core::cmp::min(len as usize, 32usize)
    };
    let mut buf = [0u8; 32];
    for i in 0..core::cmp::min(8, n) {
        buf[i] = ((lo >> (8 * i)) & 0xFF) as u8;
    }
    if n > 8 {
        for i in 0..core::cmp::min(8, n - 8) {
            buf[8 + i] = ((hi >> (8 * i)) & 0xFF) as u8;
        }
    }
    let name = match std::str::from_utf8(&buf[..n]) {
        Ok(t) => t.to_string(),
        Err(_) => String::from_utf8_lossy(&buf[..n]).to_string(),
    };
    let registry = crate::runtime::get_global_unified_registry();
    if let Ok(reg) = registry.lock() {
        match reg.create_box(&name, &[]) {
            Ok(b) => {
                let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::from(b);
                return crate::jit::rt::handles::to_handle(arc) as i64;
            }
            Err(_) => return 0,
        }
    }
    0
}
