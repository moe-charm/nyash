/*!
 * Expression Processing Module
 * 
 * Extracted from core.rs lines 408-787 (~380 lines)
 * Handles expression evaluation, binary operations, method calls, and field access
 * Core philosophy: "Everything is Box" with clean expression evaluation
 */

use super::*;
use crate::ast::UnaryOperator;
use crate::boxes::{buffer::BufferBox, JSONBox, HttpClientBox, StreamBox, RegexBox, IntentBox, P2PBox, FloatBox};
use crate::boxes::{MathBox, ConsoleBox, TimeBox, RandomBox, SoundBox, DebugBox, file::FileBox, MapBox};
use crate::box_trait::BoolBox;
use crate::operator_traits::OperatorResolver;
// TODO: Fix NullBox import issue later
// use crate::NullBox;

impl NyashInterpreter {
    /// 式を実行 - Expression evaluation engine
    pub(super) fn execute_expression(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match expression {
            ASTNode::Literal { value, .. } => {
                Ok(value.to_nyash_box())
            }
            
            ASTNode::Variable { name, .. } => {
                // 🌍 革命的変数解決：local変数 → GlobalBoxフィールド → エラー
                self.resolve_variable(name)
                    .map_err(|_| RuntimeError::UndefinedVariableAt { 
                        name: name.clone(), 
                        span: expression.span() 
                    })
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
                self.execute_field_access(object, field)
            }
            
            ASTNode::New { class, arguments, type_arguments, .. } => {
                self.execute_new(class, arguments, type_arguments)
            }
            
            ASTNode::This { .. } => {
                // 🌍 革命的this解決：local変数から取得
                self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is only available inside methods".to_string(),
                    })
            }
            
            ASTNode::Me { .. } => {
                
                // 🌍 革命的me解決：local変数から取得（thisと同じ）
                let result = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'me' is only available inside methods".to_string(),
                    });
                    
                result
            }
            
            ASTNode::ThisField { field, .. } => {
                // 🌍 革命的this.fieldアクセス：local変数から取得
                let this_value = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' is not bound in the current context".to_string(),
                    })?;
                
                if let Some(instance) = this_value.as_any().downcast_ref::<InstanceBox>() {
                    instance.get_field(field)
                        .ok_or_else(|| RuntimeError::InvalidOperation { 
                            message: format!("Field '{}' not found on this", field)
                        })
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
                
                if let Some(instance) = me_value.as_any().downcast_ref::<InstanceBox>() {
                    instance.get_field(field)
                        .ok_or_else(|| RuntimeError::InvalidOperation { 
                            message: format!("Field '{}' not found on me", field)
                        })
                } else {
                    Err(RuntimeError::TypeError {
                        message: "'this' is not an instance".to_string(),
                    })
                }
            }
            
            ASTNode::FunctionCall { name, arguments, .. } => {
                self.execute_function_call(name, arguments)
            }
            
            ASTNode::Arrow { sender, receiver, .. } => {
                self.execute_arrow(sender, receiver)
            }
            
            ASTNode::Include { filename, .. } => {
                self.execute_include(filename)?;
                Ok(Box::new(VoidBox::new()))
            }
            
            ASTNode::FromCall { parent, method, arguments, .. } => {
                self.execute_from_call(parent, method, arguments)
            }
            
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Cannot execute {:?} as expression", expression.node_type()),
            }),
        }
    }
    
    /// 二項演算を実行 - Binary operation processing
    pub(super) fn execute_binary_op(&mut self, op: &BinaryOperator, left: &ASTNode, right: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let left_val = self.execute_expression(left)?;
        let right_val = self.execute_expression(right)?;
        
        match op {
            BinaryOperator::Add => {
                // 🚀 New trait-based operator resolution system!
                OperatorResolver::resolve_add(left_val.as_ref(), right_val.as_ref())
                    .map_err(|e| RuntimeError::InvalidOperation { message: e.to_string() })
            }
            
            BinaryOperator::Equal => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::NotEqual => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(BoolBox::new(!result.value)))
            }
            
            BinaryOperator::And => {
                let left_bool = self.is_truthy(&left_val);
                if !left_bool {
                    Ok(Box::new(BoolBox::new(false)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
            
            BinaryOperator::Or => {
                let left_bool = self.is_truthy(&left_val);
                if left_bool {
                    Ok(Box::new(BoolBox::new(true)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }
            
            BinaryOperator::Subtract => {
                // 🚀 New trait-based subtraction
                OperatorResolver::resolve_sub(left_val.as_ref(), right_val.as_ref())
                    .map_err(|e| RuntimeError::InvalidOperation { message: e.to_string() })
            }
            
            BinaryOperator::Multiply => {
                // 🚀 New trait-based multiplication
                OperatorResolver::resolve_mul(left_val.as_ref(), right_val.as_ref())
                    .map_err(|e| RuntimeError::InvalidOperation { message: e.to_string() })
            }
            
            BinaryOperator::Divide => {
                // 🚀 New trait-based division
                OperatorResolver::resolve_div(left_val.as_ref(), right_val.as_ref())
                    .map_err(|e| RuntimeError::InvalidOperation { message: e.to_string() })
            }
            
            BinaryOperator::Less => {
                let result = CompareBox::less(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::Greater => {
                let result = CompareBox::greater(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::LessEqual => {
                let result = CompareBox::less_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
            
            BinaryOperator::GreaterEqual => {
                let result = CompareBox::greater_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
        }
    }
    
    /// 単項演算を実行 - Unary operation processing
    pub(super) fn execute_unary_op(&mut self, operator: &UnaryOperator, operand: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        let operand_val = self.execute_expression(operand)?;
        
        match operator {
            UnaryOperator::Minus => {
                // 数値の符号反転
                if let Some(int_box) = operand_val.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(-int_box.value)))
                } else if let Some(float_box) = operand_val.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(-float_box.value)))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "Unary minus can only be applied to Integer or Float".to_string(),
                    })
                }
            }
            UnaryOperator::Not => {
                // 論理否定
                if let Some(bool_box) = operand_val.as_any().downcast_ref::<BoolBox>() {
                    Ok(Box::new(BoolBox::new(!bool_box.value)))
                } else {
                    // どんな値でもtruthyness判定してnot演算を適用
                    let is_truthy = self.is_truthy(&operand_val);
                    Ok(Box::new(BoolBox::new(!is_truthy)))
                }
            }
        }
    }
    
    /// メソッド呼び出しを実行 - Method call processing
    pub(super) fn execute_method_call(&mut self, object: &ASTNode, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🔥 static関数のチェック
        if let ASTNode::Variable { name, .. } = object {
            // static関数が存在するかチェック
            let static_func = {
                let static_funcs = self.shared.static_functions.read().unwrap();
                if let Some(box_statics) = static_funcs.get(name) {
                    if let Some(func) = box_statics.get(method) {
                        Some(func.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            if let Some(static_func) = static_func {
                // static関数を実行
                if let ASTNode::FunctionDeclaration { params, body, .. } = static_func {
                        // 引数を評価
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.execute_expression(arg)?);
                        }
                        
                        // パラメータ数チェック
                        if arg_values.len() != params.len() {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!("Static method {}.{} expects {} arguments, got {}", 
                                               name, method, params.len(), arg_values.len()),
                            });
                        }
                        
                        // 🌍 local変数スタックを保存・クリア（static関数呼び出し開始）
                        let saved_locals = self.save_local_vars();
                        self.local_vars.clear();
                        
                        // 📤 outbox変数スタックも保存・クリア（static関数専用）
                        let saved_outbox = self.save_outbox_vars();
                        self.outbox_vars.clear();
                        
                        // 引数をlocal変数として設定
                        for (param, value) in params.iter().zip(arg_values.iter()) {
                            self.declare_local_variable(param, value.clone_box());
                        }
                        
                        // static関数の本体を実行
                        let mut result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
                        for statement in &body {
                            result = self.execute_statement(statement)?;
                            
                            // return文チェック
                            if let super::ControlFlow::Return(return_val) = &self.control_flow {
                                result = return_val.clone_box();
                                self.control_flow = super::ControlFlow::None;
                                break;
                            }
                        }
                        
                        // local変数スタックを復元
                        self.restore_local_vars(saved_locals);
                        
                        // outbox変数スタックを復元
                        self.restore_outbox_vars(saved_outbox);
                        
                        return Ok(result);
                }
            }
        }
        
        // オブジェクトを評価（通常のメソッド呼び出し）
        let obj_value = self.execute_expression(object)?;
        
        // StringBox method calls
        if let Some(string_box) = obj_value.as_any().downcast_ref::<StringBox>() {
            return self.execute_string_method(string_box, method, arguments);
        }
        
        // IntegerBox method calls
        if let Some(integer_box) = obj_value.as_any().downcast_ref::<IntegerBox>() {
            return self.execute_integer_method(integer_box, method, arguments);
        }
        
        // FloatBox method calls
        if let Some(float_box) = obj_value.as_any().downcast_ref::<FloatBox>() {
            return self.execute_float_method(float_box, method, arguments);
        }
        
        // BoolBox method calls
        if let Some(bool_box) = obj_value.as_any().downcast_ref::<BoolBox>() {
            return self.execute_bool_method(bool_box, method, arguments);
        }
        
        // ArrayBox method calls  
        if let Some(array_box) = obj_value.as_any().downcast_ref::<ArrayBox>() {
            return self.execute_array_method(array_box, method, arguments);
        }
        
        // BufferBox method calls
        if let Some(buffer_box) = obj_value.as_any().downcast_ref::<BufferBox>() {
            return self.execute_buffer_method(buffer_box, method, arguments);
        }
        
        // FileBox method calls
        if let Some(file_box) = obj_value.as_any().downcast_ref::<crate::boxes::file::FileBox>() {
            return self.execute_file_method(file_box, method, arguments);
        }
        
        // ResultBox method calls
        if let Some(result_box) = obj_value.as_any().downcast_ref::<ResultBox>() {
            return self.execute_result_method(result_box, method, arguments);
        }
        
        // FutureBox method calls
        if let Some(future_box) = obj_value.as_any().downcast_ref::<FutureBox>() {
            return self.execute_future_method(future_box, method, arguments);
        }
        
        // ChannelBox method calls
        if let Some(channel_box) = obj_value.as_any().downcast_ref::<ChannelBox>() {
            return self.execute_channel_method(channel_box, method, arguments);
        }
        
        // JSONBox method calls
        if let Some(json_box) = obj_value.as_any().downcast_ref::<JSONBox>() {
            return self.execute_json_method(json_box, method, arguments);
        }
        
        // HttpClientBox method calls
        if let Some(http_box) = obj_value.as_any().downcast_ref::<HttpClientBox>() {
            return self.execute_http_method(http_box, method, arguments);
        }
        
        // StreamBox method calls
        if let Some(stream_box) = obj_value.as_any().downcast_ref::<StreamBox>() {
            return self.execute_stream_method(stream_box, method, arguments);
        }
        
        // RegexBox method calls
        if let Some(regex_box) = obj_value.as_any().downcast_ref::<RegexBox>() {
            return self.execute_regex_method(regex_box, method, arguments);
        }
        
        // MathBox method calls
        if let Some(math_box) = obj_value.as_any().downcast_ref::<MathBox>() {
            return self.execute_math_method(math_box, method, arguments);
        }
        
        // NullBox method calls
        if let Some(null_box) = obj_value.as_any().downcast_ref::<crate::boxes::null_box::NullBox>() {
            return self.execute_null_method(null_box, method, arguments);
        }
        
        // TimeBox method calls
        if let Some(time_box) = obj_value.as_any().downcast_ref::<TimeBox>() {
            return self.execute_time_method(time_box, method, arguments);
        }
        
        // DateTimeBox method calls
        if let Some(datetime_box) = obj_value.as_any().downcast_ref::<DateTimeBox>() {
            return self.execute_datetime_method(datetime_box, method, arguments);
        }
        
        // TimerBox method calls
        if let Some(timer_box) = obj_value.as_any().downcast_ref::<TimerBox>() {
            return self.execute_timer_method(timer_box, method, arguments);
        }
        
        // MapBox method calls
        if let Some(map_box) = obj_value.as_any().downcast_ref::<MapBox>() {
            return self.execute_map_method(map_box, method, arguments);
        }
        
        // RandomBox method calls
        if let Some(random_box) = obj_value.as_any().downcast_ref::<RandomBox>() {
            return self.execute_random_method(random_box, method, arguments);
        }
        
        // SoundBox method calls
        if let Some(sound_box) = obj_value.as_any().downcast_ref::<SoundBox>() {
            return self.execute_sound_method(sound_box, method, arguments);
        }
        
        // DebugBox method calls
        if let Some(debug_box) = obj_value.as_any().downcast_ref::<DebugBox>() {
            return self.execute_debug_method(debug_box, method, arguments);
        }
        
        // ConsoleBox method calls
        if let Some(console_box) = obj_value.as_any().downcast_ref::<crate::boxes::console_box::ConsoleBox>() {
            return self.execute_console_method(console_box, method, arguments);
        }
        
        // IntentBox method calls
        if let Some(intent_box) = obj_value.as_any().downcast_ref::<IntentBox>() {
            return self.execute_intent_box_method(intent_box, method, arguments);
        }
        
        // P2PBox method calls
        if let Some(p2p_box) = obj_value.as_any().downcast_ref::<P2PBox>() {
            return self.execute_p2p_box_method(p2p_box, method, arguments);
        }
        
        // EguiBox method calls (非WASM環境のみ)
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(egui_box) = obj_value.as_any().downcast_ref::<crate::boxes::EguiBox>() {
            return self.execute_egui_method(egui_box, method, arguments);
        }
        
        // WebDisplayBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_display_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebDisplayBox>() {
            return self.execute_web_display_method(web_display_box, method, arguments);
        }
        
        // WebConsoleBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_console_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebConsoleBox>() {
            return self.execute_web_console_method(web_console_box, method, arguments);
        }
        
        // WebCanvasBox method calls (WASM環境のみ)
        #[cfg(target_arch = "wasm32")]
        if let Some(web_canvas_box) = obj_value.as_any().downcast_ref::<crate::boxes::WebCanvasBox>() {
            return self.execute_web_canvas_method(web_canvas_box, method, arguments);
        }
        
        // MethodBox method calls
        if let Some(method_box) = obj_value.as_any().downcast_ref::<crate::method_box::MethodBox>() {
            return self.execute_method_box_method(method_box, method, arguments);
        }
        
        // IntegerBox method calls  
        if let Some(integer_box) = obj_value.as_any().downcast_ref::<IntegerBox>() {
            return self.execute_integer_method(integer_box, method, arguments);
        }
        
        // FloatBox method calls (将来的に追加予定)
        
        // RangeBox method calls (将来的に追加予定)
        
        // InstanceBox method calls
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // fini()は特別処理
            if method == "fini" {
                // 既に解放済みの場合は何もしない（二重fini()対策）
                if instance.is_finalized() {
                    return Ok(Box::new(VoidBox::new()));
                }
                
                // まず、Box内で定義されたfini()メソッドがあれば実行
                if let Some(fini_method) = instance.get_method("fini") {
                    if let ASTNode::FunctionDeclaration { body, .. } = fini_method.clone() {
                        // 🌍 革命的メソッド実行：local変数スタックを使用
                        let saved_locals = self.save_local_vars();
                        self.local_vars.clear();
                        
                        // thisをlocal変数として設定
                        self.declare_local_variable("me", obj_value.clone_box());
                        
                        // fini()メソッドの本体を実行
                        let mut _result = Box::new(VoidBox::new()) as Box<dyn NyashBox>;
                        for statement in &body {
                            _result = self.execute_statement(statement)?;
                            
                            // return文チェック
                            if let super::ControlFlow::Return(_) = &self.control_flow {
                                self.control_flow = super::ControlFlow::None;
                                break;
                            }
                        }
                        
                        // local変数スタックを復元
                        self.restore_local_vars(saved_locals);
                    }
                }
                
                // インスタンスの内部的な解放処理
                instance.fini().map_err(|e| RuntimeError::InvalidOperation {
                    message: e,
                })?;
                finalization::mark_as_finalized(instance.box_id());
                return Ok(Box::new(VoidBox::new()));
            }
            
            // メソッドを取得
            let method_ast = instance.get_method(method)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Method '{}' not found in {}", method, instance.class_name),
                })?
                .clone();
            
            // メソッドが関数宣言の形式であることを確認
            if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast {
                // 🚨 FIX: 引数評価を完全に現在のコンテキストで完了させる
                let mut arg_values = Vec::new();
                for (i, arg) in arguments.iter().enumerate() {
                    let arg_value = self.execute_expression(arg)?;
                    arg_values.push(arg_value);
                }
                
                // パラメータ数チェック
                if arg_values.len() != params.len() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Method {} expects {} arguments, got {}", 
                                       method, params.len(), arg_values.len()),
                    });
                }
                
                // 🌍 NOW SAFE: すべての引数評価完了後にコンテキスト切り替え
                let saved_locals = self.save_local_vars();
                self.local_vars.clear();
                
                // thisをlocal変数として設定
                self.declare_local_variable("me", obj_value.clone_box());
                
                // パラメータをlocal変数として設定
                for (param, value) in params.iter().zip(arg_values.iter()) {
                    self.declare_local_variable(param, value.clone_box());
                }
                
                // メソッド本体を実行
                let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
                for statement in &body {
                    result = self.execute_statement(statement)?;
                    
                    // return文チェック
                    if let super::ControlFlow::Return(return_val) = &self.control_flow {
                        result = return_val.clone_box();
                        self.control_flow = super::ControlFlow::None;
                        break;
                    }
                }
                
                // local変数スタックを復元
                self.restore_local_vars(saved_locals);
                
                Ok(result)
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Method '{}' is not a valid function declaration", method),
                })
            }
        } else {
            Err(RuntimeError::TypeError {
                message: format!("Cannot call method '{}' on non-instance type", method),
            })
        }
    }
    
    /// フィールドアクセスを実行 - Field access processing
    pub(super) fn execute_field_access(&mut self, object: &ASTNode, field: &str) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🔥 Static Boxアクセスチェック
        if let ASTNode::Variable { name, .. } = object {
            // Static boxの可能性をチェック
            if self.is_static_box(name) {
                return self.execute_static_field_access(name, field);
            }
        }
        
        
        // オブジェクトを評価（通常のフィールドアクセス）  
        let obj_value = self.execute_expression(object);
        
        let obj_value = obj_value?;
        
        // InstanceBoxにキャスト
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // フィールドの値を取得
            instance.get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in {}", field, instance.class_name),
                })
        } else {
            Err(RuntimeError::TypeError {
                message: format!("Cannot access field '{}' on non-instance type. Type: {}", field, obj_value.type_name()),
            })
        }
    }
    
    /// 🔥 Static Box名前空間のフィールドアクセス
    fn execute_static_field_access(&mut self, static_box_name: &str, field: &str) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 1. Static Boxの初期化を確実に実行
        self.ensure_static_box_initialized(static_box_name)?;
        
        // 2. GlobalBox.statics.{static_box_name} からインスタンスを取得
        let global_box = self.shared.global_box.lock()
            .map_err(|_| RuntimeError::RuntimeFailure {
                message: "Failed to acquire global box lock".to_string()
            })?;
            
        let statics_box = global_box.get_field("statics")
            .ok_or(RuntimeError::RuntimeFailure {
                message: "statics namespace not found in GlobalBox".to_string()
            })?;
            
        let statics_instance = statics_box.as_any()
            .downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: "statics field is not an InstanceBox".to_string()
            })?;
            
        let static_box_instance = statics_instance.get_field(static_box_name)
            .ok_or(RuntimeError::RuntimeFailure {
                message: format!("Static box '{}' instance not found in statics namespace", static_box_name)
            })?;
            
        let instance = static_box_instance.as_any()
            .downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: format!("Static box '{}' is not an InstanceBox", static_box_name)
            })?;
        
        // 3. フィールドアクセス
        instance.get_field(field)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Field '{}' not found in static box '{}'", field, static_box_name),
            })
    }
    
    
    /// await式を実行 - Execute await expression
    pub(super) fn execute_await(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value = self.execute_expression(expression)?;
        
        // FutureBoxなら待機して結果を取得
        if let Some(future) = value.as_any().downcast_ref::<FutureBox>() {
            future.wait_and_get()
                .map_err(|msg| RuntimeError::InvalidOperation { message: msg })
        } else {
            // FutureBoxでなければそのまま返す
            Ok(value)
        }
    }
    
    /// 🔄 循環参照検出: オブジェクトの一意IDを取得
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
    fn hash_string(&self, s: &str) -> usize {
        let mut hash = 0usize;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }
    
    /// 🔥 FromCall実行処理 - from Parent.method(arguments) or from Parent.constructor(arguments)
    pub(super) fn execute_from_call(&mut self, parent: &str, method: &str, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 1. 現在のコンテキストで'me'変数を取得（現在のインスタンス）
        let current_instance_val = self.resolve_variable("me")
            .map_err(|_| RuntimeError::InvalidOperation {
                message: "'from' can only be used inside methods".to_string(),
            })?;
        
        let current_instance = current_instance_val.as_any().downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: "'from' requires current instance to be InstanceBox".to_string(),
            })?;
        
        // 2. 現在のクラスのデリゲーション関係を検証
        let current_class = &current_instance.class_name;
        let box_declarations = self.shared.box_declarations.read().unwrap();
        
        let current_box_decl = box_declarations.get(current_class)
            .ok_or(RuntimeError::UndefinedClass { 
                name: current_class.clone() 
            })?;
        
        // extendsまたはimplementsでparentが指定されているか確認
        let is_valid_delegation = current_box_decl.extends.as_ref().map(|s| s.as_str()) == Some(parent) || 
                                 current_box_decl.implements.contains(&parent.to_string());
        
        if !is_valid_delegation {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Class '{}' does not delegate to '{}'. Use 'box {} from {}' to establish delegation.", 
                               current_class, parent, current_class, parent),
            });
        }
        
        // 🔥 ビルトインBoxかチェック
        let is_builtin = matches!(parent, 
            "IntegerBox" | "StringBox" | "BoolBox" | "ArrayBox" | "MapBox" | 
            "FileBox" | "ResultBox" | "FutureBox" | "ChannelBox" | "MathBox" | 
            "TimeBox" | "DateTimeBox" | "TimerBox" | "RandomBox" | "SoundBox" | 
            "DebugBox" | "MethodBox" | "NullBox" | "ConsoleBox" | "FloatBox" |
            "BufferBox" | "RegexBox" | "JSONBox" | "StreamBox" | "HTTPClientBox" |
            "IntentBox" | "P2PBox" | "EguiBox"
        );
        
        if is_builtin {
            // ビルトインBoxの場合、ロックを解放してからメソッド呼び出し
            drop(box_declarations);
            return self.execute_builtin_box_method(parent, method, current_instance_val.clone_box(), arguments);
        }
        
        // 3. 親クラスのBox宣言を取得（ユーザー定義Boxの場合）
        let parent_box_decl = box_declarations.get(parent)
            .ok_or(RuntimeError::UndefinedClass { 
                name: parent.to_string() 
            })?
            .clone();
        
        drop(box_declarations); // ロック早期解放
        
        // 4. constructorまたはinitまたはpackの場合の特別処理
        if method == "constructor" || method == "init" || method == "pack" || method == parent {
            return self.execute_from_parent_constructor(parent, &parent_box_decl, current_instance_val.clone_box(), arguments);
        }
        
        // 5. 親クラスのメソッドを取得
        let parent_method = parent_box_decl.methods.get(method)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in parent class '{}'", method, parent),
            })?
            .clone();
        
        // 6. 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // 7. 親メソッドを実行
        if let ASTNode::FunctionDeclaration { params, body, .. } = parent_method {
            // パラメータ数チェック
            if arg_values.len() != params.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parent method {}.{} expects {} arguments, got {}", 
                                   parent, method, params.len(), arg_values.len()),
                });
            }
            
            // 🌍 local変数スタックを保存・クリア（親メソッド実行開始）
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // 'me'を現在のインスタンスに設定（重要：現在のインスタンスを維持）
            self.declare_local_variable("me", current_instance_val.clone_box());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // 親メソッドの本体を実行
            let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
            for statement in &body {
                result = self.execute_statement(statement)?;
                
                // return文チェック
                if let super::ControlFlow::Return(return_val) = &self.control_flow {
                    result = return_val.clone_box();
                    self.control_flow = super::ControlFlow::None;
                    break;
                }
            }
            
            // 🔍 DEBUG: FromCall実行結果をログ出力
            eprintln!("🔍 DEBUG: FromCall {}.{} result: {}", parent, method, result.to_string_box().value);
            
            // local変数スタックを復元
            self.restore_local_vars(saved_locals);
            
            Ok(result)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Parent method '{}' is not a valid function declaration", method),
            })
        }
    }
    
    /// 🔥 fromCall専用親コンストラクタ実行処理 - from Parent.constructor(arguments)
    fn execute_from_parent_constructor(&mut self, parent: &str, parent_box_decl: &super::BoxDeclaration, 
                                       current_instance: Box<dyn NyashBox>, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 1. 親クラスのコンストラクタを取得（引数の数でキーを作成）
        // "pack/引数数"、"init/引数数"、"Box名/引数数" の順で試す
        let pack_key = format!("pack/{}", arguments.len());
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", parent, arguments.len());
        
        let parent_constructor = parent_box_decl.constructors.get(&pack_key)
            .or_else(|| parent_box_decl.constructors.get(&init_key))
            .or_else(|| parent_box_decl.constructors.get(&box_name_key))
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("No constructor found for parent class '{}' with {} arguments", parent, arguments.len()),
            })?
            .clone();
        
        // 2. 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // 3. 親コンストラクタを実行
        if let ASTNode::FunctionDeclaration { params, body, .. } = parent_constructor {
            // パラメータ数チェック
            if arg_values.len() != params.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Parent constructor {} expects {} arguments, got {}", 
                                   parent, params.len(), arg_values.len()),
                });
            }
            
            // 🌍 local変数スタックを保存・クリア（親コンストラクタ実行開始）
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // 'me'を現在のインスタンスに設定
            self.declare_local_variable("me", current_instance.clone_box());
            
            // 引数をlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // 親コンストラクタの本体を実行
            let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());
            for statement in &body {
                result = self.execute_statement(statement)?;
                
                // return文チェック
                if let super::ControlFlow::Return(return_val) = &self.control_flow {
                    result = return_val.clone_box();
                    self.control_flow = super::ControlFlow::None;
                    break;
                }
            }
            
            // local変数スタックを復元
            self.restore_local_vars(saved_locals);
            
            // 親コンストラクタは通常現在のインスタンスを返す
            Ok(current_instance)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Parent constructor is not a valid function declaration"),
            })
        }
    }
    
    /// 🔥 ビルトインBoxのメソッド呼び出し
    fn execute_builtin_box_method(&mut self, parent: &str, method: &str, mut current_instance: Box<dyn NyashBox>, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // ビルトインBoxのインスタンスを作成または取得
        // 現在のインスタンスからビルトインBoxのデータを取得し、ビルトインBoxとしてメソッド実行
        
        match parent {
            "StringBox" => {
                // StringBoxのインスタンスを作成（デフォルト値）
                let string_box = StringBox::new("");
                self.execute_string_method(&string_box, method, arguments)
            }
            "IntegerBox" => {
                // IntegerBoxのインスタンスを作成（デフォルト値）
                let integer_box = IntegerBox::new(0);
                self.execute_integer_method(&integer_box, method, arguments)
            }
            "ArrayBox" => {
                let array_box = ArrayBox::new();
                self.execute_array_method(&array_box, method, arguments)
            }
            "MapBox" => {
                let map_box = MapBox::new();
                self.execute_map_method(&map_box, method, arguments)
            }
            "MathBox" => {
                let math_box = MathBox::new();
                self.execute_math_method(&math_box, method, arguments)
            }
            "P2PBox" => {
                // P2PBoxの場合、現在のインスタンスからP2PBoxインスタンスを取得する必要がある
                // TODO: 現在のインスタンスのフィールドからP2PBoxを取得
                return Err(RuntimeError::InvalidOperation {
                    message: format!("P2PBox delegation not yet fully implemented: {}.{}", parent, method),
                });
            }
            "FileBox" => {
                let file_box = crate::boxes::file::FileBox::new();
                self.execute_file_method(&file_box, method, arguments)
            }
            "ConsoleBox" => {
                let console_box = ConsoleBox::new();
                self.execute_console_method(&console_box, method, arguments)
            }
            "TimeBox" => {
                let time_box = TimeBox::new();
                self.execute_time_method(&time_box, method, arguments)
            }
            "RandomBox" => {
                let random_box = RandomBox::new();
                self.execute_random_method(&random_box, method, arguments)
            }
            "DebugBox" => {
                let debug_box = DebugBox::new();
                self.execute_debug_method(&debug_box, method, arguments)
            }
            "SoundBox" => {
                let sound_box = SoundBox::new();
                self.execute_sound_method(&sound_box, method, arguments)
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown built-in Box type for delegation: {}", parent),
                })
            }
        }
    }
}