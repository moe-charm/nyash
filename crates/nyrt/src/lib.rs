// Minimal NyRT static shim library (libnyrt.a)
// Exposes C ABI entry points used by AOT/JIT-emitted objects.

// Internal helpers to avoid nested mutable borrows across closures
fn nyrt_encode_from_legacy_at(buf: &mut Vec<u8>, pos: usize) {
    use nyash_rust::backend::vm::VMValue;
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    nyash_rust::jit::rt::with_legacy_vm_args(|args| {
        if let Some(v) = args.get(pos) {
            match v {
                VMValue::String(s) => {
                    nyash_rust::runtime::plugin_ffi_common::encode::string(buf, s)
                }
                VMValue::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, *i),
                VMValue::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(buf, *f),
                VMValue::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(buf, *b),
                VMValue::BoxRef(b) => {
                    if let Some(bufbox) = b
                        .as_any()
                        .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                    {
                        nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                            buf,
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
                                            buf, &s.value,
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
                                    if let Some(i) = ibx
                                        .as_any()
                                        .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                    {
                                        nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                            buf, i.value,
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                        nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                            buf,
                            p.inner.type_id,
                            p.instance_id(),
                        );
                    } else {
                        let s = b.to_string_box().value;
                        nyash_rust::runtime::plugin_ffi_common::encode::string(buf, &s)
                    }
                }
                _ => {}
            }
        }
    });
}

fn nyrt_encode_arg_or_legacy(buf: &mut Vec<u8>, val: i64, pos: usize) {
    use nyash_rust::jit::rt::handles;
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    let mut appended = false;
    if val > 0 {
        if let Some(obj) = handles::get(val as u64) {
            if let Some(bufbox) = obj
                .as_any()
                .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
            {
                nyash_rust::runtime::plugin_ffi_common::encode::bytes(buf, &bufbox.to_vec());
                return;
            }
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                let host = nyash_rust::runtime::get_global_plugin_host();
                if let Ok(hg) = host.read() {
                    if p.box_type == "StringBox" {
                        if let Ok(Some(sb)) =
                            hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[])
                        {
                            if let Some(s) = sb
                                .as_any()
                                .downcast_ref::<nyash_rust::box_trait::StringBox>()
                            {
                                nyash_rust::runtime::plugin_ffi_common::encode::string(
                                    buf, &s.value,
                                );
                                return;
                            }
                        }
                    } else if p.box_type == "IntegerBox" {
                        if let Ok(Some(ibx)) =
                            hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                        {
                            if let Some(i) = ibx
                                .as_any()
                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                            {
                                nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, i.value);
                                return;
                            }
                        }
                    }
                }
                nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                    buf,
                    p.inner.type_id,
                    p.instance_id(),
                );
                return;
            }
        }
    }
    let before = buf.len();
    nyrt_encode_from_legacy_at(buf, pos);
    if buf.len() == before {
        nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, val);
    }
}

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
    let mut real_type_id: u32 = 0;
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
    if invoke.is_none()
        && a0 >= 0
        && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1")
    {
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
        return 0;
    }
    // Build TLV args from a1/a2 if present. Prefer handles/StringBox/IntegerBox via runtime host.
    use nyash_rust::{backend::vm::VMValue, jit::rt::handles};
    // argc from LLVM lowering is explicit arg count (excludes receiver)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    // Encode legacy VM arg at position into provided buffer (avoid capturing &mut buf)
    let mut encode_from_legacy_into = |dst: &mut Vec<u8>, arg_pos: usize| {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(arg_pos) {
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
                        // BufferBox → TLV bytes
                        if let Some(bufbox) = b
                            .as_any()
                            .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                        {
                            nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                                dst,
                                &bufbox.to_vec(),
                            );
                            return;
                        }
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            // Prefer StringBox/IntegerBox primitives when possible
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
                                        if let Some(i) =
                                            ibx.as_any()
                                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                        {
                                            nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                                dst, i.value,
                                            );
                                            return;
                                        }
                                    }
                                }
                            }
                            // Fallback: pass handle as plugin-handle TLV
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                                dst,
                                p.inner.type_id,
                                p.instance_id(),
                            );
                        } else {
                            // Stringify unknown boxes
                            let s = b.to_string_box().value;
                            nyash_rust::runtime::plugin_ffi_common::encode::string(dst, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    // Encode argument value or fallback to legacy slot (avoid capturing &mut buf)
    let mut encode_arg_into = |dst: &mut Vec<u8>, val: i64, pos: usize| {
        let mut appended = false;
        // Try handle first
        if val > 0 {
            if let Some(obj) = handles::get(val as u64) {
                // BufferBox handle → TLV bytes
                if let Some(bufbox) = obj
                    .as_any()
                    .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                {
                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(dst, &bufbox.to_vec());
                    appended = true;
                    return;
                }
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
                                if let Some(s) = sb
                                    .as_any()
                                    .downcast_ref::<nyash_rust::box_trait::StringBox>()
                                {
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
                                if let Some(i) = ibx
                                    .as_any()
                                    .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                {
                                    nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                        dst, i.value,
                                    );
                                    appended = true;
                                    return;
                                }
                            }
                        }
                    }
                    // Otherwise, pass as handle TLV
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
        // Legacy VM args by positional index (1-based for a1)
        let before = dst.len();
        encode_from_legacy_into(dst, pos);
        if dst.len() != before {
            appended = true;
        }
        // If still nothing appended (no-op), fallback to raw i64
        if !appended {
            nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, val);
        }
    };
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
    // Prepare output buffer (dynamic growth on short buffer)
    let mut cap: usize = 256;
    let (mut tag_ret, mut sz_ret, mut payload_ret): (u8, usize, Vec<u8>) = (0, 0, Vec::new());
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = unsafe {
            invoke.unwrap()(
                type_id as u32,
                method_id as u32,
                instance_id,
                buf.as_ptr(),
                buf.len(),
                out.as_mut_ptr(),
                &mut out_len,
            )
        };
        if rc != 0 {
            // Retry on short buffer hint (-1) or when plugin wrote beyond capacity (len > cap)
            if rc == -1 || out_len > cap {
                cap = cap.saturating_mul(2).max(out_len + 16);
                if cap > 1 << 20 {
                    break;
                }
                continue;
            }
            return 0;
        }
        let slice = &out[..out_len];
        if let Some((t, s, p)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice) {
            tag_ret = t;
            sz_ret = s;
            payload_ret = p.to_vec();
        }
        break;
    }
    if payload_ret.is_empty() {
        return 0;
    }
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        match tag {
            2 => {
                // I32
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
            }
            3 => {
                // I64
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return i64::from_le_bytes(b);
                }
            }
            6 | 7 => {
                // String/Bytes -> register StringBox handle
                use nyash_rust::box_trait::{NyashBox, StringBox};
                let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                return h as i64;
            }
            8 => {
                // Handle(tag=8) -> register and return handle id (i64)
                if sz == 8 {
                    let mut t = [0u8; 4];
                    t.copy_from_slice(&payload[0..4]);
                    let mut i = [0u8; 4];
                    i.copy_from_slice(&payload[4..8]);
                    let r_type = u32::from_le_bytes(t);
                    let r_inst = u32::from_le_bytes(i);
                    // Build PluginBoxV2 and register into handle-registry
                    let box_type_name =
                        nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
                            .read()
                            .ok()
                            .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
                            .and_then(|m| {
                                m.into_iter().find(|(_k, v)| *v == r_type).map(|(k, _v)| k)
                            })
                            .unwrap_or_else(|| "PluginBox".to_string());
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                        box_type_name.clone(),
                        r_type,
                        r_inst,
                        invoke.unwrap(),
                    );
                    let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                        std::sync::Arc::new(pb);
                    let h = nyash_rust::jit::rt::handles::to_handle(arc);
                    return h as i64;
                }
            }
            1 => {
                // Bool
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                    .unwrap_or(false)
                {
                    1
                } else {
                    0
                };
            }
            5 => {
                // F64 → optional conversion to i64
                if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                    if sz == 8 {
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
    let mut encode_arg = |val: i64, pos: usize| {
        let mut appended = false;
        if val > 0 {
            if let Some(obj) = handles::get(val as u64) {
                if let Some(bufbox) = obj
                    .as_any()
                    .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                {
                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                        &mut buf,
                        &bufbox.to_vec(),
                    );
                    appended = true;
                    return;
                }
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
                                if let Some(s) = sb
                                    .as_any()
                                    .downcast_ref::<nyash_rust::box_trait::StringBox>()
                                {
                                    nyash_rust::runtime::plugin_ffi_common::encode::string(
                                        &mut buf, &s.value,
                                    );
                                    appended = true;
                                    return;
                                }
                            }
                        } else if p.box_type == "IntegerBox" {
                            if let Ok(Some(ibx)) =
                                hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                            {
                                if let Some(i) = ibx
                                    .as_any()
                                    .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                {
                                    nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                        &mut buf, i.value,
                                    );
                                    appended = true;
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
                    appended = true;
                    return;
                }
            }
        }
        let before = buf.len();
        // Use global helper to avoid nested mutable borrows on buf
        nyrt_encode_from_legacy_at(&mut buf, pos);
        if buf.len() != before {
            appended = true;
        }
        if !appended {
            nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val);
        }
    };
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
    // Prepare output buffer (dynamic growth on short buffer)
    let mut cap: usize = 256;
    let (mut tag_ret, mut sz_ret, mut payload_ret): (u8, usize, Vec<u8>) = (0, 0, Vec::new());
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = unsafe {
            invoke.unwrap()(
                type_id as u32,
                method_id as u32,
                instance_id,
                buf.as_ptr(),
                buf.len(),
                out.as_mut_ptr(),
                &mut out_len,
            )
        };
        if rc != 0 {
            // Retry on short buffer (-1) or when plugin wrote beyond capacity
            if rc == -1 || out_len > cap {
                cap = cap.saturating_mul(2).max(out_len + 16);
                if cap > 1 << 20 {
                    break;
                }
                continue;
            }
            return 0.0;
        }
        let slice = &out[..out_len];
        if let Some((t, s, p)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(slice) {
            tag_ret = t;
            sz_ret = s;
            payload_ret = p.to_vec();
        }
        break;
    }
    if payload_ret.is_empty() {
        return 0.0;
    }
    if let Some((tag, sz, payload)) = Some((tag_ret, sz_ret, payload_ret.as_slice())) {
        match tag {
            5 => {
                // F64
                if sz == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return f64::from_le_bytes(b);
                }
            }
            3 => {
                // I64 -> f64
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as f64;
                }
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return (i64::from_le_bytes(b)) as f64;
                }
            }
            1 => {
                // Bool -> f64
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                    .unwrap_or(false)
                {
                    1.0
                } else {
                    0.0
                };
            }
            _ => {}
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
        (argc.saturating_sub(1).max(0) as u16),
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
    if let Some((tag, _sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(out_slice)
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
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                        box_type.clone(),
                        r_type,
                        r_inst,
                        invoke.unwrap(),
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
    if let Some((tag, _sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        match tag {
            2 => {
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
            }
            3 => {
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return i64::from_le_bytes(b);
                }
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
            }
            6 | 7 => {
                use nyash_rust::box_trait::{NyashBox, StringBox};
                let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                return h as i64;
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
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                        "PluginBox".into(),
                        r_type,
                        r_inst,
                        invoke.unwrap(),
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
    if let Some((tag, _sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        match tag {
            2 => {
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
            }
            3 => {
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    return i64::from_le_bytes(b);
                }
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                    return v as i64;
                }
            }
            6 | 7 => {
                use nyash_rust::box_trait::{NyashBox, StringBox};
                let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                return h as i64;
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
                    let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                        "PluginBox".into(),
                        r_type,
                        r_inst,
                        invoke.unwrap(),
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
                            let box_type_name =
                                nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host(
                                )
                                .read()
                                .ok()
                                .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
                                .and_then(|m| {
                                    m.into_iter().find(|(_k, v)| *v == r_type).map(|(k, _v)| k)
                                })
                                .unwrap_or_else(|| "PluginBox".to_string());
                            let pb = nyash_rust::runtime::plugin_loader_v2::construct_plugin_box(
                                box_type_name,
                                r_type,
                                inv,
                                r_inst,
                                None,
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
    let box_type_name = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read()
        .ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| {
            m.into_iter()
                .find(|(_k, v)| *v == real_type_id)
                .map(|(k, _v)| k)
        })
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

// ---- Handle-based birth shims for AOT/JIT object linkage ----
// These resolve symbols like "nyash.string.birth_h" referenced by ObjectModule.

// Generic birth by type_id -> handle (no args). Exported as nyash.box.birth_h
#[export_name = "nyash.box.birth_h"]
pub extern "C" fn nyash_box_birth_h_export(type_id: i64) -> i64 {
    if type_id <= 0 {
        return 0;
    }
    let tid = type_id as u32;
    // Map type_id back to type name
    let name_opt = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read()
        .ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| m.into_iter().find(|(_k, v)| *v == tid).map(|(k, _v)| k));
    if let Some(box_type) = name_opt {
        if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host_g.create_box(&box_type, &[]) {
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                    std::sync::Arc::from(b);
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!(
                        "nyrt: birth_h {} (type_id={}) -> handle={}",
                        box_type, tid, h
                    );
                }
                return h as i64;
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "nyrt: birth_h {} (type_id={}) FAILED: create_box",
                    box_type, tid
                );
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
    if type_id <= 0 {
        return 0;
    }
    let mut invoke: Option<
        unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    > = None;
    // Resolve invoke_fn via temporary instance
    let box_type_name = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host()
        .read()
        .ok()
        .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
        .and_then(|m| {
            m.into_iter()
                .find(|(_k, v)| *v == (type_id as u32))
                .map(|(k, _v)| k)
        })
        .unwrap_or_else(|| "PluginBox".to_string());
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box(&box_type_name, &[]) {
            if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("nyrt: birth_i64 (type_id={}) FAILED: no invoke", type_id);
        }
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
                            if let Ok(Some(ibx)) =
                                hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                            {
                                if let Some(i) = ibx
                                    .as_any()
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
                    return;
                }
            }
        }
        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, h);
    };
    if nargs >= 1 {
        encode_handle(a1);
    }
    if nargs >= 2 {
        encode_handle(a2);
    }
    // Extra birth args from legacy VM when present
    if nargs > 2 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        for pos in 3..=nargs {
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
                        V::BoxRef(bx) => {
                            if let Some(pb) = bx.as_any().downcast_ref::<PluginBoxV2>() {
                                if let Some(bufbox) =
                                    bx.as_any()
                                        .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                                {
                                    nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                                        &mut buf,
                                        &bufbox.to_vec(),
                                    );
                                } else {
                                    nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                                        &mut buf,
                                        pb.inner.type_id,
                                        pb.instance_id(),
                                    );
                                }
                            } else {
                                let s = bx.to_string_box().value;
                                nyash_rust::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
    }
    let mut out = vec![0u8; 1024];
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
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!(
                "nyrt: birth_i64 (type_id={}) FAILED: invoke rc={}",
                type_id, rc
            );
        }
        return 0;
    }
    if let Some((tag, _sz, payload)) =
        nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len])
    {
        if tag == 8 && payload.len() == 8 {
            let mut t = [0u8; 4];
            t.copy_from_slice(&payload[0..4]);
            let mut i = [0u8; 4];
            i.copy_from_slice(&payload[4..8]);
            let r_type = u32::from_le_bytes(t);
            let r_inst = u32::from_le_bytes(i);
            let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                box_type_name.clone(),
                r_type,
                r_inst,
                invoke.unwrap(),
            );
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::new(pb);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!(
                    "nyrt: birth_i64 {} (type_id={}) argc={} -> handle={}",
                    box_type_name, type_id, nargs, h
                );
            }
            return h as i64;
        }
    }
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("nyrt: birth_i64 (type_id={}) FAILED: decode", type_id);
    }
    0
}

// ---- String helpers for LLVM lowering ----
// Exported as: nyash_string_new(i8* ptr, i32 len) -> i8*
#[no_mangle]
pub extern "C" fn nyash_string_new(ptr: *const u8, len: i32) -> *mut i8 {
    use std::ptr;
    if ptr.is_null() || len < 0 {
        return std::ptr::null_mut();
    }
    let n = len as usize;
    // Allocate n+1 and null-terminate for C interop (puts, etc.)
    let mut buf = Vec::<u8>::with_capacity(n + 1);
    unsafe {
        ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr(), n);
        buf.set_len(n);
    }
    buf.push(0);
    let boxed = buf.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// ---- Unified semantics shims (handle-based) ----
// Exported as: nyash.semantics.add_hh(i64 lhs_handle, i64 rhs_handle) -> i64 (NyashBox handle)
#[export_name = "nyash.semantics.add_hh"]
pub extern "C" fn nyash_semantics_add_hh_export(lhs_h: i64, rhs_h: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    use nyash_rust::{
        box_trait::{IntegerBox, StringBox},
        runtime::semantics,
    };
    if lhs_h <= 0 || rhs_h <= 0 {
        return 0;
    }
    let lhs = if let Some(obj) = handles::get(lhs_h as u64) {
        obj
    } else {
        return 0;
    };
    let rhs = if let Some(obj) = handles::get(rhs_h as u64) {
        obj
    } else {
        return 0;
    };
    let ls_opt = semantics::coerce_to_string(lhs.as_ref());
    let rs_opt = semantics::coerce_to_string(rhs.as_ref());
    if ls_opt.is_some() || rs_opt.is_some() {
        let ls = ls_opt.unwrap_or_else(|| lhs.to_string_box().value);
        let rs = rs_opt.unwrap_or_else(|| rhs.to_string_box().value);
        let s = format!("{}{}", ls, rs);
        let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
            std::sync::Arc::new(StringBox::new(s));
        return handles::to_handle(arc) as i64;
    }
    if let (Some(li), Some(ri)) = (
        semantics::coerce_to_i64(lhs.as_ref()),
        semantics::coerce_to_i64(rhs.as_ref()),
    ) {
        let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
            std::sync::Arc::new(IntegerBox::new(li + ri));
        return handles::to_handle(arc) as i64;
    }
    // Fallback: stringify both and concat to preserve total order
    let ls = lhs.to_string_box().value;
    let rs = rhs.to_string_box().value;
    let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
        std::sync::Arc::new(StringBox::new(format!("{}{}", ls, rs)));
    handles::to_handle(arc) as i64
}

// ---- Array helpers for LLVM lowering (handle-based) ----
// Exported as: nyash_array_get_h(i64 handle, i64 idx) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_get_h(handle: i64, idx: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, jit::rt::handles};
    if handle <= 0 || idx < 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            let val = arr.get(Box::new(IntegerBox::new(idx)));
            if let Some(ib) = val.as_any().downcast_ref::<IntegerBox>() {
                return ib.value;
            }
        }
    }
    0
}

// ---- ExternCall helpers for LLVM lowering ----
// Exported as: nyash.console.log(i8* cstr) -> i64
#[export_name = "nyash.console.log"]
pub extern "C" fn nyash_console_log_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            println!("{}", s);
        }
    }
    0
}

// Exported as: nyash.console.log_handle(i64 handle) -> i64  
#[export_name = "nyash.console.log_handle"]
pub extern "C" fn nyash_console_log_handle(handle: i64) -> i64 {
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        let s = obj.to_string_box().value; // 既存の統一文字列変換メソッド
        println!("{}", s);
    } else {
        println!("{}", handle);
    }
    0
}

// Exported as: nyash.console.warn_handle(i64 handle) -> i64  
#[export_name = "nyash.console.warn_handle"]
pub extern "C" fn nyash_console_warn_handle(handle: i64) -> i64 {
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("WARN: {}", s);
    } else {
        eprintln!("WARN: {}", handle);
    }
    0
}

// Exported as: nyash.console.error_handle(i64 handle) -> i64  
#[export_name = "nyash.console.error_handle"]
pub extern "C" fn nyash_console_error_handle(handle: i64) -> i64 {
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("ERROR: {}", s);
    } else {
        eprintln!("ERROR: {}", handle);
    }
    0
}

// Exported as: nyash.debug.trace_handle(i64 handle) -> i64  
#[export_name = "nyash.debug.trace_handle"]
pub extern "C" fn nyash_debug_trace_handle(handle: i64) -> i64 {
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("TRACE: {}", s);
    } else {
        eprintln!("TRACE: {}", handle);
    }
    0
}

// Exported as: nyash.console.warn(i8* cstr) -> i64
#[export_name = "nyash.console.warn"]
pub extern "C" fn nyash_console_warn_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[warn] {}", s);
        }
    }
    0
}

// Exported as: nyash.console.error(i8* cstr) -> i64
#[export_name = "nyash.console.error"]
pub extern "C" fn nyash_console_error_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[error] {}", s);
        }
    }
    0
}

// Exported as: nyash.debug.trace(i8* cstr) -> i64
#[export_name = "nyash.debug.trace"]
pub extern "C" fn nyash_debug_trace_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[trace] {}", s);
        }
    }
    0
}

// Exported as: nyash.console.readline() -> i8*
#[export_name = "nyash.console.readline"]
pub extern "C" fn nyash_console_readline_export() -> *mut i8 {
    use std::io::{self, Read};
    // Read a line from stdin; normalize to UTF-8 and strip trailing CR/LF
    let mut input = String::new();
    // Use read_to_end if stdin is not a TTY? Simpler: read_line through BufRead
    // For simplicity, read from stdin into buffer until newline or EOF
    let mut buf = String::new();
    let mut handle = io::stdin();
    // On failure or EOF, return empty string
    match io::stdin().read_line(&mut buf) {
        Ok(_n) => {
            input = buf;
        }
        Err(_) => {
            input.clear();
        }
    }
    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }
    // Allocate C string (null-terminated)
    let mut bytes = input.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// ---- String concat helpers for LLVM lowering ----
// Exported as: nyash.string.concat_ss(i8* a, i8* b) -> i8*
#[export_name = "nyash.string.concat_ss"]
pub extern "C" fn nyash_string_concat_ss(a: *const i8, b: *const i8) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() {
                s.push_str(sa);
            }
        }
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() {
                s.push_str(sb);
            }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Exported as: nyash.string.concat_si(i8* a, i64 b) -> i8*
#[export_name = "nyash.string.concat_si"]
pub extern "C" fn nyash_string_concat_si(a: *const i8, b: i64) -> *mut i8 {
    let mut s = String::new();
    unsafe {
        if !a.is_null() {
            if let Ok(sa) = std::ffi::CStr::from_ptr(a).to_str() {
                s.push_str(sa);
            }
        }
    }
    s.push_str(&b.to_string());
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// Exported as: nyash.string.concat_is(i64 a, i8* b) -> i8*
#[export_name = "nyash.string.concat_is"]
pub extern "C" fn nyash_string_concat_is(a: i64, b: *const i8) -> *mut i8 {
    let mut s = a.to_string();
    unsafe {
        if !b.is_null() {
            if let Ok(sb) = std::ffi::CStr::from_ptr(b).to_str() {
                s.push_str(sb);
            }
        }
    }
    let mut bytes = s.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}

// ---- Instance field helpers for LLVM lowering (handle-based) ----
// Exported as: nyash.instance.get_field_h(i64 handle, i8* name) -> i64
#[export_name = "nyash.instance.get_field_h"]
pub extern "C" fn nyash_instance_get_field_h(handle: i64, name: *const i8) -> i64 {
    if handle <= 0 || name.is_null() {
        return 0;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(name) };
    let Ok(field) = name.to_str() else { return 0 };
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        if let Some(inst) = obj
            .as_any()
            .downcast_ref::<nyash_rust::instance_v2::InstanceBox>()
        {
            if let Some(shared) = inst.get_field(field) {
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                    std::sync::Arc::from(shared);
                let h = nyash_rust::jit::rt::handles::to_handle(arc);
                return h as i64;
            }
        }
    }
    0
}

// Exported as: nyash.instance.set_field_h(i64 handle, i8* name, i64 val_h) -> i64
#[export_name = "nyash.instance.set_field_h"]
pub extern "C" fn nyash_instance_set_field_h(handle: i64, name: *const i8, val_h: i64) -> i64 {
    if handle <= 0 || name.is_null() {
        return 0;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(name) };
    let Ok(field) = name.to_str() else { return 0 };
    if let Some(obj) = nyash_rust::jit::rt::handles::get(handle as u64) {
        if let Some(inst) = obj
            .as_any()
            .downcast_ref::<nyash_rust::instance_v2::InstanceBox>()
        {
            if val_h > 0 {
                if let Some(val) = nyash_rust::jit::rt::handles::get(val_h as u64) {
                    let shared: nyash_rust::box_trait::SharedNyashBox = std::sync::Arc::clone(&val);
                    let _ = inst.set_field(field, shared);
                    return 0;
                }
            }
        }
    }
    0
}

// Exported as: nyash_array_set_h(i64 handle, i64 idx, i64 val) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_set_h(handle: i64, idx: i64, val: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, jit::rt::handles};
    if handle <= 0 || idx < 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            let i = idx as usize;
            let len = arr.len();
            if i < len {
                let _ = arr.set(
                    Box::new(IntegerBox::new(idx)),
                    Box::new(IntegerBox::new(val)),
                );
            } else if i == len {
                let _ = arr.push(Box::new(IntegerBox::new(val)));
            } else {
                // Do nothing for gaps (keep behavior conservative)
            }
            return 0;
        }
    }
    0
}

// Exported as: nyash_array_push_h(i64 handle, i64 val) -> i64 (returns new length)
#[no_mangle]
pub extern "C" fn nyash_array_push_h(handle: i64, val: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            // If val is handle, try to use it; otherwise treat as integer
            let vbox: Box<dyn NyashBox> = if val > 0 {
                if let Some(o) = handles::get(val as u64) {
                    o.clone_box()
                } else {
                    Box::new(IntegerBox::new(val))
                }
            } else {
                Box::new(IntegerBox::new(val))
            };
            let _ = arr.push(vbox);
            return arr.len() as i64;
        }
    }
    0
}

// Exported as: nyash_array_length_h(i64 handle) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_length_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            return arr.len() as i64;
        }
    }
    0
}

// --- AOT ObjectModule dotted-name aliases (Array) ---
// Provide dotted symbol names expected by ObjectBuilder lowering, forwarding to existing underscored exports.
#[export_name = "nyash.array.get_h"]
pub extern "C" fn nyash_array_get_h_alias(handle: i64, idx: i64) -> i64 {
    nyash_array_get_h(handle, idx)
}

#[export_name = "nyash.array.set_h"]
pub extern "C" fn nyash_array_set_h_alias(handle: i64, idx: i64, val: i64) -> i64 {
    nyash_array_set_h(handle, idx, val)
}

#[export_name = "nyash.array.push_h"]
pub extern "C" fn nyash_array_push_h_alias(handle: i64, val: i64) -> i64 {
    nyash_array_push_h(handle, val)
}

#[export_name = "nyash.array.len_h"]
pub extern "C" fn nyash_array_len_h_alias(handle: i64) -> i64 {
    nyash_array_length_h(handle)
}

// --- AOT ObjectModule dotted-name exports (Map) ---
// Provide dotted symbol names expected by ObjectBuilder lowering for MapBox operations.
// size: (handle) -> i64
#[export_name = "nyash.map.size_h"]
pub extern "C" fn nyash_map_size_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
    }
    0
}

// get_h: (map_handle, key_i64) -> value_handle
#[export_name = "nyash.map.get_h"]
pub extern "C" fn nyash_map_get_h(handle: i64, key: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = Box::new(IntegerBox::new(key));
            let v = map.get(kbox);
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(v);
            let h = handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// get_hh: (map_handle, key_handle) -> value_handle
#[export_name = "nyash.map.get_hh"]
pub extern "C" fn nyash_map_get_hh(handle: i64, key_h: i64) -> i64 {
    use nyash_rust::{box_trait::NyashBox, jit::rt::handles};
    if handle <= 0 || key_h <= 0 {
        return 0;
    }
    if let (Some(obj), Some(key)) = (handles::get(handle as u64), handles::get(key_h as u64)) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let v = map.get(key.clone_box());
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(v);
            let h = handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// set_h: (map_handle, key_i64, val) -> i64 (ignored/0)
#[export_name = "nyash.map.set_h"]
pub extern "C" fn nyash_map_set_h(handle: i64, key: i64, val: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = Box::new(IntegerBox::new(key));
            let vbox: Box<dyn NyashBox> = if val > 0 {
                if let Some(o) = handles::get(val as u64) {
                    o.clone_box()
                } else {
                    Box::new(IntegerBox::new(val))
                }
            } else {
                Box::new(IntegerBox::new(val))
            };
            let _ = map.set(kbox, vbox);
            return 0;
        }
    }
    0
}

// has_h: (map_handle, key_i64) -> i64 (0/1)
#[export_name = "nyash.map.has_h"]
pub extern "C" fn nyash_map_has_h(handle: i64, key: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, jit::rt::handles};
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox = Box::new(IntegerBox::new(key));
            let v = map.get(kbox);
            // Consider present if not VoidBox
            let present = !v.as_any().is::<nyash_rust::box_trait::VoidBox>();
            return if present { 1 } else { 0 };
        }
    }
    0
}

// --- AOT ObjectModule dotted-name exports (String/Any helpers) ---
// String.len_h(handle) -> i64
#[export_name = "nyash.string.len_h"]
pub extern "C" fn nyash_string_len_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        let present = if handle > 0 {
            handles::get(handle as u64).is_some()
        } else {
            false
        };
        eprintln!(
            "[AOT-LEN_H] string.len_h handle={} present={}",
            handle, present
        );
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return sb.value.len() as i64;
        }
    }
    0
}

// String.charCodeAt_h(handle, idx) -> i64 (byte-based; -1 if OOB)
#[export_name = "nyash.string.charCodeAt_h"]
pub extern "C" fn nyash_string_charcode_at_h_export(handle: i64, idx: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if idx < 0 {
        return -1;
    }
    if handle <= 0 {
        return -1;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            let s = &sb.value;
            let i = idx as usize;
            if i < s.len() {
                return s.as_bytes()[i] as i64;
            }
            return -1;
        }
    }
    -1
}

// String.concat_hh(lhs_h, rhs_h) -> handle
#[export_name = "nyash.string.concat_hh"]
pub extern "C" fn nyash_string_concat_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    let s = format!("{}{}", to_s(a_h), to_s(b_h));
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// String.eq_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.eq_hh"]
pub extern "C" fn nyash_string_eq_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    if to_s(a_h) == to_s(b_h) {
        1
    } else {
        0
    }
}

// box.from_i8_string(ptr) -> handle
// Helper: build a StringBox from i8* and return a handle for AOT marshalling
#[export_name = "nyash.box.from_i8_string"]
pub extern "C" fn nyash_box_from_i8_string(ptr: *const i8) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    use std::ffi::CStr;
    if ptr.is_null() {
        return 0;
    }
    let c = unsafe { CStr::from_ptr(ptr) };
    let s = match c.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => return 0,
    };
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// box.from_f64(val) -> handle
// Helper: build a FloatBox and return a handle
#[export_name = "nyash.box.from_f64"]
pub extern "C" fn nyash_box_from_f64(val: f64) -> i64 {
    use nyash_rust::{box_trait::NyashBox, boxes::FloatBox, jit::rt::handles};
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(FloatBox::new(val));
    handles::to_handle(arc) as i64
}

// env.box.new(type_name: *const i8) -> handle (i64)
// Minimal shim for Core-13 pure AOT: constructs Box via registry by name (no args)
#[export_name = "nyash.env.box.new"]
pub extern "C" fn nyash_env_box_new(type_name: *const i8) -> i64 {
    use nyash_rust::{
        box_trait::NyashBox, jit::rt::handles, runtime::box_registry::get_global_registry,
    };
    use std::ffi::CStr;
    if type_name.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(type_name) };
    let ty = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let reg = get_global_registry();
    match reg.create_box(ty, &[]) {
        Ok(b) => {
            let arc: std::sync::Arc<dyn NyashBox> = b.into();
            handles::to_handle(arc) as i64
        }
        Err(_) => 0,
    }
}

// env.box.new_i64x(type_name: *const i8, argc: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> handle (i64)
// Minimal shim: construct args from handles or wrap i64 as IntegerBox
#[export_name = "nyash.env.box.new_i64x"]
pub extern "C" fn nyash_env_box_new_i64x(
    type_name: *const i8,
    argc: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
        runtime::box_registry::get_global_registry,
    };
    use std::ffi::CStr;
    if type_name.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(type_name) };
    let ty = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    // Build args vec from provided i64 words
    let mut argv: Vec<Box<dyn NyashBox>> = Vec::new();
    let push_val = |dst: &mut Vec<Box<dyn NyashBox>>, v: i64| {
        if v > 0 {
            if let Some(obj) = handles::get(v as u64) {
                dst.push(obj.share_box());
                return;
            }
        }
        dst.push(Box::new(IntegerBox::new(v)));
    };
    if argc >= 1 {
        push_val(&mut argv, a1);
    }
    if argc >= 2 {
        push_val(&mut argv, a2);
    }
    if argc >= 3 {
        push_val(&mut argv, a3);
    }
    if argc >= 4 {
        push_val(&mut argv, a4);
    }

    let reg = get_global_registry();
    match reg.create_box(ty, &argv) {
        Ok(b) => {
            let arc: std::sync::Arc<dyn NyashBox> = b.into();
            handles::to_handle(arc) as i64
        }
        Err(_) => 0,
    }
}

// String.lt_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.lt_hh"]
pub extern "C" fn nyash_string_lt_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    if to_s(a_h) < to_s(b_h) {
        1
    } else {
        0
    }
}

// Any.length_h(handle) -> i64 (Array/String/Map)
#[export_name = "nyash.any.length_h"]
pub extern "C" fn nyash_any_length_h_export(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        let present = if handle > 0 {
            handles::get(handle as u64).is_some()
        } else {
            false
        };
        eprintln!(
            "[AOT-LEN_H] any.length_h handle={} present={}",
            handle, present
        );
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            if let Some(ib) = arr
                .length()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return sb.value.len() as i64;
        }
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
    }
    0
}

// Any.is_empty_h(handle) -> i64 (0/1)
#[export_name = "nyash.any.is_empty_h"]
pub extern "C" fn nyash_any_is_empty_h_export(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if handle <= 0 {
        return 1;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            if let Ok(items) = arr.items.read() {
                return if items.is_empty() { 1 } else { 0 };
            }
        }
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return if sb.value.is_empty() { 1 } else { 0 };
        }
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return if ib.value == 0 { 1 } else { 0 };
            }
        }
    }
    1
}

// Instance birth by name (packed u64x2 + len) -> handle
// export: nyash.instance.birth_name_u64x2(lo, hi, len) -> i64
#[export_name = "nyash.instance.birth_name_u64x2"]
pub extern "C" fn nyash_instance_birth_name_u64x2_export(lo: i64, hi: i64, len: i64) -> i64 {
    use nyash_rust::runtime::get_global_plugin_host;
    let mut bytes = Vec::with_capacity(len.max(0) as usize);
    let lo_u = lo as u64;
    let hi_u = hi as u64;
    let l = len.max(0) as usize;
    let take = core::cmp::min(16, l);
    for i in 0..take.min(8) {
        bytes.push(((lo_u >> (8 * i)) & 0xff) as u8);
    }
    for i in 0..take.saturating_sub(8) {
        bytes.push(((hi_u >> (8 * i)) & 0xff) as u8);
    }
    // If len > 16, remaining bytes are not represented in (lo,hi); assume names <=16 bytes for now.
    if bytes.len() != l {
        bytes.resize(l, 0);
    }
    let name = String::from_utf8_lossy(&bytes).to_string();
    if let Ok(host_g) = get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box(&name, &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// Construct StringBox from two u64 words (little-endian) + length (<=16) and return handle
// export: nyash.string.from_u64x2(lo, hi, len) -> i64
#[export_name = "nyash.string.from_u64x2"]
pub extern "C" fn nyash_string_from_u64x2_export(lo: i64, hi: i64, len: i64) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    let l = if len < 0 {
        0
    } else {
        core::cmp::min(len as usize, 16)
    };
    let mut bytes: Vec<u8> = Vec::with_capacity(l);
    let lo_u = lo as u64;
    let hi_u = hi as u64;
    for i in 0..l.min(8) {
        bytes.push(((lo_u >> (8 * i)) & 0xff) as u8);
    }
    for i in 0..l.saturating_sub(8) {
        bytes.push(((hi_u >> (8 * i)) & 0xff) as u8);
    }
    let s = String::from_utf8_lossy(&bytes).to_string();
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// Convert a VM argument (param index or existing handle) into a runtime handle
// Exported as: nyash.handle.of
#[export_name = "nyash.handle.of"]
pub extern "C" fn nyash_handle_of_export(v: i64) -> i64 {
    use nyash_rust::box_trait::NyashBox;
    use nyash_rust::jit::rt::{handles, with_legacy_vm_args};
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

// ---- Reserved runtime/GC externs for AOT linking ----
// Exported as: nyash.rt.checkpoint
#[export_name = "nyash.rt.checkpoint"]
pub extern "C" fn nyash_rt_checkpoint_export() -> i64 {
    if std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE")
        .ok()
        .as_deref()
        == Some("1")
    {
        eprintln!("[nyrt] nyash.rt.checkpoint reached");
    }
    0
}

// Exported as: nyash.gc.barrier_write
#[export_name = "nyash.gc.barrier_write"]
pub extern "C" fn nyash_gc_barrier_write_export(handle_or_ptr: i64) -> i64 {
    let _ = handle_or_ptr;
    if std::env::var("NYASH_GC_BARRIER_TRACE").ok().as_deref() == Some("1") {
        eprintln!("[nyrt] nyash.gc.barrier_write h=0x{:x}", handle_or_ptr);
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
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize plugin host: prefer nyash.toml next to the executable; fallback to CWD
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    // Windows: assist DLL/plugin discovery by extending PATH and normalizing PYTHONHOME
    #[cfg(target_os = "windows")]
    if let Some(dir) = &exe_dir {
        use std::path::PathBuf;
        // Extend PATH with exe_dir and exe_dir\plugins if not already present
        let mut path_val = std::env::var("PATH").unwrap_or_default();
        let add_path = |pv: &mut String, p: &PathBuf| {
            let ps = p.display().to_string();
            if !pv.split(';').any(|seg| seg.eq_ignore_ascii_case(&ps)) {
                if !pv.is_empty() {
                    pv.push(';');
                }
                pv.push_str(&ps);
            }
        };
        add_path(&mut path_val, dir);
        let plug = dir.join("plugins");
        if plug.is_dir() {
            add_path(&mut path_val, &plug);
        }
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
            let _ =
                nyash_rust::runtime::init_global_plugin_host(candidate.to_string_lossy().as_ref());
            inited = true;
        }
    }
    if !inited {
        let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
    }
    // Optional verbosity
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        println!(
            "🔌 nyrt: plugin host init attempted (exe_dir={}, cwd={})",
            exe_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "?".into()),
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "?".into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
        runtime::plugin_loader_v2::make_plugin_box_v2,
    };
    use std::sync::Arc;

    unsafe extern "C" fn fake_i32(
        _t: u32,
        _m: u32,
        _i: u32,
        _a: *const u8,
        _al: usize,
        res: *mut u8,
        len: *mut usize,
    ) -> i32 {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.push(2);
        buf.push(0);
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&123i32.to_le_bytes());
        if res.is_null() || len.is_null() || unsafe { *len } < buf.len() {
            unsafe {
                if !len.is_null() {
                    *len = buf.len();
                }
            }
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), res, buf.len());
            *len = buf.len();
        }
        0
    }

    unsafe extern "C" fn fake_str(
        _t: u32,
        _m: u32,
        _i: u32,
        _a: *const u8,
        _al: usize,
        res: *mut u8,
        len: *mut usize,
    ) -> i32 {
        let s = b"hi";
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.push(7);
        buf.push(0);
        buf.extend_from_slice(&(s.len() as u16).to_le_bytes());
        buf.extend_from_slice(s);
        if res.is_null() || len.is_null() || unsafe { *len } < buf.len() {
            unsafe {
                if !len.is_null() {
                    *len = buf.len();
                }
            }
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), res, buf.len());
            *len = buf.len();
        }
        0
    }

    #[test]
    fn decode_i32_and_string_returns() {
        let pb = make_plugin_box_v2("Dummy".into(), 1, 1, fake_i32);
        let arc: Arc<dyn NyashBox> = Arc::new(pb);
        let handle = handles::to_handle(arc) as i64;
        let val = nyash_plugin_invoke3_tagged_i64(1, 0, 0, handle, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(val, 123);

        let pb = make_plugin_box_v2("Dummy".into(), 1, 2, fake_str);
        let arc: Arc<dyn NyashBox> = Arc::new(pb);
        let handle = handles::to_handle(arc) as i64;
        let h = nyash_plugin_invoke3_tagged_i64(1, 0, 0, handle, 0, 0, 0, 0, 0, 0, 0, 0);
        assert!(h > 0);
        let obj = handles::get(h as u64).unwrap();
        let sb = obj.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(sb.value, "hi");
    }
}
