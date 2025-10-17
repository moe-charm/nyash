#![cfg(feature = "legacy-boxes")]
//! Extern function execution (exit, panic, etc.)

use super::super::super::*;
use crate::runtime::meta::future::future_box::FutureBox;

impl MirInterpreter {
    fn normalize_extern_arg(&mut self, value: VMValue) -> VMValue {
        match value {
            VMValue::BoxRef(bx) => {
                if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                    return VMValue::String(sb.value.clone());
                }
                if let Some(ib) = bx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    return VMValue::Integer(ib.value);
                }
                if let Some(bb) = bx.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                    return VMValue::Bool(bb.value);
                }
                if let Some(pb) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "ArrayBox" {
                        let recv = VMValue::BoxRef(bx.clone());
                        if let Ok(VMValue::Integer(n)) = crate::runtime::method_router_box::route(self, &recv, &"length".to_string(), &[]) {
                            if n == 1 {
                                let idx = VMValue::Integer(0);
                                if let Ok(elem) = crate::runtime::method_router_box::route(self, &recv, &"get".to_string(), &[idx]) {
                                    return self.normalize_extern_arg(elem);
                                }
                            }
                        }
                    } else if pb.box_type == "StringBox" {
                        if let Ok(VMValue::String(s)) = crate::runtime::method_router_box::route(self, &VMValue::BoxRef(bx.clone()), &"value".to_string(), &[]) {
                            return VMValue::String(s);
                        }
                    }
                }
                VMValue::BoxRef(bx)
            }
            other => other,
        }
    }
    pub(crate) fn execute_extern_function(
        &mut self,
        extern_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // Unified dotted extern support: e.g., "nyrt.ops.op_eq"
        if let Some((iface, method)) = extern_name.rsplit_once('.') {
            // Trampoline for nyvm (Ny implementation): delegate externs back to Rust handlers
            if iface == "env.extern" && method == "invoke" {
                // Expect (name: String, args: Array?)
                if args.len() < 1 { return Err(VMError::InvalidInstruction("env.extern.invoke requires name".into())); }
                let target = self.reg_load(args[0])?.to_string();
                // Optional: collect forwarded args when second arg is an ArrayBox (builtin or plugin)
                let mut _forwarded: Vec<VMValue> = Vec::new();
                if let Some(arg_id) = args.get(1) {
                    let val = self.reg_load(*arg_id)?;
                    if let VMValue::BoxRef(bx) = &val {
                        if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() || bx.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
                            // no-op
                        } else if let Some(arr) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                            for i in 0..arr.len() { _forwarded.push(VMValue::from_nyash_box(arr.get(Box::new(crate::box_trait::IntegerBox::new(i as i64))))); }
                        } else if bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                            // Plugin path: try length/get via method router
                            let len = crate::runtime::method_router_box::route(self, &val, &"length".to_string(), &[])?;
                            let n = match len { VMValue::Integer(n) => n as usize, VMValue::BoxRef(b) => { if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { ib.value as usize } else { 0 } }, _ => 0 };
                            for i in 0..n { let idx = VMValue::Integer(i as i64); let vv = crate::runtime::method_router_box::route(self, &val, &"get".to_string(), &[idx])?; _forwarded.push(vv); }
                        } else {
                            // Unknown box kind for args: ignore (no forwarded args)
                        }
                    }
                }
                if !target.contains('.') {
                    return Err(VMError::InvalidInstruction(format!("env.extern.invoke: invalid target '{}' : expected dotted iface.method", target)));
                }
                let mut parts = target.rsplitn(2, '.');
                let method = parts.next().unwrap_or("");
                let iface = parts.next().unwrap_or("");
                if iface == "hostbridge" && method == "box_new" {
                    if _forwarded.len() < 1 { return Err(VMError::InvalidInstruction("hostbridge.box_new: missing name".into())); }
                    let name = _forwarded[0].to_string();
                    let mut ny_args: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                    for v in _forwarded.iter().skip(1) { ny_args.push(v.to_nyash_box()); }
                    crate::runtime::provider_box::ensure_loaded(None);
                    match crate::runtime::provider_box::new_box(&name, &ny_args) {
                        Ok(bx) => return Ok(VMValue::from_nyash_box(bx)),
                        Err(e) => return Err(VMError::InvalidInstruction(format!("hostbridge.box_new error: {:?}", e))),
                    }
                }
                if iface == "hostbridge" && method == "box_call" {
                    if _forwarded.len() < 2 { return Err(VMError::InvalidInstruction("hostbridge.box_call: missing receiver/method".into())); }
                    let recv = _forwarded[0].clone();
                    let method_name = _forwarded[1].to_string();
                    // Third element is original args array; flatten into call_args
                    let mut call_args: Vec<VMValue> = Vec::new();
                    if let Some(arg_array) = _forwarded.get(2) {
                        match arg_array {
                            VMValue::BoxRef(bx) => {
                                if let Some(arr) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                    for i in 0..arr.len() {
                                        call_args.push(VMValue::from_nyash_box(
                                            arr.get(Box::new(crate::box_trait::IntegerBox::new(i as i64)))
                                        ));
                                    }
                                } else if bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                                    let len = crate::runtime::method_router_box::route(self, arg_array, &"length".to_string(), &[])?;
                                    let n = match len { VMValue::Integer(n) => n as usize, VMValue::BoxRef(b) => { if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { ib.value as usize } else { 0 } }, _ => 0 };
                                    for i in 0..n { let idx = VMValue::Integer(i as i64); let vv = crate::runtime::method_router_box::route(self, arg_array, &"get".to_string(), &[idx])?; call_args.push(vv); }
                                }
                            }
                            _ => {}
                        }
                    }
                    let ret = crate::runtime::method_router_box::route(self, &recv, &method_name, &call_args)?;
                    if let VMValue::BoxRef(bx) = &ret {
                        if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                            return Ok(VMValue::Void);
                        }
                    }
                    if method_name == "root" {
                        if let VMValue::Void = ret {
                            return Ok(VMValue::BoxRef(std::sync::Arc::new(
                                crate::boxes::null_box::NullBox::new(),
                            )));
                        }
                    }

                    if std::env::var("HAKO_DEBUG_TRAMPOLINE").is_ok() {
                        eprintln!("[TRAMPOLINE] box_call: method={}, result={}",
                                  method_name,
                                  match &ret {
                                      VMValue::BoxRef(b) => format!("BoxRef({})", b.type_name()),
                                      VMValue::Void => "Void".to_string(),
                                      _ => format!("{:?}", ret),
                                  });
                    }

                    return Ok(ret);
                }
                if let Some(r) = self.extern_call_public(iface, method, &_forwarded) { return r; }
                return Err(VMError::InvalidInstruction(format!("env.extern.invoke: unknown target '{}'", target)));
            }

            // Route selected iface.method pairs to dedicated handlers
            if iface == "nyrt.ops" && method == "op_eq" {
                if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.ops.op_eq requires 2 args".into())); }
                let a = self.reg_load(args[0])?;
                let b = self.reg_load(args[1])?;
                let ok = self.eval_equals(&a, &b)?;
                return Ok(VMValue::Bool(ok));
            }
            // Async spawn: env.future.spawn_instance(receiver, methodName, ...args)
            if iface == "env.future" && method == "spawn_instance" {
                // Load all args eagerly
                let mut loaded: Vec<VMValue> = Vec::with_capacity(args.len());
                for a in args { loaded.push(self.reg_load(*a)?); }
                // Expect at least (receiver, methodName)
                if loaded.len() < 2 {
                    return Err(VMError::InvalidInstruction("env.future.spawn_instance requires receiver and method name".into()));
                }
                let receiver = loaded.remove(0);
                let method_name = loaded.remove(0).to_string();
                let call_args: Vec<VMValue> = loaded; // remaining are args
                // Route synchronously via MethodRouter and resolve the result now
                let result = crate::runtime::method_router_box::route(self, &receiver, &method_name, &call_args)?;
                let fut = FutureBox::new();
                fut.set_result(result.to_nyash_box());
                return Ok(VMValue::Future(fut));
            }
            // HostBridge minimal externs (Phase B): hostbridge.box_new(name), hostbridge.box_call(recv, method, ...args)
            if iface == "hostbridge" && method == "box_new" {
                // Expect at least (name)
                if args.len() < 1 { return Err(VMError::InvalidInstruction("hostbridge.box_new requires name".into())); }
                let name = self.reg_load(args[0])?.to_string();
                // Load all remaining args as NyashBox for future expansion (Phase A ignores)
                let mut ny_args: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                for a in args.iter().skip(1) { ny_args.push(self.reg_load(*a)?.to_nyash_box()); }
                crate::runtime::provider_box::ensure_loaded(None);
                match crate::runtime::provider_box::new_box(&name, &ny_args) {
                    Ok(bx) => return Ok(VMValue::from_nyash_box(bx)),
                    Err(e) => return Err(VMError::InvalidInstruction(format!("hostbridge.box_new error: {:?}", e))),
                }
            }
            if iface == "hostbridge" && method == "box_call" {
                // Expect at least (receiver, method)
                if args.len() < 2 { return Err(VMError::InvalidInstruction("hostbridge.box_call requires receiver and method".into())); }
                let recv = self.reg_load(args[0])?;
                let method_name = self.reg_load(args[1])?.to_string();
                let mut call_args: Vec<VMValue> = Vec::new();
                for a in args.iter().skip(2) { call_args.push(self.reg_load(*a)?); }
                let ret = crate::runtime::method_router_box::route(self, &recv, &method_name, &call_args)?;

                if std::env::var("HAKO_DEBUG_TRAMPOLINE").is_ok() {
                    eprintln!("[TRAMPOLINE] box_call (fallback): method={}, result={}",
                              method_name,
                              match &ret {
                                  VMValue::BoxRef(b) => format!("BoxRef({})", b.type_name()),
                                  VMValue::Void => "Void".to_string(),
                                  _ => format!("{:?}", ret),
                              });
                }

                return Ok(ret);
            }
            if iface == "hostbridge" && method == "extern_invoke" {
                // Trampoline for nyvm: forward extern calls to Rust handlers
                // Expect (target_name: String, args_array: Array?)
                if args.len() < 1 { return Err(VMError::InvalidInstruction("hostbridge.extern_invoke requires target name".into())); }
                let target = self.reg_load(args[0])?.to_string();

                // Load args array if provided
                let mut forwarded: Vec<VMValue> = Vec::new();
                if let Some(arg_id) = args.get(1) {
                    let val = self.reg_load(*arg_id)?;
                    if let VMValue::BoxRef(bx) = &val {
                        if let Some(arr) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                            for i in 0..arr.len() {
                                let elem = arr.get(Box::new(crate::box_trait::IntegerBox::new(i as i64)));
                                forwarded.push(self.normalize_extern_arg(VMValue::from_nyash_box(elem)));
                            }
                        } else if bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some() {
                            // Plugin ArrayBox
                            let len = crate::runtime::method_router_box::route(self, &val, &"length".to_string(), &[])?;
                            let n = match len { VMValue::Integer(n) => n as usize, _ => 0 };
                            for i in 0..n {
                                let idx = VMValue::Integer(i as i64);
                                let vv = crate::runtime::method_router_box::route(self, &val, &"get".to_string(), &[idx])?;
                                forwarded.push(self.normalize_extern_arg(vv));
                            }
                        } else {
                            // Unknown box — treat as primitive via normalize
                            forwarded.push(self.normalize_extern_arg(VMValue::BoxRef(bx.clone())));
                        }
                    } else {
                        forwarded.push(self.normalize_extern_arg(val));
                    }
                }

                // Parse target as iface.method
                if !target.contains('.') {
                    return Err(VMError::InvalidInstruction(format!("hostbridge.extern_invoke: invalid target '{}': expected dotted iface.method", target)));
                }
                let mut parts = target.rsplitn(2, '.');
                let method_name = parts.next().unwrap_or("");
                let iface_name = parts.next().unwrap_or("");

                // Try extern adapter
                if std::env::var("HAKO_DEBUG_TRAMPOLINE").is_ok() {
                    eprintln!("[TRAMPOLINE] extern_invoke: target={}.{} forwarded_len={} forwarded[0]={:?}",
                              iface_name, method_name, forwarded.len(),
                              forwarded.get(0));
                }
                if let Some(r) = self.extern_call_public(iface_name, method_name, &forwarded) {
                    if std::env::var("HAKO_DEBUG_TRAMPOLINE").is_ok() {
                        eprintln!("[TRAMPOLINE] extern_invoke result: {:?}", match &r {
                            Ok(VMValue::String(s)) => format!("Ok(String({}))", s),
                            Ok(VMValue::BoxRef(b)) => format!("Ok(BoxRef({}))", b.type_name()),
                            Ok(v) => format!("Ok({:?})", v),
                            Err(e) => format!("Err({:?})", e),
                        });
                    }
                    return r;
                }

                return Err(VMError::InvalidInstruction(format!("hostbridge.extern_invoke: unknown target '{}'", target)));
            }
            // Generic VM extern adapter (nyrt.time.now_ms, nyrt.array.size, nyrt.map.size, ...)
            {
                let mut loaded: Vec<VMValue> = Vec::with_capacity(args.len());
                for a in args { loaded.push(self.reg_load(*a)?); }
                if let Some(r) = crate::backend::mir_interpreter::extern_adapter::try_call(iface, method, &loaded) {
                    return r;
                }
            }
            // Unknown dotted extern — return error (use legacy ExternCall path only for supported iface.method)
            return Err(VMError::InvalidInstruction(format!(
                "ExternCall {}.{} not supported",
                iface, method
            )));
        }
        match extern_name {
            "exit" => {
                let code = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.as_integer().unwrap_or(0)
                } else {
                    0
                };
                std::process::exit(code as i32);
            }
            "panic" => {
                let msg = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.to_string()
                } else {
                    "VM panic".to_string()
                };
                panic!("{}", msg);
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown extern function: {}",
                extern_name
            ))),
        }
    }
}
