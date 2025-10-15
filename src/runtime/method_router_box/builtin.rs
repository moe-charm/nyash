//! builtin.rs — Legacy builtin (Rust) routing adapter (Phase 15.75 split)
use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;
use crate::vm_ops::boxcall;
use super::tables::{ARRAY_HOST_ROUTES, MAP_HOST_ROUTES, STRING_HOST_ROUTES};

/// Try routing a legacy builtin box (FileBox/CallableBox/ArrayBox/MapBox)
/// Phase 0-mini: move FileBox arm only; others will be migrated incrementally.
/// Returns Ok(None) when not handled.
pub fn try_route_builtin_box(
    _interp: &mut MirInterpreter,
    bx: &std::sync::Arc<dyn NyashBox>,
    method: &str,
    args: &[VMValue],
) -> Result<Option<VMValue>, VMError> {
    // FileBox
    if bx.type_name() == "FileBox" {
        #[cfg(feature = "legacy-boxes")]
        if let Some(fb) = bx.as_any().downcast_ref::<crate::boxes::file::FileBox>() {
            let _ = super::maybe_arity_guard("FileBox", method, args.len());
            let out = match method {
                "open" => {
                    if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("FileBox","open", args.len(), &[2]))); }
                    let path = args[0].to_string();
                    let mode = args[1].to_string();
                    let ok = fb.open_in_place(&path, &mode);
                    Ok(VMValue::Bool(ok))
                }
                "read" => { Ok(VMValue::from_nyash_box(fb.read())) }
                "exists" => { Ok(VMValue::from_nyash_box(fb.exists())) }
                "write" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("FileBox","write", args.len(), &[1]))); }
                    let content = args[0].to_nyash_box();
                    Ok(VMValue::from_nyash_box(fb.write(content)))
                }
                "append" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("FileBox","append", args.len(), &[1]))); }
                    let content = args[0].to_nyash_box();
                    Ok(VMValue::from_nyash_box(fb.append(content)))
                }
                "close" => { let _ = fb.close_in_place(); Ok(VMValue::Void) }
                _ => Err(boxcall::unknown_method_err("FileBox", method, args.len())),
            }?;
            return Ok(Some(out));
        }
        #[cfg(feature = "legacy-boxes")]
        {
            // Downcast failed under legacy path
            return Err(crate::vm_ops::boxcall::downcast_failed("FileBox"));
        }
        #[cfg(not(feature = "legacy-boxes"))]
        {
            return Err(boxcall::method_not_supported(method, &VMValue::from_nyash_box(bx.clone_box())));
        }
    }
    // CallableBox
    if bx.type_name() == "CallableBox" {
        #[cfg(feature = "legacy-boxes")]
        if let Some(cb) = bx.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
            let _ = crate::vm_ops::boxcall::arity_guard_for("CallableBox", method, args.len());
            if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("CallableBox", method, args.len()) {
                let res = match slot as u32 {
                    500 => Ok(VMValue::Integer(cb.arity() as i64)),
                    503 => Ok(VMValue::String(cb.to_string_box().value)),
                    501 => {
                        // Flatten argv via helper (see array_flatten_helper/README.md)
                        use crate::runtime::array_flatten_helper as afh;
                        let argv: Vec<VMValue> = hako_core_callable::flatten_argv(args,
                            afh::is_array,
                            afh::get_len,
                            afh::get_element);
                        if let Some(recv) = &cb.receiver {
                            let recv_vm = VMValue::BoxRef(std::sync::Arc::from(recv.share_box()));
                            crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
                        } else {
                            Err(VMError::InvalidInstruction("CallableBox without receiver is not supported yet".into()))
                        }
                    }
                    502 => {
                        let fut = crate::boxes::future::FutureBox::new();
                        crate::runtime::global_hooks::register_future_to_current_group(&fut);
                        let use_async = hako_core_callable::async_enabled_from_env();
                        if use_async {
                            if let Some(recv) = &cb.receiver {
                                if let Some(p) = recv.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                    let flat_vm: Vec<VMValue> = hako_core_callable::flatten_argv(args,
                                        |v: &VMValue| { if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() } else { false } },
                                        |v: &VMValue| { if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().map(|a| a.items.read().unwrap().len()).unwrap_or(0) } else { 0 } },
                                        |v: &VMValue, i: usize| { if let VMValue::BoxRef(bx)=v { if let Some(a)=bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() { let guard=a.items.read().unwrap(); VMValue::from_nyash_box(guard[i].clone_box()) } else { v.clone() } } else { v.clone() } });
                                    let mut argv: Vec<Box<dyn NyashBox>> = Vec::new();
                                    for v in &flat_vm {
                                        match v { VMValue::BoxRef(bx) => {
                                            if crate::runtime::type_registry::is_core_box(bx.type_name()) {
                                                let h = crate::runtime::host_handles::to_handle_arc(bx.clone());
                                                argv.push(Box::new(crate::runtime::host_handle_box::HostHandleBox::new(h)));
                                            } else { argv.push(v.to_nyash_box()); }
                                        }, _ => argv.push(v.to_nyash_box()) }
                                    }
                                    let typ = p.box_type.clone();
                                    let inst = p.inner.instance_id;
                                    let method_sched = cb.method.clone();
                                    let fut_clone = fut.clone();
                                    let name = format!("callable.callAsync({}.{}#{})", typ, method, inst);
                                    let _scheduled = crate::runtime::global_hooks::spawn_task(&name, Box::new(move || {
                                        match crate::runtime::plugin_host_box::invoke_instance_method(&typ, &method_sched, inst, &argv) {
                                            Ok(Some(ret)) => fut_clone.set_result(ret),
                                            Ok(None) => fut_clone.set_result(Box::new(crate::box_trait::VoidBox::new())),
                                            Err(_) => fut_clone.set_result(Box::new(crate::box_trait::StringBox::new("invoke_failed"))),
                                        }
                                    }));
                                    crate::runtime::global_hooks::safepoint_and_poll();
                                    Ok(VMValue::from_nyash_box(Box::new(fut)))
                                } else {
                                    // Builtin receiver — schedule via router
                                    let recv_vm = VMValue::BoxRef(std::sync::Arc::from(recv.share_box()));
                                    let mut argv_vm: Vec<VMValue> = Vec::new();
                                    if args.len() == 1 {
                                        if let VMValue::BoxRef(arrbx) = &args[0] {
                                            if let Some(arr) = arrbx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                                let guard = arr.items.read().unwrap();
                                                for it in guard.iter() { argv_vm.push(VMValue::from_nyash_box(it.clone_box())); }
                                            } else { argv_vm.push(args[0].clone()); }
                                        } else { argv_vm.push(args[0].clone()); }
                                    } else { argv_vm.extend_from_slice(args); }
                                    let method_sched = cb.method.clone();
                                    let fut_clone = fut.clone();
                                    let name = format!("callable.callAsync(builtin.{})", method_sched);
                                    let _scheduled = crate::runtime::global_hooks::spawn_task(&name, Box::new(move || {
                                        let mut vm = crate::backend::mir_interpreter::MirInterpreter::new();
                                        match crate::runtime::method_router_box::route(&mut vm, &recv_vm, &method_sched, &argv_vm) {
                                            Ok(v) => fut_clone.set_result(v.to_nyash_box()),
                                            Err(_) => fut_clone.set_result(Box::new(crate::box_trait::StringBox::new("invoke_failed")))
                                        }
                                    }));
                                    crate::runtime::global_hooks::safepoint_and_poll();
                                    Ok(VMValue::from_nyash_box(Box::new(fut)))
                                }
                            } else { Err(VMError::InvalidInstruction("CallableBox without receiver is not supported yet".into())) }
                        } else {
                            // 同期 call: 直後にFutureへ結果を詰めて返す
                            if let Some(recv) = &cb.receiver {
                                let recv_vm = VMValue::BoxRef(std::sync::Arc::from(recv.share_box()));
                                use crate::runtime::array_flatten_helper as afh;
                                let argv: Vec<VMValue> = hako_core_callable::flatten_argv(args, afh::is_array, afh::get_len, afh::get_element);
                                match crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv) {
                                    Ok(v) => { fut.set_result(v.to_nyash_box()); Ok(VMValue::from_nyash_box(Box::new(fut))) }
                                    Err(e) => Err(e),
                                }
                            } else { Err(VMError::InvalidInstruction("CallableBox without receiver is not supported yet".into())) }
                        }
                    }
                    _ => Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::unknown_slot("CallableBox", method, slot))),
                }?;
                return Ok(Some(res));
            }
            return Err(crate::vm_ops::boxcall::unknown_method_err("CallableBox", method, args.len()));
        }
        #[cfg(feature = "legacy-boxes")]
        { return Err(crate::vm_ops::boxcall::downcast_failed("CallableBox")); }
        #[cfg(not(feature = "legacy-boxes"))]
        { return Err(boxcall::method_not_supported(method, &VMValue::from_nyash_box(bx.clone_box()))); }
    }
    // ArrayBox
    if bx.type_name() == "ArrayBox" {
        #[cfg(feature = "legacy-boxes")]
        if let Some(arr) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let _ = crate::vm_ops::boxcall::arity_guard_for("ArrayBox", method, args.len());
            if let Some(route) = ARRAY_HOST_ROUTES.pick(method, args.len()) {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                if let Some(v) = super::host_slot::invoke(hh, route, args) {
                    return Ok(Some(v));
                }
            }
            let res = if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("ArrayBox", method, args.len()) {
                match slot as u32 {
                    100 => { // get(index)
                        if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("ArrayBox","get", args.len(), &[1]))); }
                        let idx = args[0].as_integer().unwrap_or(0) as usize;
                        let items = arr.items.read().unwrap();
                        Ok(if idx < items.len() { VMValue::from_nyash_box(items[idx].clone_box()) } else { VMValue::from_nyash_box(Box::new(crate::boxes::null_box::NullBox::new())) })
                    }
                    101 => { // set(index,value)
                        if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("ArrayBox","set", args.len(), &[2]))); }
                        let idx = args[0].as_integer().unwrap_or(0);
                        let val = args[1].to_nyash_box();
                        let _ = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), val);
                        Ok(VMValue::Void)
                    }
                    102 => { // size/len/length
                        Ok(VMValue::Integer(arr.items.read().unwrap().len() as i64))
                    }
                    103 => { // slice(start,end)
                        if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("ArrayBox","slice", args.len(), &[2]))); }
                        let start = args[0].as_integer().unwrap_or(0);
                        let end = args[1].as_integer().unwrap_or(arr.items.read().unwrap().len() as i64);
                        Ok(VMValue::from_nyash_box(arr.slice(
                            Box::new(crate::box_trait::IntegerBox::new(start)),
                            Box::new(crate::box_trait::IntegerBox::new(end)),
                        )))
                    }
                    _ => Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::unknown_slot("ArrayBox", method, slot))),
                }
            } else {
                Err(crate::vm_ops::boxcall::unknown_method_err("ArrayBox", method, args.len()))
            }?;
            return Ok(Some(res));
        }
        #[cfg(feature = "legacy-boxes")]
        { return Err(crate::vm_ops::boxcall::downcast_failed("ArrayBox")); }
        #[cfg(not(feature = "legacy-boxes"))]
        { return Err(boxcall::method_not_supported(method, &VMValue::from_nyash_box(bx.clone_box()))); }
    }
    // MapBox
    if bx.type_name() == "MapBox" {
        #[cfg(feature = "legacy-boxes")]
        if let Some(mp) = bx.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            let _ = crate::vm_ops::boxcall::arity_guard_for("MapBox", method, args.len());
            if let Some(route) = MAP_HOST_ROUTES.pick(method, args.len()) {
                let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
                if let Some(v) = super::host_slot::invoke(hh, route, args) {
                    return Ok(Some(v));
                }
            }
            let res = if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("MapBox", method, args.len()) {
                match slot as u32 {
                    200 => Ok(VMValue::from_nyash_box(mp.size())),
                    204 => { // set(key,value)
                        if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","set", args.len(), &[2]))); }
                        let key_box = args[0].to_nyash_box(); let val_box = args[1].to_nyash_box();
                        let _ = mp.set(key_box, val_box);
                        Ok(VMValue::Void)
                    }
                    203 => { // get(key)
                        if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","get", args.len(), &[1]))); }
                        let key_box = args[0].to_nyash_box();
                        Ok(VMValue::from_nyash_box(mp.get(key_box)))
                    }
                    202 => { // has(key)
                        if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","has", args.len(), &[1]))); }
                        let key_box = args[0].to_nyash_box();
                        let bb = mp.has(key_box);
                        if let Some(b) = bb.as_any().downcast_ref::<crate::box_trait::BoolBox>() { Ok(VMValue::Bool(b.value)) } else { Ok(VMValue::Bool(false)) }
                    }
                    208 => { let _ = mp.clear(); Ok(VMValue::Void) }
                    205 => { // delete
                        if args.len() != 1 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","delete", args.len(), &[1]))); }
                        let key_box = args[0].to_nyash_box();
                        Ok(VMValue::from_nyash_box(mp.delete(key_box)))
                    }
                    206 => Ok(VMValue::from_nyash_box(mp.keys())),
                    207 => Ok(VMValue::from_nyash_box(mp.values())),
                    209 => { let s = crate::boxes::json::stringify_any(mp.clone_box()); Ok(VMValue::String(s)) },
                    210 => { // call(key, argsArray)
                        if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","call", args.len(), &[2]))); }
                        let key_box = args[0].to_nyash_box();
                        let callee = mp.get(key_box);
                        if let Some(cb) = callee.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
                            let recv_vm = VMValue::BoxRef(std::sync::Arc::from(cb.receiver.as_ref().ok_or_else(|| VMError::InvalidInstruction("CallableBox without receiver".into()))?.share_box()));
                            use crate::runtime::array_flatten_helper as afh;
                            let argv: Vec<VMValue> = hako_core_callable::flatten_argv(&args[1..2], afh::is_array, afh::get_len, afh::get_element);
                            crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
                        } else { Err(VMError::InvalidInstruction("Map.call: value is not CallableBox".into())) }
                    }
                    211 => { // callAsync
                        if args.len() != 2 { return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity("MapBox","callAsync", args.len(), &[2]))); }
                        let key_box = args[0].to_nyash_box();
                        let callee = mp.get(key_box);
                        if let Some(cb) = callee.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
                            let cb_vm = VMValue::BoxRef(std::sync::Arc::new(cb.clone()));
                            let args_vm = VMValue::BoxRef(std::sync::Arc::from(args[1].to_nyash_box()));
                            crate::runtime::method_router_box::route(_interp, &cb_vm, "callAsync", &vec![args_vm])
                        } else { Err(VMError::InvalidInstruction("Map.callAsync: value is not CallableBox".into())) }
                    }
                    _ => Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::unknown_slot("MapBox", method, slot))),
                }
            } else { Err(crate::vm_ops::boxcall::unknown_method_err("MapBox", method, args.len())) }?;
            return Ok(Some(res));
        }
        #[cfg(feature = "legacy-boxes")]
        { return Err(crate::vm_ops::boxcall::downcast_failed("MapBox")); }
        #[cfg(not(feature = "legacy-boxes"))]
        { return Err(boxcall::method_not_supported(method, &VMValue::from_nyash_box(bx.clone_box()))); }
    }
    Ok(None)
}

/// Handle primitive String receiver (non-BoxRef) via TypeRegistry slots.
/// Returns Ok(Some(VMValue)) when handled; Ok(None) if receiver is not String or method unknown.
pub fn try_route_string_primitive(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<Option<VMValue>, VMError> {
    if let VMValue::String(s) = receiver {
        let _ = super::maybe_arity_guard("StringBox", method, args.len());
        if let Some(route) = STRING_HOST_ROUTES.pick(method, args.len()) {
            let sb = Box::new(crate::box_trait::StringBox::new(s.clone())) as Box<dyn crate::box_trait::NyashBox>;
            let hh = crate::runtime::host_handles::to_handle_box(sb);
            if let Some(v) = super::host_slot::invoke(hh, route, args) {
                return Ok(Some(v));
            }
        }
        if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("StringBox", method, args.len()) {
            let res = match slot as u32 {
                300 => {
                    Ok(Some(VMValue::Integer(hako_core_string::length_bytes(s))))
                },
                301 => {
                    let start = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    let end = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(Some(VMValue::String(hako_core_string::substring_bytes(s, start, end))))
                }
                303 => {
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(Some(VMValue::Integer(hako_core_string::index_of(s, &needle, from))))
                }
                313 => {
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(Some(VMValue::Integer(hako_core_string::last_index_of(s, &needle, from))))
                }
                314 => {
                    let idx = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(Some(VMValue::String(hako_core_string::char_at_byte(s, idx))))
                }
                302 => {
                    let rhs = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let mut out = String::with_capacity(s.len() + rhs.len());
                    out.push_str(s);
                    out.push_str(&rhs);
                    Ok(Some(VMValue::String(out)))
                }
                304 => {
                    let from = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let to = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    Ok(Some(VMValue::String(hako_core_string::replace_all(s, &from, &to))))
                }
                305 => Ok(Some(VMValue::String(s.trim().to_string()))),
                306 => Ok(Some(VMValue::String(s.to_uppercase()))),
                307 => Ok(Some(VMValue::String(s.to_lowercase()))),
                308 => Ok(Some(VMValue::String(s.clone()))),
                309 => Ok(Some(VMValue::String(s.clone()))),
                310 => { let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default(); Ok(Some(VMValue::Bool(s.starts_with(&needle)))) }
                311 => { let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default(); Ok(Some(VMValue::Bool(s.ends_with(&needle)))) }
                312 => Ok(Some(VMValue::Bool(hako_core_string::is_empty(s)))) ,
                _ => Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::unknown_slot("StringBox", method, slot))),
            }?;
            return Ok(res);
        } else {
            return Err(crate::vm_ops::boxcall::unknown_method_err("StringBox", method, args.len()));
        }
    }
    Ok(None)
}
