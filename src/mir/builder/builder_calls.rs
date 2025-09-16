// Extracted call-related builders from builder.rs to keep files lean
use super::{Effect, EffectMask, FunctionSignature, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue, MethodCallExpr};
use crate::mir::{slot_registry, TypeOpKind};

impl super::MirBuilder {
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

        let dst = self.value_gen.next();

        // Special-case: math.* as function-style (sin/cos/abs/min/max)
        let is_math_func = matches!(name.as_str(), "sin" | "cos" | "abs" | "min" | "max");
        if is_math_func {
            // Build numeric args directly for math.* to preserve f64 typing
            let mut math_args: Vec<ValueId> = Vec::new();
            for a in raw_args.into_iter() {
                match a {
                    ASTNode::New {
                        class, arguments, ..
                    } if class == "FloatBox" && arguments.len() == 1 => {
                        let v = self.build_expression(arguments[0].clone())?;
                        math_args.push(v);
                    }
                    ASTNode::New {
                        class, arguments, ..
                    } if class == "IntegerBox" && arguments.len() == 1 => {
                        let iv = self.build_expression(arguments[0].clone())?;
                        let fv = self.value_gen.next();
                        self.emit_instruction(MirInstruction::TypeOp {
                            dst: fv,
                            op: TypeOpKind::Cast,
                            value: iv,
                            ty: MirType::Float,
                        })?;
                        math_args.push(fv);
                    }
                    ASTNode::Literal {
                        value: LiteralValue::Float(_),
                        ..
                    } => {
                        let v = self.build_expression(a)?;
                        math_args.push(v);
                    }
                    other => {
                        let v = self.build_expression(other)?;
                        math_args.push(v);
                    }
                }
            }
            // new MathBox()
            let math_recv = self.value_gen.next();
            self.emit_instruction(MirInstruction::NewBox {
                dst: math_recv,
                box_type: "MathBox".to_string(),
                args: vec![],
            })?;
            self.value_origin_newbox
                .insert(math_recv, "MathBox".to_string());
            // birth()
            let birt_mid = slot_registry::resolve_slot_by_type_name("MathBox", "birth");
            self.emit_box_or_plugin_call(
                None,
                math_recv,
                "birth".to_string(),
                birt_mid,
                vec![],
                EffectMask::READ,
            )?;
            // call method
            self.emit_box_or_plugin_call(
                Some(dst),
                math_recv,
                name,
                None,
                math_args,
                EffectMask::READ,
            )?;
            return Ok(dst);
        }

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
        // ExternCall: env.X.* pattern via field access (e.g., env.future.delay)
        if let ASTNode::FieldAccess {
            object: env_obj,
            field: env_field,
            ..
        } = object.clone()
        {
            if let ASTNode::Variable { name: env_name, .. } = *env_obj {
                if env_name == "env" {
                    let mut arg_values = Vec::new();
                    for arg in &arguments {
                        arg_values.push(self.build_expression(arg.clone())?);
                    }
                    match (env_field.as_str(), method.as_str()) {
                        ("future", "delay") => {
                            let result_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: Some(result_id),
                                iface_name: "env.future".to_string(),
                                method_name: "delay".to_string(),
                                args: arg_values,
                                effects: EffectMask::READ.add(Effect::Io),
                            })?;
                            return Ok(result_id);
                        }
                        ("task", "currentToken") => {
                            let result_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: Some(result_id),
                                iface_name: "env.task".to_string(),
                                method_name: "currentToken".to_string(),
                                args: arg_values,
                                effects: EffectMask::READ,
                            })?;
                            return Ok(result_id);
                        }
                        ("task", "cancelCurrent") => {
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: None,
                                iface_name: "env.task".to_string(),
                                method_name: "cancelCurrent".to_string(),
                                args: arg_values,
                                effects: EffectMask::IO,
                            })?;
                            let void_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Const {
                                dst: void_id,
                                value: super::ConstValue::Void,
                            })?;
                            return Ok(void_id);
                        }
                        ("console", "log") => {
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: None,
                                iface_name: "env.console".to_string(),
                                method_name: "log".to_string(),
                                args: arg_values,
                                effects: EffectMask::IO,
                            })?;
                            let void_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Const {
                                dst: void_id,
                                value: super::ConstValue::Void,
                            })?;
                            return Ok(void_id);
                        }
                        ("console", "readLine") => {
                            let result_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: Some(result_id),
                                iface_name: "env.console".to_string(),
                                method_name: "readLine".to_string(),
                                args: arg_values,
                                effects: EffectMask::IO,
                            })?;
                            return Ok(result_id);
                        }
                        ("canvas", m @ ("fillRect" | "fillText")) => {
                            self.emit_instruction(MirInstruction::ExternCall {
                                dst: None,
                                iface_name: "env.canvas".to_string(),
                                method_name: m.to_string(),
                                args: arg_values,
                                effects: EffectMask::IO,
                            })?;
                            let void_id = self.value_gen.next();
                            self.emit_instruction(MirInstruction::Const {
                                dst: void_id,
                                value: super::ConstValue::Void,
                            })?;
                            return Ok(void_id);
                        }
                        _ => {}
                    }
                }
            }
        }
        // If object is `me` within a static box, lower to direct Call: BoxName.method/N
        if let ASTNode::Me { .. } = object {
            if let Some(cls_name) = self.current_static_box.clone() {
                let mut arg_values: Vec<ValueId> = Vec::new();
                for a in &arguments {
                    arg_values.push(self.build_expression(a.clone())?);
                }
                let result_id = self.value_gen.next();
                let fun_name = format!(
                    "{}.{}{}",
                    cls_name,
                    method,
                    format!("/{}", arg_values.len())
                );
                let fun_val = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: fun_val,
                    value: super::ConstValue::String(fun_name),
                })?;
                self.emit_instruction(MirInstruction::Call {
                    dst: Some(result_id),
                    func: fun_val,
                    args: arg_values,
                    effects: EffectMask::READ.add(Effect::ReadHeap),
                })?;
                return Ok(result_id);
            }
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
            "Integer" | "Int" | "I64" => super::MirType::Integer,
            "Float" | "F64" => super::MirType::Float,
            "Bool" | "Boolean" => super::MirType::Bool,
            "String" => super::MirType::String,
            "Void" | "Unit" => super::MirType::Void,
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
        let mut returns_value = false;
        for st in &body {
            if let ASTNode::Return { value: Some(_), .. } = st {
                returns_value = true;
                break;
            }
        }
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
        let mut returns_value = false;
        for st in &body {
            if let ASTNode::Return { value: Some(_), .. } = st {
                returns_value = true;
                break;
            }
        }
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
