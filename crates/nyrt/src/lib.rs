// Minimal NyRT static shim library (libnyrt.a)
// Exposes C ABI entry points used by AOT/JIT-emitted objects.

#[no_mangle]
pub extern "C" fn nyash_plugin_invoke3_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver instance from handle first; fallback to legacy VM args (param index)
    let mut instance_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() && a0 >= 0 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0 as usize;
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() { return 0; }
    // Build TLV args from a1/a2 if present. Prefer handles/StringBox/IntegerBox via runtime host.
    use nyash_rust::{jit::rt::handles, backend::vm::VMValue};
    let nargs = argc.saturating_sub(1).max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut encode_from_legacy = |arg_pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(arg_pos) {
                match v {
                    VMValue::String(s) => nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                    VMValue::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                    VMValue::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                    VMValue::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                    VMValue::BoxRef(b) => {
                        // BufferBox → TLV bytes
                        if let Some(bufbox) = b.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec());
                            return;
                        }
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            // Prefer StringBox/IntegerBox primitives when possible
                            let host = nyash_rust::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                                if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                        if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value);
                                            return;
                                        }
                                    }
                                } else if p.box_type == "IntegerBox" {
                                    if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value);
                                            return;
                                        }
                                    }
                                }
                            }
                            // Fallback: pass handle as plugin-handle TLV
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                        } else {
                            // Stringify unknown boxes
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    let mut encode_arg = |val: i64, pos: usize| {
        let mut appended = false;
        // Try handle first
        if val > 0 {
            if let Some(obj) = handles::get(val as u64) {
                // BufferBox handle → TLV bytes
                if let Some(bufbox) = obj.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec());
                    appended = true; return;
                }
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = nyash_rust::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); appended = true;
                                    return;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); appended = true;
                                    return;
                                }
                            }
                        }
                    }
                    // Otherwise, pass as handle TLV
                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id()); appended = true;
                    return;
                }
            }
        }
        // Legacy VM args by positional index (1-based for a1)
        let before = buf.len();
        encode_from_legacy(pos);
        if buf.len() != before { appended = true; }
        // If still nothing appended (no-op), fallback to raw i64
        if !appended { nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val); }
    };
    if nargs >= 1 { encode_arg(a1, 1); }
    if nargs >= 2 { encode_arg(a2, 2); }
    // Extra args from legacy VM args (positions 3..nargs)
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs { encode_from_legacy(pos); }
    }
    // Prepare output buffer (dynamic growth on short buffer)
    let mut cap: usize = 256;
    let (mut tag_ret, mut sz_ret, mut payload_ret): (u8, usize, Vec<u8>) = (0, 0, Vec::new());
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = unsafe { invoke.unwrap()(type_id as u32, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
        if rc != 0 {
            // Retry on short buffer hint (-1) or when plugin wrote beyond capacity (len > cap)
            if rc == -1 || out_len > cap { cap = cap.saturating_mul(2).max(out_len + 16); if cap > 1<<20 { break; } continue; }
            return 0;
        }
        let slice = &out[..out_len];
        if let Some((t, s, p)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice) {
            tag_ret = t; sz_ret = s; payload_ret = p.to_vec();
        }
        break;
    }
    if payload_ret.is_empty() { return 0; }
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        match tag {
            3 => { // I64
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) { return v as i64; }
                if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); }
            }
            8 => { // Handle(tag=8) -> register and return handle id (i64)
                if sz == 8 {
                    let mut t = [0u8;4]; t.copy_from_slice(&payload[0..4]);
                    let mut i = [0u8;4]; i.copy_from_slice(&payload[4..8]);
                    let r_type = u32::from_le_bytes(t);
                    let r_inst = u32::from_le_bytes(i);
                    // Build PluginBoxV2 and register into handle-registry
                    let box_type_name = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
                        .read()
                        .ok()
                        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
                        .and_then(|m| m.into_iter().find(|(_k,v)| *v == r_type).map(|(k,_v)| k))
                        .unwrap_or_else(|| "PluginBox".to_string());
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(box_type_name, r_type, r_inst, invoke.unwrap());
                    let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::new(pb);
                    let h = nyash_rust::jit::rt::handles::to_handle(arc);
                    return h as i64;
                }
            }
            1 => { // Bool
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 };
            }
            5 => { // F64 → optional conversion to i64
                if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                    if sz == 8 {
                        let mut b=[0u8;8]; b.copy_from_slice(payload);
                        let f = f64::from_le_bytes(b);
                        return f as i64;
                    }
                }
            }
            _ => {}
        }
    }
    0
}

// F64-typed shim: decode TLV first entry and return f64 when possible
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke3_f64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> f64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver from legacy VM args or handle registry
    let mut instance_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if a0 >= 0 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0 as usize;
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() {
        // Fallback scan for any PluginBoxV2 in args to pick invoke_fn
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let nyash_rust::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        if p.inner.type_id == (type_id as u32) || invoke.is_none() {
                            instance_id = p.instance_id();
                            invoke = Some(p.inner.invoke_fn);
                            if p.inner.type_id == (type_id as u32) { break; }
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() { return 0.0; }
    // Build TLV args from a1/a2 with String/Integer support
    use nyash_rust::{jit::rt::handles, backend::vm::VMValue};
    let nargs = argc.saturating_sub(1).max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut encode_from_legacy = |arg_pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(arg_pos) {
                match v {
                    VMValue::String(s) => nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                    VMValue::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                    VMValue::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                    VMValue::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                    VMValue::BoxRef(b) => {
                        if let Some(bufbox) = b.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec());
                            return;
                        }
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            let host = nyash_rust::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                                if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                        if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value);
                                            return;
                                        }
                                    }
                                } else if p.box_type == "IntegerBox" {
                                    if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value);
                                            return;
                                        }
                                    }
                                }
                            }
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                        } else {
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    let mut encode_arg = |val: i64, pos: usize| {
        let mut appended = false;
        if val > 0 {
            if let Some(obj) = handles::get(val as u64) {
                if let Some(bufbox) = obj.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec());
                    appended = true; return;
                }
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = nyash_rust::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); appended = true;
                                    return;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); appended = true;
                                    return;
                                }
                            }
                        }
                    }
                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id()); appended = true;
                    return;
                }
            }
        }
        let before = buf.len();
        encode_from_legacy(pos);
        if buf.len() != before { appended = true; }
        if !appended { nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val); }
    };
    if nargs >= 1 { encode_arg(a1, 1); }
    if nargs >= 2 { encode_arg(a2, 2); }
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs { encode_from_legacy(pos); }
    }
    // Prepare output buffer (dynamic growth on short buffer)
    let mut cap: usize = 256;
    let (mut tag_ret, mut sz_ret, mut payload_ret): (u8, usize, Vec<u8>) = (0, 0, Vec::new());
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = unsafe { invoke.unwrap()(type_id as u32, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
        if rc != 0 {
            // Retry on short buffer (-1) or when plugin wrote beyond capacity
            if rc == -1 || out_len > cap { cap = cap.saturating_mul(2).max(out_len + 16); if cap > 1<<20 { break; } continue; }
            return 0.0;
        }
        let slice = &out[..out_len];
        if let Some((t, s, p)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice) {
            tag_ret = t; sz_ret = s; payload_ret = p.to_vec();
        }
        break;
    }
    if payload_ret.is_empty() { return 0.0; }
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        match tag {
            5 => { // F64
                if sz == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return f64::from_le_bytes(b); }
            }
            3 => { // I64 -> f64
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) { return v as f64; }
                if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return (i64::from_le_bytes(b)) as f64; }
            }
            1 => { // Bool -> f64
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1.0 } else { 0.0 };
            }
            _ => {}
        }
    }
    0.0
}
// By-name shims for common method names (getattr/call)
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke_name_getattr_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("getattr", argc, a0, a1, a2)
}

#[no_mangle]
pub extern "C" fn nyash_plugin_invoke_name_call_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("call", argc, a0, a1, a2)
}

fn nyash_plugin_invoke_name_common_i64(method: &str, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0.max(0) as usize;
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let nyash_rust::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                        invoke = Some(p.inner.invoke_fn); break;
                    }
                }
            }
        });
    }
    if invoke.is_none() { return 0; }
    let box_type = box_type.unwrap_or_default();
    // Resolve method_id via PluginHost by name
    let mh = if let Ok(host) = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host().read() {
        host.resolve_method(&box_type, method)
    } else { return 0 };
    let method_id = match mh { Ok(h) => h.method_id, Err(_) => return 0 } as u32;
    // Build TLV args from legacy VM args (skip receiver slot)
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header((argc.saturating_sub(1).max(0) as u16));
    let mut add_from_legacy = |pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(pos) {
                use nyash_rust::backend::vm::VMValue as V;
                match v {
                    V::String(s) => nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                    V::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                    V::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                    V::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                    V::BoxRef(b) => {
                        if let Some(bufbox) = b.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec()); return;
                        }
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            let host = nyash_rust::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                                if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                        if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); return;
                                        }
                                    }
                                } else if p.box_type == "IntegerBox" {
                                    if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); return;
                                        }
                                    }
                                }
                            }
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                        } else {
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    if argc >= 2 { add_from_legacy(1); }
    if argc >= 3 { add_from_legacy(2); }
    if argc > 3 { for pos in 3..(argc as usize) { add_from_legacy(pos); } }
    let mut out = vec![0u8; 4096]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { return 0; }
    let out_slice = &out[..out_len];
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(out_slice) {
        match tag {
            3 => { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); } }
            1 => { return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => {}
        }
    }
    0
}

// ---- Handle-based birth shims for AOT/JIT object linkage ----
// These resolve symbols like "nyash.string.birth_h" referenced by ObjectModule.

// Generic birth by type_id -> handle (no args). Exported as nyash.box.birth_h
#[export_name = "nyash.box.birth_h"]
pub extern "C" fn nyash_box_birth_h_export(type_id: i64) -> i64 {
    if type_id <= 0 { return 0; }
    let tid = type_id as u32;
    // Map type_id back to type name
    let name_opt = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read()
        .ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| m.into_iter().find(|(_k,v)| *v == tid).map(|(k,_v)| k));
    if let Some(box_type) = name_opt {
        if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host_g.create_box(&box_type, &[]) {
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("nyrt: birth_h {} (type_id={}) -> handle={}", box_type, tid, h);
                }
                return h as i64;
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("nyrt: birth_h {} (type_id={}) FAILED: create_box", box_type, tid);
            }
        }
    } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("nyrt: birth_h (type_id={}) FAILED: type map not found", tid);
    }
    0
}
// Generic birth with args: (type_id, argc, a1, a2) -> handle
// Export name: nyash.box.birth_i64
#[export_name = "nyash.box.birth_i64"]
pub extern "C" fn nyash_box_birth_i64_export(type_id: i64, argc: i64, a1: i64, a2: i64) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    if type_id <= 0 { return 0; }
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    // Resolve invoke_fn via temporary instance
    let box_type_name = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read().ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| m.into_iter().find(|(_k,v)| *v == (type_id as u32)).map(|(k,_v)| k))
        .unwrap_or_else(|| "PluginBox".to_string());
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box(&box_type_name, &[]) {
            if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() { invoke = Some(p.inner.invoke_fn); }
        }
    }
    if invoke.is_none() {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") { eprintln!("nyrt: birth_i64 (type_id={}) FAILED: no invoke", type_id); }
        return 0;
    }
    let method_id: u32 = 0; // birth
    let instance_id: u32 = 0; // static
    // Build TLV args
    use nyash_rust::jit::rt::handles;
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut encode_handle = |h: i64| {
        if h > 0 {
            if let Some(obj) = handles::get(h as u64) {
                if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                    let host = nyash_rust::runtime::get_global_plugin_host();
                    if let Ok(hg) = host.read() {
                        if p.box_type == "StringBox" {
                            if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                if let Some(s) = sb.as_any().downcast_ref::<nyash_rust::box_trait::StringBox>() { nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); return; }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                if let Some(i) = ibx.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() { nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); return; }
                            }
                        }
                    }
                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                    return;
                }
            }
        }
        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, h);
    };
    if nargs >= 1 { encode_handle(a1); }
    if nargs >= 2 { encode_handle(a2); }
    // Extra birth args from legacy VM when present
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs {
            nyash_rust::jit::rt::with_legacy_vm_args(|args| {
                if let Some(v) = args.get(pos) {
                    use nyash_rust::backend::vm::VMValue as V;
                    match v {
                        V::String(s) => nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                        V::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                        V::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                        V::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                        V::BoxRef(bx) => {
                            if let Some(pb) = bx.as_any().downcast_ref::<PluginBoxV2>() {
                                if let Some(bufbox) = bx.as_any().downcast_ref::<nyash_rust::boxes::buffer::BufferBox>() {
                                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(&mut buf, &bufbox.to_vec());
                                } else {
                                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, pb.inner.type_id, pb.instance_id());
                                }
                            } else {
                                let s = bx.to_string_box().value; nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
    }
    let mut out = vec![0u8; 1024]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") { eprintln!("nyrt: birth_i64 (type_id={}) FAILED: invoke rc={}", type_id, rc); } return 0; }
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        if tag == 8 && payload.len() == 8 {
            let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]);
            let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]);
            let r_type = u32::from_le_bytes(t); let r_inst = u32::from_le_bytes(i);
            let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(box_type_name, r_type, r_inst, invoke.unwrap());
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::new(pb);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("nyrt: birth_i64 {} (type_id={}) argc={} -> handle={}", box_type_name, type_id, nargs, h);
            }
            return h as i64;
        }
    }
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") { eprintln!("nyrt: birth_i64 (type_id={}) FAILED: decode", type_id); }
    0
}

// Convert a VM argument (param index or existing handle) into a runtime handle
// Exported as: nyash.handle.of
#[export_name = "nyash.handle.of"]
pub extern "C" fn nyash_handle_of_export(v: i64) -> i64 {
    use nyash_rust::jit::rt::{handles, with_legacy_vm_args};
    use nyash_rust::box_trait::NyashBox;
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // If already a positive handle, pass through
    if v > 0 {
        return v;
    }
    // Otherwise treat as legacy param index and box-ref → handleize
    if v >= 0 {
        let idx = v as usize;
        let mut out: i64 = 0;
        with_legacy_vm_args(|args| {
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                // If it's a PluginBoxV2 or any NyashBox, register into handle registry
                // Note: store as NyashBox for uniform access
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(b.clone());
                out = handles::to_handle(arc) as i64;
            } else if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(b.clone());
                out = handles::to_handle(arc) as i64;
            }
        });
        return out;
    }
    0
}

#[export_name = "nyash.string.birth_h"]
pub extern "C" fn nyash_string_birth_h_export() -> i64 {
    // Create a new StringBox via unified plugin host; return runtime handle as i64
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("StringBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

#[export_name = "nyash.integer.birth_h"]
pub extern "C" fn nyash_integer_birth_h_export() -> i64 {
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("IntegerBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}
// ConsoleBox birth shim for AOT/JIT handle-based creation
#[export_name = "nyash.console.birth_h"]
pub extern "C" fn nyash_console_birth_h_export() -> i64 {
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("ConsoleBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// ---- Process entry (driver) ----
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize plugin host: prefer nyash.toml next to the executable; fallback to CWD
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));

    // Windows: assist DLL/plugin discovery by extending PATH and normalizing PYTHONHOME
    #[cfg(target_os = "windows")]
    if let Some(dir) = &exe_dir {
        use std::path::PathBuf;
        // Extend PATH with exe_dir and exe_dir\plugins if not already present
        let mut path_val = std::env::var("PATH").unwrap_or_default();
        let add_path = |pv: &mut String, p: &PathBuf| {
            let ps = p.display().to_string();
            if !pv.split(';').any(|seg| seg.eq_ignore_ascii_case(&ps)) {
                if !pv.is_empty() { pv.push(';'); }
                pv.push_str(&ps);
            }
        };
        add_path(&mut path_val, dir);
        let plug = dir.join("plugins"); if plug.is_dir() { add_path(&mut path_val, &plug); }
        std::env::set_var("PATH", &path_val);

        // Normalize PYTHONHOME: if unset, point to exe_dir\python when present.
        match std::env::var("PYTHONHOME") {
            Ok(v) => {
                // If relative, make absolute under exe_dir
                let pb = PathBuf::from(&v);
                if pb.is_relative() {
                    let abs = dir.join(pb);
                    std::env::set_var("PYTHONHOME", abs.display().to_string());
                }
            }
            Err(_) => {
                let cand = dir.join("python");
                if cand.is_dir() {
                    std::env::set_var("PYTHONHOME", cand.display().to_string());
                }
            }
        }
    }
    let mut inited = false;
    if let Some(dir) = &exe_dir {
        let candidate = dir.join("nyash.toml");
        if candidate.exists() {
            let _ = nyash_rust::runtime::init_global_plugin_host(candidate.to_string_lossy().as_ref());
            inited = true;
        }
    }
    if !inited {
        let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
    }
    // Optional verbosity
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        println!("🔌 nyrt: plugin host init attempted (exe_dir={}, cwd={})",
            exe_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(||"?".into()),
            std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_|"?".into())
        );
    }
    // Call exported Nyash entry if linked: `ny_main` (i64 -> return code normalized)
    unsafe {
        extern "C" {
            fn ny_main() -> i64;
        }
        // SAFETY: if not linked, calling will be an unresolved symbol at link-time; we rely on link step to include ny_main.
        let v = ny_main();
        // Print standardized result line for golden comparisons
        println!("Result: {}", v);
        v as i32
    }
}
