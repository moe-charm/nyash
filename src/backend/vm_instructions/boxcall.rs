use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use crate::backend::vm::ControlFlow;
use crate::backend::{VM, VMError, VMValue};

impl VM {
    /// Execute BoxCall instruction
    pub(crate) fn execute_boxcall(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, method_id: Option<u16>, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        let recv = self.get_value(box_val)?;
        // Debug logging if enabled
        let debug_boxcall = std::env::var("NYASH_VM_DEBUG_BOXCALL").is_ok();

        // Fast-path: ConsoleBox.readLine — provide safe stdin fallback with EOF→Void
        if let VMValue::BoxRef(arc_box) = &recv {
            if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                if p.box_type == "ConsoleBox" && method == "readLine" {
                    use std::io::Read;
                    let mut s = String::new();
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 1];
                    loop {
                        match stdin.read(&mut buf) {
                            Ok(0) => {
                                if let Some(dst_id) = dst { let nb = crate::boxes::null_box::NullBox::new(); self.set_value(dst_id, VMValue::from_nyash_box(Box::new(nb))); }
                                return Ok(ControlFlow::Continue);
                            }
                            Ok(_) => {
                                let ch = buf[0] as char;
                                if ch == '\n' { break; }
                                s.push(ch);
                                if s.len() > 1_000_000 { break; }
                            }
                            Err(_) => {
                                if let Some(dst_id) = dst { let nb = crate::boxes::null_box::NullBox::new(); self.set_value(dst_id, VMValue::from_nyash_box(Box::new(nb))); }
                                return Ok(ControlFlow::Continue);
                            }
                        }
                    }
                    if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::String(s)); }
                    return Ok(ControlFlow::Continue);
                }
            }
        }

        // VTable-first stub path (optional)
        if crate::config::env::abi_vtable() {
            if let Some(res) = self.try_boxcall_vtable_stub(dst, &recv, method, method_id, args) { return res; }
        }

        // Record PIC hit (per-receiver-type × method)
        let pic_key = self.build_pic_key(&recv, method, method_id);
        self.pic_record_hit(&pic_key);

        // Explicit fast-paths
        if let VMValue::BoxRef(arc_box) = &recv {
            // ArrayBox get/set
            if arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() {
                let get_slot = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "get");
                let set_slot = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "set");
                let is_get = (method_id.is_some() && method_id == get_slot) || method == "get";
                let is_set = (method_id.is_some() && method_id == set_slot) || method == "set";
                if is_get && args.len() >= 1 {
                    let idx_val = self.get_value(args[0])?;
                    let idx_box = idx_val.to_nyash_box();
                    let arr = arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().unwrap();
                    let out = arr.get(idx_box);
                    if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                    return Ok(ControlFlow::Continue);
                } else if is_set && args.len() >= 2 {
                    crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.fastpath.Array.set");
                    let idx_val = self.get_value(args[0])?;
                    let val_val = self.get_value(args[1])?;
                    let idx_box = idx_val.to_nyash_box();
                    let val_box = val_val.to_nyash_box();
                    let arr = arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().unwrap();
                    let out = arr.set(idx_box, val_box);
                    if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                    return Ok(ControlFlow::Continue);
                }
            }
            // InstanceBox getField/setField by name
            if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                let is_getf = method == "getField";
                let is_setf = method == "setField";
                if (is_getf && args.len() >= 1) || (is_setf && args.len() >= 2) {
                    let name_val = self.get_value(args[0])?;
                    let field_name = match &name_val { VMValue::String(s) => s.clone(), _ => name_val.to_string(), };
                    if is_getf {
                        let out_opt = inst.get_field_unified(&field_name);
                        let out_vm = match out_opt {
                            Some(nv) => match nv {
                                crate::value::NyashValue::Integer(i) => VMValue::Integer(i),
                                crate::value::NyashValue::Float(f) => VMValue::Float(f),
                                crate::value::NyashValue::Bool(b) => VMValue::Bool(b),
                                crate::value::NyashValue::String(s) => VMValue::String(s),
                                crate::value::NyashValue::Void | crate::value::NyashValue::Null => VMValue::Void,
                                crate::value::NyashValue::Box(b) => { if let Ok(g) = b.lock() { VMValue::from_nyash_box(g.share_box()) } else { VMValue::Void } }
                                _ => VMValue::Void,
                            },
                            None => VMValue::Void,
                        };
                        if let Some(dst_id) = dst { self.set_value(dst_id, out_vm); }
                        return Ok(ControlFlow::Continue);
                    } else {
                        // setField
                        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.fastpath.Instance.setField");
                        let val_vm = self.get_value(args[1])?;
                        let nv_opt = match val_vm.clone() {
                            VMValue::Integer(i) => Some(crate::value::NyashValue::Integer(i)),
                            VMValue::Float(f) => Some(crate::value::NyashValue::Float(f)),
                            VMValue::Bool(b) => Some(crate::value::NyashValue::Bool(b)),
                            VMValue::String(s) => Some(crate::value::NyashValue::String(s)),
                            VMValue::Void => Some(crate::value::NyashValue::Void),
                            _ => None,
                        };
                        if let Some(nv) = nv_opt { let _ = inst.set_field_unified(field_name, nv); if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Void); } return Ok(ControlFlow::Continue); }
                    }
                }
            }
        }

        // VTable-like thunk path via TypeMeta and method_id
        if let (Some(mid), VMValue::BoxRef(arc_box)) = (method_id, &recv) {
            let mut class_label: Option<String> = None;
            let mut is_instance = false;
            let mut is_plugin = false;
            let mut is_builtin = false;
            if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() { class_label = Some(inst.class_name.clone()); is_instance = true; }
            else if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() { class_label = Some(p.box_type.clone()); is_plugin = true; }
            else { class_label = Some(arc_box.type_name().to_string()); is_builtin = true; }
            if let Some(label) = class_label {
                let tm = crate::runtime::type_meta::get_or_create_type_meta(&label);
                if let Some(th) = tm.get_thunk(mid as usize) {
                    if let Some(target) = th.get_target() {
                        match target {
                            crate::runtime::type_meta::ThunkTarget::MirFunction(func_name) => {
                                if std::env::var("NYASH_VM_VT_STATS").ok().as_deref() == Some("1") { eprintln!("[VT] hit class={} slot={} -> {}", label, mid, func_name); }
                                let mut vm_args = Vec::with_capacity(1 + args.len());
                                vm_args.push(recv.clone());
                                for a in args { vm_args.push(self.get_value(*a)?); }
                                let res = self.call_function_by_name(&func_name, vm_args)?;
                                self.record_poly_pic(&pic_key, &recv, &func_name);
                                let threshold = crate::config::env::vm_pic_threshold();
                                if self.pic_hits(&pic_key) >= threshold { self.boxcall_pic_funcname.insert(pic_key.clone(), func_name.clone()); }
                                if is_instance {
                                    let vkey = self.build_vtable_key(&label, mid, args.len());
                                    self.boxcall_vtable_funcname.entry(vkey).or_insert(func_name.clone());
                                }
                                if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                                return Ok(ControlFlow::Continue);
                            }
                            crate::runtime::type_meta::ThunkTarget::PluginInvoke { method_id: mid2 } => {
                                if is_plugin {
                                    if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                        self.enter_root_region();
                                        let nyash_args: Vec<Box<dyn NyashBox>> = args.iter().map(|arg| { let val = self.get_value(*arg)?; Ok(val.to_nyash_box()) }).collect::<Result<Vec<_>, VMError>>()?;
                                        self.pin_roots(std::iter::once(&recv));
                                        let pinned_args: Vec<VMValue> = args.iter().filter_map(|a| self.get_value(*a).ok()).collect();
                                        self.pin_roots(pinned_args.iter());
                                        let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(nyash_args.len() as u16);
                                        let mut enc_failed = false;
                                        for a in &nyash_args {
                                            if let Some(s) = a.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &s.value); }
                                            else if let Some(i) = a.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { crate::runtime::plugin_ffi_common::encode::i32(&mut tlv, i.value as i32); }
                                            else if let Some(h) = a.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() { crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id); }
                                            else { enc_failed = true; break; }
                                        }
                                        if !enc_failed {
                                            let mut out = vec![0u8; 4096];
                                            let mut out_len: usize = out.len();
                                            crate::runtime::host_api::set_current_vm(self as *mut _);
                                            let code = unsafe { (p.inner.invoke_fn)(p.inner.type_id, mid2 as u32, p.inner.instance_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len) };
                                            crate::runtime::host_api::clear_current_vm();
                                            if code == 0 {
                                                let vm_out = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                                                    match tag {
                                                        6 | 7 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                                                        2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                                                        9 => {
                                                            if let Some(h) = crate::runtime::plugin_ffi_common::decode::u64(payload) { if let Some(arc) = crate::runtime::host_handles::get(h) { VMValue::BoxRef(arc) } else { VMValue::Void } } else { VMValue::Void }
                                                        }
                                                        _ => VMValue::Void,
                                                    }
                                                } else { VMValue::Void };
                                                if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                                                self.leave_root_region();
                                                return Ok(ControlFlow::Continue);
                                            }
                                            self.leave_root_region();
                                        }
                                    }
                                }
                            }
                            crate::runtime::type_meta::ThunkTarget::BuiltinCall { method: ref m } => {
                                if is_builtin {
                                    let nyash_args: Vec<Box<dyn NyashBox>> = args.iter().map(|arg| { let val = self.get_value(*arg)?; Ok(val.to_nyash_box()) }).collect::<Result<Vec<_>, VMError>>()?;
                                    if crate::backend::gc_helpers::is_mutating_builtin_call(&recv, m) { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.builtin"); }
                                    else if m == "setField" { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.setField"); }
                                    let cloned_box = arc_box.share_box();
                                    self.boxcall_hits_generic = self.boxcall_hits_generic.saturating_add(1);
                                    let out = self.call_box_method(cloned_box, m, nyash_args)?;
                                    let vm_out = VMValue::from_nyash_box(out);
                                    if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                                    return Ok(ControlFlow::Continue);
                                }
                            }
                        }
                    }
                }
                // Legacy vtable cache for InstanceBox
                if is_instance {
                    let inst = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().unwrap();
                    let vkey = self.build_vtable_key(&inst.class_name, mid, args.len());
                    if let Some(func_name) = self.boxcall_vtable_funcname.get(&vkey).cloned() {
                        let mut vm_args = Vec::with_capacity(1 + args.len());
                        vm_args.push(recv.clone());
                        for a in args { vm_args.push(self.get_value(*a)?); }
                        let res = self.call_function_by_name(&func_name, vm_args)?;
                        if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                        return Ok(ControlFlow::Continue);
                    }
                }
            }
        }

        // Poly-PIC direct call to lowered function if present
        if let VMValue::BoxRef(arc_box) = &recv {
            if arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() {
                if let Some(func_name) = self.try_poly_pic(&pic_key, &recv) {
                    if crate::config::env::vm_pic_trace() { eprintln!("[PIC] poly hit {}", pic_key); }
                    self.boxcall_hits_poly_pic = self.boxcall_hits_poly_pic.saturating_add(1);
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                    return Ok(ControlFlow::Continue);
                }
                if let Some(func_name) = self.boxcall_pic_funcname.get(&pic_key).cloned() {
                    if crate::config::env::vm_pic_trace() { eprintln!("[PIC] mono hit {}", pic_key); }
                    self.boxcall_hits_mono_pic = self.boxcall_hits_mono_pic.saturating_add(1);
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                    return Ok(ControlFlow::Continue);
                }
            }
        }

        // Fast universal slots (0..3)
        if let Some(mid) = method_id {
            if let Some(fast_res) = self.try_fast_universal(mid, &recv, args)? { if let Some(dst_id) = dst { self.set_value(dst_id, fast_res); } return Ok(ControlFlow::Continue); }
        }

        // Generic path: convert args to NyashBox and call
        let nyash_args: Vec<Box<dyn NyashBox>> = args.iter().map(|arg| { let val = self.get_value(*arg)?; Ok(val.to_nyash_box()) }).collect::<Result<Vec<_>, VMError>>()?;

        // PluginBoxV2 fast-path via direct invoke_fn when method_id present
        if let (Some(mid), VMValue::BoxRef(arc_box)) = (method_id, &recv) {
            if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                self.enter_root_region();
                let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(nyash_args.len() as u16);
                let mut enc_failed = false;
                for a in &nyash_args {
                    if let Some(buf) = a.as_any().downcast_ref::<crate::boxes::buffer::BufferBox>() { let snapshot = buf.to_vec(); crate::runtime::plugin_ffi_common::encode::bytes(&mut tlv, &snapshot); continue; }
                    if let Some(s) = a.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &s.value); }
                    else if let Some(i) = a.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { if i.value >= i32::MIN as i64 && i.value <= i32::MAX as i64 { crate::runtime::plugin_ffi_common::encode::i32(&mut tlv, i.value as i32); } else { crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, i.value as i64); } }
                    else if let Some(b) = a.as_any().downcast_ref::<crate::box_trait::BoolBox>() { crate::runtime::plugin_ffi_common::encode::bool(&mut tlv, b.value); }
                    else if let Some(f) = a.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() { crate::runtime::plugin_ffi_common::encode::f64(&mut tlv, f.value); }
                    else if let Some(arr) = a.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        let items = arr.items.read().unwrap();
                        let mut tmp = Vec::with_capacity(items.len());
                        let mut ok = true;
                        for item in items.iter() {
                            if let Some(intb) = item.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { if intb.value >= 0 && intb.value <= 255 { tmp.push(intb.value as u8); } else { ok = false; break; } } else { ok = false; break; }
                        }
                        if ok { crate::runtime::plugin_ffi_common::encode::bytes(&mut tlv, &tmp); } else { enc_failed = true; break; }
                    } else if let Some(h) = a.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() { crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id); }
                    else { enc_failed = true; break; }
                }
                if !enc_failed {
                    let mut out = vec![0u8; 4096];
                    let mut out_len: usize = out.len();
                    let code = unsafe { (p.inner.invoke_fn)(p.inner.type_id, mid as u32, p.inner.instance_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len) };
                    if code == 0 {
                        // Record TypeMeta thunk for plugin invoke so next time VT path can hit
                        let tm = crate::runtime::type_meta::get_or_create_type_meta(&p.box_type);
                        tm.set_thunk_plugin_invoke(mid as usize, mid as u16);
                        let vm_out = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                            match tag {
                                1 => crate::runtime::plugin_ffi_common::decode::bool(payload).map(VMValue::Bool).unwrap_or(VMValue::Void),
                                2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                                5 => crate::runtime::plugin_ffi_common::decode::f64(payload).map(VMValue::Float).unwrap_or(VMValue::Void),
                                6 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                                _ => VMValue::Void,
                            }
                        } else { VMValue::Void };
                        if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                        self.leave_root_region();
                        return Ok(ControlFlow::Continue);
                    }
                    self.leave_root_region();
                }
            }
        }

        if debug_boxcall { self.debug_log_boxcall(&recv, method, &nyash_args, "START", None); }

        // Call the method based on receiver type
        let result = match &recv {
            VMValue::BoxRef(arc_box) => {
                if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let func_name = format!("{}.{}{}", inst.class_name, method, format!("/{}", args.len()));
                    if let Some(mid) = method_id {
                        let tm = crate::runtime::type_meta::get_or_create_type_meta(&inst.class_name);
                        tm.set_thunk_mir_target(mid as usize, func_name.clone());
                        let vkey = self.build_vtable_key(&inst.class_name, mid, args.len());
                        self.boxcall_vtable_funcname.entry(vkey).or_insert(func_name.clone());
                    }
                    self.record_poly_pic(&pic_key, &recv, &func_name);
                    let threshold = crate::config::env::vm_pic_threshold();
                    if self.pic_hits(&pic_key) >= threshold { self.boxcall_pic_funcname.insert(pic_key.clone(), func_name.clone()); }
                    if debug_boxcall { eprintln!("[BoxCall] InstanceBox -> call {}", func_name); }
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    return { if let Some(dst_id) = dst { self.set_value(dst_id, res); } Ok(ControlFlow::Continue) };
                }
                if debug_boxcall { eprintln!("[BoxCall] Taking BoxRef path for method '{}'", method); }
                if let Some(mid) = method_id {
                    let label = arc_box.type_name().to_string();
                    let tm = crate::runtime::type_meta::get_or_create_type_meta(&label);
                    tm.set_thunk_builtin(mid as usize, method.to_string());
                }
                if crate::backend::gc_helpers::is_mutating_builtin_call(&recv, method) { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall"); }
                else if method == "setField" { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.setField"); }
                let cloned_box = arc_box.share_box();
                self.call_box_method(cloned_box, method, nyash_args)?
            }
            _ => {
                if debug_boxcall { eprintln!("[BoxCall] Converting primitive to box for method '{}'", method); }
                let box_value = recv.to_nyash_box();
                self.call_box_method(box_value, method, nyash_args)?
            }
        };

        let result_val = VMValue::from_nyash_box(result);
        if debug_boxcall { self.debug_log_boxcall(&recv, method, &[], "END", Some(&result_val)); }
        if let Some(dst_id) = dst { self.set_value(dst_id, result_val); }
        Ok(ControlFlow::Continue)
    }

    /// Phase 12 Tier-0: vtable-first stub for selected types
    pub(super) fn try_boxcall_vtable_stub(&mut self, _dst: Option<ValueId>, _recv: &VMValue, _method: &str, _method_id: Option<u16>, _args: &[ValueId]) -> Option<Result<ControlFlow, VMError>> {
        if crate::config::env::vm_vt_trace() {
            match _recv { VMValue::BoxRef(b) => eprintln!("[VT] probe recv_ty={} method={} argc={}", b.type_name(), _method, _args.len()), other => eprintln!("[VT] probe recv_prim={:?} method={} argc={}", other, _method, _args.len()), }
        }
        if let VMValue::BoxRef(b) = _recv {
            let ty_name = b.type_name();
            let ty_name_for_reg: std::borrow::Cow<'_, str> = if let Some(p) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() { std::borrow::Cow::Owned(p.box_type.clone()) } else { std::borrow::Cow::Borrowed(ty_name) };
            if let Some(_tb) = crate::runtime::type_registry::resolve_typebox_by_name(&ty_name_for_reg) {
                let slot = crate::runtime::type_registry::resolve_slot_by_name(&ty_name_for_reg, _method, _args.len());
                if let Some(p) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if crate::config::env::vm_vt_trace() { eprintln!("[VT] plugin recv ty={} method={} arity={}", ty_name, _method, _args.len()); }
                    let mut nyash_args: Vec<Box<dyn NyashBox>> = Vec::with_capacity(_args.len());
                    for aid in _args.iter() { if let Ok(v) = self.get_value(*aid) { nyash_args.push(v.to_nyash_box()); } else { nyash_args.push(Box::new(crate::box_trait::VoidBox::new())); } }
                    match ty_name {
                        "MapBox" => {
                            match slot {
                                Some(200) | Some(201) => {
                                    let host = crate::runtime::get_global_plugin_host();
                                    let ro = host.read().unwrap();
                                    if let Ok(val_opt) = ro.invoke_instance_method("MapBox", _method, p.inner.instance_id, &[]) {
                                        if let Some(out) = val_opt { if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); } }
                                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                                        return Some(Ok(ControlFlow::Continue));
                                    }
                                }
                                Some(202) | Some(203) | Some(204) => {
                                    if matches!(slot, Some(204)) { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "VTable.Plugin.Map.set"); }
                                    let host = crate::runtime::get_global_plugin_host();
                                    let ro = host.read().unwrap();
                                    let mut method_eff = _method;
                                    if (matches!(slot, Some(202)) && _args.len() >= 1) || (matches!(slot, Some(203)) && _args.len() >= 1) {
                                        if let Ok(a0v) = self.get_value(_args[0]) { if matches!(a0v, VMValue::String(_)) { method_eff = if matches!(slot, Some(203)) { "getS" } else { "hasS" }; } }
                                    }
                                    if let Ok(val_opt) = ro.invoke_instance_method("MapBox", method_eff, p.inner.instance_id, &nyash_args) {
                                        if let Some(out) = val_opt { if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); } }
                                        else if _dst.is_some() { if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::Void); } }
                                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                                        return Some(Ok(ControlFlow::Continue));
                                    }
                                }
                                _ => {}
                            }
                        }
                        "ArrayBox" => {
                            match slot {
                                Some(100) | Some(101) | Some(102) => {
                                    if matches!(slot, Some(101)) { crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "VTable.Plugin.Array.set"); }
                                    let host = crate::runtime::get_global_plugin_host();
                                    let ro = host.read().unwrap();
                                    if let Ok(val_opt) = ro.invoke_instance_method("ArrayBox", _method, p.inner.instance_id, &nyash_args) {
                                        if let Some(out) = val_opt { if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); } }
                                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                                        return Some(Ok(ControlFlow::Continue));
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                // Builtin boxes
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    if matches!(slot, Some(200)) || matches!(slot, Some(201)) {
                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                        if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.size/len"); }
                        let out = map.size();
                        if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                        return Some(Ok(ControlFlow::Continue));
                    }
                    if matches!(slot, Some(202)) {
                        if let Ok(arg_v) = self.get_value(_args[0]) {
                            let key_box: Box<dyn NyashBox> = match arg_v {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.has"); }
                            let out = map.has(key_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                    if matches!(slot, Some(203)) {
                        if let Ok(arg_v) = self.get_value(_args[0]) {
                            let key_box: Box<dyn NyashBox> = match arg_v {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.get"); }
                            let out = map.get(key_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                    if matches!(slot, Some(204)) {
                        if let (Ok(a0), Ok(a1)) = (self.get_value(_args[0]), self.get_value(_args[1])) {
                            if let VMValue::String(ref s) = a0 { let vb: Box<dyn NyashBox> = match a1 { VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)), VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)), VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)), VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)), VMValue::BoxRef(ref bx) => bx.share_box(), VMValue::Future(ref fut) => Box::new(fut.clone()), VMValue::Void => Box::new(crate::box_trait::VoidBox::new()), }; let out = map.set(Box::new(crate::box_trait::StringBox::new(s)), vb); if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); } return Some(Ok(ControlFlow::Continue)); }
                            let key_box: Box<dyn NyashBox> = match a0 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            let val_box: Box<dyn NyashBox> = match a1 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            // Barrier: mutation
                            crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "VTable.Map.set");
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.set"); }
                            let out = map.set(key_box, val_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                }
                // StringBox: len
                if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                    if matches!(slot, Some(300)) {
                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                        if crate::config::env::vm_vt_trace() { eprintln!("[VT] StringBox.len"); }
                        let out = crate::box_trait::IntegerBox::new(sb.value.len() as i64);
                        if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(Box::new(out))); }
                        return Some(Ok(ControlFlow::Continue));
                    }
                }
                if crate::config::env::abi_strict() {
                    let known = crate::runtime::type_registry::known_methods_for(ty_name).unwrap_or_default().join(", ");
                    let msg = format!("ABI_STRICT: undefined vtable method {}.{}(arity={}) — known: {}", ty_name, _method, _args.len(), known);
                    return Some(Err(VMError::TypeError(msg)));
                }
            }
        }
        None
    }
}

impl VM {
    /// Try fast universal-thunk dispatch using reserved method slots 0..3.
    fn try_fast_universal(&mut self, method_id: u16, recv: &VMValue, args: &[ValueId]) -> Result<Option<VMValue>, VMError> {
        match method_id {
            0 => { let s = recv.to_string(); return Ok(Some(VMValue::String(s))); }
            1 => {
                let t = match recv {
                    VMValue::Integer(_) => "Integer".to_string(),
                    VMValue::Float(_) => "Float".to_string(),
                    VMValue::Bool(_) => "Bool".to_string(),
                    VMValue::String(_) => "String".to_string(),
                    VMValue::Future(_) => "Future".to_string(),
                    VMValue::Void => "Void".to_string(),
                    VMValue::BoxRef(b) => b.type_name().to_string(),
                };
                return Ok(Some(VMValue::String(t)));
            }
            2 => {
                let other = if let Some(arg0) = args.get(0) { self.get_value(*arg0)? } else { VMValue::Void };
                let res = match (recv, &other) {
                    (VMValue::Integer(a), VMValue::Integer(b)) => a == b,
                    (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
                    (VMValue::String(a), VMValue::String(b)) => a == b,
                    (VMValue::Void, VMValue::Void) => true,
                    _ => recv.to_string() == other.to_string(),
                };
                return Ok(Some(VMValue::Bool(res)));
            }
            3 => {
                let v = match recv {
                    VMValue::Integer(i) => VMValue::Integer(*i),
                    VMValue::Float(f) => VMValue::Float(*f),
                    VMValue::Bool(b) => VMValue::Bool(*b),
                    VMValue::String(s) => VMValue::String(s.clone()),
                    VMValue::Future(f) => VMValue::Future(f.clone()),
                    VMValue::Void => VMValue::Void,
                    VMValue::BoxRef(b) => VMValue::from_nyash_box(b.share_box()),
                };
                return Ok(Some(v));
            }
            _ => {}
        }
        Ok(None)
    }
}
