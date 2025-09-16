// Spawn a plugin instance method asynchronously and return a Future handle (i64)
// Exported as: nyash.future.spawn_method_h(type_id, method_id, argc, recv_h, vals*, tags*) -> i64 (FutureBox handle)
#[export_name = "nyash.future.spawn_method_h"]
pub extern "C" fn nyash_future_spawn_method_h(
    type_id: i64,
    method_id: i64,
    argc: i64,
    recv_h: i64,
    vals: *const i64,
    tags: *const i64,
) -> i64 {
    use nyash_rust::box_trait::{IntegerBox, NyashBox, StringBox};
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    if recv_h <= 0 {
        return 0;
    }
    // Resolve receiver invoke
    let mut instance_id: u32 = 0;
    let mut real_type_id: u32 = type_id as u32;
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if let Some(obj) = nyash_rust::jit::rt::handles::get(recv_h as u64) {
        if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
            instance_id = p.instance_id();
            real_type_id = p.inner.type_id;
            invoke = Some(p.inner.invoke_fn);
        }
    }
    if invoke.is_none() {
        return 0;
    }
    // Build TLV from tagged arrays (argc includes receiver)
    let nargs = argc.saturating_sub(1).max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let vals_slice = if !vals.is_null() && nargs > 0 {
        unsafe { std::slice::from_raw_parts(vals, nargs) }
    } else {
        &[]
    };
    let tags_slice = if !tags.is_null() && nargs > 0 {
        unsafe { std::slice::from_raw_parts(tags, nargs) }
    } else {
        &[]
    };
    for i in 0..nargs {
        let v = vals_slice.get(i).copied().unwrap_or(0);
        let t = tags_slice.get(i).copied().unwrap_or(3); // default i64
        match t {
            3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, v),
            5 => {
                let bits = v as u64;
                let f = f64::from_bits(bits);
                nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, f);
            }
            8 => {
                if v > 0 {
                    if let Some(obj) = nyash_rust::jit::rt::handles::get(v as u64) {
                        if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                            // Try common coercions: String/Integer to TLV primitives
                            let host = nyash_rust::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                                if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method(
                                        "StringBox",
                                        "toUtf8",
                                        p.instance_id(),
                                        &[],
                                    ) {
                                        if let Some(s) = sb.as_any().downcast_ref::<StringBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(
                                                &mut buf, &s.value,
                                            );
                                            continue;
                                        }
                                    }
                                } else if p.box_type == "IntegerBox" {
                                    if let Ok(Some(ibx)) = hg.invoke_instance_method(
                                        "IntegerBox",
                                        "get",
                                        p.instance_id(),
                                        &[],
                                    ) {
                                        if let Some(i) = ibx.as_any().downcast_ref::<IntegerBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                                &mut buf, i.value,
                                            );
                                            continue;
                                        }
                                    }
                                }
                            }
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                                &mut buf,
                                p.inner.type_id,
                                p.instance_id(),
                            );
                        } else {
                            let s = obj.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s);
                        }
                    } else {
                        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, v);
                    }
                } else {
                    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, 0);
                }
            }
            _ => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, v),
        }
    }
    // Prepare FutureBox and register handle
    let fut_box = std::sync::Arc::new(nyash_rust::boxes::future::FutureBox::new());
    let handle =
        nyash_rust::jit::rt::handles::to_handle(fut_box.clone() as std::sync::Arc<dyn NyashBox>);
    // Copy data for async task
    let mut cap: usize = 512;
    let tlv = buf.clone();
    let inv = invoke.unwrap();
    nyash_rust::runtime::global_hooks::spawn_task(
        "nyash.future.spawn_method_h",
        Box::new(move || {
            // Growable output buffer loop
            let mut out = vec![0u8; cap];
            let mut out_len: usize = out.len();
            let rc = unsafe {
                inv(
                    real_type_id,
                    method_id as u32,
                    instance_id,
                    tlv.as_ptr(),
                    tlv.len(),
                    out.as_mut_ptr(),
                    &mut out_len,
                )
            };
            if rc != 0 {
                // Set simple error string on failure
                fut_box.set_result(Box::new(StringBox::new(format!("invoke_failed rc={}", rc))));
                return;
            }
            let slice = &out[..out_len];
            if let Some((tag, sz, payload)) =
                nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice)
            {
                match tag {
                    3 => {
                        // I64
                        if payload.len() == 8 {
                            let mut b = [0u8; 8];
                            b.copy_from_slice(payload);
                            let n = i64::from_le_bytes(b);
                            fut_box.set_result(Box::new(IntegerBox::new(n)));
                            return;
                        }
                    }
                    2 => {
                        if let Some(v) =
                            nyash_rust::runtime::plugin_ffi_common::decode::i32(payload)
                        {
                            fut_box.set_result(Box::new(IntegerBox::new(v as i64)));
                            return;
                        }
                    }
                    1 => {
                        // Bool
                        let v = nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                            .unwrap_or(false);
                        fut_box.set_result(Box::new(nyash_rust::box_trait::BoolBox::new(v)));
                        return;
                    }
                    5 => {
                        // F64
                        if payload.len() == 8 {
                            let mut b = [0u8; 8];
                            b.copy_from_slice(payload);
                            let f = f64::from_le_bytes(b);
                            fut_box.set_result(Box::new(
                                nyash_rust::boxes::math_box::FloatBox::new(f),
                            ));
                            return;
                        }
                    }
                    6 | 7 => {
                        // String/Bytes as string
                        let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                        fut_box.set_result(Box::new(StringBox::new(s)));
                        return;
                    }
                    8 => {
                        // Handle -> PluginBoxV2 boxed
                        if sz == 8 {
                            let mut t = [0u8; 4];
                            t.copy_from_slice(&payload[0..4]);
                            let mut i = [0u8; 4];
                            i.copy_from_slice(&payload[4..8]);
                            let r_type = u32::from_le_bytes(t);
                            let r_inst = u32::from_le_bytes(i);
                            // Map type_id -> box type name (best-effort)
                            let meta_opt =
                                nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(r_type);
                            let (box_type_name, invoke_ptr, fini_id) = if let Some(meta) = meta_opt
                            {
                                (meta.box_type.clone(), meta.invoke_fn, meta.fini_method_id)
                            } else {
                                ("PluginBox".to_string(), inv, None)
                            };
                            let pb = nyash_rust::runtime::plugin_loader_v2::construct_plugin_box(
                                box_type_name,
                                r_type,
                                invoke_ptr,
                                r_inst,
                                fini_id,
                            );
                            fut_box.set_result(Box::new(pb));
                            return;
                        }
                    }
                    9 => {
                        // Void
                        fut_box.set_result(Box::new(nyash_rust::box_trait::VoidBox::new()));
                        return;
                    }
                    _ => {}
                }
            }
            // Fallback: store raw buffer as string preview
            fut_box.set_result(Box::new(StringBox::new("<unknown>")));
        }),
    );
    handle as i64
}

// Simpler spawn shim for JIT: pass argc(total explicit args incl. method_name),
// receiver handle (a0), method name (a1), and first payload (a2). Extra args
// are read from legacy VM args, same as plugin_invoke3_*.
// Returns a handle (i64) to FutureBox.
#[export_name = "nyash.future.spawn_instance3_i64"]
pub extern "C" fn nyash_future_spawn_instance3_i64(a0: i64, a1: i64, a2: i64, argc: i64) -> i64 {
    use nyash_rust::box_trait::{IntegerBox, NyashBox, StringBox};
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    if a0 <= 0 {
        return 0;
    }
    // Resolve receiver invoke and type id/name
    let (instance_id, real_type_id, invoke) =
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                (p.instance_id(), p.inner.type_id, Some(p.inner.invoke_fn))
            } else {
                (0, 0, None)
            }
        } else {
            (0, 0, None)
        };
    if invoke.is_none() {
        return 0;
    }
    let invoke = invoke.unwrap();
    // Determine box type name from type_id
    let box_type_name = nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(real_type_id)
        .map(|meta| meta.box_type)
        .unwrap_or_else(|| "PluginBox".to_string());
    // Determine method name string (from a1 handle→StringBox, or a1 as C string pointer, or legacy VM args)
    let mut method_name: Option<String> = None;
    if a1 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a1 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                if p.box_type == "StringBox" {
                    // Limit the lifetime of the read guard to this inner block by avoiding an outer binding
                    if let Ok(hg) = nyash_rust::runtime::get_global_plugin_host().read() {
                        if let Ok(Some(sb)) =
                            hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[])
                        {
                            if let Some(s) = sb.as_any().downcast_ref::<StringBox>() {
                                method_name = Some(s.value.clone());
                            }
                        }
                    }
                }
            }
        }
        // If not a handle, try to decode as C string pointer (LLVM path)
        if method_name.is_none() {
            let cptr = a1 as *const i8;
            if !cptr.is_null() {
                unsafe {
                    if let Ok(cs) = std::ffi::CStr::from_ptr(cptr).to_str() {
                        method_name = Some(cs.to_string());
                    }
                }
            }
        }
    }
    if method_name.is_none() {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            // method name is explicit arg position 1 (after receiver)
            if let Some(nyash_rust::backend::vm::VMValue::String(s)) = args.get(1) {
                method_name = Some(s.clone());
            }
        });
    }
    let method_name = match method_name {
        Some(s) => s,
        None => return 0,
    };
    // Resolve method_id via PluginHost
    let mh_opt = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read()
        .ok()
        .and_then(|h| h.resolve_method(&box_type_name, &method_name).ok());
    let method_id = if let Some(mh) = mh_opt {
        mh.method_id
    } else {
        0
    };
    if method_id == 0 { /* dynamic plugins may use 0 for birth; disallow here */ }
    // Build TLV args for payload (excluding method name)
    let nargs_total = argc.max(0) as usize; // includes method_name
    let nargs_payload = nargs_total.saturating_sub(1);
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs_payload as u16);
    let mut encode_from_legacy_into = |dst: &mut Vec<u8>, pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(pos) {
                use nyash_rust::backend::vm::VMValue;
                match v {
                    VMValue::String(s) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::string(dst, s)
                    }
                    VMValue::Integer(i) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, *i)
                    }
                    VMValue::Float(f) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::f64(dst, *f)
                    }
                    VMValue::Bool(b) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::bool(dst, *b)
                    }
                    VMValue::BoxRef(b) => {
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            let host = nyash_rust::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                                if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method(
                                        "StringBox",
                                        "toUtf8",
                                        p.instance_id(),
                                        &[],
                                    ) {
                                        if let Some(s) = sb.as_any().downcast_ref::<StringBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(
                                                dst, &s.value,
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
                                        if let Some(i) = ibx.as_any().downcast_ref::<IntegerBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                                dst, i.value,
                                            );
                                            return;
                                        }
                                    }
                                }
                            }
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                                dst,
                                p.inner.type_id,
                                p.instance_id(),
                            );
                            return;
                        }
                        // Fallback: stringify
                        let s = b.to_string_box().value;
                        nyash_rust::runtime::plugin_ffi_common::encode::string(dst, &s);
                    }
                    _ => {}
                }
            }
        });
    };
    let mut encode_arg_into = |dst: &mut Vec<u8>, val: i64, pos: usize| {
        let mut appended = false;
        if val > 0 {
            if let Some(obj) = nyash_rust::jit::rt::handles::get(val as u64) {
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = nyash_rust::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method(
                                "StringBox",
                                "toUtf8",
                                p.instance_id(),
                                &[],
                            ) {
                                if let Some(s) = sb.as_any().downcast_ref::<StringBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::string(
                                        dst, &s.value,
                                    );
                                    appended = true;
                                    return;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) =
                                hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                            {
                                if let Some(i) = ibx.as_any().downcast_ref::<IntegerBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                        dst, i.value,
                                    );
                                    appended = true;
                                    return;
                                }
                            }
                        }
                    }
                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                        dst,
                        p.inner.type_id,
                        p.instance_id(),
                    );
                    appended = true;
                    return;
                }
            }
        }
        let before = dst.len();
        encode_from_legacy_into(dst, pos);
        if dst.len() != before {
            appended = true;
        }
        if !appended {
            nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, val);
        }
    };
    // a1 is method name; payload starts at position 2
    if nargs_payload >= 1 {
        encode_arg_into(&mut buf, a2, 2);
    }
    if nargs_payload > 1 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1")
    {
        for pos in 3..=nargs_payload {
            encode_from_legacy_into(&mut buf, pos);
        }
    }
    // Create Future and schedule async invoke
    let fut_box = std::sync::Arc::new(nyash_rust::boxes::future::FutureBox::new());
    let handle =
        nyash_rust::jit::rt::handles::to_handle(fut_box.clone() as std::sync::Arc<dyn NyashBox>);
    let tlv = buf.clone();
    nyash_rust::runtime::global_hooks::spawn_task(
        "nyash.future.spawn_instance3_i64",
        Box::new(move || {
            // Dynamic output buffer with growth
            let mut cap: usize = 512;
            loop {
                let mut out = vec![0u8; cap];
                let mut out_len: usize = out.len();
                let rc = unsafe {
                    invoke(
                        real_type_id,
                        method_id as u32,
                        instance_id,
                        tlv.as_ptr(),
                        tlv.len(),
                        out.as_mut_ptr(),
                        &mut out_len,
                    )
                };
                if rc == -1 || out_len > cap {
                    cap = cap.saturating_mul(2).max(out_len + 16);
                    if cap > 1 << 20 {
                        break;
                    }
                    continue;
                }
                if rc != 0 {
                    fut_box
                        .set_result(Box::new(StringBox::new(format!("invoke_failed rc={}", rc))));
                    return;
                }
                let slice = &out[..out_len];
                if let Some((tag, sz, payload)) =
                    nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice)
                {
                    match tag {
                        3 => {
                            if payload.len() == 8 {
                                let mut b = [0u8; 8];
                                b.copy_from_slice(payload);
                                let n = i64::from_le_bytes(b);
                                fut_box.set_result(Box::new(IntegerBox::new(n)));
                                return;
                            }
                        }
                        1 => {
                            let v = nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                                .unwrap_or(false);
                            fut_box.set_result(Box::new(nyash_rust::box_trait::BoolBox::new(v)));
                            return;
                        }
                        5 => {
                            if payload.len() == 8 {
                                let mut b = [0u8; 8];
                                b.copy_from_slice(payload);
                                let f = f64::from_le_bytes(b);
                                fut_box.set_result(Box::new(
                                    nyash_rust::boxes::math_box::FloatBox::new(f),
                                ));
                                return;
                            }
                        }
                        6 | 7 => {
                            let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                            fut_box.set_result(Box::new(StringBox::new(s)));
                            return;
                        }
                        8 => {
                            if sz == 8 {
                                let mut t = [0u8; 4];
                                t.copy_from_slice(&payload[0..4]);
                                let mut i = [0u8; 4];
                                i.copy_from_slice(&payload[4..8]);
                                let r_type = u32::from_le_bytes(t);
                                let r_inst = u32::from_le_bytes(i);
                                let pb =
                                    nyash_rust::runtime::plugin_loader_v2::construct_plugin_box(
                                        box_type_name.clone(),
                                        r_type,
                                        invoke,
                                        r_inst,
                                        None,
                                    );
                                fut_box.set_result(Box::new(pb));
                                return;
                            }
                        }
                        9 => {
                            fut_box.set_result(Box::new(nyash_rust::box_trait::VoidBox::new()));
                            return;
                        }
                        _ => {}
                    }
                }
                fut_box.set_result(Box::new(StringBox::new("<unknown>")));
                return;
            }
        }),
    );
    handle as i64
}
