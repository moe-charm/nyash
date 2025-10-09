//! function.rs — Global function calls
//!
//! Behavior-preserving extraction of execute_global_function from legacy.

use super::super::*;

impl MirInterpreter {
    /// Helper: Check reentrancy and emit trace if enabled.
    /// Returns depth on success, or error if limit exceeded.
    fn check_and_trace_reenter(
        &mut self,
        prefix: &str,
        func_name: &str,
        argc: usize,
    ) -> Result<(), VMError> {
        if crate::config::env::vm_reenter_trace() || crate::config::env::vm_reenter_limit().is_some() {
            let clean = if let Some(pos) = func_name.rfind('/') { &func_name[..pos] } else { func_name };
            let key = format!("{}:{}:{}", prefix, clean, argc);
            let depth = match crate::common::reenter_guard::bump_and_check(
                &mut self.reenter_count,
                &key,
                crate::config::env::vm_reenter_limit()
            ) {
                Ok(d) => d,
                Err(e) => return Err(VMError::InvalidInstruction(e)),
            };
            if crate::config::env::vm_reenter_trace() && (depth == 64 || depth % 256 == 0) {
                eprintln!("[vm-reenter] {} depth={}", key, depth);
            }
        }
        Ok(())
    }

    /// Helper: Emit detailed arg trace for diagnosing arg marshalling.
    /// Dev-only, enabled by NYASH_VM_CALL_ARG_TRACE=1.
    fn emit_call_arg_trace(&mut self, prefix: &str, func_name: &str, args: &[ValueId]) {
        if !super::super::VmConfig::global().call_arg_trace {
            return;
        }
        let mut kinds: Vec<String> = Vec::new();
        let mut preview: Vec<String> = Vec::new();
        for a in args.iter().take(3) {
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
            "[vm-args] callee={}:{} argc={} a0={:?} a1={:?} a2={:?} kind0={} kind1={} kind2={}",
            prefix,
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
        self.check_and_trace_reenter("Global", func_name, args.len())?;
        // Narrow safety valve to break Json* index_of_from recursion (dev):
        // Handle dotted canonical or base names here when arity==3.
        if args.len() == 3 {
            let fname = if let Some(pos) = func_name.rfind('/') { &func_name[..pos] } else { func_name };
            if fname == "JsonCursorBox.index_of_from" || fname == "JsonFragBox.index_of_from" {
                let hay = self.reg_load(args[0])?.to_string();
                let needle = self.reg_load(args[1])?.to_string();
                let pos = self.reg_load(args[2])?.as_integer().unwrap_or(0).max(0) as usize;
                let idx: i64 = if needle.is_empty() { 0 } else if pos >= hay.len() { -1 } else { hay[pos..].find(&needle).map(|i| (pos + i) as i64).unwrap_or(-1) };
                return Ok(VMValue::Integer(idx));
            }
        }
        if let Some(r) = self.try_dev_json_stringify_bridge_global(func_name, args) { return r; }
        let label = format!("Global:{}", func_name);
        self.emit_call_trace_label(&label, args.len(), None);
        self.emit_call_arg_trace("Global", func_name, args);
        let r = self.execute_global_function(func_name, args);
        if let Ok(ref v) = r { self.maybe_register_scope_value(v); }
        r
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
        self.check_and_trace_reenter("Global", func_name, args.len())?;
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
                // Strict-first: if dotted, only accept exact canonical Class.method/arity
                if clean_name.contains('.') {
                    let canon = format!("{}/{}", clean_name, args.len());
                    if self.functions.contains_key(&canon) {
                        return self.handle_callee_module_function(&canon, args);
                    }
                    // Allow alias-alias form only under legacy fallback policy
                    if crate::config::env::vm_global_tail_fallback() {
                        if let Some((cls, meth)) = clean_name.split_once('.') {
                            let alias_alias = format!("{}_{}.{}{}", cls, cls, meth, format!("/{}", args.len()));
                            if self.functions.contains_key(&alias_alias) {
                                return self.handle_callee_module_function(&alias_alias, args);
                            }
                        }
                    }
                    if crate::config::env::vm_global_tail_fallback() {
                        if let Some(pick) = crate::mir::resolve::module_function_resolver::resolve_strict(self.functions.keys().cloned(), clean_name, args.len(), true) {
                            return self.handle_callee_module_function(&pick, args);
                        }
                        let wide = std::env::var("NYASH_VM_GLOBAL_TAIL_WIDE").ok().as_deref() == Some("1");
                        if let Some((cls, method)) = clean_name.split_once('.') {
                            let cands = crate::common::call_policy::tail_candidates(self.functions.keys(), cls, method, args.len(), wide);
                            if let Some(pick) = cands.first() {
                                if !crate::common::call_policy::is_immediate_cycle(pick.as_str(), clean_name) {
                                    return self.handle_callee_module_function(pick, args);
                                }
                            }
                        }
                    }
                    return Err(VMError::InvalidInstruction(format!(
                        "Unknown module function (strict global): {} (arity={})",
                        clean_name, args.len()
                    )));
                }
                // Non-dotted: legacy global resolution
                if let Some(pick) = crate::mir::resolve::module_function_resolver::resolve_strict(self.functions.keys().cloned(), clean_name, args.len(), true) {
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
        self.check_and_trace_reenter("ModuleFn", name, args.len())?;
        // Dev safety valve: intercept hot recursive helpers to avoid resolver-induced cycles.
        // JsonCursorBox.index_of_from/3 and JsonFragBox.index_of_from/3 are pure functions;
        // implement minimal native evaluation to break potential recursion.
        if (name == "JsonCursorBox.index_of_from" || name == "JsonFragBox.index_of_from"
            || name == "JsonCursorBox.index_of_from/3" || name == "JsonFragBox.index_of_from/3")
            && args.len() == 3
        {
            let hay = self.reg_load(args[0])?.to_string();
            let needle = self.reg_load(args[1])?.to_string();
            let pos = self.reg_load(args[2])?.as_integer().unwrap_or(0).max(0) as usize;
            let idx: i64 = if needle.is_empty() {
                0
            } else if pos >= hay.len() {
                -1
            } else {
                match hay[pos..].find(&needle) {
                    Some(i) => (pos + i) as i64,
                    None => -1,
                }
            };
            return Ok(VMValue::Integer(idx));
        }
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
                        if super::super::VmConfig::global().birth_trace {
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
        self.emit_call_arg_trace("ModuleFn", name, args);

        // Normalize name: ensure canonical "/arity" suffix
        let want_name = if name.contains('/') {
            name.to_string()
        } else {
            format!("{}/{}", name, args.len())
        };

        // Bridge: Builtin vtable dispatch for dotted names (ArrayBox/MapBox/StringBox/ConsoleBox)
        // This allows tests and builder to use unified ModuleFunction while VM routes to BoxCall.
        if let Some((class, method_arity)) = name.split_once('.') {
            let method = if let Some((m, _arity)) = method_arity.split_once('/') { m } else { method_arity };
            if !args.is_empty() {
                match class {
                    "ArrayBox" | "MapBox" | "StringBox" | "ConsoleBox" => {
                        if method != "birth" {
                            // Build a Method callee and reuse the legacy method path which returns VMValue
                            let recv = args[0];
                            let rest: Vec<ValueId> = args.iter().copied().skip(1).collect();
                            let box_name = match self.reg_load(recv) {
                                Ok(super::super::VMValue::BoxRef(bx)) => {
                                    if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                                        inst.class_name.clone()
                                    } else { bx.type_name().to_string() }
                                }
                                Ok(super::super::VMValue::String(_)) => "StringBox".to_string(),
                                _ => class.to_string(),
                            };
                            let cal = crate::mir::Callee::Method { box_name, method: method.to_string(), receiver: Some(recv), certainty: crate::mir::definitions::call_unified::TypeCertainty::Known };
                            return self.execute_callee_call(&cal, &rest);
                        }
                    }
                    _ => {}
                }
            }
        }


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
        if let Some((class_or_alias, method_part)) = name.split_once('.') {
            let method_only = match method_part.split_once('/') { Some((m, _)) => m, None => method_part };
            let wide = std::env::var("NYASH_VM_MODULE_TAIL_WIDE").ok().as_deref() == Some("1");
            let cands = crate::common::call_policy::tail_candidates(self.functions.keys(), class_or_alias, method_only, args.len(), wide);
            if !cands.is_empty() {
                let pick = cands[0].clone();
                if crate::common::call_policy::is_immediate_cycle(&pick, name) {
                    return Err(VMError::InvalidInstruction(
                        crate::common::diagnostics::msg::circular_tail_resolution(
                            crate::common::call_policy::base_without_arity(&pick)
                        )
                    ));
                }
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
