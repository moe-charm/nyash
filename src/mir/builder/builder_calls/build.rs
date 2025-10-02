// Build functions for function and method calls
use super::super::{Effect, EffectMask, MirBuilder, MirInstruction, MirType, ValueId};
use crate::ast::ASTNode;
use crate::mir::builder::calls::{call_unified, method_resolution, special_handlers};
use crate::mir::builder::calls::call_target::CallTarget;
use crate::mir::TypeOpKind;

impl MirBuilder {
    // === ChatGPT5 Pro Design: Type-safe Call Resolution System ===

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
        name: String,
        args: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
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
        let module_fn_unify = std::env::var("NYASH_MIR_CALL_MODULE_FN").ok().as_deref() == Some("1");
        // Canonical safe step（envガード・既定OFF）: dotted+arity の完全一致は ModuleFunction を許可
        let module_fn_canon = std::env::var("NYASH_MIR_CALL_MODULE_FN_CANON").ok().as_deref() == Some("1");
        if module_fn_canon {
            if let Some(ref module) = self.current_module {
                let arity = args.len();
                let cand2 = format!("{}/{}", name, arity);
                let dotted = name.contains('.') && name.contains('/');
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
            }
        }

        // Build argument values first (needed for arity-aware fallback)
        let mut arg_values = Vec::new();
        for a in args {
            arg_values.push(self.build_expression(a)?);
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
                    let tail_res = if let Some((cls, meth)) = name.split_once('.') {
                        idx.tail_unique(Some(cls), meth, arity)
                    } else {
                        idx.tail_unique(None, &name, arity)
                    };
                    match tail_res {
                        TailQueryResult::Unique(n) => chosen = Some(n),
                        TailQueryResult::Ambiguous(mut ambig_list) => {
                            if std::env::var("NYASH_MIR_CALL_MODULE_FN_STRICT").ok().as_deref() == Some("1") {
                                ambig_list.sort();
                                let shown: usize = ambig_list.len().min(10);
                                let mut msg = format!(
                                    "Ambiguous module function resolution for '{}', arity={} ({} candidates, showing {}):\n",
                                    name, arity, ambig_list.len(), shown
                                );
                                for k in ambig_list.iter().take(shown) { msg.push_str("  - "); msg.push_str(k); msg.push('\n'); }
                                if ambig_list.len() > shown { msg.push_str(&format!("  ... and {} more\n", ambig_list.len() - shown)); }
                                msg.push_str("Hint: qualify with Class.method/Arity, or set NYASH_MIR_CALL_MODULE_FN_STRICT=0 to fallback.");
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
                        callee: None,
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
            && (super::super::call_resolution::is_builtin_function(&name)
                || super::super::call_resolution::is_extern_function(&name));

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
                    // Secondary fallback (tail-based) is disabled by default to avoid ambiguous resolution.
                    // Enable only when explicitly requested: NYASH_BUILDER_TAIL_RESOLVE=1
                    if std::env::var("NYASH_BUILDER_TAIL_RESOLVE").ok().as_deref() == Some("1") {
                        if let Some(ref module) = self.current_module {
                            let tail = format!(".{}{}", name, format!("/{}", arg_values.len()));
                            let mut cands: Vec<String> = module
                                .functions
                                .keys()
                                .filter(|k| k.ends_with(&tail))
                                .cloned()
                                .collect();
                            if cands.len() == 1 {
                                let func_name = cands.remove(0);
                                let dst = self.value_gen.next();
                                self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                                return Ok(dst);
                            }
                        }
                    }
                    // Propagate original error
                    return Err(format!("Unresolved function: '{}'. {}", name, super::super::call_resolution::suggest_resolution(&name)));
                }
            };

            // Legacy compatibility: Create dummy func value for old systems
            let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &name)?;

            // Emit legacy-compatible Call (do not set callee to keep module/global resolution stable)
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,                  // Name-based resolution via legacy path
                callee: None,
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            if std::env::var("NYASH_WARN_LEGACY_CALL").ok().as_deref() == Some("1") {
                eprintln!("[legacy-call] Global('{}') emitted without callee (enable NYASH_MIR_CALL_MODULE_FN* to unify)", name);
            }
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
                        let use_modfn = std::env::var("NYASH_MIR_CALL_MODULE_FN").ok().as_deref() == Some("1");
                        if use_modfn {
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dst),
                                func: c,
                                callee: Some(crate::mir::Callee::ModuleFunction(fname.clone())),
                                args: call_args,
                                effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                            })?;
                        } else {
                            // Legacy path: do NOT set callee to keep legacy NameConst resolution
                            self.emit_instruction(MirInstruction::Call {
                                dst: Some(dst),
                                func: c,
                                callee: None,
                                args: call_args,
                                effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                            })?;
                            if std::env::var("NYASH_WARN_LEGACY_CALL").ok().as_deref() == Some("1") {
                                eprintln!("[legacy-call] Method('{}' via me) lowered to Global('{}') without callee", method, fname);
                            }
                        }
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
            if self
                .origin_get(object_value)
                .map(|name| name == "TimerBox")
                .unwrap_or(false)
            {
                return self.emit_timer_now_ms_call();
            }
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
        self.emit_instruction(MirInstruction::ExternCall {
            dst: Some(dst),
            iface_name: "nyrt.time".to_string(),
            method_name: "now_ms".to_string(),
            args: vec![],
            effects: EffectMask::READ,
        })?;
        Ok(dst)
    }

    fn emit_array_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
        let recv_local = self.local_recv(receiver);
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::ExternCall {
            dst: Some(dst),
            iface_name: "nyrt.array".to_string(),
            method_name: "size".to_string(),
            args: vec![recv_local],
            effects: EffectMask::READ,
        })?;
        self.value_types.insert(dst, MirType::Integer);
        Ok(dst)
    }

    fn emit_map_size_call(&mut self, receiver: ValueId) -> Result<ValueId, String> {
        let recv_local = self.local_recv(receiver);
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::ExternCall {
            dst: Some(dst),
            iface_name: "nyrt.map".to_string(),
            method_name: "size".to_string(),
            args: vec![recv_local],
            effects: EffectMask::READ,
        })?;
        self.value_types.insert(dst, MirType::Integer);
        Ok(dst)
    }
}
