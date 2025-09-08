//! Plugin invoke shims (variable-length / tagged)

// Variable-length by-name invoke with tagged args
// Export name: nyash.plugin.invoke_by_name_tagged_v_i64
#[no_mangle]
#[export_name = "nyash.plugin.invoke_by_name_tagged_v_i64"]
pub extern "C" fn nyash_plugin_invoke_by_name_tagged_v_i64(
    recv_handle: i64,
    method: *const i8,
    argc: i64,
    vals: *const i64,
    tags: *const i64,
) -> i64 {
    let wrap_i64 = std::env::var("NYASH_LLVM_VINVOKE_BYNAME_WRAP_I64").ok().as_deref() == Some("1");
    let trace = std::env::var("NYASH_LLVM_VINVOKE_TRACE").ok().as_deref() == Some("1");
    if method.is_null() { return 0; }
    let mname = unsafe { std::ffi::CStr::from_ptr(method) };
    let Ok(method_str) = mname.to_str() else { return 0 };
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver plugin
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
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
    if trace {
        eprintln!(
            "nyrt: vinvoke.by_name: recv_h={} argc={} vals_ptr={:p} tags_ptr={:p}",
            recv_handle, argc, vals, tags
        );
    }
    if invoke.is_none() { return 0; }
    let box_type = box_type.unwrap_or_default();
    // Resolve method id by name
    let mh = if let Ok(host) = nyash_rust::runtime::plugin_loader_unified::get_global_plugin_host().read() {
        host.resolve_method(&box_type, method_str)
    } else { return 0 };
    let method_id = match mh { Ok(h) => h.method_id, Err(_) => return 0 } as u32;
    // Prepare arguments
    let nargs = argc.max(0) as usize;
    let (vals, tags) = if nargs > 0 && !vals.is_null() && !tags.is_null() {
        unsafe {
            (std::slice::from_raw_parts(vals, nargs), std::slice::from_raw_parts(tags, nargs))
        }
    } else { (&[][..], &[][..]) };
    if trace {
        let sample = std::cmp::min(nargs, 8);
        eprintln!(
            "nyrt: vinvoke.by_name: type_id={} method_id={} nargs={} tags[..{}]={:?}",
            type_id, method_id, nargs, sample, &tags[..sample]
        );
    }
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let enc = |dst: &mut Vec<u8>, val: i64, tag: i64| {
        match tag {
            3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, val),
            5 => { let f = f64::from_bits(val as u64); nyash_rust::runtime::plugin_ffi_common::encode::f64(dst, f); },
            8 => {
                if let Some(obj) = nyash_rust::jit::rt::handles::get(val as u64) {
                    if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                        nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(dst, p.inner.type_id, p.instance_id());
                    } else {
                        let s = obj.to_string_box().value;
                        nyash_rust::runtime::plugin_ffi_common::encode::string(dst, &s);
                    }
                } else {
                    nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, 0);
                }
            }
            _ => nyash_rust::runtime::plugin_ffi_common::encode::i64(dst, val),
        }
    };
    for i in 0..nargs { enc(&mut buf, vals[i], tags[i]); }
    // Invoke
    let mut out = vec![0u8; 512]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if trace { eprintln!("nyrt: vinvoke.by_name: rc={} out_len={}", rc, out_len); }
    if rc != 0 { return 0; }
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        if trace { eprintln!("nyrt: vinvoke.by_name: ret_tag={}", tag); }
        match tag {
            3 => {
                if wrap_i64 {
                    // Wrap integer into IntegerBox handle for by-name path (helps string ops like toString/concat)
                    if payload.len()==8 {
                        use nyash_rust::box_trait::IntegerBox;
                        let mut b=[0u8;8]; b.copy_from_slice(payload); let n=i64::from_le_bytes(b);
                        let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::new(IntegerBox::new(n));
                        let h = nyash_rust::jit::rt::handles::to_handle(arc);
                        return h as i64;
                    }
                } else if payload.len()==8 {
                    let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b);
                }
            }
            1 => { return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            8 => { if payload.len()==8 { let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]); let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]); let r_type=u32::from_le_bytes(t); let r_inst=u32::from_le_bytes(i); let pb=nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(box_type.clone(), r_type, r_inst, invoke.unwrap()); let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox>=std::sync::Arc::new(pb); let h=nyash_rust::jit::rt::handles::to_handle(arc); return h as i64; } }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => {}
        }
    }
    0
}

// Tagged by-id invoke (supports f64/int/handle for first two args)
// tag: 3=I64, 5=F64(bits), 8=Handle
#[no_mangle]
#[no_mangle]
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
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = nyash_rust::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                real_type_id = p.inner.type_id;
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() { return 0; }
    // Build TLV from tags
    // argc is the number of explicit arguments (receiver excluded)
    let nargs = argc.max(0) as usize;
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    let mut enc = |val: i64, tag: i64| {
        match tag {
            3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, val),
            5 => { let bits = val as u64; let f = f64::from_bits(bits); nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, f); },
            8 => {
                if val > 0 {
                    if let Some(obj) = nyash_rust::jit::rt::handles::get(val as u64) {
                        if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                            nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
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
        }
    };
    if nargs >= 1 { enc(a1, tag1); }
    if nargs >= 2 { enc(a2, tag2); }
    if nargs >= 3 { enc(a3, tag3); }
    if nargs >= 4 { enc(a4, tag4); }
    // Invoke
    let mut out = vec![0u8; 512]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(real_type_id, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { return 0; }
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        match tag {
            3 => { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); } }
            1 => { return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            8 => { if payload.len()==8 { let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]); let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]); let r_type=u32::from_le_bytes(t); let r_inst=u32::from_le_bytes(i); let pb=nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2("PluginBox".into(), r_type, r_inst, invoke.unwrap()); let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox>=std::sync::Arc::new(pb); let h=nyash_rust::jit::rt::handles::to_handle(arc); return h as i64; } }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => {}
        }
    }
    0
}

// Variable-length tagged invoke by-id
// Exported as: nyash.plugin.invoke_tagged_v_i64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, i64* vals, i64* tags) -> i64
#[no_mangle]
#[no_mangle]
#[export_name = "nyash.plugin.invoke_tagged_v_i64"]
pub extern "C" fn nyash_plugin_invoke_tagged_v_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    recv_h: i64,
    vals: *const i64,
    tags: *const i64,
) -> i64 {
    let trace = std::env::var("NYASH_LLVM_VINVOKE_TRACE").ok().as_deref() == Some("1");
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    if recv_h <= 0 { return 0; }
    // Resolve receiver invoke
    let mut instance_id: u32 = 0;
    let mut real_type_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if let Some(obj) = nyash_rust::jit::rt::handles::get(recv_h as u64) {
        if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
            instance_id = p.instance_id();
            real_type_id = p.inner.type_id;
            invoke = Some(p.inner.invoke_fn);
        }
    }
    if trace {
        eprintln!(
            "nyrt: vinvoke.by_id: type_id={} method_id={} recv_h={} argc={} vals_ptr={:p} tags_ptr={:p}",
            type_id, method_id, recv_h, argc, vals, tags
        );
    }
    if invoke.is_none() { return 0; }
    let nargs = argc.saturating_sub(1).max(0) as usize;
    let (vals, tags) = if nargs > 0 && !vals.is_null() && !tags.is_null() {
        unsafe {
            (std::slice::from_raw_parts(vals, nargs), std::slice::from_raw_parts(tags, nargs))
        }
    } else { (&[][..], &[][..]) };
    if trace {
        let sample = std::cmp::min(nargs, 8);
        eprintln!(
            "nyrt: vinvoke.by_id: real_type_id={} nargs={} tags[..{}]={:?}",
            real_type_id, nargs, sample, &tags[..sample]
        );
    }

    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(nargs as u16);
    for i in 0..nargs {
        match tags[i] {
            3 => nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, vals[i]),
            5 => { let f = f64::from_bits(vals[i] as u64); nyash_rust::runtime::plugin_ffi_common::encode::f64(&mut buf, f); },
            8 => {
                if let Some(obj) = nyash_rust::jit::rt::handles::get(vals[i] as u64) {
                    if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                        nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
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
    let mut out = vec![0u8; 1024]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(real_type_id, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if trace { eprintln!("nyrt: vinvoke.by_id: rc={} out_len={}", rc, out_len); }
    if rc != 0 { return 0; }
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        if trace { eprintln!("nyrt: vinvoke.by_id: ret_tag={}", tag); }
        match tag {
            3 => { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); } }
            1 => { return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            8 => { if payload.len()==8 { let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]); let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]); let r_type=u32::from_le_bytes(t); let r_inst=u32::from_le_bytes(i); let pb=nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2("PluginBox".into(), r_type, r_inst, invoke.unwrap()); let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox>=std::sync::Arc::new(pb); let h=nyash_rust::jit::rt::handles::to_handle(arc); return h as i64; } }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => {}
        }
    }
    0
}

// imports
use std::ffi::CStr;
use std::os::raw::c_char as i8;
