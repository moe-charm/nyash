// Extracted call-related builders from builder.rs to keep files lean
use super::{Effect, EffectMask, FunctionSignature, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue, MethodCallExpr};

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

impl super::MirBuilder {
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
        if let Err(e) = self.emit_instruction(MirInstruction::NewBox { dst: math_recv, box_type: "MathBox".to_string(), args: vec![] }) { return Some(Err(e)); }
        self.value_origin_newbox.insert(math_recv, "MathBox".to_string());
        // birth()
        let birt_mid = slot_registry::resolve_slot_by_type_name("MathBox", "birth");
        if let Err(e) = self.emit_box_or_plugin_call(None, math_recv, "birth".to_string(), birt_mid, vec![], EffectMask::READ) { return Some(Err(e)); }
        // call method
        let dst = self.value_gen.next();
        if let Err(e) = self.emit_box_or_plugin_call(Some(dst), math_recv, name.to_string(), None, math_args, EffectMask::READ) { return Some(Err(e)); }
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
        if let Err(e) = self.emit_instruction(MirInstruction::Call { dst: Some(result_id), func: fun_val, args: arg_values, effects: EffectMask::READ.add(Effect::ReadHeap) }) { return Some(Err(e)); }
        Some(Ok(result_id))
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

        let dst = self.value_gen.next();

        // Default: call via fully-qualified function name string
        let mut arg_values = Vec::new();
        for a in args {
            arg_values.push(self.build_expression(a)?);
        }
        let fun_val = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: fun_val,
            value: super::ConstValue::String(name),
        })?;
        self.emit_instruction(MirInstruction::Call {
            dst: Some(dst),
            func: fun_val,
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        })?;
        Ok(dst)
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
