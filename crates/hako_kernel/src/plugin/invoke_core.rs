use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;

/// Thin shared helpers for plugin invoke shims (i64/f64)
///
/// Goal: centralize receiver resolution and the dynamic buffer call loop,
/// keeping extern functions in invoke.rs small and consistent.

pub struct Receiver {
    pub instance_id: u32,
    pub real_type_id: u32,
    pub invoke: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
}

/// Resolve receiver from a0: prefer handle registry; fallback to legacy VM args when allowed.
pub fn resolve_receiver_for_a0(a0: i64) -> Option<Receiver> {
    // 1) Handle registry (preferred)
    if a0 > 0 {
        if let Some(obj) = nyash_rust::runtime::host_handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                return Some(Receiver {
                    instance_id: p.instance_id(),
                    real_type_id: p.inner.type_id,
                    invoke: p.inner.invoke_fn,
                });
            }
        }
    }
    // ✂️ REMOVED: Legacy VM argument receiver resolution
    // Plugin-First architecture requires explicit handle-based receiver resolution only
    None
}

/// Call plugin invoke with dynamic buffer growth, returning first TLV entry on success.
pub fn plugin_invoke_call(
    invoke: unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32,
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    tlv_args: &[u8],
) -> Option<(u8, usize, Vec<u8>)> {
    let mut cap: usize = 256;
    let mut tag_ret: u8 = 0;
    let mut sz_ret: usize = 0;
    let mut payload_ret: Vec<u8> = Vec::new();
    loop {
        let mut out = vec![0u8; cap];
        let mut out_len: usize = out.len();
        let rc = unsafe {
            invoke(
                type_id,
                method_id,
                instance_id,
                tlv_args.as_ptr(),
                tlv_args.len(),
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
            return None;
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
        return None;
    }
    Some((tag_ret, sz_ret, payload_ret))
}

/// Decode a single TLV entry to i64 with side-effects (handle registration) when applicable.
pub fn decode_entry_to_i64(
    tag: u8,
    sz: usize,
    payload: &[u8],
    fallback_invoke: unsafe extern "C" fn(
        u32,
        u32,
        u32,
        *const u8,
        usize,
        *mut u8,
        *mut usize,
    ) -> i32,
) -> Option<i64> {
    match tag {
        2 => nyash_rust::runtime::plugin_ffi_common::decode::i32(payload).map(|v| v as i64),
        3 => {
            if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                return Some(v as i64);
            }
            if payload.len() == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                return Some(i64::from_le_bytes(b));
            }
            None
        }
        6 | 7 => {
            use nyash_rust::box_trait::{NyashBox, StringBox};
            let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
            let h = nyash_rust::runtime::host_handles::to_handle_arc(arc);
            Some(h as i64)
        }
        8 => {
            if sz == 8 {
                let mut t = [0u8; 4];
                t.copy_from_slice(&payload[0..4]);
                let mut i = [0u8; 4];
                i.copy_from_slice(&payload[4..8]);
                let r_type = u32::from_le_bytes(t);
                let r_inst = u32::from_le_bytes(i);
                // Use metadata if available to set box_type/invoke_fn
                let meta_opt = nyash_rust::runtime::plugin_loader_v2::metadata_for_type_id(r_type);
                let (box_type_name, invoke_ptr) = if let Some(meta) = meta_opt {
                    (meta.box_type.clone(), meta.invoke_fn)
                } else {
                    ("PluginBox".to_string(), fallback_invoke)
                };
                let pb = nyash_rust::runtime::plugin_loader_v2::make_plugin_box_v2(
                    box_type_name,
                    r_type,
                    r_inst,
                    invoke_ptr,
                );
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                    std::sync::Arc::new(pb);
                let h = nyash_rust::runtime::host_handles::to_handle_arc(arc);
                return Some(h as i64);
            }
            None
        }
        1 => {
            nyash_rust::runtime::plugin_ffi_common::decode::bool(payload)
                .map(|b| if b { 1 } else { 0 })
        }
        5 => {
            if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") && sz == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                let f = f64::from_le_bytes(b);
                return Some(f as i64);
            }
            None
        }
        _ => None,
    }
}

/// Decode a single TLV entry to f64 when possible.
pub fn decode_entry_to_f64(tag: u8, sz: usize, payload: &[u8]) -> Option<f64> {
    match tag {
        5 => {
            if sz == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                Some(f64::from_le_bytes(b))
            } else {
                None
            }
        }
        3 => {
            if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) {
                return Some(v as f64);
            }
            if payload.len() == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                return Some((i64::from_le_bytes(b)) as f64);
            }
            None
        }
        1 => nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).map(|b| {
            if b {
                1.0
            } else {
                0.0
            }
        }),
        _ => None,
    }
}
