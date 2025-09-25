// Extracted call-related builders from builder.rs to keep files lean
use super::{Effect, EffectMask, FunctionSignature, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue, MethodCallExpr};
use crate::mir::definitions::call_unified::{Callee, CallFlags, MirCall};
use crate::mir::TypeOpKind;
use super::call_resolution;

// Import from new modules
use super::calls::*;
pub use super::calls::call_target::CallTarget;

impl super::MirBuilder {
    /// Unified call emission - replaces all emit_*_call methods
    /// ChatGPT5 Pro A++ design for complete call unification
    pub fn emit_unified_call(
        &mut self,
        dst: Option<ValueId>,
        target: CallTarget,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        // Check environment variable for unified call usage
        if !call_unified::is_unified_call_enabled() {
            // Fall back to legacy implementation
            return self.emit_legacy_call(dst, target, args);
        }

        // Convert CallTarget to Callee using the new module
        let callee = call_unified::convert_target_to_callee(
            target,
            &self.value_origin_newbox,
            &self.value_types,
        )?;

        // Validate call arguments
        call_unified::validate_call_args(&callee, &args)?;

        // Create MirCall instruction using the new module
        let mir_call = call_unified::create_mir_call(dst, callee.clone(), args.clone());

        // For Phase 2: Convert to legacy Call instruction with new callee field
        let legacy_call = MirInstruction::Call {
            dst: mir_call.dst,
            func: ValueId::new(0), // Dummy value for legacy compatibility
            callee: Some(mir_call.callee),
            args: mir_call.args,
            effects: mir_call.effects,
        };

        self.emit_instruction(legacy_call)
    }

    /// Legacy call fallback - preserves existing behavior
    pub(super) fn emit_legacy_call(
        &mut self,
        dst: Option<ValueId>,
        target: CallTarget,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        match target {
            CallTarget::Method { receiver, method, .. } => {
                // Use existing emit_box_or_plugin_call
                self.emit_box_or_plugin_call(dst, receiver, method, None, args, EffectMask::IO)
            },
            CallTarget::Constructor(box_type) => {
                // Use existing NewBox
                let dst = dst.ok_or("Constructor must have destination")?;
                self.emit_instruction(MirInstruction::NewBox {
                    dst,
                    box_type,
                    args,
                })
            },
            CallTarget::Extern(name) => {
                // Use existing ExternCall
                let parts: Vec<&str> = name.splitn(2, '.').collect();
                let (iface, method) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    ("nyash".to_string(), name)
                };

                self.emit_instruction(MirInstruction::ExternCall {
                    dst,
                    iface_name: iface,
                    method_name: method,
                    args,
                    effects: EffectMask::IO,
                })
            },
            CallTarget::Global(name) => {
                // Create a string constant for the function name
                let name_const = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: name_const,
                    value: super::ConstValue::String(name),
                })?;

                self.emit_instruction(MirInstruction::Call {
                    dst,
                    func: name_const,
                    callee: None, // Legacy mode
                    args,
                    effects: EffectMask::IO,
                })
            },
            CallTarget::Value(func_val) => {
                self.emit_instruction(MirInstruction::Call {
                    dst,
                    func: func_val,
                    callee: None, // Legacy mode
                    args,
                    effects: EffectMask::IO,
                })
            },
            CallTarget::Closure { params, captures, me_capture } => {
                let dst = dst.ok_or("Closure creation must have destination")?;
                self.emit_instruction(MirInstruction::NewClosure {
                    dst,
                    params,
                    body: vec![], // Empty body for now
                    captures,
                    me: me_capture,
                })
            },
        }
    }

    // Phase 2 Migration: Convenience methods that use emit_unified_call

    /// Emit a global function call (print, panic, etc.)
    pub fn emit_global_call(
        &mut self,
        dst: Option<ValueId>,
        name: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(dst, CallTarget::Global(name), args)
    }

    /// Emit a method call (box.method)
    pub fn emit_method_call(
        &mut self,
        dst: Option<ValueId>,
        receiver: ValueId,
        method: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(
            dst,
            CallTarget::Method {
                box_type: None, // Auto-infer
                method,
                receiver,
            },
            args,
        )
    }

    /// Emit a constructor call (new BoxType)
    pub fn emit_constructor_call(
        &mut self,
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
    ) -> Result<(), String> {
        self.emit_unified_call(
            Some(dst),
            CallTarget::Constructor(box_type),
            args,
        )
    }

    /// Try handle math.* function in function-style (sin/cos/abs/min/max).
    /// Returns Some(result) if handled, otherwise None.
    fn try_handle_math_function(
        &mut self,
        name: &str,
        raw_args: Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        if !special_handlers::is_math_function(name) {
            return None;
        }
        // Build numeric args directly for math.* to preserve f64 typing
        let mut math_args: Vec<ValueId> = Vec::new();
        for a in raw_args.into_iter() {
            match a {
                ASTNode::New { class, arguments, .. } if class == "FloatBox" && arguments.len() == 1 => {
                    match self.build_expression(arguments[0].clone()) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                ASTNode::New { class, arguments, .. } if class == "IntegerBox" && arguments.len() == 1 => {
                    let iv = match self.build_expression(arguments[0].clone()) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let fv = self.value_gen.next();
                    if let Err(e) = self.emit_instruction(MirInstruction::TypeOp { dst: fv, op: TypeOpKind::Cast, value: iv, ty: MirType::Float }) { return Some(Err(e)); }
                    math_args.push(fv);
                }
                ASTNode::Literal { value: LiteralValue::Float(_), .. } => {
                    match self.build_expression(a) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                other => {
                    match self.build_expression(other) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
            }
        }
        // new MathBox()
        let math_recv = self.value_gen.next();
        if let Err(e) = self.emit_constructor_call(math_recv, "MathBox".to_string(), vec![]) { return Some(Err(e)); }
        self.value_origin_newbox.insert(math_recv, "MathBox".to_string());
        // birth()
        if let Err(e) = self.emit_method_call(None, math_recv, "birth".to_string(), vec![]) { return Some(Err(e)); }
        // call method
        let dst = self.value_gen.next();
        if let Err(e) = self.emit_method_call(Some(dst), math_recv, name.to_string(), math_args) { return Some(Err(e)); }
        Some(Ok(dst))
    }

    /// Try handle env.* extern methods like env.console.log via FieldAccess(object, field).
    fn try_handle_env_method(
        &mut self,
        object: &ASTNode,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let ASTNode::FieldAccess { object: env_obj, field: env_field, .. } = object else { return None; };
        if let ASTNode::Variable { name: env_name, .. } = env_obj.as_ref() {
            if env_name != "env" { return None; }
            // Build arguments once
            let mut arg_values = Vec::new();
            for arg in arguments {
                match self.build_expression(arg.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
            }
            let iface = env_field.as_str();
            let m = method;
            let mut extern_call = |iface_name: &str, method_name: &str, effects: EffectMask, returns: bool| -> Result<ValueId, String> {
                let result_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::ExternCall { dst: if returns { Some(result_id) } else { None }, iface_name: iface_name.to_string(), method_name: method_name.to_string(), args: arg_values.clone(), effects })?;
                if returns {
                    Ok(result_id)
                } else {
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const { dst: void_id, value: super::ConstValue::Void })?;
                    Ok(void_id)
                }
            };
            // Use the new module for env method spec
            if let Some((iface_name, method_name, effects, returns)) =
                extern_calls::get_env_method_spec(iface, m)
            {
                return Some(extern_call(&iface_name, &method_name, effects, returns));
            }
            return None;
        }
        None
    }

    /// Try direct static call for `me` in static box
    pub(super) fn try_handle_me_direct_call(
        &mut self,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let Some(cls_name) = self.current_static_box.clone() else { return None; };
        // Build args
        let mut arg_values = Vec::new();
        for a in arguments {
            match self.build_expression(a.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
        }
        let result_id = self.value_gen.next();
        let fun_name = format!("{}.{}{}", cls_name, method, format!("/{}", arg_values.len()));
        let fun_val = self.value_gen.next();
        if let Err(e) = self.emit_instruction(MirInstruction::Const { dst: fun_val, value: super::ConstValue::String(fun_name) }) { return Some(Err(e)); }
        if let Err(e) = self.emit_instruction(MirInstruction::Call {
            dst: Some(result_id),
            func: fun_val,
            callee: None, // Legacy math function - use old resolution
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap)
        }) { return Some(Err(e)); }
        Some(Ok(result_id))
    }

    // === ChatGPT5 Pro Design: Type-safe Call Resolution System ===

    /// Resolve function call target to type-safe Callee
    /// Implements the core logic of compile-time function resolution
    fn resolve_call_target(&self, name: &str) -> Result<super::super::Callee, String> {
        method_resolution::resolve_call_target(
            name,
            &self.current_static_box,
            &self.variable_map,
        )
    }

    // Build function call: name(args)
    pub(super) fn build_function_call(
        &mut self,
        name: String,
        args: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
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

        // Build argument values first (needed for arity-aware fallback)
        let mut arg_values = Vec::new();
        for a in args {
            arg_values.push(self.build_expression(a)?);
        }

        // Phase 3.2: Use unified call for basic functions like print
        let use_unified = std::env::var("NYASH_MIR_UNIFIED_CALL").unwrap_or_default() == "1";

        if use_unified {
            // New unified path - use emit_unified_call with Global target
            let dst = self.value_gen.next();
            self.emit_unified_call(
                Some(dst),
                CallTarget::Global(name),
                arg_values,
            )?;
            Ok(dst)
        } else {
            // Legacy path
            let dst = self.value_gen.next();

            // === ChatGPT5 Pro Design: Type-safe function call resolution ===
            // Resolve call target using new type-safe system; if it fails, try static-method fallback
            let callee = match self.resolve_call_target(&name) {
                Ok(c) => c,
                Err(_e) => {
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
                            // Emit legacy global call to the lowered static method function
                            self.emit_legacy_call(Some(dst), CallTarget::Global(func_name), arg_values)?;
                            return Ok(dst);
                        }
                    }
                    // Secondary fallback: search already-materialized functions in the current module
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
                    // Propagate original error
                    return Err(format!("Unresolved function: '{}'. {}", name, super::call_resolution::suggest_resolution(&name)));
                }
            };

            // Legacy compatibility: Create dummy func value for old systems
            let fun_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: fun_val,
                value: super::ConstValue::String(name.clone()),
            })?;

            // Emit new-style Call with type-safe callee
            self.emit_instruction(MirInstruction::Call {
                dst: Some(dst),
                func: fun_val,                  // Legacy compatibility
                callee: Some(callee),           // New type-safe resolution
                args: arg_values,
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            Ok(dst)
        }
    }

    // Build method call: object.method(arguments)
    pub(super) fn build_method_call(
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
                return self.handle_static_method_call(obj_name, &method, &arguments);
            }
        }

        // 2. Handle env.* methods
        if let Some(res) = self.try_handle_env_method(&object, &method, &arguments) {
            return res;
        }

        // 3. Handle me.method() calls
        if let ASTNode::Me { .. } = object {
            if let Some(res) = self.handle_me_method_call(&method, &arguments)? {
                return Ok(res);
            }
        }

        // 4. Build object value for remaining cases
        let object_value = self.build_expression(object)?;

        // 5. Handle TypeOp methods: value.is("Type") / value.as("Type")
        // Note: This was duplicated in original code - now unified!
        if let Some(type_name) = special_handlers::is_typeop_method(&method, &arguments) {
            return self.handle_typeop_method(object_value, &method, &type_name);
        }

        // 6. Fallback: standard Box/Plugin method call
        self.handle_standard_method_call(object_value, method, &arguments)
    }

    // Map a user-facing type name to MIR type
    pub(super) fn parse_type_name_to_mir(name: &str) -> super::MirType {
        special_handlers::parse_type_name_to_mir(name)
    }

    // Extract string literal from AST node if possible
    pub(super) fn extract_string_literal(node: &ASTNode) -> Option<String> {
        special_handlers::extract_string_literal(node)
    }

    // Build from expression: from Parent.method(arguments)
    pub(super) fn build_from_expression(
        &mut self,
        parent: String,
        method: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg)?);
        }
        let parent_value = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: parent_value,
            value: super::ConstValue::String(parent),
        })?;
        let result_id = self.value_gen.next();
        self.emit_box_or_plugin_call(
            Some(result_id),
            parent_value,
            method,
            None,
            arg_values,
            EffectMask::READ.add(Effect::ReadHeap),
        )?;
        Ok(result_id)
    }

    // Lower a box method into a standalone MIR function (with `me` parameter)
    pub(super) fn lower_method_as_function(
        &mut self,
        func_name: String,
        box_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        let signature = function_lowering::prepare_method_signature(
            func_name,
            &box_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = super::MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            let me_id = self.value_gen.next();
            f.params.push(me_id);
            self.variable_map.insert("me".to_string(), me_id);
            self.value_origin_newbox.insert(me_id, box_name.clone());
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value && !self.is_current_block_terminated() {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: super::ConstValue::Void,
            })?;
            self.emit_instruction(MirInstruction::Return {
                value: Some(void_val),
            })?;
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized_function = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized_function);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_gen = saved_value_gen;
        Ok(())
    }

    // Lower a static method body into a standalone MIR function (no `me` parameter)
    pub(super) fn lower_static_method_as_function(
        &mut self,
        func_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        // Derive static box context from function name prefix, e.g., "BoxName.method/N"
        let saved_static_ctx = self.current_static_box.clone();
        if let Some(pos) = func_name.find('.') {
            let box_name = &func_name[..pos];
            if !box_name.is_empty() {
                self.current_static_box = Some(box_name.to_string());
            }
        }
        let signature = function_lowering::prepare_static_method_signature(
            func_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = super::MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value {
            if let Some(ref mut f) = self.current_function {
                if let Some(block) = f.get_block(self.current_block.unwrap()) {
                    if !block.is_terminated() {
                        let void_val = self.value_gen.next();
                        self.emit_instruction(MirInstruction::Const {
                            dst: void_val,
                            value: super::ConstValue::Void,
                        })?;
                        self.emit_instruction(MirInstruction::Return {
                            value: Some(void_val),
                        })?;
                    }
                }
            }
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_gen = saved_value_gen;
        // Restore static box context
        self.current_static_box = saved_static_ctx;
        Ok(())
    }
}
