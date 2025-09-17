use crate::encode::{nyrt_encode_arg_or_legacy, nyrt_encode_from_legacy_at};
use crate::plugin::invoke_core;
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke3_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    // Resolve receiver via shared core helper
    let recv = match invoke_core::resolve_receiver_for_a0(a0) {
        Some(r) => r,
        None => return 0,
    };
    let instance_id: u32 = recv.instance_id;
    let _real_type_id: u32 = recv.real_type_id;
    let invoke = recv.invoke;
    // Build TLV args from a1/a2 if present. Prefer handles/StringBox/IntegerBox via runtime host.
    use nyash_rust::{backend::vm::VMValue, jit::rt::handles};
    // argc from LLVM lowering is explicit arg count (excludes receiver)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    // Encode legacy VM arg at position into provided buffer (avoid capturing &mut buf)
    let mut encode_from_legacy_into =
        |dst: &mut Vec<u8>, arg_pos: usize| nyrt_encode_from_legacy_at(dst, arg_pos);
    // Encode argument value or fallback to legacy slot (avoid capturing &mut buf)
    let mut encode_arg_into =
        |dst: &mut Vec<u8>, val: i64, pos: usize| nyrt_encode_arg_or_legacy(dst, val, pos);
    if nargs >= 1 {
        encode_arg_into(&mut buf, a1, 1);
    }
    if nargs >= 2 {
        encode_arg_into(&mut buf, a2, 2);
    }
    // Extra args from legacy VM args (positions 3..nargs)
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs {
            encode_from_legacy_into(&mut buf, pos);
        }
    }
    // Call invoke with dynamic buffer logic centralized
    let (tag_ret, sz_ret, payload_ret): (u8, usize, Vec<u8>) = match invoke_core::plugin_invoke_call(
        invoke,
        type_id as u32,
        method_id as u32,
        instance_id,
        &buf,
    ) {
        Some((t, s, p)) => (t, s, p),
        None => return 0,
    };
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        if let Some(v) = invoke_core::decode_entry_to_i64(tag, sz, payload, invoke) {
            return v;
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
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
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
                            if p.inner.type_id == (type_id as u32) {
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() {
        return 0.0;
    }
    // Build TLV args from a1/a2 with String/Integer support
    use nyash_rust::{backend::vm::VMValue, jit::rt::handles};
    // argc from LLVM lowering is explicit arg count (excludes receiver)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut encode_from_legacy = |arg_pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(arg_pos) {
                match v {
                    VMValue::String(s) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s)
                    }
                    VMValue::Integer(i) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i)
                    }
                    VMValue::Float(f) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f)
                    }
                    VMValue::Bool(b) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b)
                    }
                    VMValue::BoxRef(b) => {
                        if let Some(bufbox) = b
                            .as_any()
                            .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                        {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                                &mut buf,
                                &bufbox.to_vec(),
                            );
                            return;
                        }
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
                                        if let Some(s) = sb
                                            .as_any()
                                            .downcast_ref::<nyash_rust::box_trait::StringBox>()
                                        {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(
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
                                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                        {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                                &mut buf, i.value,
                                            );
                                            return;
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
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    let mut encode_arg =
        |val: i64, pos: usize| crate::encode::nyrt_encode_arg_or_legacy(&mut buf, val, pos);
    if nargs >= 1 {
        encode_arg(a1, 1);
    }
    if nargs >= 2 {
        encode_arg(a2, 2);
    }
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs {
            nyrt_encode_from_legacy_at(&mut buf, pos);
        }
    }
    // Invoke via shared helper
    let (mut tag_ret, mut sz_ret, mut payload_ret): (u8, usize, Vec<u8>) =
        match invoke_core::plugin_invoke_call(
            invoke.unwrap(),
            type_id as u32,
            method_id as u32,
            instance_id,
            &buf,
        ) {
            Some((t, s, p)) => (t, s, p),
            None => return 0.0,
        };
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        if let Some(f) = invoke_core::decode_entry_to_f64(tag, sz, payload) {
            return f;
        }
    }
    0.0
}
// By-name shims for common method names (getattr/call)
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke_name_getattr_i64(
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
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
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
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
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0.max(0) as usize;
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
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
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let nyash_rust::backend::vm::VMValue::BoxRef(b) = v {
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
    // Resolve method_id via PluginHost by name
    let mh = if let Ok(host) =
        nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host().read()
    {
        host.resolve_method(&box_type, method)
    } else {
        return 0;
    };
    let method_id = match mh {
        Ok(h) => h.method_id,
        Err(_) => return 0,
    } as u32;
    // Build TLV args from legacy VM args (skip receiver slot)
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(
        argc.saturating_sub(1).max(0) as u16,
    );
    let mut add_from_legacy = |pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(pos) {
                use nyash_rust::backend::vm::VMValue as V;
                match v {
                    V::String(s) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, s)
                    }
                    V::Integer(i) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, *i)
                    }
                    V::Float(f) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, *f)
                    }
                    V::Bool(b) => {
                        nyash_rust::runtime::plugin_ffi_common::encode::bool(&mut buf, *b)
                    }
                    V::BoxRef(b) => {
                        if let Some(bufbox) = b
                            .as_any()
                            .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                        {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                                &mut buf,
                                &bufbox.to_vec(),
                            );
                            return;
                        }
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
                                        if let Some(s) = sb
                                            .as_any()
                                            .downcast_ref::<nyash_rust::box_trait::StringBox>()
                                        {
                                            nyash_rust::runtime::plugin_ffi_common::encode::string(
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
                                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                        {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                                &mut buf, i.value,
                                            );
                                            return;
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
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
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
    if argc > 3 {
        for pos in 3..(argc as usize) {
            add_from_legacy(pos);
        }
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
        return 0;
    }
    let out_slice = &out[..out_len];
    if let Some((tag, sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(out_slice)
    {
        if let Some(v) = super::invoke_core::decode_entry_to_i64(tag, sz, payload, invoke.unwrap())
        {
            return v;
        }
    }
    0
}

// General by-name invoke: (recv_handle, method_cstr, argc, a1, a2) -> i64
// Export name: nyash.plugin.invoke_by_name_i64
#[export_name = "nyash.plugin.invoke_by_name_i64"]
pub extern "C" fn nyash_plugin_invoke_by_name_i64(
    recv_handle: i64,
    method: *const i8,
    argc: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    if method.is_null() {
        return 0;
    }
    let mname = unsafe { std::ffi::CStr::from_ptr(method) };
    let Ok(method_str) = mname.to_str() else {
        return 0;
    };
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if recv_handle > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(recv_handle as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                type_id = p.inner.type_id;
                box_type = Some(p.box_type.clone());
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() {
        return 0;
    }
    let box_type = box_type.unwrap_or_default();
    // Resolve method_id via PluginHost by name
    let mh = if let Ok(host) =
        nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host().read()
    {
        host.resolve_method(&box_type, method_str)
    } else {
        return 0;
    };
    let method_id = match mh {
        Ok(h) => h.method_id,
        Err(_) => return 0,
    } as u32;
    // Build TLV args from a1/a2 (no legacy in LLVM path)
    // argc is the number of explicit arguments (receiver excluded)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    nyrt_encode_arg_or_legacy(&mut buf, a1, 1);
    if nargs >= 2 {
        nyrt_encode_arg_or_legacy(&mut buf, a2, 2);
    }
    // Execute
    let mut out = vec![0u8; 512];
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
        return 0;
    }
    if let Some((tag, _sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
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
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                    .unwrap_or(false)
                {
                    1
                } else {
                    0
                };
            }
            8 => {
                if payload.len() == 8 {
                    let mut t = [0u8; 4];
                    t.copy_from_slice(&payload[0..4]);
                    let mut i = [0u8; 4];
                    i.copy_from_slice(&payload[4..8]);
                    let r_type = u32::from_le_bytes(t);
                    let r_inst = u32::from_le_bytes(i);
                    let meta_opt =
                        nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(r_type);
                    let (box_type_name, invoke_ptr) = if let Some(meta) = meta_opt {
                        (meta.box_type.clone(), meta.invoke_fn)
                    } else {
                        (box_type.clone(), invoke.unwrap())
                    };
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                        box_type_name,
                        r_type,
                        r_inst,
                        invoke_ptr,
                    );
                    let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                        std::sync::Arc::new(pb);
                    let h = nyash_rust::jit::rt::handles::to_handle(arc);
                    return h as i64;
                }
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
            _ => {}
        }
    }
    0
}

// Tagged by-id invoke (supports f64/int/handle for first two args)
// tag: 3=I64, 5=F64(bits), 8=Handle
#[export_name = "nyash_plugin_invoke3_tagged_i64"]
pub extern "C" fn nyash_plugin_invoke3_tagged_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    tag1: i64,
    a2: i64,
    tag2: i64,
    a3: i64,
    tag3: i64,
    a4: i64,
    tag4: i64,
) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver invoke and actual plugin type_id
    let mut instance_id: u32 = 0;
    let mut real_type_id: u32 = type_id as u32;
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                real_type_id = p.inner.type_id;
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() {
        return 0;
    }
    // Build TLV from tags
    // argc is the number of explicit arguments (receiver excluded)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut enc = |val: i64, tag: i64| match tag {
        3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val),
        5 => {
            let bits = val as u64;
            let f = f64::from_bits(bits);
            nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, f);
        }
        8 => {
            if val > 0 {
                if let Some(obj) = nyash_rust::jit::rt::handles::get(val as u64) {
                    if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                        nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                            &mut buf,
                            p.inner.type_id,
                            p.instance_id(),
                        );
                    } else {
                        let s = obj.to_string_box().value;
                        nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s);
                    }
                }
            } else {
                nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, 0);
            }
        }
        _ => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val),
    };
    if nargs >= 1 {
        enc(a1, tag1);
    }
    if nargs >= 2 {
        enc(a2, tag2);
    }
    if nargs >= 3 {
        enc(a3, tag3);
    }
    if nargs >= 4 {
        enc(a4, tag4);
    }
    // Invoke
    let mut out = vec![0u8; 512];
    let mut out_len: usize = out.len();
    let rc = unsafe {
        invoke.unwrap()(
            real_type_id,
            method_id as u32,
            instance_id,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if rc != 0 {
        return 0;
    }
    if let Some((tag, sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        if let Some(v) = invoke_core::decode_entry_to_i64(tag, sz, payload, invoke.unwrap()) {
            return v;
        }
    }
    0
}

// Variable-length tagged invoke by-id
// Exported as: nyash.plugin.invoke_tagged_v_i64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, i64* vals, i64* tags) -> i64
#[export_name = "nyash.plugin.invoke_tagged_v_i64"]
pub extern "C" fn nyash_plugin_invoke_tagged_v_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    recv_h: i64,
    vals: *const i64,
    tags: *const i64,
) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    if recv_h <= 0 {
        return 0;
    }
    // Resolve receiver invoke
    let mut instance_id: u32 = 0;
    let mut real_type_id: u32 = 0;
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
    let nargs = argc.saturating_sub(1).max(0) as usize;
    let (vals, tags) = if nargs > 0 && !vals.is_null() && !tags.is_null() {
        unsafe {
            (
                std::slice::from_raw_parts(vals, nargs),
                std::slice::from_raw_parts(tags, nargs),
            )
        }
    } else {
        (&[][..], &[][..])
    };

    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    for i in 0..nargs {
        match tags[i] {
            3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, vals[i]),
            5 => {
                let f = f64::from_bits(vals[i] as u64);
                nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, f);
            }
            8 => {
                if let Some(obj) = nyash_rust::jit::rt::handles::get(vals[i] as u64) {
                    if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
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
                    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, 0);
                }
            }
            _ => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, vals[i]),
        }
    }
    let mut out = vec![0u8; 1024];
    let mut out_len: usize = out.len();
    let rc = unsafe {
        invoke.unwrap()(
            real_type_id,
            method_id as u32,
            instance_id,
            buf.as_ptr(),
            buf.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if rc != 0 {
        return 0;
    }
    if let Some((tag, sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        if let Some(v) = invoke_core::decode_entry_to_i64(tag, sz, payload, invoke.unwrap()) {
            return v;
        }
    }
    0
}
