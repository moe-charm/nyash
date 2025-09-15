/*!
 * Expression Processing Module
 * 
 * Extracted from core.rs lines 408-787 (~380 lines)
 * Handles expression evaluation, binary operations, method calls, and field access
 * Core philosophy: "Everything is Box" with clean expression evaluation
 */

// Module declarations
mod operators;
mod calls;
mod access;
mod builtins;

use super::*;
use std::sync::Arc;
// Direct implementation approach to avoid import issues

// TODO: Fix NullBox import issue later
// use crate::NullBox;

impl NyashInterpreter {
    /// Build closure environment by capturing 'me' and free variables by value (P1)
    fn build_closure_env(&mut self, params: &Vec<String>, body: &Vec<ASTNode>) -> Result<crate::boxes::function_box::ClosureEnv, RuntimeError> {
        use std::collections::HashSet;
        let mut env = crate::boxes::function_box::ClosureEnv::new();
        // Capture 'me' if bound
        if let Ok(mev) = self.resolve_variable("me") { env.me_value = Some(Arc::downgrade(&mev)); }

        // Collect free variables
        let mut used: HashSet<String> = HashSet::new();
        let mut locals: HashSet<String> = HashSet::new();
        // params are considered local
        for p in params { locals.insert(p.clone()); }
        // BFS walk statements
        fn collect(node: &ASTNode, used: &mut HashSet<String>, locals: &mut HashSet<String>) {
            match node {
                ASTNode::Variable { name, .. } => {
                    if !locals.contains(name) && name != "me" && name != "this" { used.insert(name.clone()); }
                }
                ASTNode::Local { variables, .. } => { for v in variables { locals.insert(v.clone()); } }
                ASTNode::Assignment { target, value, .. } => { collect(target, used, locals); collect(value, used, locals); }
                ASTNode::BinaryOp { left, right, .. } => { collect(left, used, locals); collect(right, used, locals); }
                ASTNode::UnaryOp { operand, .. } => { collect(operand, used, locals); }
                ASTNode::MethodCall { object, arguments, .. } => { collect(object, used, locals); for a in arguments { collect(a, used, locals);} }
                ASTNode::FunctionCall { arguments, .. } => { for a in arguments { collect(a, used, locals);} }
                ASTNode::Call { callee, arguments, .. } => { collect(callee, used, locals); for a in arguments { collect(a, used, locals);} }
                ASTNode::FieldAccess { object, .. } => { collect(object, used, locals); }
                ASTNode::New { arguments, .. } => { for a in arguments { collect(a, used, locals);} }
                ASTNode::If { condition, then_body, else_body, .. } => {
                    collect(condition, used, locals);
                    for st in then_body { collect(st, used, locals); }
                    if let Some(eb) = else_body { for st in eb { collect(st, used, locals); } }
                }
                ASTNode::Loop { condition, body, .. } => { collect(condition, used, locals); for st in body { collect(st, used, locals);} }
                ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                    for st in try_body { collect(st, used, locals); }
                    for c in catch_clauses { for st in &c.body { collect(st, used, locals); } }
                    if let Some(fb) = finally_body { for st in fb { collect(st, used, locals); } }
                }
                ASTNode::Throw { expression, .. } => { collect(expression, used, locals); }
                ASTNode::Print { expression, .. } => { collect(expression, used, locals); }
                ASTNode::Return { value, .. } => { if let Some(v) = value { collect(v, used, locals); } }
                ASTNode::AwaitExpression { expression, .. } => { collect(expression, used, locals); }
                ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                    collect(scrutinee, used, locals);
                    for (_, e) in arms { collect(e, used, locals); }
                    collect(else_expr, used, locals);
                }
                ASTNode::Program { statements, .. } => { for st in statements { collect(st, used, locals); } }
                ASTNode::FunctionDeclaration { params, body, .. } => {
                    let mut inner = locals.clone();
                    for p in params { inner.insert(p.clone()); }
                    for st in body { collect(st, used, &mut inner); }
                }
                _ => {}
            }
        }
        for st in body { collect(st, &mut used, &mut locals); }

        // Materialize captures: local by-ref via RefCellBox, others by-value
        for name in used.into_iter() {
            if let Some(local_arc) = self.local_vars.get(&name) {
                let lb: &dyn NyashBox = &**local_arc;
                // If already RefCellBox, reuse inner; else wrap and replace local binding
                if let Some(rc) = lb.as_any().downcast_ref::<crate::boxes::ref_cell_box::RefCellBox>() {
                    env.captures.insert(name.clone(), rc.share_box());
                } else {
                    // wrap existing into RefCell and replace local binding
                    let wrapped = crate::boxes::ref_cell_box::RefCellBox::new(lb.clone_box());
                    self.local_vars.insert(name.clone(), wrapped.clone_arc());
                    env.captures.insert(name, wrapped.share_box());
                }
            } else {
                // non-local (global/static): by-value capture
                if let Ok(v) = self.resolve_variable(&name) { env.captures.insert(name, v.clone_or_share()); }
            }
        }
        Ok(env)
    }
    /// 式を実行 - Expression evaluation engine
    pub(super) fn execute_expression(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match expression {
            // P1: allow block (Program) as expression; value = last statement's value
            ASTNode::Program { statements, .. } => {
                let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
                let last = statements.len().saturating_sub(1);
                for (i, st) in statements.iter().enumerate() {
                    let prev = self.discard_context;
                    self.discard_context = i != last;
                    result = self.execute_statement(st)?;
                    self.discard_context = prev;
                    match &self.control_flow {
                        ControlFlow::Break => { return Err(RuntimeError::BreakOutsideLoop); }
                        ControlFlow::Continue => { return Err(RuntimeError::BreakOutsideLoop); }
                        ControlFlow::Return(_) => { return Err(RuntimeError::ReturnOutsideFunction); }
                        ControlFlow::Throw(_) => { return Err(RuntimeError::UncaughtException); }
                        ControlFlow::None => {}
                    }
                }
                Ok(result)
            }
            ASTNode::Literal { value, .. } => {
                Ok(value.to_nyash_box())
            }
            
            ASTNode::Variable { name, .. } => {
                // 🌍 革命的変数解決：local変数 → GlobalBoxフィールド → エラー
                let shared_var = self.resolve_variable(name)
                    .map_err(|_| RuntimeError::UndefinedVariableAt { 
                        name: name.clone(), 
                        span: expression.span() 
                    })?;
                Ok((*shared_var).share_box())  // 🎯 State-sharing instead of cloning
            }
            
            ASTNode::BinaryOp { operator, left, right, .. } => {
                self.execute_binary_op(operator, left, right)
            }
            
            ASTNode::UnaryOp { operator, operand, .. } => {
                self.execute_unary_op(operator, operand)
            }
            
            ASTNode::AwaitExpression { expression, .. } => {
                self.execute_await(expression)
            }
            
            ASTNode::MethodCall { object, method, arguments, .. } => {
                let result = self.execute_method_call(object, method, arguments);
                result
            }
            
            ASTNode::FieldAccess { object, field, .. } => {
                let shared_result = self.execute_field_access(object, field)?;
                Ok((*shared_result).clone_or_share())
            }
            
            ASTNode::New { class, arguments, type_arguments, .. } => {
                self.execute_new(class, arguments, type_arguments)
            }
            
            ASTNode::This { .. } => {
                // 🌍 革命的this解決：local変数から取得
                let shared_this = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is only available inside methods".to_string(),
                    })?;
                Ok((*shared_this).clone_or_share())
            }
            
            ASTNode::Me { .. } => {
                
                // 🌍 革命的me解決：local変数から取得（thisと同じ）
                let shared_me = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'me' is only available inside methods".to_string(),
                    })?;
                    
                Ok((*shared_me).clone_or_share())
            }
            
            ASTNode::ThisField { field, .. } => {
                // 🌍 革命的this.fieldアクセス：local変数から取得
                let this_value = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is not bound in the current context".to_string(),
                    })?;
                
                if let Some(instance) = (*this_value).as_any().downcast_ref::<InstanceBox>() {
                    let shared_field = instance.get_field(field)
                        .ok_or_else(|| RuntimeError::InvalidOperation { 
                            message: format!("Field '{}' not found on this", field)
                        })?;
                    Ok((*shared_field).clone_or_share())
                } else {
                    Err(RuntimeError::TypeError {
                        message: "'this' is not an instance".to_string(),
                    })
                }
            }
            
            ASTNode::MeField { field, .. } => {
                // 🌍 革命的me.fieldアクセス：local変数から取得
                let me_value = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is not bound in the current context".to_string(),
                    })?;
                
                if let Some(instance) = (*me_value).as_any().downcast_ref::<InstanceBox>() {
                    let shared_field = instance.get_field(field)
                        .ok_or_else(|| RuntimeError::InvalidOperation { 
                            message: format!("Field '{}' not found on me", field)
                        })?;
                    Ok((*shared_field).clone_or_share())
                } else {
                    Err(RuntimeError::TypeError {
                        message: "'this' is not an instance".to_string(),
                    })
                }
            }
            
            ASTNode::FunctionCall { name, arguments, .. } => {
                self.execute_function_call(name, arguments)
            }
            ASTNode::Call { callee, arguments, .. } => {
                // callee を評価して FunctionBox なら本体を実行
                let callee_val = self.execute_expression(callee)?;
                if let Some(fun) = callee_val.as_any().downcast_ref::<crate::boxes::function_box::FunctionBox>() {
                    // 引数評価
                    let mut arg_values: Vec<Box<dyn NyashBox>> = Vec::new();
                    for a in arguments { arg_values.push(self.execute_expression(a)?); }
                    if arg_values.len() != fun.params.len() {
                        return Err(RuntimeError::InvalidOperation { message: format!("Function expects {} args, got {}", fun.params.len(), arg_values.len()) });
                    }
                    // スコープ保存
                    let saved_locals = self.save_local_vars();
                    self.local_vars.clear();
                    // キャプチャ注入（by-value）
                    for (k, v) in fun.env.captures.iter() { self.declare_local_variable(k, v.clone_or_share()); }
                    if let Some(me_w) = &fun.env.me_value {
                        if let Some(me_arc) = me_w.upgrade() {
                            self.declare_local_variable("me", (*me_arc).clone_or_share());
                        } else {
                            self.declare_local_variable("me", Box::new(crate::boxes::null_box::NullBox::new()));
                        }
                    }
                    for (p, v) in fun.params.iter().zip(arg_values.iter()) {
                        self.declare_local_variable(p, v.clone_or_share());
                    }
                    // 実行
                    crate::runtime::global_hooks::push_task_scope();
                    let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
                    for st in &fun.body {
                        result = self.execute_statement(st)?;
                        if let super::ControlFlow::Return(rv) = &self.control_flow {
                            result = rv.clone_box();
                            self.control_flow = super::ControlFlow::None;
                            break;
                        }
                    }
                    crate::runtime::global_hooks::pop_task_scope();
                    self.restore_local_vars(saved_locals);
                    Ok(result)
                } else if let ASTNode::Lambda { params, body, .. } = callee.as_ref() {
                    // 直書きLambdaは従来通り実行（後方互換）
                    let mut arg_values: Vec<Box<dyn NyashBox>> = Vec::new();
                    for a in arguments { arg_values.push(self.execute_expression(a)?); }
                    if arg_values.len() != params.len() {
                        return Err(RuntimeError::InvalidOperation { message: format!("Lambda expects {} args, got {}", params.len(), arg_values.len()) });
                    }
                    let saved_locals = self.save_local_vars();
                    self.local_vars.clear();
                    for (p, v) in params.iter().zip(arg_values.iter()) { self.declare_local_variable(p, v.clone_or_share()); }
                    crate::runtime::global_hooks::push_task_scope();
                    let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
                    for st in body { result = self.execute_statement(st)?; if let super::ControlFlow::Return(rv) = &self.control_flow { result = rv.clone_box(); self.control_flow = super::ControlFlow::None; break; } }
                    crate::runtime::global_hooks::pop_task_scope();
                    self.restore_local_vars(saved_locals);
                    Ok(result)
                } else {
                    Err(RuntimeError::InvalidOperation { message: "Callee is not callable".to_string() })
                }
            }
            
            ASTNode::Arrow { sender, receiver, .. } => {
                self.execute_arrow(sender, receiver)
            }
            ASTNode::QMarkPropagate { expression, .. } => {
                let v = self.execute_expression(expression)?;
                if let Some(res) = v.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
                    // ok -> unwrap, err -> early return (propagate)
                    if matches!(res, crate::boxes::result::NyashResultBox::Ok(_)) {
                        return Ok(res.get_value());
                    } else {
                        // Early return the Result itself
                        self.control_flow = super::ControlFlow::Return(v.clone_box());
                        return Ok(Box::new(crate::box_trait::VoidBox::new()));
                    }
                }
                // Not a Result: pass-through
                Ok(v)
            }
            ASTNode::PeekExpr { scrutinee, arms, else_expr, .. } => {
                let val = self.execute_expression(scrutinee)?;
                let sval = val.to_string_box().value;
                for (pat, expr) in arms {
                    let pv = match pat {
                        crate::ast::LiteralValue::String(s) => s.clone(),
                        crate::ast::LiteralValue::Integer(i) => i.to_string(),
                        crate::ast::LiteralValue::Float(f) => f.to_string(),
                        crate::ast::LiteralValue::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                        crate::ast::LiteralValue::Null => "null".to_string(),
                        crate::ast::LiteralValue::Void => "void".to_string(),
                    };
                    if pv == sval {
                        return self.execute_expression(expr);
                    }
                }
                self.execute_expression(else_expr)
            }
            ASTNode::Lambda { params, body, .. } => {
                // 値としての関数ボックスを生成（ClosureEnv: me/by-val captures）
                let env = self.build_closure_env(&params, body)?;
                Ok(Box::new(crate::boxes::function_box::FunctionBox::with_env(params.clone(), body.clone(), env)))
            }
            
            ASTNode::Include { filename, .. } => {
                // include式: 最初のstatic boxを返す
                self.execute_include_expr(filename)
            }
            
            ASTNode::FromCall { parent, method, arguments, .. } => {
                self.execute_from_call(parent, method, arguments)
            }
            
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Cannot execute {:?} as expression", expression.node_type()),
            }),
        }
    }
    
    
    
    
    /// 🔄 循環参照検出: オブジェクトの一意IDを取得
    #[allow(dead_code)]
    fn get_object_id(&self, node: &ASTNode) -> Option<usize> {
        match node {
            ASTNode::Variable { name, .. } => {
                // 変数名のハッシュをIDとして使用
                Some(self.hash_string(name))
            }
            ASTNode::Me { .. } => {
                // 'me'参照の特別なID
                Some(usize::MAX) 
            }
            ASTNode::This { .. } => {
                // 'this'参照の特別なID  
                Some(usize::MAX - 1)
            }
            _ => None, // 他のノードタイプはID追跡しない
        }
    }
    
    /// 🔄 文字列のシンプルなハッシュ関数
    #[allow(dead_code)]
    fn hash_string(&self, s: &str) -> usize {
        let mut hash = 0usize;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }
    
    // fn box_to_nyash_value(&self, box_val: &Box<dyn NyashBox>) -> Option<nyash_rust::value::NyashValue> {
    //     // Try to convert the box back to NyashValue for weak reference operations
    //     // This is a simplified conversion - in reality we might need more sophisticated logic
    //     use nyash_rust::value::NyashValue;
    //     use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
    //     
    //     if let Some(string_box) = box_val.as_any().downcast_ref::<StringBox>() {
    //         Some(NyashValue::String(string_box.value.clone()))
    //     } else if let Some(int_box) = box_val.as_any().downcast_ref::<IntegerBox>() {
    //         Some(NyashValue::Integer(int_box.value))
    //     } else if let Some(bool_box) = box_val.as_any().downcast_ref::<BoolBox>() {
    //         Some(NyashValue::Bool(bool_box.value))
    //     } else if box_val.as_any().downcast_ref::<VoidBox>().is_some() {
    //         Some(NyashValue::Void)
    //     } else if box_val.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
    //         Some(NyashValue::Null)
    //     } else {
    //         // For complex types, create a Box variant
    //         // Note: This is where we'd store the weak reference
    //         None // Simplified for now
    //     }
    // }
    
    
}
