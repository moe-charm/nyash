use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use crate::backend::vm::ControlFlow;
use crate::backend::{VM, VMError, VMValue};

impl VM {
    /// Execute ExternCall instruction
    pub(crate) fn execute_extern_call(&mut self, dst: Option<ValueId>, iface_name: &str, method_name: &str, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Optional routing to name→slot handlers for stability and diagnostics
        if crate::config::env::extern_route_slots() {
            if let Some(slot) = crate::runtime::extern_registry::resolve_slot(iface_name, method_name) {
                // Decode args to VMValue as needed by handlers below
                let vm_args: Vec<VMValue> = args.iter().filter_map(|a| self.get_value(*a).ok()).collect();
                match (iface_name, method_name, slot) {
                    ("env.console", m @ ("log" | "warn" | "error" | "println"), 10) => {
                        if let Some(a0) = vm_args.get(0) {
                            match m { "warn" => eprintln!("[warn] {}", a0.to_string()), "error" => eprintln!("[error] {}", a0.to_string()), _ => println!("{}", a0.to_string()), }
                        }
                        if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.debug", "trace", 11) => {
                        if let Some(a0) = vm_args.get(0) { eprintln!("[trace] {}", a0.to_string()); }
                        if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.runtime", "checkpoint", 12) => {
                        if crate::config::env::runtime_checkpoint_trace() {
                            let (func, bb, pc) = self.gc_site_info();
                            eprintln!("[rt] checkpoint @{} bb={} pc={}", func, bb, pc);
                        }
                        self.runtime.gc.safepoint();
                        if let Some(s) = &self.runtime.scheduler { s.poll(); }
                        if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.future", "new", 20) | ("env.future", "birth", 20) => {
                        // Create a new Future and optionally set initial value from arg0
                        let fut = crate::boxes::future::FutureBox::new();
                        if let Some(a0) = vm_args.get(0) { fut.set_result(a0.to_nyash_box()); }
                        if let Some(d) = dst { self.set_value(d, VMValue::Future(fut)); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.future", "set", 21) => {
                        // set(future, value)
                        if vm_args.len() >= 2 { if let VMValue::Future(f) = &vm_args[0] { f.set_result(vm_args[1].to_nyash_box()); } }
                        if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.future", "await", 22) => {
                        if let Some(VMValue::Future(fb)) = vm_args.get(0) {
                            // Simple blocking await
                            let start = std::time::Instant::now();
                            let max_ms = crate::config::env::await_max_ms();
                            while !fb.ready() {
                                std::thread::yield_now();
                                if start.elapsed() >= std::time::Duration::from_millis(max_ms) { break; }
                            }
                            let result = if fb.ready() { fb.get() } else { Box::new(crate::box_trait::StringBox::new("Timeout")) };
                            let ok = crate::boxes::result::NyashResultBox::new_ok(result);
                            if let Some(d) = dst { self.set_value(d, VMValue::from_nyash_box(Box::new(ok))); }
                        } else if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    ("env.task", "cancelCurrent", 30) => { if let Some(d) = dst { self.set_value(d, VMValue::Void); } return Ok(ControlFlow::Continue); }
                    ("env.task", "currentToken", 31) => { if let Some(d) = dst { self.set_value(d, VMValue::Integer(0)); } return Ok(ControlFlow::Continue); }
                    ("env.task", "yieldNow", 32) => { std::thread::yield_now(); if let Some(d) = dst { self.set_value(d, VMValue::Void); } return Ok(ControlFlow::Continue); }
                    ("env.task", "sleepMs", 33) => {
                        let ms = vm_args.get(0).map(|v| match v { VMValue::Integer(i) => *i, _ => 0 }).unwrap_or(0);
                        if ms > 0 { std::thread::sleep(std::time::Duration::from_millis(ms as u64)); }
                        if let Some(d) = dst { self.set_value(d, VMValue::Void); }
                        return Ok(ControlFlow::Continue);
                    }
                    _ => { /* fallthrough to host */ }
                }
            }
        }

        // Evaluate arguments as NyashBox for loader
        let mut nyash_args: Vec<Box<dyn NyashBox>> = Vec::new();
        for arg_id in args { let arg_value = self.get_value(*arg_id)?; nyash_args.push(arg_value.to_nyash_box()); }

        if crate::config::env::extern_trace() {
            if let Some(slot) = crate::runtime::extern_registry::resolve_slot(iface_name, method_name) {
                eprintln!("[EXT] call {}.{} slot={} argc={}", iface_name, method_name, slot, nyash_args.len());
            } else { eprintln!("[EXT] call {}.{} argc={}", iface_name, method_name, nyash_args.len()); }
        }
        let host = crate::runtime::get_global_plugin_host();
        let host = host.read().map_err(|_| VMError::InvalidInstruction("Plugin host lock poisoned".into()))?;
        match host.extern_call(iface_name, method_name, &nyash_args) {
            Ok(Some(result_box)) => { if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(result_box)); } }
            Ok(None) => { if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Void); } }
            Err(_) => {
                let strict = crate::config::env::extern_strict() || crate::config::env::abi_strict();
                let mut msg = String::new();
                if strict { msg.push_str("ExternCall STRICT: unregistered or unsupported call "); } else { msg.push_str("ExternCall failed: "); }
                msg.push_str(&format!("{}.{}", iface_name, method_name));
                if let Err(detail) = crate::runtime::extern_registry::check_arity(iface_name, method_name, nyash_args.len()) { msg.push_str(&format!(" ({})", detail)); }
                if let Some(spec) = crate::runtime::extern_registry::resolve(iface_name, method_name) {
                    msg.push_str(&format!(" (expected arity {}..{})", spec.min_arity, spec.max_arity));
                } else {
                    let known = crate::runtime::extern_registry::known_for_iface(iface_name);
                    if !known.is_empty() { msg.push_str(&format!("; known methods: {}", known.join(", "))); }
                    else { let ifaces = crate::runtime::extern_registry::all_ifaces(); msg.push_str(&format!("; known interfaces: {}", ifaces.join(", "))); }
                }
                return Err(VMError::InvalidInstruction(msg));
            }
        }
        Ok(ControlFlow::Continue)
    }
}
