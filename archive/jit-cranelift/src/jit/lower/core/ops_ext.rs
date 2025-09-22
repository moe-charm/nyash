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
        if bt == "PyRuntimeBox" && (m == "import") {
            let argc = 1 + args.len();
            if let Some(pidx) = self.param_index.get(box_val).copied() {
                b.emit_param_i64(pidx);
            } else {
                self.push_value_if_known_or_param(b, box_val);
            }
            let decision =
                crate::jit::policy::invoke::decide_box_method(&bt, m, argc, dst.is_some());
            if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                type_id,
                method_id,
                box_type,
                ..
            } = decision
            {
                b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                crate::jit::observe::lower_plugin_invoke(&box_type, m, type_id, method_id, argc);
                if let Some(d) = dst {
                    self.handle_values.insert(*d);
                }
            } else {
                if dst.is_some() {
                    b.emit_const_i64(0);
                }
            }
        } else if bt == "PyRuntimeBox" && (m == "getattr" || m == "call") {
            let argc = 1 + args.len();
            if let Some(pidx) = self.param_index.get(box_val).copied() {
                b.emit_param_i64(pidx);
            } else {
                b.emit_const_i64(-1);
            }
            for a in args.iter() {
                self.push_value_if_known_or_param(b, a);
            }
            b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.store_local_i64(slot);
            }
        } else if self.handle_values.contains(box_val) && (m == "getattr" || m == "call") {
            let argc = 1 + args.len();
            if let Some(slot) = self.local_index.get(box_val).copied() {
                b.load_local_i64(slot);
            } else {
                b.emit_const_i64(-1);
            }
            for a in args.iter() {
                self.push_value_if_known_or_param(b, a);
            }
            b.emit_plugin_invoke_by_name(m, argc, dst.is_some());
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.store_local_i64(slot);
            }
        } else if (bt == "PyRuntimeBox" && (m == "birth" || m == "eval"))
            || (bt == "IntegerBox" && m == "birth")
            || (bt == "StringBox" && m == "birth")
            || (bt == "ConsoleBox" && m == "birth")
        {
            if dst.is_some() {
                b.emit_const_i64(0);
            }
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
        if iface_name == "env.console"
            && (method_name == "log"
                || method_name == "println"
                || method_name == "warn"
                || method_name == "error")
        {
            if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
                // a0: 先頭引数を最小限で積む
                if let Some(arg0) = args.get(0) {
                    self.push_value_if_known_or_param(b, arg0);
                } else {
                    b.emit_const_i64(0);
                }
                let sym = match method_name {
                    "warn" => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_WARN,
                    "error" => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_ERROR,
                    _ => crate::jit::r#extern::host_bridge::SYM_HOST_CONSOLE_LOG,
                };
                b.emit_host_call(sym, 1, false);
                return Ok(());
            }
            // Ensure we have a Console handle (hostcall birth shim)
            b.emit_host_call(
                crate::jit::r#extern::collections::SYM_CONSOLE_BIRTH_H,
                0,
                true,
            );
            // a1: first argument best-effort
            if let Some(arg0) = args.get(0) {
                self.push_value_if_known_or_param(b, arg0);
            }
            // Resolve plugin invoke for ConsoleBox.method
            let decision = crate::jit::policy::invoke::decide_box_method(
                "ConsoleBox",
                method_name,
                2,
                dst.is_some(),
            );
            if let crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                type_id,
                method_id,
                box_type,
                ..
            } = decision
            {
                b.emit_plugin_invoke(type_id, method_id, 2, dst.is_some());
                crate::jit::observe::lower_plugin_invoke(
                    &box_type,
                    method_name,
                    type_id,
                    method_id,
                    2,
                );
            } else if dst.is_some() {
                b.emit_const_i64(0);
            }
            return Ok(());
        }
        // env.future.await(fut) → await_h + ok_h/err_h select
        if iface_name == "env.future" && method_name == "await" {
            if let Some(arg0) = args.get(0) {
                if let Some(pidx) = self.param_index.get(arg0).copied() {
                    b.emit_param_i64(pidx);
                } else if let Some(slot) = self.local_index.get(arg0).copied() {
                    b.load_local_i64(slot);
                } else if let Some(v) = self.known_i64.get(arg0).copied() {
                    b.emit_const_i64(v);
                } else {
                    b.emit_const_i64(-1);
                }
            } else {
                b.emit_const_i64(-1);
            }
            // await_h → handle(0 timeout)
            b.emit_host_call(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, 1, true);
            let hslot = {
                let id = self.next_local;
                self.next_local += 1;
                id
            };
            b.store_local_i64(hslot);
            // ok_h(handle)
            b.load_local_i64(hslot);
            b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_OK_H, 1, true);
            let ok_slot = {
                let id = self.next_local;
                self.next_local += 1;
                id
            };
            b.store_local_i64(ok_slot);
            // err_h(0)
            b.emit_const_i64(0);
            b.emit_host_call(crate::jit::r#extern::result::SYM_RESULT_ERR_H, 1, true);
            let err_slot = {
                let id = self.next_local;
                self.next_local += 1;
                id
            };
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
                let slot = *self.local_index.entry(*d).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
                b.store_local_i64(slot);
            }
            return Ok(());
        }
        // env.future.spawn_instance(recv, method_name, args...)
        if iface_name == "env.future" && method_name == "spawn_instance" {
            // a0 receiver
            if let Some(recv) = args.get(0) {
                if let Some(pidx) = self.param_index.get(recv).copied() {
                    b.emit_param_i64(pidx);
                } else {
                    b.emit_const_i64(-1);
                }
            } else {
                b.emit_const_i64(-1);
            }
            // a1 method name (best-effort)
            if let Some(meth) = args.get(1) {
                self.push_value_if_known_or_param(b, meth);
            } else {
                b.emit_const_i64(0);
            }
            // a2 first payload (optional)
            if let Some(a2) = args.get(2) {
                self.push_value_if_known_or_param(b, a2);
            } else {
                b.emit_const_i64(0);
            }
            // argc_total = explicit args including method name and payload (exclude receiver)
            let argc_total = args.len().saturating_sub(1).max(0);
            b.emit_const_i64(argc_total as i64);
            // call spawn shim → Future handle
            b.emit_host_call(
                crate::jit::r#extern::r#async::SYM_FUTURE_SPAWN_INSTANCE3_I64,
                4,
                true,
            );
            if let Some(d) = dst {
                self.handle_values.insert(*d);
                let slot = *self.local_index.entry(*d).or_insert_with(|| {
                    let id = self.next_local;
                    self.next_local += 1;
                    id
                });
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
        // 非コアBox（例: EguiBox など）は共通処理として名前ベースの plugin_invoke にフォールバック
        // コアBoxの目安: StringBox/ArrayBox/MapBox（この後の分岐で処理）と PyRuntimeBox（専用分岐済）
        if let Some(bt) = self.box_type_map.get(array).cloned() {
            let is_core =
                bt == "StringBox" || bt == "ArrayBox" || bt == "MapBox" || bt == "PyRuntimeBox";
            if !is_core {
                // receiver: prefer existing local slot/param; ensure a valid runtime handle
                if let Some(slot) = self.local_index.get(array).copied() {
                    b.load_local_i64(slot);
                } else if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                } else {
                    self.push_value_if_known_or_param(b, array);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                }
                // push up to 2 args (name-shim supports at most 2 positional args beyond receiver)
                let take_n = core::cmp::min(args.len(), 2);
                for i in 0..take_n {
                    if let Some(v) = args.get(i) {
                        self.push_value_if_known_or_param(b, v);
                    }
                }
                let argc = 1 + take_n;
                b.emit_plugin_invoke_by_name(method, argc, dst.is_some());
                if std::env::var("NYASH_JIT_TRACE_LOWER").ok().as_deref() == Some("1") {
                    crate::jit::events::emit_lower(
                        serde_json::json!({
                            "id": format!("plugin_name:{}:{}", bt, method),
                            "decision": "allow",
                            "reason": "plugin_invoke_by_name",
                            "argc": argc
                        }),
                        "plugin",
                        "<jit>",
                    );
                }
                if let Some(d) = dst {
                    self.handle_values.insert(d);
                    let slot = *self.local_index.entry(d).or_insert_with(|| {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    });
                    b.store_local_i64(slot);
                }
                if std::env::var("NYASH_JIT_TRACE_LOWER").ok().as_deref() == Some("1") {
                    eprintln!("[LOWER] {}.{} via name-invoke (argc={})", bt, method, argc);
                }
                return Ok(true);
            }
        }
        // Builtins-to-plugin path (subset for String/Array/Map critical ops)
        // Builtins-to-plugin path (subset for String/Array/Map critical ops)
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1") {
            // StringBox (length/is_empty/charCodeAt)
            if matches!(method, "length" | "is_empty" | "charCodeAt") {
                if method == "length" {
                    // Prefer robust fallback path (param/local/literal/handle.of)
                    if let Some(pidx) = self.param_index.get(array).copied() {
                        self.emit_len_with_fallback_param(b, pidx);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                    if let Some(slot) = self.local_index.get(array).copied() {
                        self.emit_len_with_fallback_local_handle(b, slot);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                    // literal?
                    let mut lit: Option<String> = None;
                    for (_bid, bb) in func.blocks.iter() {
                        for ins in bb.instructions.iter() {
                            if let crate::mir::MirInstruction::NewBox {
                                dst,
                                box_type,
                                args,
                            } = ins
                            {
                                if dst == array && box_type == "StringBox" && args.len() == 1 {
                                    if let Some(src) = args.get(0) {
                                        if let Some(s) = self.known_str.get(src).cloned() {
                                            lit = Some(s);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if lit.is_some() {
                            break;
                        }
                    }
                    if let Some(s) = lit {
                        let n = s.len() as i64;
                        b.emit_const_i64(n);
                        if let Some(d) = dst {
                            self.known_i64.insert(d, n);
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                    // last resort: handle.of + any.length_h
                    self.push_value_if_known_or_param(b, array);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                    b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, true);
                    if let Some(d) = dst {
                        let slot = *self.local_index.entry(d).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(slot);
                    }
                    return Ok(true);
                }
                // is_empty / charCodeAt: keep mapped hostcall path
                // Ensure receiver is a valid runtime handle (param or materialized via handle.of)
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                } else if let Some(slot) = self.local_index.get(array).copied() {
                    b.load_local_i64(slot);
                } else {
                    self.push_value_if_known_or_param(b, array);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                }
                let mut argc = 1usize;
                if method == "charCodeAt" {
                    if let Some(v) = args.get(0) {
                        self.push_value_if_known_or_param(b, v);
                    } else {
                        b.emit_const_i64(0);
                    }
                    argc = 2;
                }
                if method == "is_empty" {
                    b.hint_ret_bool(true);
                }
                let decision = crate::jit::policy::invoke::decide_box_method(
                    "StringBox",
                    method,
                    argc,
                    dst.is_some(),
                );
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(
                            &symbol,
                            argc,
                            &if argc == 1 {
                                ["Handle"][..].to_vec()
                            } else {
                                ["Handle", "I64"][..].to_vec()
                            },
                            "allow",
                            "mapped_symbol",
                        );
                        b.emit_host_call(&symbol, argc, dst.is_some());
                        return Ok(true);
                    }
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                        type_id,
                        method_id,
                        box_type,
                        ..
                    } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, dst.is_some());
                        crate::jit::observe::lower_plugin_invoke(
                            &box_type, method, type_id, method_id, argc,
                        );
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
                    if let Some(v) = args.get(0) {
                        let _ = v;
                    } // keep args in scope
                    self.push_value_if_known_or_param(b, array);
                    // name: if const string, build a StringBox handle from literal; else best-effort push
                    if let Some(name_id) = args.get(0) {
                        if let Some(s) = self.known_str.get(name_id).cloned() {
                            b.emit_string_handle_from_literal(&s);
                        } else {
                            b.emit_const_i64(0);
                        }
                    } else {
                        b.emit_const_i64(0);
                    }
                    // value for setField
                    let argc = if method == "setField" {
                        if let Some(val_id) = args.get(1) {
                            if let Some(s) = self.known_str.get(val_id).cloned() {
                                b.emit_string_handle_from_literal(&s);
                            } else {
                                self.push_value_if_known_or_param(b, val_id);
                            }
                        } else {
                            b.emit_const_i64(0);
                        }
                        3
                    } else {
                        2
                    };
                    // Unified 3-arity call: getField uses val=-1 sentinel
                    let sym = crate::jit::r#extern::host_bridge::SYM_HOST_INSTANCE_FIELD3;
                    if method == "getField" {
                        b.emit_const_i64(-1);
                    }
                    b.emit_host_call_fixed3(sym, dst.is_some());
                    return Ok(true);
                }
            }
            // String.len/length: robust handling
            "len" => {
                let trace = std::env::var("NYASH_JIT_TRACE_LOWER_LEN").ok().as_deref() == Some("1");
                // (1) const string literal case
                let mut lit_len: Option<i64> = None;
                for (_bbid, bb) in func.blocks.iter() {
                    for ins in bb.instructions.iter() {
                        if let crate::mir::MirInstruction::Const { dst, value } = ins {
                            if dst == array {
                                if let crate::mir::ConstValue::String(s) = value {
                                    lit_len = Some(s.len() as i64);
                                }
                                break;
                            }
                        }
                    }
                    if lit_len.is_some() {
                        break;
                    }
                }
                if let Some(n) = lit_len {
                    if trace {
                        eprintln!(
                            "[LOWER] StringBox.len: literal length={} (dst?={})",
                            n,
                            dst.is_some()
                        );
                    }
                    b.emit_const_i64(n);
                    if let Some(d) = dst {
                        // Persist literal length so Return can reliably load
                        let slot = *self.local_index.entry(d).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(slot);
                    }
                    return Ok(true);
                }
                // (2) prefer host-bridge when enabled
                if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
                    if self
                        .box_type_map
                        .get(array)
                        .map(|s| s == "StringBox")
                        .unwrap_or(false)
                    {
                        if std::env::var("NYASH_JIT_TRACE_BRIDGE").ok().as_deref() == Some("1") {
                            eprintln!("[LOWER]string.len via host-bridge");
                        }
                        if trace {
                            eprintln!(
                                "[LOWER] StringBox.len via host-bridge (dst?={})",
                                dst.is_some()
                            );
                        }
                        self.push_value_if_known_or_param(b, array);
                        b.emit_host_call(
                            crate::jit::r#extern::host_bridge::SYM_HOST_STRING_LEN,
                            1,
                            dst.is_some(),
                        );
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                }
                // (3) Fallback: emit string.len_h with Any.length_h guard
                if self
                    .box_type_map
                    .get(array)
                    .map(|s| s == "StringBox")
                    .unwrap_or(false)
                {
                    // Strong constant fold when literal mapping is known
                    if let Some(s) = self.string_box_literal.get(array).cloned() {
                        let n = s.len() as i64;
                        b.emit_const_i64(n);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                            self.known_i64.insert(d, n);
                        }
                        return Ok(true);
                    }
                    // Prefer literal reconstruction so JIT-AOT path is deterministic
                    let mut lit: Option<String> = None;
                    for (_bid, bb) in func.blocks.iter() {
                        for ins in bb.instructions.iter() {
                            if let crate::mir::MirInstruction::NewBox {
                                dst,
                                box_type,
                                args,
                            } = ins
                            {
                                if dst == array && box_type == "StringBox" && args.len() == 1 {
                                    if let Some(src) = args.get(0) {
                                        if let Some(s) = self.known_str.get(src).cloned() {
                                            lit = Some(s);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if lit.is_some() {
                            break;
                        }
                    }
                    if let Some(s) = lit {
                        if trace {
                            eprintln!(
                                "[LOWER] StringBox.len reconstructed literal '{}' (dst?={})",
                                s,
                                dst.is_some()
                            );
                        }
                        // Const fold: use literal length directly to avoid hostcall dependence
                        let n = s.len() as i64;
                        b.emit_const_i64(n);
                        if let Some(d) = dst {
                            let dslot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(dslot);
                            self.known_i64.insert(d, n);
                        }
                        return Ok(true);
                    }
                    // Param/local fallback when not a reconstructable literal
                    if let Some(pidx) = self.param_index.get(array).copied() {
                        if trace {
                            eprintln!(
                                "[LOWER] StringBox.len param p{} (dst?={})",
                                pidx,
                                dst.is_some()
                            );
                        }
                        self.emit_len_with_fallback_param(b, pidx);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                    if let Some(slot) = self.local_index.get(array).copied() {
                        if trace {
                            eprintln!(
                                "[LOWER] StringBox.len local slot#{} (dst?={})",
                                slot,
                                dst.is_some()
                            );
                        }
                        self.emit_len_with_fallback_local_handle(b, slot);
                        if let Some(d) = dst {
                            let dslot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(dslot);
                        }
                        return Ok(true);
                    }
                    // As a last resort, convert receiver to handle via nyash.handle.of and apply fallback on temp slot
                    if trace {
                        eprintln!(
                            "[LOWER] StringBox.len last-resort handle.of + fallback (dst?={})",
                            dst.is_some()
                        );
                    }
                    self.push_value_if_known_or_param(b, array);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                    let t_recv = {
                        let id = self.next_local;
                        self.next_local += 1;
                        id
                    };
                    b.store_local_i64(t_recv);
                    self.emit_len_with_fallback_local_handle(b, t_recv);
                    if let Some(d) = dst {
                        let dslot = *self.local_index.entry(d).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(dslot);
                    }
                    return Ok(true);
                }
                // Not a StringBox: let other branches handle
                if trace {
                    eprintln!(
                        "[LOWER] StringBox.len not handled (box_type={:?})",
                        self.box_type_map.get(array)
                    );
                }
                return Ok(false);
            }
            // Alias: String.length → same as len
            "length" => {
                let trace = std::env::var("NYASH_JIT_TRACE_LOWER_LEN").ok().as_deref() == Some("1");
                if self
                    .box_type_map
                    .get(array)
                    .map(|s| s == "StringBox")
                    .unwrap_or(false)
                {
                    // Try literal constant fold first for stability
                    let mut lit: Option<String> = None;
                    for (_bid, bb) in func.blocks.iter() {
                        for ins in bb.instructions.iter() {
                            if let crate::mir::MirInstruction::NewBox {
                                dst,
                                box_type,
                                args,
                            } = ins
                            {
                                if dst == array && box_type == "StringBox" && args.len() == 1 {
                                    if let Some(src) = args.get(0) {
                                        if let Some(s) = self.known_str.get(src).cloned() {
                                            lit = Some(s);
                                            break;
                                        }
                                        // Fallback: scan Const directly
                                        for (_b2, bb2) in func.blocks.iter() {
                                            for ins2 in bb2.instructions.iter() {
                                                if let crate::mir::MirInstruction::Const {
                                                    dst: cdst,
                                                    value,
                                                } = ins2
                                                {
                                                    if cdst == src {
                                                        if let crate::mir::ConstValue::String(sv) =
                                                            value
                                                        {
                                                            lit = Some(sv.clone());
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            if lit.is_some() {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if lit.is_some() {
                            break;
                        }
                    }
                    if let Some(s) = lit {
                        let n = s.len() as i64;
                        b.emit_const_i64(n);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                            self.known_i64.insert(d, n);
                        }
                        return Ok(true);
                    }
                    // Reuse len handler, but ensure dst persistence if len handler did not handle
                    let handled = self.lower_box_call(func, b, array, "len", args, dst)?;
                    if handled {
                        // len branch already persisted when dst.is_some()
                        return Ok(true);
                    }
                    // As a conservative fallback, try direct any.length_h on handle.of
                    if trace {
                        eprintln!(
                            "[LOWER] StringBox.length fallback any.length_h on handle.of (dst?={})",
                            dst.is_some()
                        );
                    }
                    self.push_value_if_known_or_param(b, array);
                    b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_ANY_LEN_H,
                        1,
                        dst.is_some(),
                    );
                    if let Some(d) = dst {
                        let slot = *self.local_index.entry(d).or_insert_with(|| {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        });
                        b.store_local_i64(slot);
                    }
                    return Ok(true);
                }
                // Array length is handled below; otherwise not handled here
                return Ok(false);
            }
            // Array/String length variants (length/len)
            "len" | "length" => {
                match self.box_type_map.get(array).map(|s| s.as_str()) {
                    Some("StringBox") => {
                        // Strong constant fold when literal mapping is known (allow disabling via NYASH_JIT_DISABLE_LEN_CONST=1)
                        if std::env::var("NYASH_JIT_DISABLE_LEN_CONST").ok().as_deref() != Some("1")
                            && self.string_box_literal.get(array).is_some()
                        {
                            let s = self.string_box_literal.get(array).cloned().unwrap();
                            let n = s.len() as i64;
                            b.emit_const_i64(n);
                            if let Some(d) = dst {
                                let slot = *self.local_index.entry(d).or_insert_with(|| {
                                    let id = self.next_local;
                                    self.next_local += 1;
                                    id
                                });
                                b.store_local_i64(slot);
                                self.known_i64.insert(d, n);
                            }
                            return Ok(true);
                        }
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            self.emit_len_with_fallback_param(b, pidx);
                            // Persist into dst local so Return can reliably pick it up
                            if let Some(d) = dst {
                                let slot = *self.local_index.entry(d).or_insert_with(|| {
                                    let id = self.next_local;
                                    self.next_local += 1;
                                    id
                                });
                                b.store_local_i64(slot);
                            }
                            return Ok(true);
                        }
                        if let Some(slot) = self.local_index.get(array).copied() {
                            self.emit_len_with_fallback_local_handle(b, slot);
                            // Persist into dst local so Return can reliably pick it up
                            if let Some(d) = dst {
                                let slot = *self.local_index.entry(d).or_insert_with(|| {
                                    let id = self.next_local;
                                    self.next_local += 1;
                                    id
                                });
                                b.store_local_i64(slot);
                            }
                            return Ok(true);
                        }
                        // Try literal reconstruction (skipped if disabled by env)
                        let mut lit: Option<String> = None;
                        for (_bid, bb) in func.blocks.iter() {
                            for ins in bb.instructions.iter() {
                                if let crate::mir::MirInstruction::NewBox {
                                    dst,
                                    box_type,
                                    args,
                                } = ins
                                {
                                    if dst == array && box_type == "StringBox" && args.len() == 1 {
                                        if let Some(src) = args.get(0) {
                                            if let Some(s) = self.known_str.get(src).cloned() {
                                                lit = Some(s);
                                                break;
                                            }
                                            // Fallback: scan Const directly
                                            for (_b2, bb2) in func.blocks.iter() {
                                                for ins2 in bb2.instructions.iter() {
                                                    if let crate::mir::MirInstruction::Const {
                                                        dst: cdst,
                                                        value,
                                                    } = ins2
                                                    {
                                                        if cdst == src {
                                                            if let crate::mir::ConstValue::String(
                                                                sv,
                                                            ) = value
                                                            {
                                                                lit = Some(sv.clone());
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                if lit.is_some() {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if lit.is_some() {
                                break;
                            }
                        }
                        if let Some(s) = lit {
                            let n = s.len() as i64;
                            b.emit_const_i64(n);
                            if let Some(d) = dst {
                                let slot = *self.local_index.entry(d).or_insert_with(|| {
                                    let id = self.next_local;
                                    self.next_local += 1;
                                    id
                                });
                                b.store_local_i64(slot);
                                self.known_i64.insert(d, n);
                            }
                            return Ok(true);
                        }
                        // Last resort: handle.of
                        self.push_value_if_known_or_param(b, array);
                        b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                        let slot = {
                            let id = self.next_local;
                            self.next_local += 1;
                            id
                        };
                        b.store_local_i64(slot);
                        self.emit_len_with_fallback_local_handle(b, slot);
                        // Persist into dst local so Return can reliably pick it up
                        if let Some(d) = dst {
                            let dslot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(dslot);
                        }
                        return Ok(true);
                    }
                    Some("ArrayBox") => {}
                    _ => {
                        // Unknown receiver type: generic Any.length_h on a handle
                        self.push_value_if_known_or_param(b, array);
                        b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
                        b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, true);
                        if let Some(d) = dst {
                            let slot = *self.local_index.entry(d).or_insert_with(|| {
                                let id = self.next_local;
                                self.next_local += 1;
                                id
                            });
                            b.store_local_i64(slot);
                        }
                        return Ok(true);
                    }
                }
                if let Ok(ph) =
                    crate::runtime::plugin_loader_unified::get_global_plugin_host().read()
                {
                    if let Ok(h) = ph.resolve_method("ArrayBox", "length") {
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            b.emit_param_i64(pidx);
                        } else {
                            b.emit_const_i64(-1);
                        }
                        b.emit_plugin_invoke(h.type_id, h.method_id, 1, dst.is_some());
                        return Ok(true);
                    }
                }
                // Hostcall fallback
                if let Some(pidx) = self.param_index.get(array).copied() {
                    crate::jit::observe::lower_hostcall(
                        crate::jit::r#extern::collections::SYM_ANY_LEN_H,
                        1,
                        &["Handle"],
                        "allow",
                        "mapped_symbol",
                    );
                    b.emit_param_i64(pidx);
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_ANY_LEN_H,
                        1,
                        dst.is_some(),
                    );
                } else {
                    crate::jit::observe::lower_hostcall(
                        crate::jit::r#extern::collections::SYM_ARRAY_LEN,
                        1,
                        &["I64"],
                        "fallback",
                        "receiver_not_param",
                    );
                    b.emit_const_i64(-1);
                    b.emit_host_call(
                        crate::jit::r#extern::collections::SYM_ARRAY_LEN,
                        1,
                        dst.is_some(),
                    );
                }
                return Ok(true);
            }
            // Array push
            "push" => {
                let argc = 2usize;
                // receiver
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                } else {
                    b.emit_const_i64(-1);
                }
                // value
                if let Some(v) = args.get(0).and_then(|vid| self.known_i64.get(vid)).copied() {
                    b.emit_const_i64(v);
                } else if let Some(v) = args.get(0) {
                    self.push_value_if_known_or_param(b, v);
                } else {
                    b.emit_const_i64(0);
                }
                // policy decide → plugin / hostcall fallback
                let decision =
                    crate::jit::policy::invoke::decide_box_method("ArrayBox", "push", argc, false);
                match decision {
                    crate::jit::policy::invoke::InvokeDecision::PluginInvoke {
                        type_id,
                        method_id,
                        box_type,
                        ..
                    } => {
                        b.emit_plugin_invoke(type_id, method_id, argc, false);
                        crate::jit::observe::lower_plugin_invoke(
                            &box_type, "push", type_id, method_id, argc,
                        );
                    }
                    crate::jit::policy::invoke::InvokeDecision::HostCall { symbol, .. } => {
                        crate::jit::observe::lower_hostcall(
                            &symbol,
                            argc,
                            &["Handle", "I64"],
                            "allow",
                            "mapped_symbol",
                        );
                        b.emit_host_call(&symbol, argc, false);
                    }
                    _ => {
                        // Fallback hostcall
                        let sym = if self.param_index.get(array).is_some() {
                            crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H
                        } else {
                            crate::jit::r#extern::collections::SYM_ARRAY_PUSH
                        };
                        let arg_types = if self.param_index.get(array).is_some() {
                            &["Handle", "I64"][..]
                        } else {
                            &["I64", "I64"][..]
                        };
                        crate::jit::observe::lower_hostcall(
                            sym,
                            argc,
                            arg_types,
                            "fallback",
                            "policy_or_unknown",
                        );
                        b.emit_host_call(sym, argc, false);
                    }
                }
                return Ok(true);
            }
            // Map ops
            "size" | "get" | "has" | "set" => {
                let is_set = method == "set";
                if is_set && crate::jit::policy::current().read_only {
                    // deny under read-only policy
                    if let Some(_) = dst {
                        b.emit_const_i64(0);
                    }
                    return Ok(true);
                }
                let argc = match method {
                    "size" => 1,
                    "get" | "has" => 2,
                    "set" => 3,
                    _ => 1,
                };
                // If receiver is a local handle (AOT/JIT-AOT), prefer handle-based hostcalls directly
                if self.handle_values.contains(array) {
                    self.push_value_if_known_or_param(b, array);
                    match method {
                        "size" => b.emit_host_call(
                            crate::jit::r#extern::collections::SYM_MAP_SIZE_H,
                            argc,
                            dst.is_some(),
                        ),
                        "get" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_GET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "has" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_HAS_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "set" => {
                            if let Some(k) = args.get(0) {
                                self.push_value_if_known_or_param(b, k);
                            } else {
                                b.emit_const_i64(0);
                            }
                            if let Some(v) = args.get(1) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_SET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        _ => {}
                    }
                    return Ok(true);
                }
                if let Ok(ph) =
                    crate::runtime::plugin_loader_unified::get_global_plugin_host().read()
                {
                    if let Ok(h) = ph.resolve_method("MapBox", method) {
                        // receiver
                        if let Some(pidx) = self.param_index.get(array).copied() {
                            b.emit_param_i64(pidx);
                        } else {
                            b.emit_const_i64(-1);
                        }
                        // args
                        match method {
                            "size" => {}
                            "get" | "has" => {
                                if let Some(v) = args.get(0) {
                                    self.push_value_if_known_or_param(b, v);
                                } else {
                                    b.emit_const_i64(0);
                                }
                            }
                            "set" => {
                                if let Some(k) = args.get(0) {
                                    self.push_value_if_known_or_param(b, k);
                                } else {
                                    b.emit_const_i64(0);
                                }
                                if let Some(v) = args.get(1) {
                                    self.push_value_if_known_or_param(b, v);
                                } else {
                                    b.emit_const_i64(0);
                                }
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
                            "plugin",
                            "<jit>",
                        );
                        return Ok(true);
                    }
                }
                // Hostcall fallback symbols
                if let Some(pidx) = self.param_index.get(array).copied() {
                    b.emit_param_i64(pidx);
                    match method {
                        "size" => b.emit_host_call(
                            crate::jit::r#extern::collections::SYM_MAP_SIZE_H,
                            argc,
                            dst.is_some(),
                        ),
                        "get" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_GET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "has" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_HAS_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "set" => {
                            if let Some(k) = args.get(0) {
                                self.push_value_if_known_or_param(b, k);
                            } else {
                                b.emit_const_i64(0);
                            }
                            if let Some(v) = args.get(1) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_SET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        _ => {}
                    }
                } else {
                    // receiver unknown: try local handle (AOT/JIT-AOT)
                    self.push_value_if_known_or_param(b, array);
                    match method {
                        "size" => b.emit_host_call(
                            crate::jit::r#extern::collections::SYM_MAP_SIZE_H,
                            argc,
                            dst.is_some(),
                        ),
                        "get" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_GET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "has" => {
                            if let Some(v) = args.get(0) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_HAS_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        "set" => {
                            if let Some(k) = args.get(0) {
                                self.push_value_if_known_or_param(b, k);
                            } else {
                                b.emit_const_i64(0);
                            }
                            if let Some(v) = args.get(1) {
                                self.push_value_if_known_or_param(b, v);
                            } else {
                                b.emit_const_i64(0);
                            }
                            b.emit_host_call(
                                crate::jit::r#extern::collections::SYM_MAP_SET_H,
                                argc,
                                dst.is_some(),
                            )
                        }
                        _ => {}
                    }
                }
                return Ok(true);
            }
            _ => {}
        }
        // Not handled here
        if std::env::var("NYASH_JIT_TRACE_LOWER").ok().as_deref() == Some("1") {
            let bt = self.box_type_map.get(array).cloned().unwrap_or_default();
            let is_param = self.param_index.contains_key(array);
            let has_local = self.local_index.contains_key(array);
            let is_handle = self.handle_values.contains(array);
            let mut arg_kinds: Vec<&'static str> = Vec::new();
            for a in args.iter().take(3) {
                let k = if self.known_i64.contains_key(a) {
                    "i"
                } else if self.known_str.contains_key(a) {
                    "s"
                } else if self.param_index.contains_key(a) {
                    "p"
                } else if self.local_index.contains_key(a) {
                    "l"
                } else if self.handle_values.contains(a) {
                    "h"
                } else {
                    "-"
                };
                arg_kinds.push(k);
            }
            let policy = crate::jit::policy::invoke::decide_box_method(
                &bt,
                method,
                1 + args.len(),
                dst.is_some(),
            );
            let policy_str = match policy {
                crate::jit::policy::invoke::InvokeDecision::HostCall { ref symbol, .. } => {
                    format!("hostcall:{}", symbol)
                }
                crate::jit::policy::invoke::InvokeDecision::PluginInvoke { .. } => {
                    "plugin_invoke".to_string()
                }
                crate::jit::policy::invoke::InvokeDecision::Fallback { ref reason } => {
                    format!("fallback:{}", reason)
                }
            };
            eprintln!(
                "[LOWER] unhandled BoxCall: box_type='{}' method='{}' recv[param?{} local?{} handle?{}] args={:?} policy={}",
                bt, method, is_param, has_local, is_handle, arg_kinds, policy_str
            );
        }
        Ok(false)
    }
}
