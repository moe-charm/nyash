use super::super::builder::IRBuilder;
use super::super::core::LowerCore;
use crate::mir::{MirFunction, ValueId};

impl LowerCore {
    pub fn lower_plugin_invoke(
        &mut self,
        b: &mut dyn IRBuilder,
        dst: &Option<ValueId>,
        box_val: &ValueId,
        method: &str,
        args: &Vec<ValueId>,
        _func: &MirFunction,
    ) -> Result<(), String> {
        // Copied logic from core.rs PluginInvoke arm (scoped to PyRuntimeBox path)
        let bt = self.box_type_map.get(box_val).cloned().unwrap_or_default();
        let m = method;
        if (bt == "PyRuntimeBox" && (m == "import")) {
            let argc = 1 + args.len();
            if let Some(pidx) = self.param_index.get(box_val).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
            let decision = crate::jit::policy::invoke::decide_box_method(&bt, m, argc, dst.is_some());
            if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, .. } = decision {
                b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                if let Some(d) = dst { self.handle_values.insert(*d); }
            } else { if dst.is_some() { b.emit_const_i64(0); } }
        } else if (bt == "PyRuntimeBox" && (m == "getattr" || m == "call")) {
            let argc = 1 + args.len();
            if let Some(pidx) = self.param_index.get(box_val).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
            for a in args.iter() { self.push_value_if_known_or_param(b, a); }
            b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.store_local_i64(slot);
            }
        } else if self.handle_values.contains(box_val) && (m == "getattr" || m == "call") {
            let argc = 1 + args.len();
            b.emit_const_i64(-1);
            for a in args.iter() { self.push_value_if_known_or_param(b, a); }
            b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.store_local_i64(slot);
            }
        } else if (bt == "PyRuntimeBox" && (m == "birth" || m == "eval"))
            || (bt == "IntegerBox" && m == "birth")
            || (bt == "StringBox" && m == "birth")
            || (bt == "ConsoleBox" && m == "birth") {
            if dst.is_some() { b.emit_const_i64(0); }
        } else {
            self.unsupported += 1;
        }
        Ok(())
    }

    pub fn lower_extern_call(
        &mut self,
        b: &mut dyn IRBuilder,
        dst: &Option<ValueId>,
        iface_name: &str,
        method_name: &str,
        args: &Vec<ValueId>,
        _func: &MirFunction,
    ) -> Result<(), String> {
        // env.console.log/warn/error/println → ConsoleBox に委譲（host-bridge有効時は直接ログ）
        if iface_name == "env.console" && (method_name == "log" || method_name == "println" || method_name == "warn" || method_name == "error") {
            if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
                // a0: 先頭引数を最小限で積む
                if let Some(arg0) = args.get(0) { self.push_value_if_known_or_param(b, arg0); } else { b.emit_const_i64(0); }
                let sym = match method_name {
                    "warn" => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_WARN,
                    "error" => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_ERROR,
                    _ => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_LOG,
                };
                b.emit_host_call(sym, 1, false);
                return Ok(());
            }
            // Ensure we have a Console handle (hostcall birth shim)
            b.emit_host_call("nyash.console.birth_h", 0, true);
            // a1: first argument best-effort
            if let Some(arg0) = args.get(0) { self.push_value_if_known_or_param(b, arg0); }
            // Resolve plugin invoke for ConsoleBox.method
            let decision = crate::jit::policy::invoke::decide_box_method("ConsoleBox", method_name, 2, dst.is_some());
            if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, .. } = decision {
                b.emit_plugin_invoke(type_id, method_id, 2, dst.is_some());
            } else if dst.is_some() { b.emit_const_i64(0); }
            return Ok(());
        }
        // env.future.await(fut) → await_h + ok_h/err_h select
        if iface_name == "env.future" && method_name == "await" {
            if let Some(arg0) = args.get(0) {
                if let Some(pidx) = self.param_index.get(arg0).copied() { b.emit_param_i64(pidx); }
                else if let Some(slot) = self.local_index.get(arg0).copied() { b.load_local_i64(slot); }
                else if let Some(v) = self.known_i64.get(arg0).copied() { b.emit_const_i64(v); }
                else { b.emit_const_i64(-1); }
            } else { b.emit_const_i64(-1); }
            // await_h → handle(0 timeout)
            b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, 1, true);
            let hslot = { let id = self.next_local; self.next_local += 1; id };
            b.store_local_i64(hslot);
            // ok_h(handle)
            b.load_local_i64(hslot);
            b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_OK_H, 1, true);
            let ok_slot = { let id = self.next_local; self.next_local += 1; id };
            b.store_local_i64(ok_slot);
            // err_h(0)
            b.emit_const_i64(0);
            b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_ERR_H, 1, true);
            let err_slot = { let id = self.next_local; self.next_local += 1; id };
            b.store_local_i64(err_slot);
            // select(handle==0 ? err : ok)
            b.load_local_i64(hslot);
            b.emit_const_i64(0);
            b.emit_compare(crate::jit::lower::builder::CmpKind::Eq);
            b.load_local_i64(err_slot);
            b.load_local_i64(ok_slot);
            b.emit_select_i64();
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.store_local_i64(slot);
            }
            return Ok(());
        }
        // env.future.spawn_instance(recv, method_name, args...)
        if iface_name == "env.future" && method_name == "spawn_instance" {
            // a0 receiver
            if let Some(recv) = args.get(0) {
                if let Some(pidx) = self.param_index.get(recv).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
            } else { b.emit_const_i64(-1); }
            // a1 method name (best-effort)
            if let Some(meth) = args.get(1) { self.push_value_if_known_or_param(b, meth); } else { b.emit_const_i64(0); }
            // a2 first payload (optional)
            if let Some(a2) = args.get(2) { self.push_value_if_known_or_param(b, a2); } else { b.emit_const_i64(0); }
            // argc_total = explicit args including method name and payload (exclude receiver)
            let argc_total = args.len().saturating_sub(1).max(0);
            b.emit_const_i64(argc_total as i64);
            // call spawn shim → Future handle
            b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_SPAWN_INSTANCE3_I64, 4, true);
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| { let id = self.next_local; self.next_local += 1; id });
                b.store_local_i64(slot);
            }
            return Ok(());
        }
        // Unhandled extern path
        self.unsupported += 1;
        Ok(())
    }

    pub fn lower_box_call(
        &mut self,
        func: &MirFunction,
        b: &mut dyn IRBuilder,
        array: &ValueId,
        method: &str,
        args: &Vec<ValueId>,
        dst: Option<ValueId>,
    ) -> Result<bool, String> {
        // Note: simple_reads は後段の分岐のフォールバックとして扱う（String/Instance優先）
        if matches!(method, "sin" | "cos" | "abs" | "min" | "max") {
            super::super::core_hostcall::lower_math_call(
                func,
                b,
                &self.known_i64,
                &self.known_f64,
                &self.float_box_values,
                method,
                args,
                dst.clone(),
            );
            return Ok(true);
        }
        // Builtins-to-plugin path (subset for String/Array/Map critical ops)
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1") {
            // StringBox (length/is_empty/charCodeAt)
            if matches!(method, "length" | "is_empty" | "charCodeAt") {
                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                let mut argc = 1usize;
                if method == "charCodeAt" {
                    if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                    argc = 2;
                }
                if method == "is_empty" { b.hint_ret_bool(true); }
                let decision = crate::jit::policy::invoke::decide_box_method("StringBox", method, argc, dst.is_some());
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                        crate::jit::observe::lower_plugin_invoke(&box_type, method, type_id, method_id, argc);
                        return Ok(true);
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(&symbol, argc, &if argc==1 { ["Handle"][..].to_vec() } else { ["Handle","I64"][..].to_vec() }, "allow", "mapped_symbol");
                        b.emit_host_call(&symbol, argc, dst.is_some());
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
        // Array/Map minimal handling
        match method {
            // Instance field ops via host-bridge
            "getField" | "setField" => {
                if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
                    // receiver: allow param/local/phi/known
                    if let Some(v) = args.get(0) { let _ = v; } // keep args in scope
                    self.push_value_if_known_or_param(b, array);
                    // name: if const string, build a StringBox handle from literal; else best-effort push
                    if let Some(name_id) = args.get(0) {
                        if let Some(s) = self.known_str.get(name_id).cloned() { b.emit_string_handle_from_literal(&s); }
                        else { b.emit_const_i64(0); }
                    } else { b.emit_const_i64(0); }
                    // value for setField
                    let argc = if method == "setField" {
                        if let Some(val_id) = args.get(1) {
                            if let Some(s) = self.known_str.get(val_id).cloned() { b.emit_string_handle_from_literal(&s); }
                            else { self.push_value_if_known_or_param(b, val_id); }
                        } else { b.emit_const_i64(0); }
                        3
                    } else { 2 };
                    // Unified 3-arity call: getField uses val=-1 sentinel
                    let sym = crate::jit::r#extern::host_bridge::SYM_HOST_INSTANCE_FIELD3;
                    if method == "getField" { b.emit_const_i64(-1); }
                    b.emit_host_call_fixed3(sym, dst.is_some());
                    return Ok(true);
                }
            }
            // String.len: (1) const string → 定数埋め込み、(2) StringBox → host-bridge
            "len" => {
                // (1) const string literal case
                let mut lit_len: Option<i64> = None;
                for (_bbid, bb) in func.blocks.iter() {
                    for ins in bb.instructions.iter() {
                        if let crate::mir::MirInstruction::Const { dst, value } = ins {
                            if dst == array {
                                if let crate::mir::ConstValue::String(s) = value { lit_len = Some(s.len() as i64); }
                                break;
                            }
                        }
                    }
                    if lit_len.is_some() { break; }
                }
                if let Some(n) = lit_len {
                    b.emit_const_i64(n);
                    return Ok(true);
                }
                // (2) StringBox via host-bridge
                if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
                    if let Some(bt) = self.box_type_map.get(array) {
                        if bt == "StringBox" {
                            if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") { eprintln!("[LOWER]string.len via host-bridge"); }
                            self.push_value_if_known_or_param(b, array);
                            b.emit_host_call(crate::jit::r#extern::host_bridge::SYM_HOST_STRING_LEN, 1, dst.is_some());
                            return Ok(true);
                        }
                    }
                }
            }
            // Array length variants (length/len)
            "len" | "length" => {
                if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                    if let Ok(h) = ph.resolve_method("ArrayBox", "length") {
                        if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                        b.emit_plugin_invoke(h.type_id, h.method_id, 1, dst.is_some());
                        return Ok(true);
                    }
                }
                // Hostcall fallback
                if let Some(pidx) = self.param_index.get(array).copied() {
                    crate::jit::observe::lower_hostcall(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, &["Handle"], "allow", "mapped_symbol");
                    b.emit_param_i64(pidx);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, dst.is_some());
                } else {
                    crate::jit::observe::lower_hostcall(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, &["I64"], "fallback", "receiver_not_param");
                    b.emit_const_i64(-1);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ARRAY_LEN, 1, dst.is_some());
                }
                return Ok(true);
            }
            // Array push
            "push" => {
                let argc = 2usize;
                // receiver
                if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                // value
                if let Some(v) = args.get(0).and_then(|vid| self.known_i64.get(vid)).copied() { b.emit_const_i64(v); }
                else if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                // policy decide → plugin / hostcall fallback
                let decision = crate::jit::policy::invoke::decide_box_method("ArrayBox", "push", argc, false);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke { type_id, method_id, box_type, .. } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                        crate::jit::observe::lower_plugin_invoke(&box_type, "push", type_id, method_id, argc);
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(&symbol, argc, &["Handle","I64"], "allow", "mapped_symbol");
                        b.emit_host_call(&symbol, argc, false);
                    }
                    _ => {
                        // Fallback hostcall
                        let sym = if self.param_index.get(array).is_some() { crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H } else { crate::jit::r#extern::collections::SYM_ARRAY_PUSH };
                        let arg_types = if self.param_index.get(array).is_some() { &["Handle","I64"][..] } else { &["I64","I64"][..] };
                        crate::jit::observe::lower_hostcall(sym, argc, arg_types, "fallback", "policy_or_unknown");
                        b.emit_host_call(sym, argc, false);
                    }
                }
                return Ok(true);
            }
            // Map ops
            "size" | "get" | "has" | "set" => {
                let is_set = method == "set";
                if is_set && crate::jit::policy::current().read_only { // deny under read-only policy
                    if let Some(_) = dst { b.emit_const_i64(0); }
                    return Ok(true);
                }
                let argc = match method { "size" => 1, "get" | "has" => 2, "set" => 3, _ => 1 };
                if let Ok(ph) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() {
                    if let Ok(h) = ph.resolve_method("MapBox", method) {
                        // receiver
                        if let Some(pidx) = self.param_index.get(array).copied() { b.emit_param_i64(pidx); } else { b.emit_const_i64(-1); }
                        // args
                        match method {
                            "size" => {}
                            "get" | "has" => {
                                if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            }
                            "set" => {
                                if let Some(k) = args.get(0) { self.push_value_if_known_or_param(b, k); } else { b.emit_const_i64(0); }
                                if let Some(v) = args.get(1) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            }
                            _ => {}
                        }
                        b.emit_plugin_invoke(h.type_id, h.method_id, argc, dst.is_some());
                        crate::jit::events::emit_lower(
                            serde_json::json!({
                                "id": format!("plugin:{}:{}", h.box_type, method),
                                "decision":"allow","reason":"plugin_invoke","argc": argc,
                                "type_id": h.type_id, "method_id": h.method_id
                            }),
                            "plugin","<jit>"
                        );
                        return Ok(true);
                    }
                }
                // Hostcall fallback symbols
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                    match method {
                        "size" => b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE_H, argc, dst.is_some()),
                        "get" => {
                            if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_GET_H, argc, dst.is_some())
                        }
                        "has" => {
                            if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_HAS_H, argc, dst.is_some())
                        }
                        "set" => {
                            if let Some(k) = args.get(0) { self.push_value_if_known_or_param(b, k); } else { b.emit_const_i64(0); }
                            if let Some(v) = args.get(1) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SET_H, argc, dst.is_some())
                        }
                        _ => {}
                    }
                } else {
                    // receiver unknown
                    b.emit_const_i64(-1);
                    match method {
                        "size" => b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SIZE, argc, dst.is_some()),
                        "get" => {
                            if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_GET_H, argc, dst.is_some())
                        }
                        "has" => {
                            if let Some(v) = args.get(0) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_HAS_H, argc, dst.is_some())
                        }
                        "set" => {
                            if let Some(k) = args.get(0) { self.push_value_if_known_or_param(b, k); } else { b.emit_const_i64(0); }
                            if let Some(v) = args.get(1) { self.push_value_if_known_or_param(b, v); } else { b.emit_const_i64(0); }
                            b.emit_host_call(crate::jit::r#extern::collections::SYM_MAP_SET, argc, dst.is_some())
                        }
                        _ => {}
                    }
                }
                return Ok(true);
            }
            _ => {}
        }
        // Not handled here
        Ok(false)
    }
}
