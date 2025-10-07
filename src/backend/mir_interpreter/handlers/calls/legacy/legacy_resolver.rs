//! Legacy function name resolution and execution
//!
//! Handles string-based function calls from NameConst MIR instructions.
//! Provides fallback resolution, arity normalization, and FunctionIndex lookup.

use super::super::super::*;
use crate::backend::mir_interpreter::VmConfig;

impl MirInterpreter {
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
                if let Some(first) = args.get(0) { self.check_unborn_guard(*first)?; }
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

        if VmConfig::global().call_trace {
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
        if VmConfig::global().call_arg_trace {
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
        let dev_trace = VmConfig::global().general_trace;
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

    

    

}
