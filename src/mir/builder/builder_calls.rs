// Extracted call-related builders from builder.rs to keep files lean
use super::{Effect, EffectMask, FunctionSignature, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue, MethodCallExpr};
use crate::mir::definitions::call_unified::{Callee, CallFlags, MirCall};
use crate::mir::definitions::call_unified::migration;
use super::call_resolution;

fn contains_value_return(nodes: &[ASTNode]) -> bool {
    fn node_has_value_return(node: &ASTNode) -> bool {
        match node {
            ASTNode::Return { value: Some(_), .. } => true,
            ASTNode::If { then_body, else_body, .. } => {
                contains_value_return(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| contains_value_return(body))
            }
            ASTNode::Loop { body, .. } => contains_value_return(body),
            ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                ..
            } => {
                contains_value_return(try_body)
                    || catch_clauses
                        .iter()
                        .any(|clause| contains_value_return(&clause.body))
                    || finally_body
                        .as_ref()
                        .map_or(false, |body| contains_value_return(body))
            }
            ASTNode::Program { statements, .. } => contains_value_return(statements),
            ASTNode::ScopeBox { body, .. } => contains_value_return(body),
            ASTNode::FunctionDeclaration { body, .. } => contains_value_return(body),
            _ => false,
        }
    }

    nodes.iter().any(node_has_value_return)
}
use crate::mir::{slot_registry, TypeOpKind};

/// Call target specification for emit_unified_call
/// Provides type-safe target resolution at the builder level
#[derive(Debug, Clone)]
pub enum CallTarget {
    /// Global function (print, panic, etc.)
    Global(String),
    /// Method call (box.method)
    Method {
        box_type: Option<String>,  // None = infer from value
        method: String,
        receiver: ValueId,
    },
    /// Constructor (new BoxType)
    Constructor(String),
    /// External function (nyash.*)
    Extern(String),
    /// Dynamic function value
    Value(ValueId),
    /// Closure creation
    Closure {
        params: Vec<String>,
        captures: Vec<(String, ValueId)>,
        me_capture: Option<ValueId>,
    },
}

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
        let use_unified = std::env::var("NYASH_MIR_UNIFIED_CALL")
            .unwrap_or_else(|_| "0".to_string()) != "0";

        if !use_unified {
            // Fall back to legacy implementation
            return self.emit_legacy_call(dst, target, args);
        }

        // Convert CallTarget to Callee
        let callee = match target {
            CallTarget::Global(name) => {
                // Check if it's a built-in function
                if call_resolution::is_builtin_function(&name) {
                    Callee::Global(name)
                } else if call_resolution::is_extern_function(&name) {
                    Callee::Extern(name)
                } else {
                    return Err(format!("Unknown global function: {}", name));
                }
            },
            CallTarget::Method { box_type, method, receiver } => {
                let inferred_box_type = box_type.unwrap_or_else(|| {
                    // Try to infer box type from value origin or type annotation
                    self.value_origin_newbox.get(&receiver)
                        .cloned()
                        .or_else(|| {
                            self.value_types.get(&receiver)
                                .and_then(|t| match t {
                                    MirType::Box(box_name) => Some(box_name.clone()),
                                    _ => None,
                                })
                        })
                        .unwrap_or_else(|| "UnknownBox".to_string())
                });

                Callee::Method {
                    box_name: inferred_box_type,
                    method,
                    receiver: Some(receiver),
                }
            },
            CallTarget::Constructor(box_type) => {
                Callee::Constructor { box_type }
            },
            CallTarget::Extern(name) => {
                Callee::Extern(name)
            },
            CallTarget::Value(func_val) => {
                Callee::Value(func_val)
            },
            CallTarget::Closure { params, captures, me_capture } => {
                Callee::Closure { params, captures, me_capture }
            },
        };

        // Create MirCall instruction
        let effects = self.compute_call_effects(&callee);
        let flags = if callee.is_constructor() {
            CallFlags::constructor()
        } else {
            CallFlags::new()
        };

        let mir_call = MirCall {
            dst,
            callee,
            args,
            flags,
            effects,
        };

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
    fn emit_legacy_call(
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

    /// Compute effects for a call based on its callee
    fn compute_call_effects(&self, callee: &Callee) -> EffectMask {
        match callee {
            Callee::Global(name) => {
                match name.as_str() {
                    "print" | "error" => EffectMask::IO,
                    "panic" | "exit" => EffectMask::IO.add(Effect::Control),
                    _ => EffectMask::IO,
                }
            },
            Callee::Method { method, .. } => {
                match method.as_str() {
                    "birth" => EffectMask::PURE.add(Effect::Alloc),
                    _ => EffectMask::READ,
                }
            },
            Callee::Constructor { .. } => EffectMask::PURE.add(Effect::Alloc),
            Callee::Closure { .. } => EffectMask::PURE.add(Effect::Alloc),
            Callee::Extern(_) => EffectMask::IO,
            Callee::Value(_) => EffectMask::IO, // Conservative
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
        let is_math_func = matches!(name, "sin" | "cos" | "abs" | "min" | "max");
        if !is_math_func {
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
        let birt_mid = slot_registry::resolve_slot_by_type_name("MathBox", "birth");
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
            if let Some((iface_name, method_name, effects, returns)) =
                Self::get_env_method_spec(iface, m)
            {
                return Some(extern_call(&iface_name, &method_name, effects, returns));
            }
            return None;
        }
        None
    }

    /// Table-like spec for env.* methods. Returns iface_name, method_name, effects, returns.
    fn get_env_method_spec(
        iface: &str,
        method: &str,
    ) -> Option<(String, String, EffectMask, bool)> {
        // This match is the table. Keep it small and explicit.
        match (iface, method) {
            ("future", "delay") => Some(("env.future".to_string(), "delay".to_string(), EffectMask::READ.add(Effect::Io), true)),
            ("task", "currentToken") => Some(("env.task".to_string(), "currentToken".to_string(), EffectMask::READ, true)),
            ("task", "cancelCurrent") => Some(("env.task".to_string(), "cancelCurrent".to_string(), EffectMask::IO, false)),
            ("console", "log") => Some(("env.console".to_string(), "log".to_string(), EffectMask::IO, false)),
            ("console", "readLine") => Some(("env.console".to_string(), "readLine".to_string(), EffectMask::IO, true)),
            ("canvas", m) if matches!(m, "fillRect" | "fillText") => Some(("env.canvas".to_string(), method.to_string(), EffectMask::IO, false)),
            _ => None,
        }
    }

    /// Try direct static call for `me` in static box
    fn try_handle_me_direct_call(
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
        // 1. Check for built-in/global functions first
        if self.is_builtin_function(name) {
            return Ok(super::super::Callee::Global(name.to_string()));
        }

        // 2. Check for static box method in current context
        if let Some(box_name) = &self.current_static_box {
            if self.has_method(box_name, name) {
                // Warn about potential self-recursion using external helper
                if super::call_resolution::is_commonly_shadowed_method(name) {
                    eprintln!("{}", super::call_resolution::generate_self_recursion_warning(box_name, name));
                }

                return Ok(super::super::Callee::Method {
                    box_name: box_name.clone(),
                    method: name.to_string(),
                    receiver: None, // Static method call
                });
            }
        }

        // 3. Check for local variable containing function value
        if self.variable_map.contains_key(name) {
            let value_id = self.variable_map[name];
            return Ok(super::super::Callee::Value(value_id));
        }

        // 4. Check for external/host functions
        if self.is_extern_function(name) {
            return Ok(super::super::Callee::Extern(name.to_string()));
        }

        // 5. Resolution failed - this prevents runtime string-based resolution
        Err(format!("Unresolved function: '{}'. {}", name, super::call_resolution::suggest_resolution(name)))
    }

    /// Check if function name is a built-in global function
    fn is_builtin_function(&self, name: &str) -> bool {
        super::call_resolution::is_builtin_function(name)
    }

    /// Check if current static box has the specified method
    fn has_method(&self, box_name: &str, method: &str) -> bool {
        // TODO: Implement proper method registry lookup
        // For now, use simple heuristics for common cases
        match box_name {
            "ConsoleStd" => matches!(method, "print" | "println" | "log"),
            "StringBox" => matches!(method, "upper" | "lower" | "length"),
            _ => false, // Conservative: assume no method unless explicitly known
        }
    }

    /// Check if function name is an external/host function
    fn is_extern_function(&self, name: &str) -> bool {
        super::call_resolution::is_extern_function(name)
    }

    // Build function call: name(args)
    pub(super) fn build_function_call(
        &mut self,
        name: String,
        args: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Minimal TypeOp wiring via function-style: isType(value, "Type"), asType(value, "Type")
        if (name == "isType" || name == "asType") && args.len() == 2 {
            if let Some(type_name) = Self::extract_string_literal(&args[1]) {
                let val = self.build_expression(args[0].clone())?;
                let ty = Self::parse_type_name_to_mir(&type_name);
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

        // Build argument values
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

            // Resolve call target using new type-safe system
            let callee = self.resolve_call_target(&name)?;

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
        // Minimal TypeOp wiring via method-style syntax: value.is("Type") / value.as("Type")
        if (method == "is" || method == "as") && arguments.len() == 1 {
            if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                let object_value = self.build_expression(object.clone())?;
                let mir_ty = Self::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if method == "is" {
                    TypeOpKind::Check
                } else {
                    TypeOpKind::Cast
                };
                self.emit_instruction(MirInstruction::TypeOp {
                    dst,
                    op,
                    value: object_value,
                    ty: mir_ty,
                })?;
                return Ok(dst);
            }
        }
        if let Some(res) = self.try_handle_env_method(&object, &method, &arguments) { return res; }
        // If object is `me` within a static box, lower to direct Call: BoxName.method/N
        if let ASTNode::Me { .. } = object {
            if let Some(res) = self.try_handle_me_direct_call(&method, &arguments) { return res; }
        }
        // Build the object expression (wrapper allows simple access if needed in future)
        let _mc = MethodCallExpr { object: Box::new(object.clone()), method: method.clone(), arguments: arguments.clone(), span: crate::ast::Span::unknown() };
        let object_value = self.build_expression(object.clone())?;
        // Secondary interception for is/as
        if (method == "is" || method == "as") && arguments.len() == 1 {
            if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                let mir_ty = Self::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if method == "is" {
                    TypeOpKind::Check
                } else {
                    TypeOpKind::Cast
                };
                self.emit_instruction(MirInstruction::TypeOp {
                    dst,
                    op,
                    value: object_value,
                    ty: mir_ty,
                })?;
                return Ok(dst);
            }
        }
        // Fallback: generic plugin invoke
        let mut arg_values: Vec<ValueId> = Vec::new();
        for a in &arguments {
            arg_values.push(self.build_expression(a.clone())?);
        }
        let result_id = self.value_gen.next();
        self.emit_box_or_plugin_call(
            Some(result_id),
            object_value,
            method,
            None,
            arg_values,
            EffectMask::READ.add(Effect::ReadHeap),
        )?;
        Ok(result_id)
    }

    // Map a user-facing type name to MIR type
    pub(super) fn parse_type_name_to_mir(name: &str) -> super::MirType {
        match name {
            // Primitive families
            "Integer" | "Int" | "I64" | "IntegerBox" | "IntBox" => super::MirType::Integer,
            "Float" | "F64" | "FloatBox" => super::MirType::Float,
            "Bool" | "Boolean" | "BoolBox" => super::MirType::Bool,
            "String" | "StringBox" => super::MirType::String,
            "Void" | "Unit" => super::MirType::Void,
            // Fallback: treat as user box type
            other => super::MirType::Box(other.to_string()),
        }
    }

    // Extract string literal from AST node if possible
    pub(super) fn extract_string_literal(node: &ASTNode) -> Option<String> {
        let mut cur = node;
        loop {
            match cur {
                ASTNode::Literal {
                    value: LiteralValue::String(s),
                    ..
                } => return Some(s.clone()),
                ASTNode::New {
                    class, arguments, ..
                } if class == "StringBox" && arguments.len() == 1 => {
                    cur = &arguments[0];
                    continue;
                }
                _ => return None,
            }
        }
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
        let mut param_types = Vec::new();
        param_types.push(MirType::Box(box_name.clone()));
        for _ in &params {
            param_types.push(MirType::Unknown);
        }
        let returns_value = contains_value_return(&body);
        let ret_ty = if returns_value {
            MirType::Unknown
        } else {
            MirType::Void
        };
        let signature = FunctionSignature {
            name: func_name,
            params: param_types,
            return_type: ret_ty,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        };
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
        let program_ast = ASTNode::Program {
            statements: body,
            span: crate::ast::Span::unknown(),
        };
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
        let mut param_types = Vec::new();
        for _ in &params {
            param_types.push(MirType::Unknown);
        }
        let returns_value = contains_value_return(&body);
        let ret_ty = if returns_value {
            MirType::Unknown
        } else {
            MirType::Void
        };
        let signature = FunctionSignature {
            name: func_name,
            params: param_types,
            return_type: ret_ty,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        };
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
        let program_ast = ASTNode::Program {
            statements: body,
            span: crate::ast::Span::unknown(),
        };
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
        Ok(())
    }
}
