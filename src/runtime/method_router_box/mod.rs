//! MethodRouterBox — Single entry to route method calls
//!
//! Phase‑1 implementation:
//! - Resolver: classify receiver into builtin/core vs plugin box
//! - Invoker:
//!   - BuiltinInvoker → delegate to core semantics (String/Array/Map)
//!   - PluginInvoker  → invoke v2 TypeBox FFI through plugin host

mod map_callable;
mod method_ref;

use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;
use map_callable::MapCallableBox;
use method_ref::MethodRefBox;


#[inline]
fn maybe_arity_guard(type_name: &str, method: &str, arity: usize) -> Result<(), crate::backend::vm_types::VMError> {
    if method == "birth" { return Ok(()); }
    if crate::runtime::type_registry::resolve_typebox_by_name(type_name).is_some() {
        if crate::runtime::type_registry::resolve_slot_by_name(type_name, method, arity).is_none() {
            if let Some(known) = crate::runtime::type_registry::known_arities_for(type_name, method) {
                if !known.is_empty() {
                    return Err(crate::backend::vm_types::VMError::InvalidInstruction(format!(
                        "No matching method: {}.{}({} args). Available arities: {:?}",
                        type_name, method, arity, known
                    )));
                }
            }
        }
    }
    Ok(())
}
pub fn route(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    if let Some(result) = MethodRefBox::try_route(receiver, method, args) {
        return result;
    }

    // Primitive String — table-driven via TypeRegistry slots
    if let VMValue::String(s) = receiver {
        let _ = maybe_arity_guard("StringBox", method, args.len());
        if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("StringBox", method, args.len()) {
            let res = match slot as u32 {
                300 => Ok(VMValue::Integer(hako_core_string::length_bytes(s))),
                301 => { // substring(start,end)
                    let start = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    let end = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(VMValue::String(hako_core_string::substring_bytes(s, start, end)))
                }
                303 => { // indexOf(needle[, from])
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(VMValue::Integer(hako_core_string::index_of(s, &needle, from)))
                }
                313 => { // lastIndexOf(needle[, from])
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let from = args.get(1).map(|v| v.as_integer().unwrap_or(hako_core_string::length_bytes(s))).unwrap_or(hako_core_string::length_bytes(s));
                    Ok(VMValue::Integer(hako_core_string::last_index_of(s, &needle, from)))
                }
                314 => { // charAt(idx)
                    let idx = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                    Ok(VMValue::String(hako_core_string::char_at_byte(s, idx)))
                }

                302 => { // concat(rhs)
                    let rhs = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let mut out = String::with_capacity(s.len() + rhs.len());
                    out.push_str(s);
                    out.push_str(&rhs);
                    Ok(VMValue::String(out))
                }
                304 => { // replace(from, to)
                    let from = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    let to = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::String(hako_core_string::replace_all(s, &from, &to)))
                }
                305 => { // trim()
                    Ok(VMValue::String(s.trim().to_string()))
                }
                306 => { // toUpper()
                    Ok(VMValue::String(s.to_uppercase()))
                }
                307 => { // toLower()
                    Ok(VMValue::String(s.to_lowercase()))
                }
                308 => { // toString()
                    Ok(VMValue::String(s.clone()))
                }
                309 => { // stringify() — keep identity for now (JSON.stringify exists as module)
                    Ok(VMValue::String(s.clone()))
                }
                310 => { // startsWith(needle)
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::Bool(s.starts_with(&needle)))
                }
                311 => { // endsWith(needle)
                    let needle = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                    Ok(VMValue::Bool(s.ends_with(&needle)))
                }
                312 => Ok(VMValue::Bool(hako_core_string::is_empty(s))),
                _ => Err(VMError::InvalidInstruction(format!("Unknown String slot {} (method={})", slot, method))),
            };
            return res;
        } else {
            return Err(VMError::InvalidInstruction(format!("Unknown String method: {} (arity={})", method, args.len())));
        }
    }
    // BoxRef
    if let VMValue::BoxRef(bx) = receiver {
        // Plugin TypeBox v2
        if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            if let Some(result) = MapCallableBox::try_route(_interp, receiver, method, args) {
                return result;
            }
            let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for v in args {
                if let VMValue::BoxRef(bx) = v {
                    if crate::runtime::type_registry::is_core_box(bx.type_name()) {
                        let h = crate::runtime::host_handles::to_handle_arc(bx.clone());
                        argv.push(Box::new(crate::runtime::host_handle_box::HostHandleBox::new(h)));
                        continue;
                    }
                }
                argv.push(v.to_nyash_box());
            }
            let out = crate::runtime::plugin_host_box::invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv);
            // Normalize mutator returns to Void for core collections（remove/delete は値返却のため除外）
            let is_core = crate::runtime::type_registry::is_core_box(&p.box_type);
            let is_void_method = is_core && matches!(method, "set" | "push" | "clear");
            return match out {
                Ok(Some(ret)) => {
                    if is_void_method { Ok(VMValue::Void) } else { Ok(VMValue::from_nyash_box(ret)) }
                }
                Ok(None) => Ok(VMValue::Void),
                Err(e) => Err(VMError::InvalidInstruction(format!("Plugin method {}.{} failed: {:?}", p.box_type, method, e))),
            };
        }
        // Builtin core boxes
        match bx.type_name() {
            "CallableBox" => {
                if let Some(cb) = bx.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
                    let _ = maybe_arity_guard("CallableBox", method, args.len());
                    if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("CallableBox", method, args.len()) {
                        return match slot as u32 {
                            500 => Ok(VMValue::Integer(cb.arity() as i64)),
                            503 => Ok(VMValue::String(cb.to_string_box().value)),
                            501 => {
                                let argv: Vec<VMValue> = hako_core_callable::flatten_argv(args,
                                    |v: &VMValue| {
                                        if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() } else { false }
                                    },
                                    |v: &VMValue| {
                                        if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().map(|a| a.items.read().unwrap().len()).unwrap_or(0) } else { 0 }
                                    },
                                    |v: &VMValue, i: usize| {
                                        if let VMValue::BoxRef(bx)=v {
                                            if let Some(a)=bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                                let guard=a.items.read().unwrap();
                                                VMValue::from_nyash_box(guard[i].clone_box())
                                            } else { v.clone() }
                                        } else { v.clone() }
                                    });
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
                                        // Early async scheduling path (plugin or builtin)
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
                                            // Proactively poll once to kick the scheduler
                                            crate::runtime::global_hooks::safepoint_and_poll();
                                            // Fallback: if not ready, compute synchronously in current interpreter context
                                            if !fut.ready() {
                                                let recv_vm2 = VMValue::BoxRef(std::sync::Arc::from(recv.share_box()));
                                                let mut argv2: Vec<VMValue> = Vec::new();
                                                if args.len() == 1 {
                                                    if let VMValue::BoxRef(arrbx) = &args[0] {
                                                        if let Some(arr) = arrbx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                                            let guard = arr.items.read().unwrap();
                                                            for it in guard.iter() { argv2.push(VMValue::from_nyash_box(it.clone_box())); }
                                                        } else { argv2.push(args[0].clone()); }
                                                    } else { argv2.push(args[0].clone()); }
                                                } else { argv2.extend_from_slice(args); }
                                                match crate::runtime::method_router_box::route(_interp, &recv_vm2, &cb.method, &argv2) {
                                                    Ok(v) => fut.set_result(v.to_nyash_box()),
                                                    Err(_) => fut.set_result(Box::new(crate::box_trait::StringBox::new("invoke_failed")))
                                                }
                                            }
                                            return Ok(VMValue::from_nyash_box(Box::new(fut)));
                                        } else {
                                            // Builtin receiver — schedule via single-thread scheduler and resolve later
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
                                            // Proactively poll once to kick the scheduler
                                            crate::runtime::global_hooks::safepoint_and_poll();
                                            return Ok(VMValue::from_nyash_box(Box::new(fut)));
                                        }
                                    }
                                }
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
                                            // Proactively poll once to kick the scheduler
                                            crate::runtime::global_hooks::safepoint_and_poll();
                                            return Ok(VMValue::from_nyash_box(Box::new(fut)));
                                        }
                                    }
                                }
                                let res = if let Some(recv) = &cb.receiver {
                                    let recv_vm = VMValue::BoxRef(std::sync::Arc::from(recv.share_box()));
                                    let mut argv: Vec<VMValue> = Vec::new();
                                    if args.len() == 1 {
                                        if let VMValue::BoxRef(arrbx) = &args[0] {
                                            if let Some(arr) = arrbx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                                let guard = arr.items.read().unwrap();
                                                for it in guard.iter() { argv.push(VMValue::from_nyash_box(it.clone_box())); }
                                            } else { argv.push(args[0].clone()); }
                                        } else { argv.push(args[0].clone()); }
                                    } else { argv.extend_from_slice(args); }
                                    crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
                                } else { Err(VMError::InvalidInstruction("CallableBox without receiver is not supported yet".into())) };
                                match res {
                                    Ok(v) => { fut.set_result(v.to_nyash_box()); Ok(VMValue::from_nyash_box(Box::new(fut))) }
                                    Err(e) => Err(e),
                                }
                            }
                            _ => Err(VMError::InvalidInstruction(format!("Unknown CallableBox slot {} (method={})", slot, method))),
                        };
                    }
                    return Err(VMError::InvalidInstruction(format!("Unknown CallableBox method: {} (arity={})", method, args.len())));
                }
                return Err(VMError::InvalidInstruction("CallableBox downcast failed".into()));
            },
            "ArrayBox" => {
                if let Some(arr) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    let _ = maybe_arity_guard("ArrayBox", method, args.len());
                    return if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("ArrayBox", method, args.len()) {
                        match slot as u32 {
                            102 => Ok(VMValue::Integer(arr.len() as i64)),
                            100 => { // get(index)
                                let idx = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                                Ok(VMValue::from_nyash_box(arr.get(Box::new(crate::box_trait::IntegerBox::new(idx)))))
                            }
                            101 => { // set(index, value) -> Void
                                let idx = args.get(0).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
                                let val = args.get(1).map(|v| v.to_nyash_box()).unwrap_or_else(|| Box::new(crate::box_trait::VoidBox::new()));
                                let _ = arr.set(Box::new(crate::box_trait::IntegerBox::new(idx)), val);
                                Ok(VMValue::Void)
                            }
                            103 => { // push(value) -> Void
                                let val = args.get(0).map(|v| v.to_nyash_box()).unwrap_or_else(|| Box::new(crate::box_trait::VoidBox::new()));
                                let _ = arr.push(val);
                                Ok(VMValue::Void)
                            }
                            111 => { // slice(start,end)
                                if args.len() != 2 { return Err(VMError::InvalidInstruction(format!("No matching method: ArrayBox.slice({} args). Available arities: [2]", args.len()))); }
                                let s_i = args[0].as_integer().unwrap_or(0);
                                let e_i = args[1].as_integer().unwrap_or(0);
                                Ok(VMValue::from_nyash_box(arr.slice(
                                    Box::new(crate::box_trait::IntegerBox::new(s_i)),
                                    Box::new(crate::box_trait::IntegerBox::new(e_i)),
                                )))
                            }
                            112 => { // toJSON
                                let s = crate::boxes::json::stringify_any(arr.clone_box());
                                Ok(VMValue::String(s))
                            }
                            // slot 113 (methodRef) is handled as a VM pseudo-method earlier
                            104 => { // pop
                                Ok(VMValue::from_nyash_box(arr.pop()))
                            }
                            105 => { let _ = arr.clear(); Ok(VMValue::Void) }
                            106 => { // contains
                                let vb = args.get(0).map(|v| v.to_nyash_box()).unwrap_or_else(|| Box::new(crate::boxes::null_box::NullBox::new()));
                                let bb = arr.contains(vb);
                                if let Some(b) = bb.as_any().downcast_ref::<crate::box_trait::BoolBox>() { Ok(VMValue::Bool(b.value)) } else { Ok(VMValue::Bool(false)) }
                            }
                            107 => { // indexOf
                                let vb = args.get(0).map(|v| v.to_nyash_box()).unwrap_or_else(|| Box::new(crate::boxes::null_box::NullBox::new()));
                                let ib = arr.indexOf(vb);
                                if let Some(ii) = ib.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { Ok(VMValue::Integer(ii.value)) } else { Ok(VMValue::Integer(-1)) }
                            }
                            108 => { // join
                                let sep = args.get(0).map(|v| v.to_string()).unwrap_or_default();
                                Ok(VMValue::from_nyash_box(arr.join(Box::new(crate::box_trait::StringBox::new(&sep)))))
                            }
                            109 => { let _ = arr.sort(); Ok(VMValue::Void) }
                            110 => { let _ = arr.reverse(); Ok(VMValue::Void) }
                            _ => Err(VMError::InvalidInstruction(format!("Unknown ArrayBox slot {} (method={})", slot, method))),
                        }
                    } else {
                        Err(VMError::InvalidInstruction(format!("Unknown ArrayBox method: {} (arity={})", method, args.len())))
                    };
                }
                return Err(VMError::InvalidInstruction("ArrayBox downcast failed".into()));
            }
            "MapBox" => {
                if let Some(mp) = bx.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let _ = maybe_arity_guard("MapBox", method, args.len());
                    return if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name("MapBox", method, args.len()) {
                        match slot as u32 {
                            200 | 201 => { // size/len
                                let ib = mp.size();
                                if let Some(ii) = ib.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { Ok(VMValue::Integer(ii.value)) } else { Ok(VMValue::Integer(0)) }
                            }
                            204 => { // set(key, val) -> Void
                                if args.len() != 2 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.set({} args). Available arities: [2]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                let val_box = args[1].to_nyash_box();
                                let _ = mp.set(key_box, val_box);
                                Ok(VMValue::Void)
                            }
                            203 => { // get(key) -> value|null
                                if args.len() != 1 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.get({} args). Available arities: [1]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                Ok(VMValue::from_nyash_box(mp.get(key_box)))
                            }
                            202 => { // has(key)
                                if args.len() != 1 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.has({} args). Available arities: [1]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                let bb = mp.has(key_box);
                                if let Some(b) = bb.as_any().downcast_ref::<crate::box_trait::BoolBox>() { Ok(VMValue::Bool(b.value)) } else { Ok(VMValue::Bool(false)) }
                            }
                            208 => { let _ = mp.clear(); Ok(VMValue::Void) }
                            205 => { // delete/remove
                                if args.len() != 1 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.delete({} args). Available arities: [1]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                Ok(VMValue::from_nyash_box(mp.delete(key_box)))
                            }
                            206 => Ok(VMValue::from_nyash_box(mp.keys())),
                            207 => Ok(VMValue::from_nyash_box(mp.values())),
                            209 => { let s = crate::boxes::json::stringify_any(mp.clone_box()); Ok(VMValue::String(s)) },
                            210 => { // call(key, argsArray)
                                if args.len() != 2 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.call({} args). Available arities: [2]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                let callee = mp.get(key_box);
                                if let Some(cb) = callee.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
                                    let recv_vm = VMValue::BoxRef(std::sync::Arc::from(cb.receiver.as_ref().ok_or_else(|| VMError::InvalidInstruction("CallableBox without receiver".into()))?.share_box()));
                                    // Reuse shared argv flattener: treat args[1] as the single argv-array input
                                    let argv: Vec<VMValue> = hako_core_callable::flatten_argv(
                                        &args[1..2],
                                        |v: &VMValue| { if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() } else { false } },
                                        |v: &VMValue| { if let VMValue::BoxRef(bx)=v { bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().map(|a| a.items.read().unwrap().len()).unwrap_or(0) } else { 0 } },
                                        |v: &VMValue, i: usize| { if let VMValue::BoxRef(bx)=v { if let Some(a)=bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() { let guard=a.items.read().unwrap(); VMValue::from_nyash_box(guard[i].clone_box()) } else { v.clone() } } else { v.clone() } }
                                    );
                                    crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
                                } else {
                                    Err(VMError::InvalidInstruction("Map.call: value is not CallableBox".into()))
                                }
                            }
                            211 => { // callAsync(key, argsArray)
                                if args.len() != 2 { return Err(VMError::InvalidInstruction(format!("No matching method: MapBox.callAsync({} args). Available arities: [2]", args.len()))); }
                                let key_box = args[0].to_nyash_box();
                                let callee = mp.get(key_box);
                                if let Some(cb) = callee.as_any().downcast_ref::<crate::boxes::callable::CallableBox>() {
                                    let cb_vm = VMValue::BoxRef(std::sync::Arc::new(cb.clone()));
                                    let args_vm = VMValue::BoxRef(std::sync::Arc::from(args[1].to_nyash_box()));
                                    crate::runtime::method_router_box::route(_interp, &cb_vm, "callAsync", &vec![args_vm])
                                } else {
                                    Err(VMError::InvalidInstruction("Map.callAsync: value is not CallableBox".into()))
                                }
                            }
                            _ => Err(VMError::InvalidInstruction(format!("Unknown MapBox slot {} (method={})", slot, method))),
                        }
                    } else {
                        Err(VMError::InvalidInstruction(format!("Unknown MapBox method: {} (arity={})", method, args.len())))
                    };
                }
                return Err(VMError::InvalidInstruction("MapBox downcast failed".into()));
            }

            _ => {}
        }
    }
    Err(VMError::InvalidInstruction(format!("Method {} not supported on {:?}", method, receiver)))
}
