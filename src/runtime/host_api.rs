#![allow(unexpected_cfgs)]
/*!
 * Host reverse-call API for plugins (Phase 12 / A-1)
 *
 * - Provides C ABI functions that plugins can call to operate on HostHandle (user/builtin boxes) via TLV.
 * - Minimal supported methods: InstanceBox.getField/setField, ArrayBox.get/set
 * - GC correctness: setField/Array.set triggers write barrier using current VM's runtime (TLS-bound during plugin calls).
 */

use crate::box_trait::NyashBox;

// ===== TLS: current VM pointer during plugin invoke =====
// VM-legacy feature removed - provide stubs only
pub fn set_current_vm(_ptr: *mut ()) {}
pub fn clear_current_vm() {}

// ===== Utilities: TLV encode helpers (single-value) =====
fn tlv_encode_one(val: &crate::backend::vm::VMValue) -> Vec<u8> {
    use crate::runtime::plugin_ffi_common as tlv;
    let mut buf = tlv::encode_tlv_header(1);
    match val {
        crate::backend::vm::VMValue::Integer(i) => tlv::encode::i64(&mut buf, *i),
        crate::backend::vm::VMValue::Float(f) => tlv::encode::f64(&mut buf, *f),
        crate::backend::vm::VMValue::Bool(b) => tlv::encode::bool(&mut buf, *b),
        crate::backend::vm::VMValue::String(s) => tlv::encode::string(&mut buf, s),
        crate::backend::vm::VMValue::BoxRef(b) => {
            // Return HostHandle for arbitrary boxes
            let h = crate::runtime::host_handles::to_handle_arc(b.clone());
            tlv::encode::host_handle(&mut buf, h);
        }
        _ => tlv::encode::string(&mut buf, "void"),
    }
    buf
}

fn vmvalue_from_tlv(tag: u8, payload: &[u8]) -> Option<crate::backend::vm::VMValue> {
    use crate::runtime::plugin_ffi_common as tlv;
    match tag {
        1 => Some(crate::backend::vm::VMValue::Bool(
            tlv::decode::bool(payload).unwrap_or(false),
        )),
        2 => tlv::decode::i32(payload).map(|v| crate::backend::vm::VMValue::Integer(v as i64)),
        3 => {
            if payload.len() == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(payload);
                Some(crate::backend::vm::VMValue::Integer(i64::from_le_bytes(b)))
            } else {
                None
            }
        }
        5 => tlv::decode::f64(payload).map(crate::backend::vm::VMValue::Float),
        6 | 7 => Some(crate::backend::vm::VMValue::String(tlv::decode::string(
            payload,
        ))),
        8 => {
            // PluginHandle(type_id, instance_id) → reconstruct PluginBoxV2 (when plugins enabled)
            if let Some((type_id, instance_id)) = tlv::decode::plugin_handle(payload) {
                if let Some(arc) = plugin_box_from_handle(type_id, instance_id) {
                    return Some(crate::backend::vm::VMValue::BoxRef(arc));
                }
            }
            None
        }
        9 => {
            if let Some(h) = tlv::decode::u64(payload) {
                crate::runtime::host_handles::get(h).map(crate::backend::vm::VMValue::BoxRef)
            } else {
                None
            }
        }
        _ => None,
    }
}

unsafe fn slice_from_raw<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    std::slice::from_raw_parts(ptr, len)
}
unsafe fn slice_from_raw_mut<'a>(ptr: *mut u8, len: usize) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(ptr, len)
}

fn encode_out(out_ptr: *mut u8, out_len: *mut usize, buf: &[u8]) -> i32 {
    unsafe {
        if out_ptr.is_null() || out_len.is_null() {
            return -2;
        }
        let cap = *out_len;
        if cap < buf.len() {
            return -3;
        }
        let out = slice_from_raw_mut(out_ptr, cap);
        out[..buf.len()].copy_from_slice(buf);
        *out_len = buf.len();
        0
    }
}

#[cfg_attr(all(not(test), any(feature = "c-abi-export", export_host_c_abi)), no_mangle)]
pub extern "C" fn nyrt_host_call_name(
    handle: u64,
    method_ptr: *const u8,
    method_len: usize,
    args_ptr: *const u8,
    args_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> i32 {
    // Resolve receiver
    let recv_arc = match crate::runtime::host_handles::get(handle) {
        Some(a) => a,
        None => return -1,
    };
    let method =
        unsafe { std::str::from_utf8(slice_from_raw(method_ptr, method_len)).unwrap_or("") }
            .to_string();
    // Parse TLV args (header + entries)
    let mut argv: Vec<crate::backend::vm::VMValue> = Vec::new();
    if args_ptr.is_null() || args_len < 4 { /* no args */
    } else {
        let buf = unsafe { slice_from_raw(args_ptr, args_len) };
        // Iterate entries
        let mut off = 4usize;
        while buf.len() >= off + 4 {
            let tag = buf[off];
            let _rsv = buf[off + 1];
            let sz = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
            if buf.len() < off + 4 + sz {
                break;
            }
            let payload = &buf[off + 4..off + 4 + sz];
            if let Some(v) = vmvalue_from_tlv(tag, payload) {
                argv.push(v);
            }
            off += 4 + sz;
        }
    }
    // Dispatch minimal supported methods
    // InstanceBox getField/setField
    if let Some(inst) = recv_arc
        .as_any()
        .downcast_ref::<crate::instance_v2::InstanceBox>()
    {
        match method.as_str() {
            "getField" if argv.len() >= 1 => {
                let field = match &argv[0] {
                    crate::backend::vm::VMValue::String(s) => s.clone(),
                    v => v.to_string(),
                };
                let out = inst
                    .get_field_unified(&field)
                    .map(|nv| match nv {
                        crate::value::NyashValue::Integer(i) => {
                            crate::backend::vm::VMValue::Integer(i)
                        }
                        crate::value::NyashValue::Float(f) => crate::backend::vm::VMValue::Float(f),
                        crate::value::NyashValue::Bool(b) => crate::backend::vm::VMValue::Bool(b),
                        crate::value::NyashValue::String(s) => {
                            crate::backend::vm::VMValue::String(s)
                        }
                        crate::value::NyashValue::Void | crate::value::NyashValue::Null => {
                            crate::backend::vm::VMValue::String("".to_string())
                        }
                        crate::value::NyashValue::Box(b) => {
                            if let Ok(g) = b.lock() {
                                crate::backend::vm::VMValue::BoxRef(std::sync::Arc::from(
                                    g.share_box(),
                                ))
                            } else {
                                crate::backend::vm::VMValue::String("".to_string())
                            }
                        }
                        _ => crate::backend::vm::VMValue::String("".to_string()),
                    })
                    .unwrap_or(crate::backend::vm::VMValue::String("".to_string()));
                let buf = tlv_encode_one(&out);
                return encode_out(out_ptr, out_len, &buf);
            }
            "setField" if argv.len() >= 2 => {
                let field = match &argv[0] {
                    crate::backend::vm::VMValue::String(s) => s.clone(),
                    v => v.to_string(),
                };
                // GC barrier (Write) — revive minimal barrier on host setField
                crate::runtime::global_hooks::gc_barrier(crate::runtime::gc::BarrierKind::Write);
                // Accept primitives only for now
                let nv_opt = match argv[1].clone() {
                    crate::backend::vm::VMValue::Integer(i) => {
                        Some(crate::value::NyashValue::Integer(i))
                    }
                    crate::backend::vm::VMValue::Float(f) => {
                        Some(crate::value::NyashValue::Float(f))
                    }
                    crate::backend::vm::VMValue::Bool(b) => Some(crate::value::NyashValue::Bool(b)),
                    crate::backend::vm::VMValue::String(s) => {
                        Some(crate::value::NyashValue::String(s))
                    }
                    crate::backend::vm::VMValue::BoxRef(_) => None,
                    _ => None,
                };
                if let Some(nv) = nv_opt {
                    let _ = inst.set_field_unified(field, nv);
                }
                let buf = tlv_encode_one(&crate::backend::vm::VMValue::Bool(true));
                return encode_out(out_ptr, out_len, &buf);
            }
            _ => {}
        }
    }
    // ArrayBox get/set
    if let Some(arr) = recv_arc
        .as_any()
        .downcast_ref::<crate::boxes::array::ArrayBox>()
    {
        match method.as_str() {
            "get" if argv.len() >= 1 => {
                let idx = match argv[0].clone() {
                    crate::backend::vm::VMValue::Integer(i) => i,
                    v => v.to_string().parse::<i64>().unwrap_or(0),
                };
                let out = arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
                let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                let buf = tlv_encode_one(&vmv);
                return encode_out(out_ptr, out_len, &buf);
            }
            "set" if argv.len() >= 2 => {
                let idx = match argv[0].clone() {
                    crate::backend::vm::VMValue::Integer(i) => i,
                    v => v.to_string().parse::<i64>().unwrap_or(0),
                };
                let vb = match argv[1].clone() {
                    crate::backend::vm::VMValue::Integer(i) => {
                        Box::new(crate::box_trait::IntegerBox::new(i)) as Box<dyn NyashBox>
                    }
                    crate::backend::vm::VMValue::Float(f) => {
                        Box::new(crate::boxes::math_box::FloatBox::new(f))
                    }
                    crate::backend::vm::VMValue::Bool(b) => {
                        Box::new(crate::box_trait::BoolBox::new(b))
                    }
                    crate::backend::vm::VMValue::String(s) => {
                        Box::new(crate::box_trait::StringBox::new(s))
                    }
                    crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                    _ => Box::new(crate::box_trait::VoidBox::new()),
                };
                // VM-legacy removed - no GC barrier needed
                let out = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), vb);
                let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                let buf = tlv_encode_one(&vmv);
                return encode_out(out_ptr, out_len, &buf);
            }
            _ => {}
        }
    }
    // Unsupported
    -10
}

// Helper: reconstruct PluginBoxV2 from (type_id, instance_id) when plugins are enabled
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
fn plugin_box_from_handle(type_id: u32, instance_id: u32) -> Option<std::sync::Arc<dyn NyashBox>> {
    let loader = crate::runtime::plugin_loader_v2::get_global_loader_v2();
    let loader = loader.read().ok()?;
    let bx = loader.construct_existing_instance(type_id, instance_id)?;
    Some(std::sync::Arc::from(bx))
}
#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
fn plugin_box_from_handle(
    _type_id: u32,
    _instance_id: u32,
) -> Option<std::sync::Arc<dyn NyashBox>> {
    None
}

// ---- by-slot variant (selector_id: u64) ----
// Minimal slot mapping (subject to consolidation with TypeRegistry):
// 1: InstanceBox.getField(name: string) -> any
// 2: InstanceBox.setField(name: string, value: any-primitive) -> bool
// 3: InstanceBox.has(name: string) -> bool
// 4: InstanceBox.size() -> i64
// 100: ArrayBox.get(index: i64) -> any
// 101: ArrayBox.set(index: i64, value: any) -> any
// 102: ArrayBox.len() -> i64
// 200: MapBox.size() -> i64
// 201: MapBox.len() -> i64
// 202: MapBox.has(key:any) -> bool
// 203: MapBox.get(key:any) -> any
// 204: MapBox.set(key:any, value:any) -> any
// 300: StringBox.len() -> i64
#[cfg_attr(all(not(test), any(feature = "c-abi-export", export_host_c_abi)), no_mangle)]
pub extern "C" fn nyrt_host_call_slot(
    handle: u64,
    selector_id: u64,
    args_ptr: *const u8,
    args_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> i32 {
    let recv_arc = match crate::runtime::host_handles::get(handle) {
        Some(a) => a,
        None => return -1,
    };
    // Parse TLV args
    let mut argv: Vec<crate::backend::vm::VMValue> = Vec::new();
    if !args_ptr.is_null() && args_len >= 4 {
        let buf = unsafe { slice_from_raw(args_ptr, args_len) };
        let mut off = 4usize;
        while buf.len() >= off + 4 {
            let tag = buf[off];
            let sz = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
            if buf.len() < off + 4 + sz {
                break;
            }
            let payload = &buf[off + 4..off + 4 + sz];
            if let Some(v) = vmvalue_from_tlv(tag, payload) {
                argv.push(v);
            }
            off += 4 + sz;
        }
    }

    match selector_id {
        1 | 2 | 3 | 4 => {
            if let Some(inst) = recv_arc
                .as_any()
                .downcast_ref::<crate::instance_v2::InstanceBox>()
            {
                if selector_id == 1 {
                    // getField(name)
                    if argv.len() >= 1 {
                        let field = match &argv[0] {
                            crate::backend::vm::VMValue::String(s) => s.clone(),
                            v => v.to_string(),
                        };
                        let out = inst
                            .get_field_unified(&field)
                            .map(|nv| match nv {
                                crate::value::NyashValue::Integer(i) => {
                                    crate::backend::vm::VMValue::Integer(i)
                                }
                                crate::value::NyashValue::Float(f) => {
                                    crate::backend::vm::VMValue::Float(f)
                                }
                                crate::value::NyashValue::Bool(b) => {
                                    crate::backend::vm::VMValue::Bool(b)
                                }
                                crate::value::NyashValue::String(s) => {
                                    crate::backend::vm::VMValue::String(s)
                                }
                                crate::value::NyashValue::Void | crate::value::NyashValue::Null => {
                                    crate::backend::vm::VMValue::String("".to_string())
                                }
                                crate::value::NyashValue::Box(b) => {
                                    if let Ok(g) = b.lock() {
                                        crate::backend::vm::VMValue::BoxRef(std::sync::Arc::from(
                                            g.share_box(),
                                        ))
                                    } else {
                                        crate::backend::vm::VMValue::String("".to_string())
                                    }
                                }
                                _ => crate::backend::vm::VMValue::String("".to_string()),
                            })
                            .unwrap_or(crate::backend::vm::VMValue::String("".to_string()));
                        let buf = tlv_encode_one(&out);
                        return encode_out(out_ptr, out_len, &buf);
                    }
                } else if selector_id == 2 {
                    // setField(name, value)
                    if argv.len() >= 2 {
                        let field = match &argv[0] {
                            crate::backend::vm::VMValue::String(s) => s.clone(),
                            v => v.to_string(),
                        };
                        // VM-legacy removed - no GC barrier needed
                        let nv_opt = match argv[1].clone() {
                            crate::backend::vm::VMValue::Integer(i) => {
                                Some(crate::value::NyashValue::Integer(i))
                            }
                            crate::backend::vm::VMValue::Float(f) => {
                                Some(crate::value::NyashValue::Float(f))
                            }
                            crate::backend::vm::VMValue::Bool(b) => {
                                Some(crate::value::NyashValue::Bool(b))
                            }
                            crate::backend::vm::VMValue::String(s) => {
                                Some(crate::value::NyashValue::String(s))
                            }
                            crate::backend::vm::VMValue::BoxRef(_b) => {
                                // Do not store BoxRef into unified map in this minimal host path
                                None
                            }
                            _ => None,
                        };
                        if let Some(nv) = nv_opt {
                            let _ = inst.set_field_unified(field, nv);
                        }
                        let buf = tlv_encode_one(&crate::backend::vm::VMValue::Bool(true));
                        return encode_out(out_ptr, out_len, &buf);
                    }
                } else if selector_id == 3 {
                    // has(name)
                    if argv.len() >= 1 {
                        let field = match &argv[0] {
                            crate::backend::vm::VMValue::String(s) => s.clone(),
                            v => v.to_string(),
                        };
                        let has = inst.get_field_unified(&field).is_some();
                        let buf = tlv_encode_one(&crate::backend::vm::VMValue::Bool(has));
                        return encode_out(out_ptr, out_len, &buf);
                    }
                } else if selector_id == 4 {
                    // size()
                    let sz = inst.fields_ng.lock().map(|m| m.len() as i64).unwrap_or(0);
                    let buf = tlv_encode_one(&crate::backend::vm::VMValue::Integer(sz));
                    return encode_out(out_ptr, out_len, &buf);
                }
            }
        }
        100 | 101 | 102 => {
            if let Some(arr) = recv_arc
                .as_any()
                .downcast_ref::<crate::boxes::array::ArrayBox>()
            {
                match selector_id {
                    100 => {
                        // get(index)
                        if argv.len() >= 1 {
                            let idx = match argv[0].clone() {
                                crate::backend::vm::VMValue::Integer(i) => i,
                                v => v.to_string().parse::<i64>().unwrap_or(0),
                            };
                            let out = arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                            let buf = tlv_encode_one(&vmv);
                            return encode_out(out_ptr, out_len, &buf);
                        }
                    }
                    101 => {
                        // set(index, value)
                        if argv.len() >= 2 {
                            let idx = match argv[0].clone() {
                                crate::backend::vm::VMValue::Integer(i) => i,
                                v => v.to_string().parse::<i64>().unwrap_or(0),
                            };
                            let vb = match argv[1].clone() {
                                crate::backend::vm::VMValue::Integer(i) => {
                                    Box::new(crate::box_trait::IntegerBox::new(i))
                                        as Box<dyn NyashBox>
                                }
                                crate::backend::vm::VMValue::Float(f) => {
                                    Box::new(crate::boxes::math_box::FloatBox::new(f))
                                }
                                crate::backend::vm::VMValue::Bool(b) => {
                                    Box::new(crate::box_trait::BoolBox::new(b))
                                }
                                crate::backend::vm::VMValue::String(s) => {
                                    Box::new(crate::box_trait::StringBox::new(s))
                                }
                                crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                                _ => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            // VM-legacy removed - no GC barrier needed
                            let out = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), vb);
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                            let buf = tlv_encode_one(&vmv);
                            return encode_out(out_ptr, out_len, &buf);
                        }
                    }
                    102 => {
                        // len()
                        let len = arr.len();
                        let buf = tlv_encode_one(&crate::backend::vm::VMValue::Integer(len as i64));
                        return encode_out(out_ptr, out_len, &buf);
                    }
                    _ => {}
                }
            }
        }
        200 | 201 | 202 | 203 | 204 => {
            if let Some(map) = recv_arc
                .as_any()
                .downcast_ref::<crate::boxes::map_box::MapBox>()
            {
                match selector_id {
                    200 | 201 => {
                        let out = map.size();
                        let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                        let buf = tlv_encode_one(&vmv);
                        return encode_out(out_ptr, out_len, &buf);
                    }
                    202 => {
                        // has(key)
                        if argv.len() >= 1 {
                            let key_box: Box<dyn NyashBox> = match argv[0].clone() {
                                crate::backend::vm::VMValue::Integer(i) => {
                                    Box::new(crate::box_trait::IntegerBox::new(i))
                                }
                                crate::backend::vm::VMValue::Float(f) => {
                                    Box::new(crate::boxes::math_box::FloatBox::new(f))
                                }
                                crate::backend::vm::VMValue::Bool(b) => {
                                    Box::new(crate::box_trait::BoolBox::new(b))
                                }
                                crate::backend::vm::VMValue::String(s) => {
                                    Box::new(crate::box_trait::StringBox::new(s))
                                }
                                crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                                crate::backend::vm::VMValue::Future(fu) => Box::new(fu),
                                crate::backend::vm::VMValue::Void => {
                                    Box::new(crate::box_trait::VoidBox::new())
                                }
                            };
                            let out = map.has(key_box);
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                            let buf = tlv_encode_one(&vmv);
                            return encode_out(out_ptr, out_len, &buf);
                        }
                    }
                    203 => {
                        // get(key)
                        if argv.len() >= 1 {
                            let key_box: Box<dyn NyashBox> = match argv[0].clone() {
                                crate::backend::vm::VMValue::Integer(i) => {
                                    Box::new(crate::box_trait::IntegerBox::new(i))
                                }
                                crate::backend::vm::VMValue::Float(f) => {
                                    Box::new(crate::boxes::math_box::FloatBox::new(f))
                                }
                                crate::backend::vm::VMValue::Bool(b) => {
                                    Box::new(crate::box_trait::BoolBox::new(b))
                                }
                                crate::backend::vm::VMValue::String(s) => {
                                    Box::new(crate::box_trait::StringBox::new(s))
                                }
                                crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                                crate::backend::vm::VMValue::Future(fu) => Box::new(fu),
                                crate::backend::vm::VMValue::Void => {
                                    Box::new(crate::box_trait::VoidBox::new())
                                }
                            };
                            let out = map.get(key_box);
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                            let buf = tlv_encode_one(&vmv);
                            return encode_out(out_ptr, out_len, &buf);
                        }
                    }
                    204 => {
                        // set(key, value)
                        if argv.len() >= 2 {
                            let key_box: Box<dyn NyashBox> = match argv[0].clone() {
                                crate::backend::vm::VMValue::Integer(i) => {
                                    Box::new(crate::box_trait::IntegerBox::new(i))
                                }
                                crate::backend::vm::VMValue::Float(f) => {
                                    Box::new(crate::boxes::math_box::FloatBox::new(f))
                                }
                                crate::backend::vm::VMValue::Bool(b) => {
                                    Box::new(crate::box_trait::BoolBox::new(b))
                                }
                                crate::backend::vm::VMValue::String(s) => {
                                    Box::new(crate::box_trait::StringBox::new(s))
                                }
                                crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                                crate::backend::vm::VMValue::Future(fu) => Box::new(fu),
                                crate::backend::vm::VMValue::Void => {
                                    Box::new(crate::box_trait::VoidBox::new())
                                }
                            };
                            let val_box: Box<dyn NyashBox> = match argv[1].clone() {
                                crate::backend::vm::VMValue::Integer(i) => {
                                    Box::new(crate::box_trait::IntegerBox::new(i))
                                }
                                crate::backend::vm::VMValue::Float(f) => {
                                    Box::new(crate::boxes::math_box::FloatBox::new(f))
                                }
                                crate::backend::vm::VMValue::Bool(b) => {
                                    Box::new(crate::box_trait::BoolBox::new(b))
                                }
                                crate::backend::vm::VMValue::String(s) => {
                                    Box::new(crate::box_trait::StringBox::new(s))
                                }
                                crate::backend::vm::VMValue::BoxRef(b) => b.share_box(),
                                crate::backend::vm::VMValue::Future(fu) => Box::new(fu),
                                crate::backend::vm::VMValue::Void => {
                                    Box::new(crate::box_trait::VoidBox::new())
                                }
                            };
                            let out = map.set(key_box, val_box);
                            let vmv = crate::backend::vm::VMValue::from_nyash_box(out);
                            let buf = tlv_encode_one(&vmv);
                            return encode_out(out_ptr, out_len, &buf);
                        }
                    }
                    _ => {}
                }
            }
        }
        300 => {
            if let Some(sb) = recv_arc
                .as_any()
                .downcast_ref::<crate::box_trait::StringBox>()
            {
                let out = crate::backend::vm::VMValue::Integer(sb.value.len() as i64);
                let buf = tlv_encode_one(&out);
                return encode_out(out_ptr, out_len, &buf);
            }
        }
        _ => {}
    }
    -10
}
