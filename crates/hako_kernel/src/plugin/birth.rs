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
    if let Some(meta) = nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(tid) {
        if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
            if let Ok(b) = host_g.create_box(&meta.box_type, &[]) {
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                    std::sync::Arc::from(b);
                let h = nyash_rust::runtime::host_handles::to_handle_arc(arc);
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!(
                        "nyrt: birth_h {} (type_id={}) -> handle={}",
                        meta.box_type, meta.type_id, h
                    );
                }
                return h as i64;
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "nyrt: birth_h {} (type_id={}) FAILED: create_box",
                    meta.box_type, tid
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
    // Resolve invoke_fn via loader metadata
    let meta = if let Some(meta) =
        nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(type_id as u32)
    {
        meta
    } else {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("nyrt: birth_i64 (type_id={}) FAILED: type map", type_id);
        }
        return 0;
    };
    let box_type_name = meta.box_type.clone();
    let invoke_fn = meta.invoke_fn;
    let method_id: u32 = 0; // birth
    let instance_id: u32 = 0; // static
                              // Build TLV args
    use nyash_rust::runtime::host_handles as handles;
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
    // ✂️ REMOVED: Legacy VM argument processing for args 3+
    // In Plugin-First architecture, birth functions are limited to 2 explicit arguments
    // Extended argument support removed with legacy VM system archival
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
                invoke_fn,
            );
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::new(pb);
            let h = nyash_rust::runtime::host_handles::to_handle_arc(arc);
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
