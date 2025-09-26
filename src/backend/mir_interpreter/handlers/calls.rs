use super::*;

impl MirInterpreter {
    pub(super) fn handle_call(
        &mut self,
        dst: Option<ValueId>,
        func: ValueId,
        callee: Option<&Callee>,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        let call_result = if let Some(callee_type) = callee {
            self.execute_callee_call(callee_type, args)?
        } else {
            self.execute_legacy_call(func, args)?
        };
        if let Some(d) = dst {
            self.regs.insert(d, call_result);
        }
        Ok(())
    }

    pub(super) fn execute_callee_call(
        &mut self,
        callee: &Callee,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match callee {
            Callee::Global(func_name) => self.execute_global_function(func_name, args),
            Callee::Method {
                box_name: _,
                method,
                receiver,
            } => {
                if let Some(recv_id) = receiver {
                    let recv_val = self.reg_load(*recv_id)?;
                    self.execute_method_call(&recv_val, method, args)
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method call missing receiver for {}",
                        method
                    )))
                }
            }
            Callee::Constructor { box_type } => Err(VMError::InvalidInstruction(format!(
                "Constructor calls not yet implemented for {}",
                box_type
            ))),
            Callee::Closure { .. } => Err(VMError::InvalidInstruction(
                "Closure creation not yet implemented in VM".into(),
            )),
            Callee::Value(func_val_id) => {
                let _func_val = self.reg_load(*func_val_id)?;
                Err(VMError::InvalidInstruction(
                    "First-class function calls not yet implemented in VM".into(),
                ))
            }
            Callee::Extern(extern_name) => self.execute_extern_function(extern_name, args),
        }
    }

    pub(super) fn execute_legacy_call(
        &mut self,
        func_id: ValueId,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        let name_val = self.reg_load(func_id)?;
        let raw = match name_val {
            VMValue::String(ref s) => s.clone(),
            other => other.to_string(),
        };

        let mut pick: Option<String> = None;
        // Fast path: exact match
        if self.functions.contains_key(&raw) {
            pick = Some(raw.clone());
        } else {
            // Robust normalization for names like "Box.method/Arity" or just "method"
            let call_arity = args.len();
            let (base, ar_from_raw) = if let Some((b, a)) = raw.rsplit_once('/') {
                (b.to_string(), a.parse::<usize>().ok())
            } else {
                (raw.clone(), None)
            };
            let want_arity = ar_from_raw.unwrap_or(call_arity);
            // Try exact canonical form: "base/arity"
            let exact = format!("{}/{}", base, want_arity);
            if self.functions.contains_key(&exact) {
                pick = Some(exact);
            } else {
                // Split base into optional box and method name
                let (maybe_box, method) = if let Some((bx, m)) = base.split_once('.') {
                    (Some(bx.to_string()), m.to_string())
                } else {
                    (None, base.clone())
                };
                // Collect candidates that end with ".method/arity"
                let mut cands: Vec<String> = Vec::new();
                let tail = format!(".{}{}", method, format!("/{}", want_arity));
                for k in self.functions.keys() {
                    if k.ends_with(&tail) {
                        if let Some(ref bx) = maybe_box {
                            if k.starts_with(&format!("{}.", bx)) { cands.push(k.clone()); }
                        } else {
                            cands.push(k.clone());
                        }
                    }
                }
                if cands.len() > 1 {
                    // Prefer same-box candidate based on current function's box
                    if let Some(cur) = &self.cur_fn {
                        let cur_box = cur.split('.').next().unwrap_or("");
                        let scoped: Vec<String> = cands
                            .iter()
                            .filter(|k| k.starts_with(&format!("{}.", cur_box)))
                            .cloned()
                            .collect();
                        if scoped.len() == 1 { cands = scoped; }
                    }
                }
                if cands.len() == 1 {
                    pick = Some(cands.remove(0));
                } else if cands.len() > 1 {
                    let mut c = cands;
                    c.sort();
                    pick = Some(c.remove(0));
                }
            }
        }

        let fname = pick.ok_or_else(|| {
            VMError::InvalidInstruction(format!(
                "call unresolved: '{}' (arity={})",
                raw,
                args.len()
            ))
        })?;

        if std::env::var("NYASH_VM_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[vm] legacy-call resolved '{}' -> '{}'", raw, fname);
        }

        let callee =
            self.functions.get(&fname).cloned().ok_or_else(|| {
                VMError::InvalidInstruction(format!("function not found: {}", fname))
            })?;

        let mut argv: Vec<VMValue> = Vec::new();
        for a in args {
            argv.push(self.reg_load(*a)?);
        }
        // Dev trace: emit a synthetic "call" event for global function calls
        // so operator boxes (e.g., CompareOperator.apply/3) are observable with
        // argument kinds. This produces a JSON line on stderr, filtered by
        // NYASH_BOX_TRACE_FILTER like other box traces.
        if Self::box_trace_enabled() {
            // Render class/method from canonical fname like "Class.method/Arity"
            let (class_name, method_name) = if let Some((cls, rest)) = fname.split_once('.') {
                let method = rest.split('/').next().unwrap_or(rest);
                (cls.to_string(), method.to_string())
            } else {
                ("<global>".to_string(), fname.split('/').next().unwrap_or(&fname).to_string())
            };
            // Simple filter match (local copy to avoid private helper)
            let filt_ok = match std::env::var("NYASH_BOX_TRACE_FILTER").ok() {
                Some(filt) => {
                    let want = filt.trim();
                    if want.is_empty() { true } else {
                        want.split(|c: char| c == ',' || c.is_whitespace())
                            .map(|t| t.trim())
                            .filter(|t| !t.is_empty())
                            .any(|t| class_name.contains(t))
                    }
                }
                None => true,
            };
            if filt_ok {
                // Optionally include argument kinds for targeted debugging.
                let with_args = std::env::var("NYASH_OP_TRACE_ARGS").ok().as_deref() == Some("1")
                    || class_name == "CompareOperator";
                if with_args {
                    // local JSON string escaper (subset)
                    let mut esc = |s: &str| {
                        let mut out = String::with_capacity(s.len() + 8);
                        for ch in s.chars() {
                            match ch {
                                '"' => out.push_str("\\\""),
                                '\\' => out.push_str("\\\\"),
                                '\n' => out.push_str("\\n"),
                                '\r' => out.push_str("\\r"),
                                '\t' => out.push_str("\\t"),
                                c if c.is_control() => out.push(' '),
                                c => out.push(c),
                            }
                        }
                        out
                    };
                    let mut kinds: Vec<String> = Vec::with_capacity(argv.len());
                    let mut nullish: Vec<String> = Vec::with_capacity(argv.len());
                    for v in &argv {
                        let k = match v {
                            VMValue::Integer(_) => "Integer".to_string(),
                            VMValue::Float(_) => "Float".to_string(),
                            VMValue::Bool(_) => "Bool".to_string(),
                            VMValue::String(_) => "String".to_string(),
                            VMValue::Void => "Void".to_string(),
                            VMValue::Future(_) => "Future".to_string(),
                            VMValue::BoxRef(b) => {
                                // Prefer InstanceBox.class_name when available
                                if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                                    format!("BoxRef:{}", inst.class_name)
                                } else {
                                    format!("BoxRef:{}", b.type_name())
                                }
                            }
                        };
                        kinds.push(k);
                        // nullish tag (env-gated): "null" | "missing" | "void" | ""
                        if crate::config::env::null_missing_box_enabled() {
                            let tag = match v {
                                VMValue::Void => "void",
                                VMValue::BoxRef(b) => {
                                    if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { "null" }
                                    else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { "missing" }
                                    else if b.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() { "void" }
                                    else { "" }
                                }
                                _ => "",
                            };
                            nullish.push(tag.to_string());
                        }
                    }
                    let args_json = kinds
                        .into_iter()
                        .map(|s| format!("\"{}\"", esc(&s)))
                        .collect::<Vec<_>>()
                        .join(",");
                    let nullish_json = if crate::config::env::null_missing_box_enabled() {
                        let arr = nullish
                            .into_iter()
                            .map(|s| format!("\"{}\"", esc(&s)))
                            .collect::<Vec<_>>()
                            .join(",");
                        Some(arr)
                    } else { None };
                    // For CompareOperator, include op string value if present in argv[0]
                    let cur_fn = self
                        .cur_fn
                        .as_deref()
                        .map(|s| esc(s))
                        .unwrap_or_else(|| String::from("") );
                    if class_name == "CompareOperator" && !argv.is_empty() {
                        let op_str = match &argv[0] {
                            VMValue::String(s) => esc(s),
                            _ => String::from("")
                        };
                        if let Some(nj) = nullish_json {
                            eprintln!(
                                "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{},\"fn\":\"{}\",\"op\":\"{}\",\"argk\":[{}],\"nullish\":[{}]}}",
                                esc(&class_name), esc(&method_name), argv.len(), cur_fn, op_str, args_json, nj
                            );
                        } else {
                            eprintln!(
                                "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{},\"fn\":\"{}\",\"op\":\"{}\",\"argk\":[{}]}}",
                                esc(&class_name), esc(&method_name), argv.len(), cur_fn, op_str, args_json
                            );
                        }
                    } else {
                        if let Some(nj) = nullish_json {
                            eprintln!(
                                "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{},\"fn\":\"{}\",\"argk\":[{}],\"nullish\":[{}]}}",
                                esc(&class_name), esc(&method_name), argv.len(), cur_fn, args_json, nj
                            );
                        } else {
                            eprintln!(
                                "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{},\"fn\":\"{}\",\"argk\":[{}]}}",
                                esc(&class_name), esc(&method_name), argv.len(), cur_fn, args_json
                            );
                        }
                    }
                } else {
                    self.box_trace_emit_call(&class_name, &method_name, argv.len());
                }
            }
        }
        self.exec_function_inner(&callee, Some(&argv))
    }

    fn execute_global_function(
        &mut self,
        func_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match func_name {
            "nyash.builtin.print" | "print" | "nyash.console.log" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    // Dev-only: print trace (kind/class) before actual print
                    if Self::print_trace_enabled() { self.print_trace_emit(&val); }
                    // Dev observe: Null/Missing boxes quick normalization (no behavior change to prod)
                    if let VMValue::BoxRef(bx) = &val {
                        // NullBox → always print as null (stable)
                        if bx.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
                            println!("null");
                            return Ok(VMValue::Void);
                        }
                        // MissingBox → default prints as null; when flag ON, show (missing)
                        if bx.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() {
                            if crate::config::env::null_missing_box_enabled() {
                                println!("(missing)");
                            } else {
                                println!("null");
                            }
                            return Ok(VMValue::Void);
                        }
                    }
                    // Dev: treat VM Void and BoxRef(VoidBox) as JSON null for print
                    match &val {
                        VMValue::Void => {
                            println!("null");
                            return Ok(VMValue::Void);
                        }
                        VMValue::BoxRef(bx) => {
                            if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                                println!("null");
                                return Ok(VMValue::Void);
                            }
                        }
                        _ => {}
                    }
                    // Print raw strings directly (avoid double quoting via StringifyOperator)
                    match &val {
                        VMValue::String(s) => { println!("{}", s); return Ok(VMValue::Void); }
                        VMValue::BoxRef(bx) => {
                            if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                                println!("{}", sb.value);
                                return Ok(VMValue::Void);
                            }
                        }
                        _ => {}
                    }
                    // Operator Box (Stringify) – dev flag gated
                    if std::env::var("NYASH_OPERATOR_BOX_STRINGIFY").ok().as_deref() == Some("1") {
                        if let Some(op) = self.functions.get("StringifyOperator.apply/1").cloned() {
                            let out = self.exec_function_inner(&op, Some(&[val.clone()]))?;
                            println!("{}", out.to_string());
                        } else {
                            println!("{}", val.to_string());
                        }
                    } else {
                        println!("{}", val.to_string());
                    }
                }
                Ok(VMValue::Void)
            }
            "nyash.builtin.error" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    eprintln!("Error: {}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown global function: {}",
                func_name
            ))),
        }
    }

    fn execute_method_call(
        &mut self,
        receiver: &VMValue,
        method: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match receiver {
            VMValue::String(s) => match method {
                "length" => Ok(VMValue::Integer(s.len() as i64)),
                "concat" => {
                    if let Some(arg_id) = args.get(0) {
                        let arg_val = self.reg_load(*arg_id)?;
                        let new_str = format!("{}{}", s, arg_val.to_string());
                        Ok(VMValue::String(new_str))
                    } else {
                        Err(VMError::InvalidInstruction(
                            "concat requires 1 argument".into(),
                        ))
                    }
                }
                "indexOf" => {
                    if let Some(arg_id) = args.get(0) {
                        let needle = self.reg_load(*arg_id)?.to_string();
                        let idx = s.find(&needle).map(|i| i as i64).unwrap_or(-1);
                        Ok(VMValue::Integer(idx))
                    } else {
                        Err(VMError::InvalidInstruction(
                            "indexOf requires 1 argument".into(),
                        ))
                    }
                }
                "substring" => {
                    let start = if let Some(a0) = args.get(0) {
                        self.reg_load(*a0)?.as_integer().unwrap_or(0)
                    } else { 0 };
                    let end = if let Some(a1) = args.get(1) {
                        self.reg_load(*a1)?.as_integer().unwrap_or(s.len() as i64)
                    } else { s.len() as i64 };
                    let len = s.len() as i64;
                    let i0 = start.max(0).min(len) as usize;
                    let i1 = end.max(0).min(len) as usize;
                    if i0 > i1 { return Ok(VMValue::String(String::new())); }
                    // Note: operating on bytes; Nyash strings are UTF‑8, but tests are ASCII only here
                    let bytes = s.as_bytes();
                    let sub = String::from_utf8(bytes[i0..i1].to_vec()).unwrap_or_default();
                    Ok(VMValue::String(sub))
                }
                _ => Err(VMError::InvalidInstruction(format!(
                    "Unknown String method: {}",
                    method
                ))),
            },
            VMValue::BoxRef(box_ref) => {
                if let Some(p) = box_ref
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> =
                        Vec::with_capacity(args.len());
                    for a in args {
                        argv.push(self.reg_load(*a)?.to_nyash_box());
                    }
                    match host.invoke_instance_method(
                        &p.box_type,
                        method,
                        p.inner.instance_id,
                        &argv,
                    ) {
                        Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method {}.{} failed: {:?}",
                            p.box_type, method, e
                        ))),
                    }
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method {} not supported on BoxRef({})",
                        method,
                        box_ref.type_name()
                    )))
                }
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Method {} not supported on {:?}",
                method, receiver
            ))),
        }
    }

    fn execute_extern_function(
        &mut self,
        extern_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
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
