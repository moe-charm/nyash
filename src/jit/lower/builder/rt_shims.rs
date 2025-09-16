#![cfg(feature = "cranelift-jit")]

// Runtime shims and helpers used by the Cranelift JIT backend

pub(crate) extern "C" fn nyash_host_stub0() -> i64 {
    0
}

pub(crate) extern "C" fn nyash_jit_dbg_i64(tag: i64, val: i64) -> i64 {
    eprintln!("[JIT-DBG] tag={} val={}", tag, val);
    val
}

pub(crate) extern "C" fn nyash_jit_block_enter(idx: i64) {
    eprintln!("[JIT-BLOCK] enter={}", idx);
}

pub(crate) extern "C" fn nyash_plugin_invoke3_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    let trace = crate::jit::observe::trace_enabled();
    crate::jit::events::emit_runtime(
        serde_json::json!({ "id": "shim.enter.i64", "type_id": type_id, "method_id": method_id, "argc": argc }),
        "shim",
        "<jit>",
    );
    let mut instance_id: u32 = 0;
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                invoke = Some(p.inner.invoke_fn);
            } else if method_id as u32 == 1 {
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
            }
        }
    }
    let mut native_array_len: Option<i64> = None;
    #[cfg(not(feature = "jit-direct-only"))]
    {
        if a0 >= 0 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
            crate::jit::rt::with_legacy_vm_args(|args| {
                let idx = a0 as usize;
                if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        instance_id = p.instance_id();
                        invoke = Some(p.inner.invoke_fn);
                    } else if let Some(arr) =
                        b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>()
                    {
                        if method_id as u32 == 1 {
                            if let Some(ib) = arr
                                .length()
                                .as_any()
                                .downcast_ref::<crate::box_trait::IntegerBox>()
                            {
                                native_array_len = Some(ib.value);
                            }
                        }
                    }
                }
            });
        }
    }
    if invoke.is_none() {
        if let Some(v) = native_array_len {
            if trace {
                eprintln!("[JIT-SHIM i64] native_fallback return {}", v);
            }
            crate::jit::events::emit_runtime(
                serde_json::json!({ "id": "shim.native.i64", "type_id": type_id, "method_id": method_id, "argc": argc, "ret": v }),
                "shim",
                "<jit>",
            );
            return v;
        }
    }
    #[cfg(not(feature = "jit-direct-only"))]
    if invoke.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let crate::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        instance_id = p.instance_id();
                        invoke = Some(p.inner.invoke_fn);
                        break;
                    }
                }
            }
        });
    }
    if invoke.is_none() {
        return 0;
    }
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(
        (argc.saturating_sub(1).max(0) as u16),
    );
    let mut add_i64 = |v: i64| {
        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, v);
    };
    if argc >= 2 {
        add_i64(a1);
    }
    if argc >= 3 {
        add_i64(a2);
    }
    let mut out = vec![0xCDu8; 4096 + 32];
    let canary_val = 0xABu8;
    let canary_len = 16usize;
    for i in 0..canary_len {
        out[i] = canary_val;
    }
    for i in 0..canary_len {
        out[4096 + canary_len + i] = canary_val;
    }
    let mut out_len: usize = 0;
    let ok = unsafe {
        (invoke.unwrap())(
            instance_id,
            type_id as u32,
            method_id as u32,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr().add(canary_len),
            &mut out_len as *mut usize,
        )
    };
    if ok != 0 {
        let out_slice = &out[canary_len..(canary_len + out_len.min(4096))];
        if let Some((tag, _sz, payload)) =
            crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice)
        {
            match tag {
                3 => {
                    if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) {
                        return v as i64;
                    }
                    if payload.len() == 8 {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(payload);
                        return i64::from_le_bytes(b);
                    }
                }
                8 => {
                    if _sz == 8 {
                        let mut t = [0u8; 4];
                        t.copy_from_slice(&payload[0..4]);
                        let mut i = [0u8; 4];
                        i.copy_from_slice(&payload[4..8]);
                        let r_type = u32::from_le_bytes(t);
                        let r_inst = u32::from_le_bytes(i);
                        let meta_opt =
                            crate::runtime::plugin_loader_v2::metadata_for_type_id(r_type);
                        let (box_type_name, invoke_ptr) = if let Some(meta) = meta_opt {
                            (meta.box_type.clone(), meta.invoke_fn)
                        } else {
                            ("PluginBox".to_string(), invoke.unwrap())
                        };
                        let pb = crate::runtime::plugin_loader_v2::make_plugin_box_v2(
                            box_type_name,
                            r_type,
                            r_inst,
                            invoke_ptr,
                        );
                        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                            std::sync::Arc::new(pb);
                        let h = crate::jit::rt::handles::to_handle(arc);
                        return h as i64;
                    }
                }
                1 => {
                    return if crate::runtime::plugin_ffi_common::decode::bool(payload)
                        .unwrap_or(false)
                    {
                        1
                    } else {
                        0
                    };
                }
                5 => {
                    if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                        if _sz == 8 {
                            let mut b = [0u8; 8];
                            b.copy_from_slice(payload);
                            let f = f64::from_le_bytes(b);
                            return f as i64;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    0
}

pub(crate) extern "C" fn nyash_plugin_invoke3_f64(
    _type_id: i64,
    _method_id: i64,
    _argc: i64,
    _a0: i64,
    _a1: i64,
    _a2: i64,
) -> f64 {
    0.0
}

// === By-name plugin shims (i64) ===
pub(crate) extern "C" fn nyash_plugin_invoke_name_getattr_i64(
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    nyash_plugin_invoke_name_common_i64("getattr", argc, a0, a1, a2)
}
pub(crate) extern "C" fn nyash_plugin_invoke_name_call_i64(
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    nyash_plugin_invoke_name_common_i64("call", argc, a0, a1, a2)
}

fn nyash_plugin_invoke_name_common_i64(method: &str, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
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
        return 0;
    }
    let box_type = box_type.unwrap_or_default();
    let mh =
        if let Ok(host) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
            host.resolve_method(&box_type, method)
        } else {
            return 0;
        };
    let method_id = match mh {
        Ok(h) => h.method_id,
        Err(_) => return 0,
    } as u32;
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header(
        (argc.saturating_sub(1).max(0) as u16),
    );
    let mut add_from_legacy = |pos: usize| {
        crate::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(pos) {
                match v {
                    crate::backend::vm::VMValue::Integer(i) => {
                        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, *i)
                    }
                    crate::backend::vm::VMValue::Float(f) => {
                        crate::runtime::plugin_ffi_common::encode::f64(&mut buf, *f)
                    }
                    crate::backend::vm::VMValue::Bool(b) => {
                        crate::runtime::plugin_ffi_common::encode::bool(&mut buf, *b)
                    }
                    crate::backend::vm::VMValue::BoxRef(_) => {
                        crate::runtime::plugin_ffi_common::encode::i64(&mut buf, 0);
                    }
                    _ => {}
                }
            }
        });
    };
    if argc >= 2 {
        add_from_legacy(1);
    }
    if argc >= 3 {
        add_from_legacy(2);
    }
    let mut out = vec![0u8; 4096];
    let mut out_len: usize = 0;
    let ok = unsafe {
        (invoke.unwrap())(
            instance_id,
            type_id as u32,
            method_id as u32,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr(),
            &mut out_len as *mut usize,
        )
    };
    if ok != 0 {
        let out_slice = &out[0..out_len.min(4096)];
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
                    if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) {
                        return v as i64;
                    }
                }
                8 => {
                    if _sz == 8 {
                        let mut t = [0u8; 4];
                        t.copy_from_slice(&payload[0..4]);
                        let mut i = [0u8; 4];
                        i.copy_from_slice(&payload[4..8]);
                        let r_type = u32::from_le_bytes(t);
                        let r_inst = u32::from_le_bytes(i);
                        let meta_opt =
                            crate::runtime::plugin_loader_v2::metadata_for_type_id(r_type);
                        let (meta_box, invoke_ptr) = if let Some(meta) = meta_opt {
                            (meta.box_type, meta.invoke_fn)
                        } else {
                            (box_type.clone(), invoke.unwrap())
                        };
                        let pb = crate::runtime::plugin_loader_v2::make_plugin_box_v2(
                            meta_box, r_type, r_inst, invoke_ptr,
                        );
                        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                            std::sync::Arc::new(pb);
                        let h = crate::jit::rt::handles::to_handle(arc);
                        return h as i64;
                    }
                }
                1 => {
                    return if crate::runtime::plugin_ffi_common::decode::bool(payload)
                        .unwrap_or(false)
                    {
                        1
                    } else {
                        0
                    };
                }
                5 => {
                    if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                        if _sz == 8 {
                            let mut b = [0u8; 8];
                            b.copy_from_slice(payload);
                            let f = f64::from_le_bytes(b);
                            return f as i64;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    0
}
