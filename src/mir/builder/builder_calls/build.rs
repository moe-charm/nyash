// Build functions for function and method calls
use super::super::{Effect, EffectMask, MirBuilder, MirInstruction, MirType, ValueId};
use crate::ast::ASTNode;
use crate::mir::builder::calls::{call_unified, method_resolution, special_handlers};
use crate::mir::builder::calls::call_target::CallTarget;
use crate::mir::TypeOpKind;

impl MirBuilder {
    // === ChatGPT5 Pro Design: Type-safe Call Resolution System ===
    /// Normalize external (builtin) ModuleFunction names so dotted+arity strings always resolve.
    /// Examples:
    /// - "String.len/1"  + args.len()==1  -> Some("StringBox.len/0")
    /// - "Array.get/2"   + args.len()==2  -> Some("ArrayBox.get/1")
    /// - "Map.set/3"     + args.len()==3  -> Some("MapBox.set/2")
    /// Returns None when the input isn't a dotted name or doesn't target known builtins.
    fn normalize_external_module_function_name(&self, name: &str, argc: usize) -> Option<String> {
        if !name.contains('.') { return None; }
        let (class, method_and_arity) = name.split_once('.')?;
        let canon_class = match class {
            "String" => Some("StringBox"),
            "Array" => Some("ArrayBox"),
            "Map" => Some("MapBox"),
            "Console" => Some("ConsoleBox"),
            s if s.ends_with("Box") => Some(s),
            _ => None,
        }?;
        let (mut method, provided_arity_opt) = if let Some((m, a)) = method_and_arity.split_once('/') {
            (m.to_string(), a.parse::<usize>().ok())
        } else {
            (method_and_arity.to_string(), None)
        };
        // Method alias normalization for builtins
        if canon_class == "StringBox" {
            if method.as_str() == "len" { method = "length".to_string(); }
        }
        let total = provided_arity_opt.unwrap_or(argc);
        let method_arity = total.saturating_sub(1);
        Some(format!("{}.{}{}", canon_class, method, format!("/{}", method_arity)))
    }

    /// Resolve function call target to type-safe Callee
    /// Implements the core logic of compile-time function resolution
    fn resolve_call_target(&self, name: &str) -> Result<super::super::super::Callee, String> {
        method_resolution::resolve_call_target(
            name,
            &self.current_static_box,
            &self.variable_map,
        )
    }

    // Build function call: name(args)
    pub(in super::super) fn build_function_call(
        &mut self,
        mut name: String,
        args: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Normalize dotted names without arity to fully qualified form
        if name.contains('.') && !name.contains('/') {
            if let Ok(norm) = crate::mir::resolve::call_resolver_core::normalize(&name, args.len()) {
                name = norm;
            }
        }

        // dev trace removed
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            let cur_fun = self.current_function.as_ref().map(|f| f.signature.name.clone()).unwrap_or_else(|| "<none>".to_string());
            eprintln!(
                "[builder] function-call name={} static_ctx={} in_fn={}",
                name,
                self.current_static_box.as_deref().unwrap_or(""),
                cur_fun
            );
        }
        // Handle built-in dispatcher: call("Box.method/N", recv, args...)
        // This allows Ny source to explicitly target a ModuleFunction without macro expansion.
        if name == "call" {
            if args.is_empty() {
                return Err("MIR compilation error: Unresolved function: call. Function call not found. Check spelling or add explicit scope qualifier".to_string());
            }
            if let Some(target_name) = special_handlers::extract_string_literal(&args[0]) {
                // Build remaining arguments
                let mut call_args = Vec::new();
                for a in args.iter().skip(1) { call_args.push(self.build_expression(a.clone())?); }
                // Direct extern mapping for Timer.now_ms as a convenience
                if (target_name == "Timer.now_ms/0" || target_name == "TimerBox.now_ms/0") && call_args.is_empty() {
                    return self.emit_timer_now_ms_call();
                }
                // Normalize builtin names like String.len/1 → StringBox.length/0
                let normalized = self
                    .normalize_external_module_function_name(&target_name, call_args.len())
                    .or_else(|| if target_name.contains(".") { Some(target_name.clone()) } else { None });
                if let Some(ext_name) = normalized {
                    let dst = self.value_gen.next();
                    let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &ext_name)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: fun_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(ext_name.clone())),
                        args: call_args,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &ext_name);
                    return Ok(dst);
                }
                return Err(format!(
                    "MIR compilation error: Unresolved call target: '{}'. Expected dotted+arity like 'String.len/1'",
                    target_name
                ));
            } else {
                return Err("MIR compilation error: Unresolved function: call. First argument must be a string literal target".to_string());
            }
        }

        // Literal wrappers: map({...}) / arr([...]) → treat as literals when macros are disabled
        if (name == "map" || name == "json") && args.len() == 1 {
            if let ASTNode::MapLiteral { .. } = &args[0] {
                return self.build_expression(args[0].clone());
            }
        }
        if name == "arr" && args.len() == 1 {
            if let ASTNode::ArrayLiteral { .. } = &args[0] {
                return self.build_expression(args[0].clone());
            }
        }

        // Minimal TypeOp wiring via function-style: isType(value, "Type"), asType(value, "Type")
        if (name == "isType" || name == "asType") && args.len() == 2 {
            if let Some(type_name) = special_handlers::extract_string_literal(&args[1]) {
                let val = self.build_expression(args[0].clone())?;
                let ty = special_handlers::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if name == "isType" {
                    TypeOpKind::Check
                } else {
                    TypeOpKind::Cast
                };
                self.emit_instruction(MirInstruction::TypeOp {
                    dst,
                    op,
                    value: val,
                    ty,
                })?;
                return Ok(dst);
            }
        }
        // Keep original args for special handling (math.*)
        let raw_args = args.clone();

        if let Some(res) = self.try_handle_math_function(&name, raw_args) { return res; }

        // Phase 2: ModuleFunction unification（envガード）
        // If enabled and current module contains a matching function, emit a Call
        // with callee=ModuleFunction to avoid Global builtin path.
        let module_fn_unify = true;
        // Canonical safe step（envガード・既定OFF）: dotted+arity の完全一致は ModuleFunction を許可
        let module_fn_canon = true;
        if module_fn_canon {
            if let Some(ref module) = self.current_module {
                let arity = args.len();
                let cand2 = format!("{}/{}", name, arity);
                let dotted = name.contains('.') && name.contains('/');
                // Phase 1: Self-recursive direct call (env-gated)
                if std::env::var("NYASH_MIR_SELFREC_DIRECT").ok().as_deref() == Some("1") {
                    if let Some(cur_fn) = self.current_function.as_ref() {
                        let cur_name = cur_fn.signature.name.clone();
                        // If this call targets current function (name or name/arity), force ModuleFunction
                        let is_self = cur_name == name || cur_name == cand2;
                        if is_self {
                            let target_key = if module.functions.contains_key(&cur_name) { cur_name.clone() } else { cand2.clone() };
                            if module.functions.contains_key(&target_key) {
                                let dst2 = self.value_gen.next();
                                let fun_val2 = crate::mir::builder::name_const::make_name_const_result(self, &target_key)?;
                                let mut arg_values2 = Vec::new();
                                for a in &args { arg_values2.push(self.build_expression(a.clone())?); }
                                self.emit_instruction(MirInstruction::Call {
                                    dst: Some(dst2),
                                    func: fun_val2,
                                    callee: Some(crate::mir::Callee::ModuleFunction(target_key.clone())),
                                    args: arg_values2,
                                    effects: EffectMask::READ.add(Effect::ReadHeap),
                                })?;
                                self.annotate_call_result_from_func_name(dst2, &target_key);
                                return Ok(dst2);
                            }
                        }
                    }
                }
                if dotted && module.functions.contains_key(&name) {
                    let dst = self.value_gen.next();
                    let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &name)?;
                    let mut arg_values = Vec::new();
                    for a in &args { arg_values.push(self.build_expression(a.clone())?); }
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst), func: fun_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(name.clone())),
                        args: arg_values,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &name);
                    return Ok(dst);
                }
                if dotted && module.functions.contains_key(&cand2) {
                    let dst = self.value_gen.next();
                    let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &cand2)?;
                    let mut arg_values = Vec::new();
                    for a in &args { arg_values.push(self.build_expression(a.clone())?); }
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst), func: fun_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(cand2.clone())),
                        args: arg_values,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &cand2);
                    return Ok(dst);
                }
                // Fallback: use shared CallResolver core when dotted but no exact match
                if dotted {
                    if let Some(fname) = crate::mir::resolve::call_resolver_core::resolve_module_function(
                        module.functions.keys().cloned(), &name, arity,
                    ) {
                        let dst2 = self.value_gen.next();
                        let fun_val2 = crate::mir::builder::name_const::make_name_const_result(self, &fname)?;
                        let mut arg_values2 = Vec::new();
                        for a in &args { arg_values2.push(self.build_expression(a.clone())?); }
                        self.emit_instruction(MirInstruction::Call {
                            dst: Some(dst2),
                            func: fun_val2,
                            callee: Some(crate::mir::Callee::ModuleFunction(fname.clone())),
                            args: arg_values2,
                            effects: EffectMask::READ.add(Effect::ReadHeap),
                        })?;
                        self.annotate_call_result_from_func_name(dst2, &fname);
                        return Ok(dst2);
                    }
                }
            }
        }

        // Build argument values first (needed for arity-aware fallback)
        let mut arg_values = Vec::new();
        for a in args {
            arg_values.push(self.build_expression(a)?);
        }
        
        // External (builtin) ModuleFunction index — always resolve dotted+arity names
        // even when not present in current module (String/Array/Map/Console families).
                // Direct extern mapping for Timer.now_ms (function-call form)
        if (name == "Timer.now_ms" || name == "TimerBox.now_ms") && arg_values.is_empty() {
            return self.emit_timer_now_ms_call();
        }
        if name == "Timer.now_ms/0" || name == "TimerBox.now_ms/0" {
            return self.emit_timer_now_ms_call();
        }

        if let Some(ext_name) = self.normalize_external_module_function_name(&name, arg_values.len()) {
            let dst = self.value_gen.next();
            let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &ext_name)?;
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,
                callee: Some(crate::mir::Callee::ModuleFunction(ext_name.clone())),
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            self.annotate_call_result_from_func_name(dst, &ext_name);
            return Ok(dst);
        }

        // General dotted fallback: treat any dotted name as a ModuleFunction even if
        // it is defined in an imported box. Normalize via CallNameResolver to append "/arity".
        if name.contains(".") {
            let target = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::normalize(&name, arg_values.len()) {
                Ok(full) => full,
                Err(_) => if name.contains('/') { name.clone() } else { format!("{}/{}", name, arg_values.len()) },
            };
            let dst = self.value_gen.next();
            let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &target)?;
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,
                callee: Some(crate::mir::Callee::ModuleFunction(target.clone())),
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            self.annotate_call_result_from_func_name(dst, &target);
            return Ok(dst);
        }

        if module_fn_unify {
            if let Some(ref module) = self.current_module {
                let arity = arg_values.len();
                // Accept canonical name provided or append /arity
                let cand1 = name.clone();
                let cand2 = format!("{}/{}", name, arity);
                let idx = crate::mir::indexes::functions::FunctionIndex::new(module);
                let mut chosen: Option<String> = if idx.contains(&cand1) { Some(cand1) } else if idx.contains(&cand2) { Some(cand2) } else { None };

                // Fallback (env-gated): unique tail match
                // - if dotted: prefer keys that start with the class prefix (Alias_ も許容) and end with ".method/arity"
                // - if bare: search unique "*.name/arity"
                if chosen.is_none() {
                    use crate::mir::indexes::functions::TailQueryResult;
                    // Strict by default: unless explicitly disabled, ambiguous tail matches are errors.
                    let strict = match std::env::var("NYASH_MIR_CALL_MODULE_FN_STRICT").ok().as_deref() {
                        Some("0") | Some("false") | Some("off") => false,
                        _ => true,
                    };
                    let tail_res = if let Some((cls, meth)) = name.split_once('.') {
                        idx.tail_unique(Some(cls), meth, arity)
                    } else {
                        idx.tail_unique(None, &name, arity)
                    };
                    match tail_res {
                        TailQueryResult::Unique(n) => chosen = Some(n),
                        TailQueryResult::Ambiguous(mut ambig_list) => {
                            if strict {
                                // Stable and informative ordering: prefer current box first, then lexical
                                let mut display = ambig_list.clone();
                                display.sort();
                                if let Some(cur_fn_name) = self.current_function.as_ref().map(|f| f.signature.name.clone()) {
                                    let preferred = crate::mir::indexes::functions::prefer_current_box(&cur_fn_name, &display);
                                    display = preferred;
                                }
                                let msg = crate::common::diagnostics::msg_ambiguous::module_function(
                                    &name, arity, &display, ambig_list.len()
                                );
                                return Err(msg);
                            } else {
                                // Apply common heuristic: prefer current box when it yields a single candidate
                                if let Some(cur_fn_name) = self.current_function.as_ref().map(|f| f.signature.name.clone()) {
                                    let filtered = crate::mir::indexes::functions::prefer_current_box(&cur_fn_name, &ambig_list);
                                    if filtered.len() == 1 {
                                        chosen = Some(filtered[0].clone());
                                    }
                                }
                            }
                        }
                        TailQueryResult::None => {}
                    }
                }
                if let Some(fname) = chosen {
                    let dst = self.value_gen.next();
                    // func field retained for legacy compatibility (diagnostics/SSA references)
                    let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &fname)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: fun_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(fname.clone())),
                        args: arg_values,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &fname);
                    return Ok(dst);
                }
            }
        }

        // Dev-only safety: inside a static box context, allow unqualified helper calls
        // like `_foo(x)` to be resolved as `Class._foo/arity(x)`.
        // This guards against accidental desugaring that strips `me.` or alias prefixes.
        if std::env::var("NYASH_DEV").ok().as_deref() == Some("1") {
            if let Some(cls_name) = self.current_static_box.clone() {
                if name.starts_with('_') && !name.contains('.') {
                    let result_id = self.value_gen.next();
                    let fun_name = format!("{}.{}{}", cls_name, name, format!("/{}", arg_values.len()));
                    let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &fun_name)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(result_id),
                        func: fun_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(fun_name.clone())),
                        args: arg_values,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(result_id, &fun_name);
                    return Ok(result_id);
                }
            }
        }

        // Special-case: global str(x) → x.str() に正規化（内部は関数へ統一される）
        if name == "str" && arg_values.len() == 1 {
            let dst = self.value_gen.next();
            // Use unified method emission; downstream rewrite will functionize as needed
            self.emit_method_call(Some(dst), arg_values[0], "str".to_string(), vec![])?;
            return Ok(dst);
        }

        // Phase 3.2: Unified call is default ON, but only use it for known builtins/externs.
        let use_unified = call_unified::is_unified_call_enabled()
            && (method_resolution::is_builtin_function(&name)
                || method_resolution::is_extern_function(&name));

        if !use_unified {
            // Legacy path（必要なら警告）
            let dst = self.value_gen.next();

            // === ChatGPT5 Pro Design: Type-safe function call resolution ===
            // Resolve call target using new type-safe system; if it fails, try static-method fallback
            let _callee = match self.resolve_call_target(&name) {
                Ok(c) => c,
                Err(_e) => {
                    // dev trace removed
                    // Fallback: if exactly one static method with this name and arity is known, call it.
                    if let Some(cands) = self.static_method_index.get(&name) {
                        let mut matches: Vec<(String, usize)> = cands
                            .iter()
                            .cloned()
                            .filter(|(_, ar)| *ar == arg_values.len())
                            .collect();
                        if matches.len() == 1 {
                            let (bx, _arity) = matches.remove(0);
                            let dst = self.value_gen.next();
                            let func_name = format!("{}.{}{}", bx, name, format!("/{}", arg_values.len()));
                            // Keep legacy emission for module-level function resolution
                            self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                            return Ok(dst);
                        }
                    } else if let Some(dot) = name.rfind('.') {
                        // Qualified input like Alias.Box.method → try method-name fallback
                        let method_only = &name[dot + 1..];
                        if let Some(cands2) = self.static_method_index.get(method_only) {
                            let mut matches: Vec<(String, usize)> = cands2
                                .iter()
                                .cloned()
                                .filter(|(_, ar)| *ar == arg_values.len())
                                .collect();
                            if matches.len() == 1 {
                                let (bx, _arity) = matches.remove(0);
                                let dst = self.value_gen.next();
                                let func_name = format!("{}.{}{}", bx, method_only, format!("/{}", arg_values.len()));
                                self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                                return Ok(dst);
                            }
                        }
                    }
                    // Secondary fallback (tail-based) via common call_policy (dev only)
                    // Enable only when explicitly requested: NYASH_BUILDER_TAIL_RESOLVE=1
                    if std::env::var("NYASH_BUILDER_TAIL_RESOLVE").ok().as_deref() == Some("1") {
                        if let Some(ref module) = self.current_module {
                            let (class_or_alias, method_only) = if let Some((cls, m)) = name.split_once('.') {
                                (cls, m)
                            } else { ("", name.as_str()) };
                            // wide=true when no qualifier; otherwise respect qualifier
                            let wide = class_or_alias.is_empty();
                            let cands = crate::common::call_policy::tail_candidates(
                                module.functions.keys(),
                                class_or_alias,
                                method_only.split('/').next().unwrap_or(method_only),
                                arg_values.len(),
                                wide,
                            );
                            if cands.len() == 1 {
                                let pick = cands[0].clone();
                                // Avoid immediate self-cycle just in case
                                if !crate::common::call_policy::is_immediate_cycle(&pick, &name) {
                                    let dst = self.value_gen.next();
                                    self.emit_legacy_call(Some(dst), CallTarget::Global(pick), arg_values)?;
                                    return Ok(dst);
                                }
                            }
                        }
                    }
                    // Propagate original error
                    return Err(format!("Unresolved function: '{}'. {}", name, method_resolution::suggest_resolution(&name)));
                }
            };

            // Legacy compatibility: Create dummy func value for old systems
            let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &name)?;

            // Emit strict Global callee to avoid legacy dynamic resolution
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,                  // Retained for diagnostics/SSA references
                callee: Some(crate::mir::Callee::Global(name.clone())),
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            Ok(dst)
        } else {
            // Unified path for builtins/externs
            let dst = self.value_gen.next();
            self.emit_unified_call(
                Some(dst),
                CallTarget::Global(name),
                arg_values,
            )?;
            Ok(dst)
        }
    }

    // Build method call: object.method(arguments)
    pub(in super::super) fn build_method_call(
        &mut self,
        object: ASTNode,
        method: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Guard: static box does not support field emulation via getField/setField on the box itself
        if let Some(cls) = self.current_static_box.clone() {
            let is_box_self = match &object {
                ASTNode::Variable { name, .. } => name == "me" || name == &cls || name == &format!("{}_{}", cls, cls),
                ASTNode::Me { .. } => true,
                ASTNode::Literal { value: crate::ast::LiteralValue::String(s), .. } => s == &cls,
                _ => false,
            };
            if is_box_self && (method == "setField" || method == "getField") {
                return Err("Static box fields are not supported: use methods or instance boxes (getField/setField)".to_string());
            }
        }
        if std::env::var("NYASH_STATIC_CALL_TRACE").ok().as_deref() == Some("1") {
            let kind = match &object {
                ASTNode::Variable { .. } => "Variable",
                ASTNode::FieldAccess { .. } => "FieldAccess",
                ASTNode::This { .. } => "This",
                ASTNode::Me { .. } => "Me",
                _ => "Other",
            };
            eprintln!("[builder] method-call object kind={} method={}", kind, method);
        }

        // 1. Static box method call: BoxName.method(args)
        if let ASTNode::Variable { name: obj_name, .. } = &object {
            let is_local_var = self.variable_map.contains_key(obj_name);
            // Phase 15.5: Treat unknown identifiers in receiver position as static type names
            if !is_local_var {
                if obj_name == "TimerBox" && method == "now_ms" && arguments.is_empty() {
                    return self.emit_timer_now_ms_call();
                }
                return self.handle_static_method_call(obj_name, &method, &arguments);
            }
        }

        // 2. Handle env.* methods
        if let Some(res) = self.try_handle_env_method(&object, &method, &arguments) {
            return res;
        }

        // 3. Handle me.method() calls
        if let ASTNode::Me { .. } = object {
            // 3-a) Static box fast path (already handled)
            if let Some(res) = self.handle_me_method_call(&method, &arguments)? {
                return Ok(res);
            }
            // 3-b) Instance box: prefer enclosing box method explicitly to avoid cross-box name collisions
            {
                // Capture enclosing class name without holding an active borrow
                let enclosing_cls: Option<String> = self
                    .current_function
                    .as_ref()
                    .and_then(|f| f.signature.name.split('.').next().map(|s| s.to_string()));
                    if let Some(cls) = enclosing_cls.as_ref() {
                    // Build arg values (avoid overlapping borrows by collecting first)
                    let built_args: Vec<ASTNode> = arguments.clone();
                    let mut arg_values = Vec::with_capacity(built_args.len());
                    for a in built_args.into_iter() { arg_values.push(self.build_expression(a)?); }
                    let arity = arg_values.len();
                    let fname = crate::mir::builder::calls::function_lowering::generate_method_function_name(cls, &method, arity);
                    let exists = if let Some(ref module) = self.current_module { module.functions.contains_key(&fname) } else { false };
                    if exists {
                        // Pass 'me' as first arg
                        let me_id = self.build_me_expression()?;
                        let mut call_args = Vec::with_capacity(arity + 1);
                        call_args.push(me_id);
                        call_args.extend(arg_values.into_iter());
                        let dst = self.value_gen.next();
                        // Emit function name via NameConstBox
                        let c = match crate::mir::builder::name_const::make_name_const_result(self, &fname) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };
                        // Prefer ModuleFunction callee when enabled; fallback to legacy (callee=None)
                        self.emit_instruction(MirInstruction::Call {
                                dst: Some(dst),
                                func: c,
                                callee: Some(crate::mir::Callee::ModuleFunction(fname.clone())),
                                args: call_args,
                                effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                            })?;
                        self.annotate_call_result_from_func_name(dst, &fname);
                        return Ok(dst);
                    }
                }
            }
        }

        // 4. Build object value for remaining cases
        let object_value = self.build_expression(object)?;

        let router = crate::mir::builder::router::call_router::CallRoutingBox::new();
        if let Some(route) = router.decide_method_route(self.origin_get(object_value), &method, arguments.len()) {
            match route {
                crate::mir::builder::router::call_router::CallRoute::DirectExtern { iface, method: ref m } => {
                    if iface == "nyrt.time" && *m == "now_ms" && arguments.is_empty() {
                        return self.emit_timer_now_ms_call();
                    }
                    if iface == "nyrt.array" && *m == "size" && arguments.is_empty() {
                        return self.emit_array_size_call(object_value);
                    }
                    if iface == "nyrt.map" && *m == "size" && arguments.is_empty() {
                        return self.emit_map_size_call(object_value);
                    }
                }
            }
        }

        if method == "now_ms" && arguments.is_empty() {
            // Prefer direct extern when receiver is TimerBox (origin known)
            if self
                .origin_get(object_value)
                .map(|name| name == "TimerBox")
                .unwrap_or(false)
            {
                return self.emit_timer_now_ms_call();
            }
            // Fallback: for unknown receiver origins, route now_ms to extern timer
            // This keeps semantics stable and avoids stub returns from script fallback.
            return self.emit_timer_now_ms_call();
        }

        // 5. Handle TypeOp methods: value.is("Type") / value.as("Type")
        // Note: This was duplicated in original code - now unified!
        if let Some(type_name) = special_handlers::is_typeop_method(&method, &arguments) {
            return self.handle_typeop_method(object_value, &method, &type_name);
        }

        // 6. Fallback: standard Box/Plugin method call
        self.handle_standard_method_call(object_value, method, &arguments)
    }

    fn emit_timer_now_ms_call(&mut self) -> Result<ValueId, String> {
        let dst = self.value_gen.next();
        let mut args: Vec<ValueId> = vec![];
        crate::mir::builder::ssa::local::finalize_args(self, &mut args);
        self.emit_unified_call(
            Some(dst),
            super::super::builder_calls::CallTarget::Extern("nyrt.time.now_ms".to_string()),
            args,
        )?;
        Ok(dst)
    }

    fn emit_array_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
        let recv_local = self.local_recv(receiver);
        let dst = self.value_gen.next();
        self.emit_unified_call(
            Some(dst),
            super::super::builder_calls::CallTarget::Extern("nyrt.array.size".to_string()),
            vec![recv_local],
        )?;
        self.value_types.insert(dst, MirType::Integer);
        Ok(dst)
    }

    fn emit_map_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
        let recv_local = self.local_recv(receiver);
        let dst = self.value_gen.next();
        self.emit_unified_call(
            Some(dst),
            super::super::builder_calls::CallTarget::Extern("nyrt.map.size".to_string()),
            vec![recv_local],
        )?;
        self.value_types.insert(dst, MirType::Integer);
        Ok(dst)
    }
}


#[cfg(test)]
mod tests {
    use super::super::super::MirBuilder;

    #[test]
    fn normalize_external_module_function_name_basic() {
        let b = MirBuilder::new();
        // String.len/1 → StringBox.length/0
        assert_eq!(
            b.normalize_external_module_function_name("String.len/1", 1),
            Some("StringBox.length/0".to_string())
        );
        // Array.get/2 → ArrayBox.get/1
        assert_eq!(
            b.normalize_external_module_function_name("Array.get/2", 2),
            Some("ArrayBox.get/1".to_string())
        );
        // Map.set (arity from argc=3) → MapBox.set/2
        assert_eq!(
            b.normalize_external_module_function_name("Map.set", 3),
            Some("MapBox.set/2".to_string())
        );
        // Already canonical with Box suffix stays canonical (arity inferred)
        assert_eq!(
            b.normalize_external_module_function_name("ConsoleBox.println/1", 1),
            Some("ConsoleBox.println/0".to_string())
        );
        // Non-dotted names return None
        assert_eq!(b.normalize_external_module_function_name("print", 1), None);
    }
}
