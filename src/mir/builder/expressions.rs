/*!
 * MIR Builder Expressions - Expression AST node conversion
 * 
 * Handles conversion of expression AST nodes to MIR instructions
 */

use super::*;
use crate::ast::{ASTNode, LiteralValue, BinaryOperator};

/// Binary operation type classification
enum BinaryOpType {
    Arithmetic(BinaryOp),
    Comparison(CompareOp),
}

impl MirBuilder {
    /// Build a literal value into MIR
    pub(super) fn build_literal(&mut self, literal: LiteralValue) -> Result<ValueId, String> {
        let const_value = match literal {
            LiteralValue::Integer(n) => ConstValue::Integer(n),
            LiteralValue::Float(f) => ConstValue::Float(f),
            LiteralValue::String(s) => ConstValue::String(s),
            LiteralValue::Bool(b) => ConstValue::Bool(b),
            LiteralValue::Null => ConstValue::Null,
            LiteralValue::Void => ConstValue::Void,
        };
        
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst,
            value: const_value,
        })?;
        
        Ok(dst)
    }

    /// Build a binary operation
    pub(super) fn build_binary_op(&mut self, left: ASTNode, operator: BinaryOperator, right: ASTNode) -> Result<ValueId, String> {
        let lhs = self.build_expression(left)?;
        let rhs = self.build_expression(right)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_binary_operator(operator)?;
        
        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                self.emit_instruction(MirInstruction::BinOp {
                    dst, op, lhs, rhs
                })?;
            },
            
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                self.emit_instruction(MirInstruction::Compare {
                    dst, op, lhs, rhs
                })?;
            },
        }
        
        Ok(dst)
    }
    
    /// Build a unary operation
    pub(super) fn build_unary_op(&mut self, operator: String, operand: ASTNode) -> Result<ValueId, String> {
        let operand_val = self.build_expression(operand)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_unary_operator(operator)?;
        
        self.emit_instruction(MirInstruction::UnaryOp {
            dst,
            op: mir_op,
            operand: operand_val,
        })?;
        
        Ok(dst)
    }

    /// Convert AST binary operator to MIR operator
    fn convert_binary_operator(&self, op: BinaryOperator) -> Result<BinaryOpType, String> {
        match op {
            BinaryOperator::Add => Ok(BinaryOpType::Arithmetic(BinaryOp::Add)),
            BinaryOperator::Subtract => Ok(BinaryOpType::Arithmetic(BinaryOp::Sub)),
            BinaryOperator::Multiply => Ok(BinaryOpType::Arithmetic(BinaryOp::Mul)),
            BinaryOperator::Divide => Ok(BinaryOpType::Arithmetic(BinaryOp::Div)),
            BinaryOperator::Modulo => Ok(BinaryOpType::Arithmetic(BinaryOp::Mod)),
            BinaryOperator::Equal => Ok(BinaryOpType::Comparison(CompareOp::Eq)),
            BinaryOperator::NotEqual => Ok(BinaryOpType::Comparison(CompareOp::Ne)),
            BinaryOperator::Less => Ok(BinaryOpType::Comparison(CompareOp::Lt)),
            BinaryOperator::LessEqual => Ok(BinaryOpType::Comparison(CompareOp::Le)),
            BinaryOperator::Greater => Ok(BinaryOpType::Comparison(CompareOp::Gt)),
            BinaryOperator::GreaterEqual => Ok(BinaryOpType::Comparison(CompareOp::Ge)),
            BinaryOperator::And => Ok(BinaryOpType::Arithmetic(BinaryOp::And)),
            BinaryOperator::Or => Ok(BinaryOpType::Arithmetic(BinaryOp::Or)),
        }
    }
    
    /// Convert AST unary operator to MIR operator
    fn convert_unary_operator(&self, op: String) -> Result<UnaryOp, String> {
        match op.as_str() {
            "-" => Ok(UnaryOp::Neg),
            "!" | "not" => Ok(UnaryOp::Not),
            "~" => Ok(UnaryOp::BitNot),
            _ => Err(format!("Unsupported unary operator: {}", op)),
        }
    }

    /// Build variable access
    pub(super) fn build_variable_access(&mut self, name: String) -> Result<ValueId, String> {
        if let Some(&value_id) = self.variable_map.get(&name) {
            Ok(value_id)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }

    /// Build field access
    pub(super) fn build_field_access(&mut self, object: ASTNode, field: String) -> Result<ValueId, String> {
        // Clone the object before building expression if we need to check it later
        let object_clone = object.clone();
        
        // First, build the object expression to get its ValueId
        let object_value = self.build_expression(object)?;

        // Get the field from the object using RefGet
        let field_val = self.value_gen.next();
        self.emit_instruction(MirInstruction::RefGet {
            dst: field_val,
            reference: object_value,
            field: field.clone(),
        })?;

        // If we recorded origin class for this field on this base object, propagate it to this value id
        if let Some(class_name) = self.field_origin_class.get(&(object_value, field.clone())).cloned() {
            self.value_origin_newbox.insert(field_val, class_name);
        }

        // If we can infer the box type and the field is weak, emit WeakLoad (+ optional barrier)
        let mut inferred_class: Option<String> = self.value_origin_newbox.get(&object_value).cloned();
        // Fallback: if the object is a nested field access like (X.Y).Z, consult recorded field origins for X.Y
        if inferred_class.is_none() {
            if let ASTNode::FieldAccess { object: inner_obj, field: ref parent_field, .. } = object_clone {
                // Build inner base to get a stable id and consult mapping
                if let Ok(base_id) = self.build_expression(*inner_obj.clone()) {
                    if let Some(cls) = self.field_origin_class.get(&(base_id, parent_field.clone())) {
                        inferred_class = Some(cls.clone());
                    }
                }
            }
        }
        if let Some(class_name) = inferred_class {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    // Barrier (read) PoC
                    let _ = self.emit_barrier_read(field_val);
                    // WeakLoad
                    let loaded = self.emit_weak_load(field_val)?;
                    return Ok(loaded);
                }
            }
        }

        Ok(field_val)
    }

    /// Build function call
    pub(super) fn build_function_call(&mut self, name: String, args: Vec<ASTNode>) -> Result<ValueId, String> {
        // Minimal TypeOp wiring via function-style: isType(value, "Type"), asType(value, "Type")
        if (name == "isType" || name == "asType") && args.len() == 2 {
            if let Some(type_name) = Self::extract_string_literal(&args[1]) {
                let val = self.build_expression(args[0].clone())?;
                let ty = Self::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if name == "isType" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                self.emit_instruction(MirInstruction::TypeOp { dst, op, value: val, ty })?;
                return Ok(dst);
            }
        }
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.build_expression(arg)?);
        }
        
        let dst = self.value_gen.next();
        
        // For now, treat all function calls as Box method calls
        self.emit_instruction(MirInstruction::Call {
            dst,
            function: name,
            arguments: arg_values,
        })?;
        
        Ok(dst)
    }

    /// Parse type name string to MIR type
    fn parse_type_name_to_mir(name: &str) -> super::MirType {
        match name {
            "Integer" | "Int" | "I64" => super::MirType::Integer,
            "Float" | "F64" => super::MirType::Float,
            "Bool" | "Boolean" => super::MirType::Bool,
            "String" => super::MirType::String,
            "Void" | "Unit" => super::MirType::Void,
            other => super::MirType::Box(other.to_string()),
        }
    }
    
    /// Extract string literal from AST node if possible
    /// Supports: Literal("Type") and new StringBox("Type")
    fn extract_string_literal(node: &ASTNode) -> Option<String> {
        let mut cur = node;
        loop {
            match cur {
                ASTNode::Literal { value: crate::ast::LiteralValue::String(s), .. } => return Some(s.clone()),
                ASTNode::New { class, arguments, .. } if class == "StringBox" && arguments.len() == 1 => {
                    cur = &arguments[0];
                    continue;
                }
                _ => return None,
            }
        }
    }

    /// Build me expression
    pub(super) fn build_me_expression(&mut self) -> Result<ValueId, String> {
        // If lowering a method/birth function, "me" should be a parameter
        if let Some(id) = self.variable_map.get("me").cloned() {
            return Ok(id);
        }

        // Fallback: use a symbolic constant (legacy behavior)
        let me_value = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: me_value,
            value: ConstValue::String("__me__".to_string()),
        })?;
        // Register a stable mapping so subsequent 'me' resolves to the same ValueId
        self.variable_map.insert("me".to_string(), me_value);
        Ok(me_value)
    }

    /// Build method call: object.method(arguments)
    pub(super) fn build_method_call(&mut self, object: ASTNode, method: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Minimal TypeOp wiring via method-style syntax: value.is("Type") / value.as("Type")
        if (method == "is" || method == "as") && arguments.len() == 1 {
            if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                // Build the object expression
                let object_value = self.build_expression(object.clone())?;
                // Map string to MIR type
                let mir_ty = Self::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if method == "is" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                self.emit_instruction(MirInstruction::TypeOp { dst, op, value: object_value, ty: mir_ty })?;
                return Ok(dst);
            }
        }
        // ExternCall判定はobjectの変数解決より先に行う（未定義変数で落とさない）
        if let ASTNode::Variable { name: object_name, .. } = object.clone() {
            // Build argument expressions first (externはobject自体を使わない)
            let mut arg_values = Vec::new();
            for arg in &arguments {
                arg_values.push(self.build_expression(arg.clone())?);
            }
            match (object_name.as_str(), method.as_str()) {
                ("console", "log") => {
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None,
                        iface_name: "env.console".to_string(),
                        method_name: "log".to_string(),
                        args: arg_values,
                        effects: EffectMask::IO,
                    })?;
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const { dst: void_id, value: ConstValue::Void })?;
                    return Ok(void_id);
                },
                ("canvas", "fillRect") | ("canvas", "fillText") => {
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None,
                        iface_name: "env.canvas".to_string(),
                        method_name: method,
                        args: arg_values,
                        effects: EffectMask::IO,
                    })?;
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const { dst: void_id, value: ConstValue::Void })?;
                    return Ok(void_id);
                },
                _ => {}
            }
        }

        // Build the object expression
        let object_value = self.build_expression(object.clone())?;

        // Secondary interception for is/as in case early path did not trigger
        if (method == "is" || method == "as") && arguments.len() == 1 {
            if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                let mir_ty = Self::parse_type_name_to_mir(&type_name);
                let dst = self.value_gen.next();
                let op = if method == "is" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                self.emit_instruction(MirInstruction::TypeOp { dst, op, value: object_value, ty: mir_ty })?;
                return Ok(dst);
            }
        }

        // Build argument expressions
        let mut arg_values = Vec::new();
        for arg in &arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // Create result value
        let result_id = self.value_gen.next();
        
        // Optimization: If the object is a direct `new ClassName(...)`, lower to a direct Call
        if let ASTNode::New { class, .. } = object {
            // Build function name and only lower to Call if the function exists (user-defined)
            let func_name = format!("{}.{}{}", class, method, format!("/{}", arg_values.len()));
            let can_lower = self.user_defined_boxes.contains(&class)
                && if let Some(ref module) = self.current_module { module.functions.contains_key(&func_name) } else { false };
            if can_lower {
                let func_val = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const { dst: func_val, value: ConstValue::String(func_name) })?;
                let mut call_args = Vec::with_capacity(arg_values.len() + 1);
                call_args.push(object_value);
                call_args.extend(arg_values);
                self.emit_instruction(MirInstruction::Call {
                    dst: Some(result_id),
                    func: func_val,
                    args: call_args,
                    effects: EffectMask::READ.add(Effect::ReadHeap),
                })?;
                return Ok(result_id);
            }
            // else fall through to BoxCall below
        } else {
            // If the object originates from a NewBox in this function, we can lower to Call as well
            if let Some(class_name) = self.value_origin_newbox.get(&object_value).cloned() {
                let func_name = format!("{}.{}{}", class_name, method, format!("/{}", arg_values.len()));
                let can_lower = self.user_defined_boxes.contains(&class_name)
                    && if let Some(ref module) = self.current_module { module.functions.contains_key(&func_name) } else { false };
                if can_lower {
                    let func_val = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const { dst: func_val, value: ConstValue::String(func_name) })?;
                    let mut call_args = Vec::with_capacity(arg_values.len() + 1);
                    call_args.push(object_value);
                    call_args.extend(arg_values);
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(result_id),
                        func: func_val,
                        args: call_args,
                        effects: EffectMask::READ.add(Effect::ReadHeap),
                    })?;
                    return Ok(result_id);
                }
            }
        }

        // Fallback: Emit a BoxCall instruction for regular or plugin/builtin method calls
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(result_id),
            box_val: object_value,
            method,
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap), // Method calls may have side effects
        })?;
        Ok(result_id)
    }

    /// Build from expression: from Parent.method(arguments)
    pub(super) fn build_from_expression(&mut self, parent: String, method: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Build argument expressions
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg)?);
        }
        
        // Get parent box's me value if available
        if let Some(&parent_me) = self.variable_map.get(&format!("{}.__me__", parent)) {
            // Emit from call with parent's me
            let result = self.value_gen.next();
            self.emit_instruction(MirInstruction::Call {
                dst: Some(result),
                func: self.value_gen.next(), // Placeholder for from resolution
                args: vec![parent_me], // Include parent's me plus other args
                effects: EffectMask::READ.add(Effect::ReadHeap),
            })?;
            Ok(result)
        } else {
            // Fallback behavior without proper parent context
            let result = self.value_gen.next();
            Ok(result)
        }
    }

    /// Build expression from AST
    pub(super) fn build_expression(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        match ast {
            ASTNode::Literal { value, .. } => {
                self.build_literal(value)
            },
            
            ASTNode::BinaryOp { left, operator, right, .. } => {
                self.build_binary_op(*left, operator, *right)
            },
            
            ASTNode::UnaryOp { operator, operand, .. } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            },
            
            ASTNode::Variable { name, .. } => {
                self.build_variable_access(name.clone())
            },
            
            ASTNode::Me { .. } => {
                self.build_me_expression()
            },
            
            ASTNode::MethodCall { object, method, arguments, .. } => {
                // Early TypeOp lowering for method-style is()/as()
                if (method == "is" || method == "as") && arguments.len() == 1 {
                    if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                        let obj_val = self.build_expression(*object.clone())?;
                        let ty = Self::parse_type_name_to_mir(&type_name);
                        let dst = self.value_gen.next();
                        let op = if method == "is" { super::TypeOpKind::Check } else { super::TypeOpKind::Cast };
                        self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
                        return Ok(dst);
                    }
                }
                self.build_method_call(*object.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::FromCall { parent, method, arguments, .. } => {
                self.build_from_expression(parent.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::Assignment { target, value, .. } => {
                // Check if target is a field access for RefSet
                if let ASTNode::FieldAccess { object, field, .. } = target.as_ref() {
                    self.build_field_assignment(*object.clone(), field.clone(), *value.clone())
                } else if let ASTNode::Variable { name, .. } = target.as_ref() {
                    self.build_assignment(name.clone(), *value.clone())
                } else {
                    Err("Invalid assignment target".to_string())
                }
            },
            
            ASTNode::FunctionCall { name, arguments, .. } => {
                self.build_function_call(name.clone(), arguments.clone())
            },
            
            ASTNode::FieldAccess { object, field, .. } => {
                self.build_field_access(*object.clone(), field.clone())
            },
            
            ASTNode::ArrayAccess { array, index, .. } => {
                // Build array and index expressions
                let array_val = self.build_expression(*array)?;
                let index_val = self.build_expression(*index)?;
                
                // Generate ArrayGet instruction
                let result = self.value_gen.next();
                self.emit_instruction(MirInstruction::ArrayGet {
                    dst: result,
                    array: array_val,
                    index: index_val,
                    effects: EffectMask::READ.add(Effect::ReadHeap),
                })?;
                
                Ok(result)
            },
            
            ASTNode::ArrayLiteral { elements, .. } => {
                // Create new ArrayBox
                let array_val = self.build_new_expression("ArrayBox".to_string(), vec![])?;
                
                // Add each element
                for elem in elements {
                    let elem_val = self.build_expression(elem)?;
                    
                    // array.push(elem)
                    self.emit_instruction(MirInstruction::BoxCall {
                        dst: None,
                        box_val: array_val,
                        method: "push".to_string(),
                        args: vec![elem_val],
                        effects: EffectMask::WRITE.add(Effect::WriteHeap),
                    })?;
                }
                
                Ok(array_val)
            },
            
            ASTNode::New { class, arguments, .. } => {
                self.build_new_expression(class.clone(), arguments.clone())
            },
            
            ASTNode::Await { expression, .. } => {
                self.build_await_expression(*expression)
            },
            
            _ => {
                Err(format!("Unsupported expression type: {:?}", ast))
            }
        }
    }

    /// Build assignment: variable = value
    pub(super) fn build_assignment(&mut self, var_name: String, value: ASTNode) -> Result<ValueId, String> {
        let value_id = self.build_expression(value)?;
        
        // In SSA form, each assignment creates a new value
        self.variable_map.insert(var_name.clone(), value_id);
        
        Ok(value_id)
    }

    /// Build field assignment: object.field = value
    pub(super) fn build_field_assignment(&mut self, object: ASTNode, field: String, value: ASTNode) -> Result<ValueId, String> {
        // Build the object and value expressions
        let object_value = self.build_expression(object)?;
        let mut value_result = self.build_expression(value)?;

        // If we can infer the box type and the field is weak, create WeakRef before store
        if let Some(class_name) = self.value_origin_newbox.get(&object_value).cloned() {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    value_result = self.emit_weak_new(value_result)?;
                }
            }
        }

        // Set the field using RefSet
        self.emit_instruction(MirInstruction::RefSet {
            reference: object_value,
            field: field.clone(),
            value: value_result,
        })?;

        // Emit a write barrier for weak fields (PoC)
        if let Some(class_name) = self.value_origin_newbox.get(&object_value).cloned() {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    let _ = self.emit_barrier_write(value_result);
                }
            }
        }

        // Record origin class for this field (if the value originates from NewBox of a known class)
        if let Some(class_name) = self.value_origin_newbox.get(&value_result).cloned() {
            self.field_origin_class.insert((object_value, field.clone()), class_name);
        }

        // Return the assigned value
        Ok(value_result)
    }

    /// Build new expression: new ClassName(arguments)
    pub(super) fn build_new_expression(&mut self, class: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Phase 9.78a: Unified Box creation using NewBox instruction
        
        // First, evaluate all arguments to get their ValueIds
        let mut arg_values = Vec::new();
        for arg in arguments {
            let arg_value = self.build_expression(arg)?;
            arg_values.push(arg_value);
        }
        
        // Generate the destination ValueId
        let dst = self.value_gen.next();
        
        // Emit NewBox instruction for all Box types
        // VM will handle optimization for basic types internally
        self.emit_instruction(MirInstruction::NewBox {
            dst,
            box_type: class.clone(),
            args: arg_values.clone(),
        })?;

        // Record origin for optimization: dst was created by NewBox of class
        self.value_origin_newbox.insert(dst, class);

        // Immediately call birth(...) on the created instance to run constructor semantics.
        // birth typically returns void; we don't capture the result here (dst: None)
        self.emit_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: dst,
            method: "birth".to_string(),
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        })?;
        
        Ok(dst)
    }

    /// Build await expression: await expression
    pub(super) fn build_await_expression(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        // Evaluate the expression (should be a Future)
        let future_value = self.build_expression(expression)?;
        
        // Create result value for the await
        let result_id = self.value_gen.next();
        
        // Emit await instruction
        self.emit_instruction(MirInstruction::Await {
            dst: result_id,
            future: future_value,
            effects: EffectMask::READ.add(Effect::Async),
        })?;
        
        Ok(result_id)
    }
}