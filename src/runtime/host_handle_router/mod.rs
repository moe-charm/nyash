//! HostHandleRouterBox
//!
//! Responsibility
//! - Centralize HostHandle slot dispatch (Array/Map/Instance minimal methods)
//! - Keep `nyrt_host_call_slot` in host_api.rs thin by delegating here
//!
//! Minimal API: mirrors `nyrt_host_call_slot` signature

use crate::box_trait::NyashBox;
use crate::backend::vm_types::VMValue;

pub extern "C" fn route_slot(
    handle: u64,
    selector_id: u64,
    args_ptr: *const u8,
    args_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> i32 {
    // For now, delegate to the existing implementation to avoid behavior drift.
    // Follow-up patches can migrate concrete branches here.
    
    // Resolve receiver
    let recv_arc = match crate::runtime::host_handles::get(handle) {
        Some(a) => a,
        None => return -1,
    };

    // Parse TLV args (reuse host_api helpers)
    let mut argv: Vec<VMValue> = Vec::new();
    if !args_ptr.is_null() && args_len >= 4 {
        let buf = unsafe { crate::runtime::host_api::slice_from_raw(args_ptr, args_len) };
        let mut off = 4usize;
        while buf.len() >= off + 4 {
            let tag = buf[off];
            let sz = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
            if buf.len() < off + 4 + sz { break; }
            let payload = &buf[off + 4..off + 4 + sz];
            if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { argv.push(v); }
            off += 4 + sz;
        }
    }

    // Plugin Box fast path by selector→method name
    if let Some(pb) = recv_arc.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
        let method = match selector_id {
            100 => Some("get"), 101 => Some("set"), 102 => Some("size"),
            200 => Some("size"), 201 => Some("len"), 202 => Some("has"),
            203 => Some("get"), 204 => Some("set"),
            300 => Some("len"),
            _ => None,
        };
        if let Some(name) = method {
            let mut args_boxes: Vec<Box<dyn NyashBox>> = Vec::with_capacity(argv.len());
            for v in &argv { args_boxes.push(v.to_nyash_box()); }
            match crate::runtime::plugin_host_box::invoke_instance_method(&pb.box_type, name, pb.instance_id(), &args_boxes) {
                Ok(Some(ret)) => {
                    let vmv = VMValue::from_nyash_box(ret);
                    let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                    return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                }
                Ok(None) => {
                    let buf = crate::runtime::host_api::tlv_encode_one(&VMValue::Void);
                    return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                }
                Err(_) => { return -5; }
            }
        }
    }

    // InstanceBox (1..4)
    if matches!(selector_id, 1|2|3|4) {
        if let Some(inst) = recv_arc.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
            match selector_id {
                1 => { // getField(name)
                    if argv.len() >= 1 {
                        let field = match &argv[0] { VMValue::String(s) => s.clone(), v => v.to_string(), };
                        let out = inst.get_field_unified(&field).map(|nv| match nv {
                            crate::value::NyashValue::Integer(i) => VMValue::Integer(i),
                            crate::value::NyashValue::Float(f) => VMValue::Float(f),
                            crate::value::NyashValue::Bool(b) => VMValue::Bool(b),
                            crate::value::NyashValue::String(s) => VMValue::String(s),
                            crate::value::NyashValue::Void|crate::value::NyashValue::Null => VMValue::String(String::new()),
                            crate::value::NyashValue::Box(b) => {
                                if let Ok(g)=b.lock() { VMValue::BoxRef(std::sync::Arc::from(g.share_box())) } else { VMValue::String(String::new()) }
                            }
                            _ => VMValue::String(String::new()),
                        }).unwrap_or(VMValue::String(String::new()));
                        let buf = crate::runtime::host_api::tlv_encode_one(&out);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                2 => { // setField(name, value)
                    if argv.len() >= 2 {
                        let field = match &argv[0] { VMValue::String(s)=>s.clone(), v=>v.to_string() };
                        let nv_opt = match argv[1].clone() {
                            VMValue::Integer(i)=>Some(crate::value::NyashValue::Integer(i)),
                            VMValue::Float(f)=>Some(crate::value::NyashValue::Float(f)),
                            VMValue::Bool(b)=>Some(crate::value::NyashValue::Bool(b)),
                            VMValue::String(s)=>Some(crate::value::NyashValue::String(s)),
                            VMValue::BoxRef(_) | VMValue::Future(_) | VMValue::Void => None,
                        };
                        if let Some(nv)=nv_opt { let _=inst.set_field_unified(field, nv); }
                        let buf = crate::runtime::host_api::tlv_encode_one(&VMValue::Bool(true));
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                3 => { // has(name)
                    if argv.len() >= 1 {
                        let field = match &argv[0] { VMValue::String(s)=>s.clone(), v=>v.to_string() };
                        let has = inst.get_field_unified(&field).is_some();
                        let buf = crate::runtime::host_api::tlv_encode_one(&VMValue::Bool(has));
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                4 => { // size()
                    let sz = inst.fields_ng.lock().map(|m| m.len() as i64).unwrap_or(0);
                    let buf = crate::runtime::host_api::tlv_encode_one(&VMValue::Integer(sz));
                    return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                }
                _ => {}
            }
        }
    }

    // Builtin Array (100..102)
    if matches!(selector_id, 100|101|102) {
        if let Some(arr) = recv_arc.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match selector_id {
                100 => { // get(index)
                    if let Some(VMValue::Integer(i)) = argv.get(0) {
                        let out = arr.get(Box::new(crate::box_trait::IntegerBox::new(*i)));
                        let vmv = VMValue::from_nyash_box(out);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                101 => { // set(index, value)
                    if argv.len() >= 2 {
                        let idx = match argv[0].clone() { VMValue::Integer(i)=>i, v=>v.to_string().parse::<i64>().unwrap_or(0) };
                        let vb: Box<dyn NyashBox> = match argv[1].clone() {
                            VMValue::Integer(i)=>Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f)=>Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b)=>Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s)=>Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b)=>b.share_box(),
                            _=>Box::new(crate::box_trait::VoidBox::new()),
                        };
                        let out = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), vb);
                        let vmv = VMValue::from_nyash_box(out);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                102 => { // len()
                    let len = arr.len();
                    let buf = crate::runtime::host_api::tlv_encode_one(&VMValue::Integer(len as i64));
                    return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                }
                _ => {}
            }
        }
    }

    // Builtin Map (200..204)
    if matches!(selector_id, 200|201|202|203|204) {
        if let Some(map) = recv_arc.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            match selector_id {
                200 | 201 => {
                    let out = map.size();
                    let vmv = VMValue::from_nyash_box(out);
                    let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                    return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                }
                202 => { // has(key)
                    if let Some(k) = argv.get(0) {
                        let key_box: Box<dyn NyashBox> = match k.clone() {
                            VMValue::Integer(i)=>Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f)=>Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b)=>Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s)=>Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b)=>b.share_box(),
                            VMValue::Future(fu)=>Box::new(fu),
                            VMValue::Void=>Box::new(crate::box_trait::VoidBox::new()),
                        };
                        let out = map.has(key_box);
                        let vmv = VMValue::from_nyash_box(out);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                203 => { // get(key)
                    if let Some(k) = argv.get(0) {
                        let key_box: Box<dyn NyashBox> = match k.clone() {
                            VMValue::Integer(i)=>Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f)=>Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b)=>Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s)=>Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b)=>b.share_box(),
                            VMValue::Future(fu)=>Box::new(fu),
                            VMValue::Void=>Box::new(crate::box_trait::VoidBox::new()),
                        };
                        let out = map.get(key_box);
                        let vmv = VMValue::from_nyash_box(out);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                204 => { // set(key, value)
                    if argv.len() >= 2 {
                        let key_box: Box<dyn NyashBox> = match argv[0].clone() {
                            VMValue::Integer(i)=>Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f)=>Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b)=>Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s)=>Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b)=>b.share_box(),
                            VMValue::Future(fu)=>Box::new(fu),
                            VMValue::Void=>Box::new(crate::box_trait::VoidBox::new()),
                        };
                        let val_box: Box<dyn NyashBox> = match argv[1].clone() {
                            VMValue::Integer(i)=>Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f)=>Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b)=>Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s)=>Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b)=>b.share_box(),
                            VMValue::Future(fu)=>Box::new(fu),
                            VMValue::Void=>Box::new(crate::box_trait::VoidBox::new()),
                        };
                        let out = map.set(key_box, val_box);
                        let vmv = VMValue::from_nyash_box(out);
                        let buf = crate::runtime::host_api::tlv_encode_one(&vmv);
                        return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
                    }
                }
                _ => {}
            }
        }
    }

    // Builtin String (300)
    if selector_id == 300 {
        if let Some(sb) = recv_arc.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            let out = VMValue::Integer(sb.value.len() as i64);
            let buf = crate::runtime::host_api::tlv_encode_one(&out);
            return crate::runtime::host_api::encode_out(out_ptr, out_len, &buf);
        }
    }

    // Not handled here
    -10

}
