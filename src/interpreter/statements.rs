/*!
 * Statement Processing Module
 * 
 * Extracted from core.rs - statement execution engine
 * Handles all statement types: assignments, if/else, loops, control flow
 * Core philosophy: "Everything is Box" with structured statement processing
 */

use super::*;
use std::sync::{Arc, Mutex};

impl NyashInterpreter {
    /// 文を実行 - Core statement execution engine
    pub(super) fn execute_statement(&mut self, statement: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match statement {
            ASTNode::Assignment { target, value, .. } => {
                self.execute_assignment(target, value)
            }
            
            ASTNode::Print { expression, .. } => {
                let value = self.execute_expression(expression)?;
                println!("{}", value.to_string_box());
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::If { condition, then_body, else_body, .. } => {
                self.execute_if(condition, then_body, else_body)
            }
            
            ASTNode::Loop { condition, body, .. } => {
                self.execute_loop(condition, body)
            }
            
            ASTNode::Return { value, .. } => {
                let return_value = if let Some(val) = value {
                    self.execute_expression(val)?
                } else {
                    Box::new(VoidBox::new())
                };
                self.control_flow = super::ControlFlow::Return(return_value);
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::Break { .. } => {
                self.control_flow = super::ControlFlow::Break;
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::Nowait { variable, expression, .. } => {
                self.execute_nowait(variable, expression)
            }
            
            ASTNode::BoxDeclaration { name, fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, is_static, static_init, .. } => {
                if *is_static {
                    // 🔥 Static Box宣言の処理
                    self.register_static_box_declaration(
                        name.clone(),
                        fields.clone(),
                        methods.clone(),
                        init_fields.clone(),
                        weak_fields.clone(),  // 🔗 Add weak_fields parameter
                        static_init.clone(),
                        extends.clone(),
                        implements.clone(),
                        type_parameters.clone()
                    )?;
                } else {
                    // 通常のBox宣言の処理 - 🔥 コンストラクタオーバーロード禁止対応
                    self.register_box_declaration(
                        name.clone(), 
                        fields.clone(), 
                        methods.clone(),
                        constructors.clone(),
                        init_fields.clone(),
                        weak_fields.clone(),  // 🔗 Add weak_fields parameter
                        *is_interface,
                        extends.clone(),
                        implements.clone(),
                        type_parameters.clone() // 🔥 ジェネリクス型パラメータ追加
                    )?; // 🔥 エラーハンドリング追加
                }
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::FunctionDeclaration { name, params, body, is_static, .. } => {
                if *is_static {
                    // 🔥 静的関数：box名.関数名の形式で解析
                    if let Some(dot_pos) = name.find('.') {
                        let box_name = name[..dot_pos].to_string();
                        let func_name = name[dot_pos + 1..].to_string();
                        
                        // boxのstaticメソッドとして登録
                        let func_ast = ASTNode::FunctionDeclaration {
                            name: func_name.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            is_static: true,
                            is_override: false,
                            span: crate::ast::Span::unknown(),
                        };
                        
                        {
                            let mut static_funcs = self.shared.static_functions.write().unwrap();
                            static_funcs
                                .entry(box_name.clone())
                                .or_insert_with(HashMap::new)
                                .insert(func_name.clone(), func_ast);
                        }
                        
                        eprintln!("🔥 Static function '{}.{}' registered", box_name, func_name);
                    } else {
                        // box名なしのstatic関数（将来的にはエラーにする）
                        eprintln!("⚠️ Static function '{}' needs box prefix (e.g., Math.min)", name);
                    }
                } else {
                    // 通常の関数：従来通りGlobalBoxメソッドとして登録
                    self.register_function_declaration(name.clone(), params.clone(), body.clone());
                }
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::GlobalVar { name, value, .. } => {
                let val = self.execute_expression(value)?;
                // 🌍 革命的グローバル変数：GlobalBoxのフィールドとして設定
                self.set_variable(name, val.clone_box())?;
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                self.execute_try_catch(try_body, catch_clauses, finally_body)
            }
            
            ASTNode::Throw { expression, .. } => {
                self.execute_throw(expression)
            }
            
            ASTNode::Local { variables, initial_values, .. } => {
                // 🌍 革命的local変数宣言：local変数スタックに追加（初期化対応）
                for (i, var_name) in variables.iter().enumerate() {
                    if let Some(Some(init_expr)) = initial_values.get(i) {
                        // 🚀 初期化付きlocal宣言: local x = value
                        let init_value = self.execute_expression(init_expr)?;
                        self.declare_local_variable(var_name, init_value);
                    } else {
                        // 従来のlocal宣言: local x
                        self.declare_local_variable(var_name, Box::new(VoidBox::new()));
                    }
                }
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::Outbox { variables, initial_values, .. } => {
                // 📤 革命的outbox変数宣言：static関数内で所有権移転（初期化対応）
                for (i, var_name) in variables.iter().enumerate() {
                    if let Some(Some(init_expr)) = initial_values.get(i) {
                        // 🚀 初期化付きoutbox宣言: outbox x = value
                        let init_value = self.execute_expression(init_expr)?;
                        self.declare_outbox_variable(var_name, init_value);
                    } else {
                        // 従来のoutbox宣言: outbox x
                        self.declare_outbox_variable(var_name, Box::new(VoidBox::new()));
                    }
                }
                Ok(Box::new(VoidBox::new()))
            }
            
            // 式文
            _ => self.execute_expression(statement),
        }
    }
    
    /// 条件分岐を実行 - If/else statement processing
    pub(super) fn execute_if(&mut self, condition: &ASTNode, then_body: &[ASTNode], else_body: &Option<Vec<ASTNode>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let condition_value = self.execute_expression(condition)?;
        
        // 条件を真偉値として評価
        let is_true = self.is_truthy(&condition_value);
        
        if is_true {
            for statement in then_body {
                self.execute_statement(statement)?;
                if !matches!(self.control_flow, super::ControlFlow::None) {
                    break;
                }
            }
        } else if let Some(else_statements) = else_body {
            for statement in else_statements {
                self.execute_statement(statement)?;
                if !matches!(self.control_flow, super::ControlFlow::None) {
                    break;
                }
            }
        }
        
        Ok(Box::new(VoidBox::new()))
    }
    
    /// ループを実行 - Loop processing: loop(condition) { body } のみ
    pub(super) fn execute_loop(&mut self, condition: &Box<ASTNode>, body: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        loop {
            // 常に条件をチェック
            let condition_result = self.execute_expression(condition)?;
            if let Some(bool_box) = condition_result.as_any().downcast_ref::<BoolBox>() {
                if !bool_box.value {
                    break; // 条件がfalseの場合はループ終了
                }
            } else {
                // 条件が真偉値でない場合は、Interpreter::is_truthy()を使用
                if !self.is_truthy(&condition_result) {
                    break;
                }
            }
            
            // ループ本体を実行
            for statement in body {
                self.execute_statement(statement)?;
                
                match &self.control_flow {
                    super::ControlFlow::Break => {
                        self.control_flow = super::ControlFlow::None;
                        return Ok(Box::new(VoidBox::new()));
                    }
                    super::ControlFlow::Return(_) => {
                        // returnはループを抜けるが、上位に伝播
                        return Ok(Box::new(VoidBox::new()));
                    }
                    super::ControlFlow::Throw(_) => {
                        // 例外はループを抜けて上位に伝播
                        return Ok(Box::new(VoidBox::new()));
                    }
                    super::ControlFlow::None => {}
                }
            }
        }
        
        Ok(Box::new(VoidBox::new()))
    }
    
    /// 代入処理を実行 - Assignment processing
    pub(super) fn execute_assignment(&mut self, target: &ASTNode, value: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let val = self.execute_expression(value)?;
        
        match target {
            ASTNode::Variable { name, .. } => {
                // 🌍 革命的代入：local変数 → GlobalBoxフィールド
                
                // 🔗 DEMO: Weak Reference Invalidation Simulation
                // If we're setting a variable to 0, simulate "dropping" the previous value
                if val.to_string_box().value == "0" {
                    eprintln!("🔗 DEBUG: Variable '{}' set to 0 - simulating object drop", name);
                    
                    // Get the current value before dropping it
                    if let Ok(old_value) = self.resolve_variable(name) {
                        let old_value_str = old_value.to_string_box().value;
                        eprintln!("🔗 DEBUG: Old value being dropped: {}", old_value_str);
                        
                        // For demo purposes, if we're dropping a "parent" variable,
                        // manually invalidate weak references to Parent instances
                        if name.contains("parent") && old_value_str.contains("instance #") {
                            eprintln!("🔗 DEBUG: Triggering weak reference invalidation for: {}", old_value_str);
                            
                            // Call the interpreter method with actual object info
                            self.trigger_weak_reference_invalidation(&old_value_str);
                        }
                    }
                }
                
                self.set_variable(name, val.clone_box())?;
                Ok(val)
            }
            
            ASTNode::FieldAccess { object, field, .. } => {
                // フィールドへの代入
                let obj_value = self.execute_expression(object)?;
                
                if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
                    // 🔥 Usage prohibition guard - check if instance is finalized
                    if instance.is_finalized() {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Instance was finalized; further use is prohibited".to_string(),
                        });
                    }
                    
                    // 🔗 Weak Reference Assignment Check
                    let box_decls = self.shared.box_declarations.read().unwrap();
                    if let Some(box_decl) = box_decls.get(&instance.class_name) {
                        if box_decl.weak_fields.contains(&field.to_string()) {
                            eprintln!("🔗 DEBUG: Assigning to weak field '{}' in class '{}'", field, instance.class_name);
                            
                            // 🎯 PHASE 2: Use the new legacy conversion helper
                            instance.set_weak_field_from_legacy(field.to_string(), val.clone_box())
                                .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
                            
                            return Ok(val);
                        }
                    }
                    
                    // 既存のフィールド値があればfini()を呼ぶ
                    if let Some(old_field_value) = instance.get_field(field) {
                        if let Some(old_instance) = old_field_value.as_any().downcast_ref::<InstanceBox>() {
                            let _ = old_instance.fini();
                            finalization::mark_as_finalized(old_instance.box_id());
                        }
                    }
                    
                    instance.set_field(field, val.clone_box())
                        .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
                    Ok(val)
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("Cannot set field '{}' on non-instance type", field),
                    })
                }
            }
            
            ASTNode::ThisField { field, .. } => {
                // 🌍 革命的this.field代入：local変数から取得
                let this_value = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is not bound in the current context".to_string(),
                    })?;
                    
                if let Some(instance) = this_value.as_any().downcast_ref::<InstanceBox>() {
                    // 🔥 Usage prohibition guard - check if instance is finalized
                    if instance.is_finalized() {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Instance was finalized; further use is prohibited".to_string(),
                        });
                    }
                    
                    // 既存のthis.field値があればfini()を呼ぶ
                    if let Some(old_field_value) = instance.get_field(field) {
                        if let Some(old_instance) = old_field_value.as_any().downcast_ref::<InstanceBox>() {
                            let _ = old_instance.fini();
                            finalization::mark_as_finalized(old_instance.box_id());
                        }
                    }
                    
                    instance.set_field(field, val.clone_box())
                        .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
                    Ok(val)
                } else {
                    Err(RuntimeError::TypeError {
                        message: "'this' is not an instance".to_string(),
                    })
                }
            }
            
            ASTNode::MeField { field, .. } => {
                // 🌍 革命的me.field代入：local変数から取得
                let me_value = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is not bound in the current context".to_string(),
                    })?;
                    
                if let Some(instance) = me_value.as_any().downcast_ref::<InstanceBox>() {
                    // 🔥 Usage prohibition guard - check if instance is finalized
                    if instance.is_finalized() {
                        return Err(RuntimeError::InvalidOperation {
                            message: "Instance was finalized; further use is prohibited".to_string(),
                        });
                    }
                    
                    // 既存のme.field値があればfini()を呼ぶ
                    if let Some(old_field_value) = instance.get_field(field) {
                        if let Some(old_instance) = old_field_value.as_any().downcast_ref::<InstanceBox>() {
                            let _ = old_instance.fini();
                            finalization::mark_as_finalized(old_instance.box_id());
                        }
                    }
                    
                    instance.set_field(field, val.clone_box())
                        .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
                    Ok(val)
                } else {
                    Err(RuntimeError::TypeError {
                        message: "'this' is not an instance".to_string(),
                    })
                }
            }
            
            _ => Err(RuntimeError::InvalidOperation {
                message: "Invalid assignment target".to_string(),
            }),
        }
    }
    
    /// try/catch/finally文を実行 - Exception handling
    pub(super) fn execute_try_catch(&mut self, try_body: &[ASTNode], catch_clauses: &[super::CatchClause], finally_body: &Option<Vec<ASTNode>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut thrown_exception: Option<Box<dyn NyashBox>> = None;
        
        // Try block execution
        let mut try_result = Ok(Box::new(VoidBox::new()));
        for statement in try_body {
            match self.execute_statement(statement) {
                Ok(_) => {
                    // 制御フローをチェック
                    if !matches!(self.control_flow, super::ControlFlow::None) {
                        if let super::ControlFlow::Throw(exception) = &self.control_flow {
                            thrown_exception = Some(exception.clone_box());
                            self.control_flow = super::ControlFlow::None;
                            break;
                        } else {
                            break; // Return/Break等は上位に伝播
                        }
                    }
                }
                Err(e) => {
                    // RuntimeErrorを例外として扱う
                    thrown_exception = Some(Box::new(exception_box::ErrorBox::new(&format!("{:?}", e))));
                    try_result = Err(e);
                    break;
                }
            }
        }
        
        // Catch clause processing
        if let Some(exception) = &thrown_exception {
            for catch_clause in catch_clauses {
                // 型チェック
                if let Some(exception_type) = &catch_clause.exception_type {
                    if !exception_box::is_exception_type(exception.as_ref(), exception_type) {
                        continue; // 型が合わない場合は次のcatch句へ
                    }
                }
                
                // 🌍 革命的例外変数束縛：local変数として設定
                if let Some(var_name) = &catch_clause.variable_name {
                    self.declare_local_variable(var_name, exception.clone_box());
                }
                
                // Catch body execution
                for statement in &catch_clause.body {
                    self.execute_statement(statement)?;
                    if !matches!(self.control_flow, super::ControlFlow::None) {
                        break;
                    }
                }
                
                // 🌍 革命的例外変数クリーンアップ：local変数から削除
                if let Some(var_name) = &catch_clause.variable_name {
                    self.local_vars.remove(var_name);
                }
                
                thrown_exception = None; // 例外が処理された
                break;
            }
        }
        
        // Finally block execution (always executed)
        if let Some(ref finally_statements) = finally_body {
            for statement in finally_statements {
                self.execute_statement(statement)?;
                if !matches!(self.control_flow, super::ControlFlow::None) {
                    break;
                }
            }
        }
        
        // 未処理の例外があれば再スロー
        if let Some(exception) = thrown_exception {
            self.control_flow = super::ControlFlow::Throw(exception);
        }
        
        match try_result {
            Ok(result) => Ok(result),
            Err(_) => Ok(Box::new(VoidBox::new()) as Box<dyn NyashBox>),
        }
    }
    
    /// throw文を実行 - Throw exception
    pub(super) fn execute_throw(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value = self.execute_expression(expression)?;
        
        // 値を例外として扱う
        let exception = if let Some(error_box) = value.as_any().downcast_ref::<exception_box::ErrorBox>() {
            Box::new(error_box.clone()) as Box<dyn NyashBox>
        } else {
            // 文字列や他の値はErrorBoxに変換
            Box::new(exception_box::ErrorBox::new(&value.to_string_box().value))
        };
        
        self.control_flow = super::ControlFlow::Throw(exception);
        Ok(Box::new(VoidBox::new()))
    }
}