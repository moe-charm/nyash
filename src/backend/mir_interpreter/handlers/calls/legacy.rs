// Import handler-level scope (parent: handlers) for types and helpers
use super::super::*;
// Also allow referring to sibling `calls` module items if/when extracted (not needed currently)

impl MirInterpreter {
    pub(crate) fn handle_call(
        &mut self,
        dst: Option<ValueId>,
        func: ValueId,
        callee: Option<&Callee>,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // LocalSSA at call-site: prefer a materialized in-block SSA id for each arg
        let args2: Vec<ValueId> = self.materialize_args_in_current_block(args);
        let call_result = if let Some(callee_type) = callee {
            self.execute_callee_call(callee_type, &args2)?
        } else {
            self.execute_legacy_call(func, &args2)?
        };
        if let Some(d) = dst {
            self.regs.insert(d, call_result);
        }
        Ok(())
    }

    pub(crate) fn execute_callee_call(
        &mut self,
        callee: &Callee,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // Optional: emit one-line JSON call trace for parity checks (refactored helper)
        match callee {
            // Global: trace emission is handled inside handle_callee_global after bridge check
            Callee::Global(_name) => {}
            Callee::ModuleFunction(_name) => {
                // Trace emitted below in specific handler; keep quiet here
            }
            Callee::Method { box_name, method, receiver, .. } => {
                let label = format!("Method:{}.{}{}", box_name, method, format!("/{}", args.len()));
                let recv_id = receiver.map(|v| v.as_u32());
                self.emit_call_trace_label(&label, args.len(), recv_id);
            }
            Callee::Constructor { box_type } => {
                let label = format!("Ctor:{}", box_type);
                self.emit_call_trace_label(&label, args.len(), None);
            }
            Callee::Closure { .. } => {
                self.emit_call_trace_label("Closure", args.len(), None);
            }
            Callee::Value(_fid) => {
                self.emit_call_trace_label("Value", args.len(), None);
            }
            // Extern: trace emission is handled inside handle_callee_extern
            Callee::Extern(_name) => {}
        }
        match callee {
            Callee::Global(func_name) => self.handle_callee_global(func_name, args),
            Callee::ModuleFunction(func_name) => self.handle_callee_module_function(func_name, args),
            Callee::Method { box_name, method, receiver, certainty: _, } => {
                if method == &"birth" && crate::config::env::cli_verbose() && !crate::config::env::cli_quiet() {
                    eprintln!("[vm-call] invoking birth() via method call");
                }
                if let Some(recv_id) = receiver {
                    // Fail-Fast: forbid operations on unborn InstanceBox until birth()
                    if method != "birth" {
                        let is_instance = match self.reg_load(*recv_id).ok() {
                            Some(VMValue::BoxRef(b)) => b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some(),
                            _ => false,
                        };
                        if is_instance {
                            let key = self.object_key_for(*recv_id);
                            let seen_new = self.contracts_new.contains(&key);
                            let seen_birth = self.contracts_born.contains(&key) || self.contracts_in_birth.contains(&key);
                            if seen_new && !seen_birth {
                                return Err(VMError::InvalidInstruction(
                                    "operation on unborn instance (call birth() first)".to_string(),
                                ));
                            }
                        }
                    }
                    // LocalSSA for receiver: prefer materialized id within current block
                    let recv_id = self.materialize_recv_in_current_block(*recv_id);
                    // Primary: load receiver by id. If undefined, attempt a best-effort
                    // recovery by resolving a local Copy(dst := recv_id) in the same block,
                    // then fall back to arg[0] or error.
                    let recv_val = match self.reg_load(recv_id) {
                        Ok(v) => v,
                        Err(e) => {
                            // Try: find a preceding Copy in the current block with src=recv_id
                            let mut recovered: Option<VMValue> = None;
                            if let (Some(fn_name), Some(bb_id)) = (self.cur_fn.clone(), self.last_block) {
                                if let Some(fun) = self.functions.get(&fn_name) {
                                    if let Some(bb) = fun.blocks.get(&bb_id) {
                                        for inst in &bb.instructions {
                                            if let crate::mir::MirInstruction::Copy { dst, src } = inst {
                                                // Pattern A: we copied into recv_id just before the call (dst == recv_id)
                                                if *dst == recv_id {
                                                    if let Ok(v2) = self.reg_load(*src) {
                                                        recovered = Some(v2);
                                                        break;
                                                    }
                                                }
                                                // Pattern B: we copied from recv_id into a local tmp (src == recv_id)
                                                if *src == recv_id {
                                                    if let Ok(v2) = self.reg_load(*dst) {
                                                        recovered = Some(v2);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(v) = recovered {
                                v
                            } else {
                                // Autoload guard (plugins profile): when using kind="dylib" autoload is active,
                                // try a best-effort recovery by scanning current registers for a BoxRef whose
                                // type matches the expected receiver box (e.g., CounterBox/FixtureBox).
                                if std::env::var("NYASH_USING_DYLIB_AUTOLOAD").ok().as_deref() == Some("1") {
                                    let mut found: Option<VMValue> = None;
                                    for (_id, val) in self.regs.iter() {
                                        if let VMValue::BoxRef(bx) = val {
                                            // Match plugin-backed boxes by inner box_type when available
                                            if let Some(pb) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                                if pb.box_type == *box_name { found = Some(val.clone()); break; }
                                            } else if bx.type_name() == box_name {
                                                // Builtin boxes expose their concrete type name
                                                found = Some(val.clone()); break;
                                            }
                                        }
                                    }
                                    if let Some(v) = found { v } else {
                                        // Fallbacks (dev-only/tolerant modes)
                                        let tolerate = std::env::var("NYASH_VM_RECV_ARG_FALLBACK").ok().as_deref() == Some("1")
                                            || std::env::var("NYASH_VM_TOLERATE_VOID").ok().as_deref() == Some("1");
                                        if tolerate {
                                            if let Some(a0) = args.get(0) { self.reg_load(*a0)? } else { return Err(e); }
                                        } else {
                                            // Narrow, behavior-preserving rescue: for ParserBox.* inside ParserBox.* functions,
                                            // fallback receiver to the `me` parameter of the current function.
                                            if box_name == "ParserBox" {
                                                if let Some(cur) = &self.cur_fn {
                                                    if cur.starts_with("ParserBox.") {
                                                        if let Some(fun) = self.functions.get(cur) {
                                                            if let Some(me_vid) = fun.params.first() {
                                                                if let Ok(mev) = self.reg_load(*me_vid) { return Ok(mev); }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // Minimal safety: common pure methods with undefined recv return neutral defaults
                                            if method == "length" { return Ok(VMValue::Integer(0)); }
                                            return Err(e);
                                        }
                                    }
                                } else {
                                    // Dev fallback: use args[0] as surrogate when enabled
                                    let tolerate = std::env::var("NYASH_VM_RECV_ARG_FALLBACK").ok().as_deref() == Some("1")
                                        || std::env::var("NYASH_VM_TOLERATE_VOID").ok().as_deref() == Some("1");
                                    if tolerate {
                                        if let Some(a0) = args.get(0) { self.reg_load(*a0)? } else { return Err(e); }
                                    } else {
                                        // Narrow, behavior-preserving rescue: for ParserBox.* inside ParserBox.* functions,
                                        // fallback receiver to the `me` parameter of the current function.
                                        if box_name == "ParserBox" {
                                            if let Some(cur) = &self.cur_fn {
                                                if cur.starts_with("ParserBox.") {
                                                    if let Some(fun) = self.functions.get(cur) {
                                                        if let Some(me_vid) = fun.params.first() {
                                                            if let Ok(mev) = self.reg_load(*me_vid) { return Ok(mev); }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Minimal safety: common pure methods with undefined recv
                                        // return neutral defaults instead of crashing (length -> 0).
                                        if method == "length" { return Ok(VMValue::Integer(0)); }
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    };
                    let dev_trace = std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1");
                    // Fast bridge for builtin boxes (Array) and common methods.
                    // Preserve legacy semantics when plugins are absent.
                    if let VMValue::BoxRef(bx) = &recv_val {
                        if let Some(arr) = bx
                            .as_any()
                            .downcast_ref::<crate::boxes::array::ArrayBox>()
                        {
                            if let Some(res) =
                                self.box_array_fastpath(arr, method.as_str(), args)
                            {
                                return res;
                            }
                        }
                        if let Some(map) = bx
                            .as_any()
                            .downcast_ref::<crate::boxes::map_box::MapBox>()
                        {
                            if let Some(res) =
                                self.box_map_fastpath(map, method.as_str(), args)
                            {
                                return res;
                            }
                        }
                    }
                    // Minimal bridge for birth(): delegate to BoxCall handler and return Void
                    if method == &"birth" {
                        let _ = self.handle_box_call(None, recv_id, method, args)?;
                        return Ok(VMValue::Void);
                    }
                    let is_kw = method == &"keyword_to_token_type";
                    if dev_trace && is_kw {
                        let a0 = args.get(0).and_then(|id| self.reg_load(*id).ok());
                        eprintln!("[vm-trace] mcall {} argv0={:?}", method, a0);
                    }
                    let out = self.execute_method_call(&recv_val, method, args)?;
                    if dev_trace && is_kw {
                        eprintln!("[vm-trace] mret  {} -> {:?}", method, out);
                    }
                    Ok(out)
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
            Callee::Extern(extern_name) => self.handle_callee_extern(extern_name, args),
        }
    }

    pub(crate) fn execute_legacy_call(
        &mut self,
        func_id: ValueId,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        let name_val = self.reg_load(func_id)?;
        let raw = match name_val {
            VMValue::String(ref s) => s.clone(),
            other => other.to_string(),
        };

        // Fail-Fast parity for legacy NameConst-based calls that target instance methods
        if let Some((_, method_part)) = raw.split_once('.') {
            let method_only = method_part.split('/').next().unwrap_or(method_part);
            if method_only != "birth" {
                if let Some(first) = args.get(0) {
                    if let VMValue::BoxRef(b) = self.reg_load(*first)? {
                        if b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() {
                            let key = self.object_key_for(*first);
                            let seen_new = self.contracts_new.contains(&key);
                            let seen_birth = self.contracts_born.contains(&key) || self.contracts_in_birth.contains(&key);
                            if seen_new && !seen_birth {
                                return Err(VMError::InvalidInstruction(
                                    "operation on unborn instance (call birth() first)".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Dev-only: built-in fallback for JSON.stringify(any)
        // This short-circuits legacy resolution to provide a stable stringify during
        // declarative MIR bring-up without requiring a user-prelude.
        if std::env::var("NYASH_JSON_STRINGIFY_DEV").ok().as_deref() == Some("1") {
            // Normalize to base without trailing "/arity" and also allow raw prefix
            let base = if let Some((b, _)) = raw.rsplit_once('/') { b } else { raw.as_str() };
            if base == "JSON.stringify" || raw.starts_with("JSON.stringify") {
                let a0 = args.get(0).ok_or_else(|| VMError::InvalidInstruction(
                    "JSON.stringify expects 1 argument".into(),
                ))?;
                let v0 = self.reg_load(*a0)?.to_nyash_box();
                let s = crate::boxes::json::stringify_any(v0);
                return Ok(VMValue::String(s));
            }
        }

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
                } else { (None, base.clone()) };
                // Use FunctionIndex for tail-unique query
                let idx = crate::mir::indexes::functions::FunctionIndex::from_map(&self.functions);
                let mut cands: Vec<String> = match &maybe_box {
                    Some(bx) => match idx.tail_unique(Some(bx.as_str()), &method, want_arity) {
                        crate::mir::indexes::functions::TailQueryResult::Unique(n) => vec![n],
                        crate::mir::indexes::functions::TailQueryResult::Ambiguous(v) => v,
                        crate::mir::indexes::functions::TailQueryResult::None => Vec::new(),
                    },
                    None => match idx.tail_unique(None, &method, want_arity) {
                        crate::mir::indexes::functions::TailQueryResult::Unique(n) => vec![n],
                        crate::mir::indexes::functions::TailQueryResult::Ambiguous(v) => v,
                        crate::mir::indexes::functions::TailQueryResult::None => Vec::new(),
                    }
                };
                if cands.len() > 1 {
                    if let Some(cur) = &self.cur_fn {
                        cands = crate::mir::indexes::functions::prefer_current_box(cur, &cands);
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

        let fname = match pick {
            Some(n) => n,
            None => {
                // As a last resort, honor dev bridge for JSON.stringify(any)
                if std::env::var("NYASH_JSON_STRINGIFY_DEV").ok().as_deref() == Some("1") {
                    let base = if let Some((b, _)) = raw.rsplit_once('/') { b } else { raw.as_str() };
                    if base == "JSON.stringify" || raw.starts_with("JSON.stringify") {
                        if let Some(a0) = args.get(0) {
                            let v0 = self.reg_load(*a0)?.to_nyash_box();
                            let s = crate::boxes::json::stringify_any(v0);
                            return Ok(VMValue::String(s));
                        }
                    }
                }
                return Err(VMError::InvalidInstruction(format!(
                    "call unresolved: {} (arity={})",
                    raw,
                    args.len()
                )));
            }
        };

        if std::env::var("NYASH_VM_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[vm] legacy-call resolved '{}' -> '{}'", raw, fname);
        }
        if std::env::var("NYASH_WARN_LEGACY_CALL").ok().as_deref() == Some("1") {
            // JSON line (stderr) for machine-friendly observation
            fn esc(s: &str) -> String { s.replace('"', "\\\"") }
            eprintln!(
                "{{\"kind\":\"legacy_call\",\"from\":\"{}\",\"to\":\"{}\",\"arity\":{}}}",
                esc(&raw), esc(&fname), args.len()
            );
        }

        // If calling a class birth function directly (e.g., "MyBox.birth/1"),
        // mark the first argument (receiver) as born before executing the body.
        if let Some((_, mpart)) = fname.split_once('.') {
            let method_only = mpart.split('/').next().unwrap_or(mpart);
            if method_only == "birth" {
                if let Some(first) = args.get(0) {
                    self.lifecycle_contracts_birth(*first, args.len().saturating_sub(1));
                }
            }
        }

        let callee =
            self.functions.get(&fname).cloned().ok_or_else(|| {
                VMError::InvalidInstruction(format!("function not found: {}", fname))
            })?;

        let mut argv: Vec<VMValue> = Vec::new();
        for a in args {
            argv.push(self.reg_load(*a)?);
        }
        if std::env::var("NYASH_VM_CALL_ARG_TRACE").ok().as_deref() == Some("1") {
            let mut kinds: Vec<String> = Vec::new();
            let mut preview: Vec<String> = Vec::new();
            for v in argv.iter().take(2) {
                kinds.push(crate::backend::abi_util::tag_of_vm(v).to_string());
                preview.push(match v {
                    VMValue::Integer(n) => format!("i64:{}", n),
                    VMValue::Float(f) => format!("f64:{:.3}", f),
                    VMValue::Bool(b) => format!("bool:{}", b),
                    VMValue::String(ref s) => format!("str:'{}'", s),
                    VMValue::Void => "void".into(),
                    VMValue::BoxRef(ref bx) => format!("box:{}", bx.type_name()),
                    VMValue::Future(_) => "future".into(),
                });
            }
            eprintln!(
                "[vm-args] callee=Legacy:{} argc={} a0={:?} a1={:?} kind0={} kind1={}",
                fname,
                argv.len(),
                preview.get(0),
                preview.get(1),
                kinds.get(0).map(|s| s.as_str()).unwrap_or("-"),
                kinds.get(1).map(|s| s.as_str()).unwrap_or("-")
            );
        }
        let dev_trace = std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1");
        let is_kw = fname.ends_with("JsonTokenizer.keyword_to_token_type/1");
        let is_sc_ident = fname.ends_with("JsonScanner.read_identifier/0");
        let is_sc_current = fname.ends_with("JsonScanner.current/0");
        let is_tok_kw = fname.ends_with("JsonTokenizer.tokenize_keyword/0");
        let is_tok_struct = fname.ends_with("JsonTokenizer.create_structural_token/2");
        if dev_trace && (is_kw || is_sc_ident || is_sc_current || is_tok_kw || is_tok_struct) {
            if let Some(a0) = argv.get(0) {
                eprintln!("[vm-trace] call {} argv0={:?}", fname, a0);
            } else {
                eprintln!("[vm-trace] call {}", fname);
            }
        }
        // Dev trace: emit a synthetic "call" event for global function calls.
        // NOTE (Operator Guard): 観測は「非再入のイベント」で行うのが原則。
        // OperatorBoxGuard が exec_function_inner の入口で採用/遮断を一本化しているため、
        // ここで観測目的の追加呼び出し（再入）を行わないこと（Guardポリシー）。
        // 目的はあくまでイベント発火のみ（NYASH_BOX_TRACE_FILTER でフィルタ可能）。
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
                    let esc = |s: &str| {
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
        let out = self.exec_function_inner(&callee, Some(&argv))?;
        if dev_trace && (is_kw || is_sc_ident || is_sc_current || is_tok_kw || is_tok_struct) {
            eprintln!("[vm-trace] ret  {} -> {:?}", fname, out);
        }
        Ok(out)
    }

    

    

    pub(crate) fn execute_extern_function(
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
