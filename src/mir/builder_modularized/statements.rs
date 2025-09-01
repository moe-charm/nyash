/*!
 * MIR Builder Statements - Statement AST node conversion
 * 
 * Handles conversion of statement AST nodes to MIR instructions
 */

use super::*;
use crate::mir::builder_modularized::core::builder_debug_log;
use crate::mir::TypeOpKind;
use crate::ast::ASTNode;

impl MirBuilder {
    /// Build print statement - converts to console output
    pub(super) fn build_print_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        builder_debug_log("enter build_print_statement");
        // 根治: print(isType(...)) / print(asType(...)) / print(obj.is(...)) / print(obj.as(...)) は必ずTypeOpを先に生成してからprintする
        match &expression {
            ASTNode::FunctionCall { name, arguments, .. } if (name == "isType" || name == "asType") && arguments.len() == 2 => {
                builder_debug_log("pattern: print(FunctionCall isType|asType)");
                if let Some(type_name) = Self::extract_string_literal(&arguments[1]) {
                    builder_debug_log(&format!("extract_string_literal OK: {}", type_name));
                    let val = self.build_expression(arguments[0].clone())?;
                    let ty = Self::parse_type_name_to_mir(&type_name);
                    let dst = self.value_gen.next();
                    let op = if name == "isType" { TypeOpKind::Check } else { TypeOpKind::Cast };
                    builder_debug_log(&format!("emit TypeOp {:?} value={} dst= {}", op, val, dst));
                    self.emit_instruction(MirInstruction::TypeOp { dst, op, value: val, ty })?;
                    self.emit_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![dst], effects: EffectMask::PURE.add(Effect::Io) })?;
                    return Ok(dst);
                } else {
                    builder_debug_log("extract_string_literal FAIL");
                }
            }
            ASTNode::MethodCall { object, method, arguments, .. } if (method == "is" || method == "as") && arguments.len() == 1 => {
                builder_debug_log("pattern: print(MethodCall is|as)");
                if let Some(type_name) = Self::extract_string_literal(&arguments[0]) {
                    builder_debug_log(&format!("extract_string_literal OK: {}", type_name));
                    let obj_val = self.build_expression(*object.clone())?;
                    let ty = Self::parse_type_name_to_mir(&type_name);
                    let dst = self.value_gen.next();
                    let op = if method == "is" { TypeOpKind::Check } else { TypeOpKind::Cast };
                    builder_debug_log(&format!("emit TypeOp {:?} obj={} dst= {}", op, obj_val, dst));
                    self.emit_instruction(MirInstruction::TypeOp { dst, op, value: obj_val, ty })?;
                    self.emit_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![dst], effects: EffectMask::PURE.add(Effect::Io) })?;
                    return Ok(dst);
                } else {
                    builder_debug_log("extract_string_literal FAIL");
                }
            }
            _ => {}
        }

        let value = self.build_expression(expression)?;
        builder_debug_log(&format!("fallback print value={}", value));
        
        self.emit_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.console".to_string(), method_name: "log".to_string(), args: vec![value], effects: EffectMask::PURE.add(Effect::Io) })?;
        
        // Return the value that was printed
        Ok(value)
    }
    
    /// Build a block of statements
    pub(super) fn build_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        let mut last_value = None;
        
        for statement in statements {
            last_value = Some(self.build_expression(statement)?);
        }
        
        // Return last value or void
        Ok(last_value.unwrap_or_else(|| {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            }).unwrap();
            void_val
        }))
    }
    
    /// Build local variable declarations with optional initial values
    pub(super) fn build_local_statement(&mut self, variables: Vec<String>, initial_values: Vec<Option<Box<ASTNode>>>) -> Result<ValueId, String> {
        let mut last_value = None;
        
        // Process each variable declaration
        for (i, var_name) in variables.iter().enumerate() {
            let value_id = if i < initial_values.len() && initial_values[i].is_some() {
                // Variable has initial value - evaluate it
                let init_expr = initial_values[i].as_ref().unwrap();
                self.build_expression(*init_expr.clone())?
            } else {
                // No initial value - do not emit a const; leave uninitialized until assigned
                // Use a fresh SSA id only for name binding; consumers should not use it before assignment
                self.value_gen.next()
            };
            
            // Register variable in SSA form
            self.variable_map.insert(var_name.clone(), value_id);
            last_value = Some(value_id);
        }
        
        // Return the last bound value id (no emission); callers shouldn't rely on this value
        Ok(last_value.unwrap_or_else(|| {
            // create a dummy id without emission
            self.value_gen.next()
        }))
    }
    
    /// Build return statement
    pub(super) fn build_return_statement(&mut self, value: Option<Box<ASTNode>>) -> Result<ValueId, String> {
        let return_value = if let Some(expr) = value {
            self.build_expression(*expr)?
        } else {
            // Return void if no value specified
            let void_dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_dst,
                value: ConstValue::Void,
            })?;
            void_dst
        };
        
        // Emit return instruction
        self.emit_instruction(MirInstruction::Return {
            value: Some(return_value),
        })?;
        
        Ok(return_value)
    }
    
    /// Build a throw statement
    pub(super) fn build_throw_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        if std::env::var("NYASH_BUILDER_DISABLE_THROW").ok().as_deref() == Some("1") {
            let v = self.build_expression(expression)?;
            self.emit_instruction(MirInstruction::ExternCall { dst: None, iface_name: "env.debug".to_string(), method_name: "trace".to_string(), args: vec![v], effects: EffectMask::PURE.add(Effect::Debug) })?;
            return Ok(v);
        }
        let exception_value = self.build_expression(expression)?;
        self.emit_instruction(MirInstruction::Throw { exception: exception_value, effects: EffectMask::PANIC })?;
        Ok(exception_value)
    }
    
    /// Build nowait statement: nowait variable = expression
    pub(super) fn build_nowait_statement(&mut self, variable: String, expression: ASTNode) -> Result<ValueId, String> {
        // If the expression is a method call on a receiver, spawn it asynchronously via env.future.spawn_instance
        if let ASTNode::MethodCall { object, method, arguments, .. } = expression.clone() {
            // Build receiver value
            let recv_val = self.build_expression(*object)?;
            // Build method name as Const String
            let mname_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const { dst: mname_id, value: crate::mir::ConstValue::String(method.clone()) })?;
            // Build argument values
            let mut arg_vals: Vec<ValueId> = Vec::with_capacity(2 + arguments.len());
            arg_vals.push(recv_val);
            arg_vals.push(mname_id);
            for a in arguments.into_iter() { arg_vals.push(self.build_expression(a)?); }
            // Emit extern call to env.future.spawn_instance, capturing Future result
            let future_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::ExternCall {
                dst: Some(future_id),
                iface_name: "env.future".to_string(),
                method_name: "spawn_instance".to_string(),
                args: arg_vals,
                effects: crate::mir::effect::EffectMask::PURE.add(crate::mir::effect::Effect::Io),
            })?;
            // Store the future in the variable
            self.variable_map.insert(variable.clone(), future_id);
            return Ok(future_id);
        }
        // Fallback: evaluate synchronously and wrap into a resolved Future
        let expression_value = self.build_expression(expression)?;
        let future_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::FutureNew { dst: future_id, value: expression_value })?;
        self.variable_map.insert(variable.clone(), future_id);
        Ok(future_id)
    }
}
