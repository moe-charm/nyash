//! function.rs — Global function calls
//!
//! Behavior-preserving extraction of execute_global_function from legacy.

use super::super::*;
use crate::backend::mir_interpreter::resolve::call_resolver;

impl MirInterpreter {
    /// Dev-only bridge: JSON.stringify(any) when invoked as a Global callee.
    /// Returns Some(result) to short-circuit normal flow.
    pub(crate) fn try_dev_json_stringify_bridge_global(
        &mut self,
        func_name: &str,
        args: &[ValueId],
    ) -> Option<Result<VMValue, VMError>> {
        if std::env::var("NYASH_JSON_STRINGIFY_DEV").ok().as_deref() == Some("1") {
            if func_name == "JSON.stringify" || func_name.starts_with("JSON.stringify/") {
                if let Some(a0) = args.get(0) {
                    let v0 = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let s = crate::boxes::json::stringify_any(v0);
                    return Some(Ok(VMValue::String(s)));
                }
            }
        }
        None
    }

    /// Handle Global callee: emit trace then dispatch to global function table.
    pub(crate) fn handle_callee_global(
        &mut self,
        func_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        if let Some(r) = self.try_dev_json_stringify_bridge_global(func_name, args) { return r; }
        let label = format!("Global:{}", func_name);
        self.emit_call_trace_label(&label, args.len(), None);
        // Dev-only: detailed arg trace for diagnosing arg marshalling (static/global)
        if std::env::var("NYASH_VM_CALL_ARG_TRACE").ok().as_deref() == Some("1") {
            let mut kinds: Vec<String> = Vec::new();
            let mut preview: Vec<String> = Vec::new();
            for (_i, a) in args.iter().enumerate().take(3) {
                match self.reg_load(*a) {
                    Ok(v) => {
                        kinds.push(crate::backend::abi_util::tag_of_vm(&v).to_string());
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
                    Err(e) => {
                        kinds.push("<err>".into());
                        preview.push(format!("err:{:?}", e));
                    }
                }
            }
            eprintln!(
                "[vm-args] callee=Global:{} argc={} a0={:?} a1={:?} a2={:?} kind0={} kind1={} kind2={}",
                func_name,
                args.len(),
                preview.get(0),
                preview.get(1),
                preview.get(2),
                kinds.get(0).map(|s| s.as_str()).unwrap_or("-"),
                kinds.get(1).map(|s| s.as_str()).unwrap_or("-"),
                kinds.get(2).map(|s| s.as_str()).unwrap_or("-")
            );
        }
        self.execute_global_function(func_name, args)
    }

    /// Handle Extern callee: emit trace then dispatch to externs.
    pub(crate) fn handle_callee_extern(
        &mut self,
        extern_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        let label = format!("Extern:{}", extern_name);
        self.emit_call_trace_label(&label, args.len(), None);
        self.execute_extern_function(extern_name, args)
    }
    pub(crate) fn execute_global_function(
        &mut self,
        func_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match func_name {
            "nyash.builtin.print" | "print" | "nyash.console.log" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    // Dev-only: print trace (kind/class) before actual print
                    if Self::print_trace_enabled() {
                        self.print_trace_emit(&val);
                    }
                    // Dev observe: Null/Missing boxes quick normalization (no behavior change to prod)
                    if let VMValue::BoxRef(bx) = &val {
                        // NullBox → always print as null (stable)
                        if bx
                            .as_any()
                            .downcast_ref::<crate::boxes::null_box::NullBox>()
                            .is_some()
                        {
                            println!("null");
                            return Ok(VMValue::Void);
                        }
                        // MissingBox → default prints as null; when flag ON, show (missing)
                        if bx
                            .as_any()
                            .downcast_ref::<crate::boxes::missing_box::MissingBox>()
                            .is_some()
                        {
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
                            if bx
                                .as_any()
                                .downcast_ref::<crate::box_trait::VoidBox>()
                                .is_some()
                            {
                                println!("null");
                                return Ok(VMValue::Void);
                            }
                        }
                        _ => {}
                    }
                    // Print raw strings directly (avoid double quoting via StringifyOperator)
                    match &val {
                        VMValue::String(s) => {
                            println!("{}", s);
                            return Ok(VMValue::Void);
                        }
                        VMValue::BoxRef(bx) => {
                            if let Some(sb) = bx
                                .as_any()
                                .downcast_ref::<crate::box_trait::StringBox>()
                            {
                                println!("{}", sb.value);
                                return Ok(VMValue::Void);
                            }
                        }
                        _ => {}
                    }
                    // Operator Box (Stringify) – dev flag gated
                    if std::env::var("NYASH_OPERATOR_BOX_STRINGIFY")
                        .ok()
                        .as_deref()
                        == Some("1")
                    {
                        if let Some(op) = self
                            .functions
                            .get("StringifyOperator.apply/1")
                            .cloned()
                        {
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
            _ => {
                let clean_name = if let Some(pos) = func_name.rfind('/') { &func_name[..pos] } else { func_name };
                if let Some(pick) = call_resolver::resolve_module_function_collect(self.functions.keys().cloned(), clean_name, args.len()) {
                    return self.handle_callee_module_function(&pick, args);
                }
                Err(VMError::InvalidInstruction(format!(
                    "Unknown global function: {}",
                    func_name
                )))
            }
        }
    }

    /// Handle ModuleFunction callee: resolves against the MIR module's function table.
    /// Name can be canonical ("BoxName.method/Arity") or base without arity; if
    /// arity is missing, it is appended using the call-site argument count.
    pub(crate) fn handle_callee_module_function(
        &mut self,
        name: &str,
        args: &[ValueId],
        ) -> Result<VMValue, VMError> {
        if !crate::mir::resolve::call_resolver_core::is_fully_qualified(name) {
            return Err(VMError::InvalidInstruction(format!(
                "VM received incomplete module function name: {}",
                name
            )));
        }

        // Lifecycle: handle birth enter (idempotence + reentrancy)
        let mut birth_key: Option<u64> = None;
        let mut is_birth_fn = false;
        if let Some((_cls, method_arity)) = name.split_once('.') {
            let method = method_arity.split('/').next().unwrap_or(method_arity);
            if method == "birth" {
                is_birth_fn = true;
                if let Some(first) = args.get(0) {
                    let key = self.object_key_for(*first);
                    if self.contracts_born.contains(&key) {
                        if std::env::var("NYASH_VM_BIRTH_TRACE").ok().as_deref() == Some("1") {
                            eprintln!("{{\"kind\":\"birth_idempotent\",\"name\":\"{}\",\"key\":{}}}", name, key);
                        }
                        return Ok(VMValue::Void);
                    }
                    if !self.contracts_in_birth.insert(key) {
                        return Err(VMError::InvalidInstruction("reentrant birth()".to_string()));
                    }
                    birth_key = Some(key);
                }
            }
        }

        
        // Fail-Fast: unified unborn guard for instance-dispatch ModuleFunction (non-birth)
        if let Some((_, method)) = name.split_once('.') {
            if method != "birth" {
                if let Some(first) = args.get(0) { self.check_unborn_guard(*first)?; }
            }
        }
        let label = format!("ModuleFn:{}", name);
        self.emit_call_trace_label(&label, args.len(), None);
        if std::env::var("NYASH_VM_CALL_ARG_TRACE").ok().as_deref() == Some("1") {
            let mut kinds: Vec<String> = Vec::new();
            let mut preview: Vec<String> = Vec::new();
            for (_i, a) in args.iter().enumerate().take(3) {
                match self.reg_load(*a) {
                    Ok(v) => {
                        kinds.push(crate::backend::abi_util::tag_of_vm(&v).to_string());
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
                    Err(e) => {
                        kinds.push("<err>".into());
                        preview.push(format!("err:{:?}", e));
                    }
                }
            }
            eprintln!(
                "[vm-args] callee=ModuleFn:{} argc={} a0={:?} a1={:?} a2={:?} kind0={} kind1={} kind2={}",
                name,
                args.len(),
                preview.get(0),
                preview.get(1),
                preview.get(2),
                kinds.get(0).map(|s| s.as_str()).unwrap_or("-"),
                kinds.get(1).map(|s| s.as_str()).unwrap_or("-"),
                kinds.get(2).map(|s| s.as_str()).unwrap_or("-")
            );
        }

        // Normalize name: ensure canonical "/arity" suffix
        let want_name = if name.contains('/') {
            name.to_string()
        } else {
            format!("{}/{}", name, args.len())
        };

        // Exact match first
        if let Some(func) = self.functions.get(&want_name).cloned() {
            let mut argv: Vec<VMValue> = Vec::new();
            for a in args { argv.push(self.reg_load(*a)?); }
            {
                let r = self.exec_function_inner(&func, Some(&argv));
                if is_birth_fn {
                    if let Some(k) = birth_key { self.contracts_in_birth.remove(&k); }
                    if r.is_ok() {
                        if let Some(first) = args.get(0) {
                            self.lifecycle_contracts_birth(*first, args.len().saturating_sub(1));
                        }
                    } else {
                        if let Some(first) = args.get(0) { self.regs.remove(first); }
                    }
                }
                return r;
            }
        }

        // Tail-based fallback: collect candidates that end with ".method/arity"
        // if the provided name was not canonical but looked like "Class.method".
        if let Some((class_or_alias, method)) = name.split_once('.') {
            let tail = format!(".{}{}", method, format!("/{}", args.len()));
            let mut cands: Vec<String> = self
                .functions
                .keys()
                .filter(|k| k.ends_with(&tail) && (k.starts_with(&format!("{}.", class_or_alias)) || k.starts_with(&format!("{}_", class_or_alias))))
                .cloned()
                .collect();
            if !cands.is_empty() {
                cands.sort();
                let pick = cands.remove(0);
                if let Some(func) = self.functions.get(&pick).cloned() {
                    let mut argv: Vec<VMValue> = Vec::new();
                    for a in args { argv.push(self.reg_load(*a)?); }
                    {
                let r = self.exec_function_inner(&func, Some(&argv));
                if is_birth_fn {
                    if let Some(k) = birth_key { self.contracts_in_birth.remove(&k); }
                    if r.is_ok() {
                        if let Some(first) = args.get(0) {
                            self.lifecycle_contracts_birth(*first, args.len().saturating_sub(1));
                        }
                    } else {
                        if let Some(first) = args.get(0) { self.regs.remove(first); }
                    }
                }
                return r;
            }
                }
            }
        }

        Err(VMError::InvalidInstruction(format!(
            "Unknown module function: {} (arity={})",
            name,
            args.len()
        )))
    }
}
