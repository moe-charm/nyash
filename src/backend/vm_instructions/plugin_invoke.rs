use crate::mir::ValueId;
use std::sync::Arc;
use crate::backend::vm::ControlFlow;
use crate::backend::{VM, VMError, VMValue};

impl VM {
    /// Execute a forced plugin invocation (no builtin fallback)
    pub(crate) fn execute_plugin_invoke(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Helper: extract UTF-8 string from internal StringBox, Result.Ok(String-like), or plugin StringBox via toUtf8
        fn extract_string_from_box(bx: &dyn crate::box_trait::NyashBox) -> Option<String> {
            if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() { return Some(sb.value.clone()); }
            if let Some(res) = bx.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
                if let crate::boxes::result::NyashResultBox::Ok(inner) = res { return extract_string_from_box(inner.as_ref()); }
            }
            if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                if p.box_type == "StringBox" {
                    let host = crate::runtime::get_global_plugin_host();
                    let tmp: Option<String> = if let Ok(ro) = host.read() {
                        if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", p.inner.instance_id, &[]) {
                            if let Some(vb) = val_opt { if let Some(sb2) = vb.as_any().downcast_ref::<crate::box_trait::StringBox>() { Some(sb2.value.clone()) } else { None } } else { None }
                        } else { None }
                    } else { None };
                    if tmp.is_some() { return tmp; }
                }
            }
            None
        }
        let recv = self.get_value(box_val)?;
        if method == "birth" && !matches!(recv, VMValue::BoxRef(ref b) if b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some()) {
            eprintln!("[VM PluginInvoke] static birth fallback recv={:?}", recv);
            let mut created: Option<VMValue> = None;
            match &recv {
                VMValue::String(s) => {
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let sb: Box<dyn crate::box_trait::NyashBox> = Box::new(crate::box_trait::StringBox::new(s.clone()));
                    if let Ok(b) = host.create_box("StringBox", &[sb]) { created = Some(VMValue::from_nyash_box(b)); }
                }
                VMValue::Integer(_n) => {
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    if let Ok(b) = host.create_box("IntegerBox", &[]) { created = Some(VMValue::from_nyash_box(b)); }
                }
                _ => {}
            }
            if let Some(val) = created { if let Some(dst_id) = dst { self.set_value(dst_id, val); } return Ok(ControlFlow::Continue); }
        }

        if let VMValue::BoxRef(pbox) = &recv {
            if let Some(p) = pbox.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                let host = crate::runtime::get_global_plugin_host();
                let host = host.read().unwrap();
                let mh = host.resolve_method(&p.box_type, method).map_err(|_| VMError::InvalidInstruction(format!("Plugin method not found: {}.{}", p.box_type, method)))?;
                let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(args.len() as u16);
                for (idx, a) in args.iter().enumerate() {
                    let v = self.get_value(*a)?;
                    match v {
                        VMValue::Integer(n) => { if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") { eprintln!("[VM→Plugin][vm] arg[{}] encode I64 {}", idx, n); } crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, n) }
                        VMValue::Float(x) => { if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") { eprintln!("[VM→Plugin][vm] arg[{}] encode F64 {}", idx, x); } crate::runtime::plugin_ffi_common::encode::f64(&mut tlv, x) }
                        VMValue::Bool(b) => crate::runtime::plugin_ffi_common::encode::bool(&mut tlv, b),
                        VMValue::String(ref s) => crate::runtime::plugin_ffi_common::encode::string(&mut tlv, s),
                        VMValue::BoxRef(ref b) => {
                            if let Some(h) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                if h.box_type == "StringBox" {
                                    let host = crate::runtime::get_global_plugin_host();
                                    let host = host.read().unwrap();
                                    if let Ok(val_opt) = host.invoke_instance_method("StringBox", "toUtf8", h.inner.instance_id, &[]) {
                                        if let Some(sb) = val_opt.and_then(|bx| bx.as_any().downcast_ref::<crate::box_trait::StringBox>().map(|s| s.value.clone())) { crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &sb); continue; }
                                    }
                                } else if h.box_type == "IntegerBox" {
                                    let host = crate::runtime::get_global_plugin_host();
                                    let host = host.read().unwrap();
                                    if let Ok(val_opt) = host.invoke_instance_method("IntegerBox", "get", h.inner.instance_id, &[]) {
                                        if let Some(ib) = val_opt.and_then(|bx| bx.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|i| i.value)) { crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, ib); continue; }
                                    }
                                }
                                crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id);
                            } else {
                                let h = crate::runtime::host_handles::to_handle_arc(b.clone());
                                crate::runtime::plugin_ffi_common::encode::host_handle(&mut tlv, h);
                            }
                        }
                        VMValue::Future(_) | VMValue::Void => {}
                    }
                }
                let mut out = vec![0u8; 32768];
                let mut out_len: usize = out.len();
                unsafe { (p.inner.invoke_fn)(p.inner.type_id, mh.method_id as u32, p.inner.instance_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len) };
                let vm_out_raw: VMValue = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                    match tag {
                        1 => crate::runtime::plugin_ffi_common::decode::bool(payload).map(VMValue::Bool).unwrap_or(VMValue::Void),
                        2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                        5 => crate::runtime::plugin_ffi_common::decode::f64(payload).map(VMValue::Float).unwrap_or(VMValue::Void),
                        6 | 7 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                        8 => {
                            if let Some(u) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                                if let Some(arc) = crate::runtime::host_handles::get(u) { VMValue::BoxRef(arc) } else { VMValue::Void }
                            } else { VMValue::Void }
                        }
                        _ => VMValue::Void,
                    }
                } else { VMValue::Void };
                // Wrap into Result.Ok when method is declared returns_result
                let vm_out = {
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let rr = host.method_returns_result(&p.box_type, method);
                    if rr {
                        let boxed: Box<dyn crate::box_trait::NyashBox> = match vm_out_raw {
                            VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s) => Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b) => b.share_box(),
                            VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            _ => Box::new(crate::box_trait::StringBox::new(vm_out_raw.to_string()))
                        };
                        let res = crate::boxes::result::NyashResultBox::new_ok(boxed);
                        VMValue::BoxRef(std::sync::Arc::from(Box::new(res) as Box<dyn crate::box_trait::NyashBox>))
                    } else { vm_out_raw }
                };
                if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                return Ok(ControlFlow::Continue);
            }
        }
        // Fallback: support common string-like methods without requiring PluginBox receiver
        if let VMValue::BoxRef(ref bx) = recv {
            if let Some(s) = extract_string_from_box(bx.as_ref()) {
                match method {
                    "length" => { if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Integer(s.len() as i64)); } return Ok(ControlFlow::Continue); }
                    "is_empty" | "isEmpty" => { if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Bool(s.is_empty())); } return Ok(ControlFlow::Continue); }
                    "charCodeAt" => {
                        let idx_v = if let Some(a0) = args.get(0) { self.get_value(*a0)? } else { VMValue::Integer(0) };
                        let idx = match idx_v { VMValue::Integer(i) => i.max(0) as usize, _ => 0 };
                        let code = s.chars().nth(idx).map(|c| c as u32 as i64).unwrap_or(0);
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Integer(code)); }
                        return Ok(ControlFlow::Continue);
                    }
                    "concat" => {
                        let rhs_v = if let Some(a0) = args.get(0) { self.get_value(*a0)? } else { VMValue::String(String::new()) };
                        let rhs_s = match rhs_v { VMValue::String(ss) => ss, VMValue::BoxRef(br) => extract_string_from_box(br.as_ref()).unwrap_or_else(|| br.to_string_box().value), _ => rhs_v.to_string(), };
                        let mut new_s = s.clone();
                        new_s.push_str(&rhs_s);
                        let out = Box::new(crate::box_trait::StringBox::new(new_s));
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::BoxRef(std::sync::Arc::from(out as Box<dyn crate::box_trait::NyashBox>))); }
                        return Ok(ControlFlow::Continue);
                    }
                    _ => {}
                }
            }
        }
        Err(VMError::InvalidInstruction(format!("PluginInvoke requires PluginBox receiver; method={} got {:?}", method, recv)))
    }
}
