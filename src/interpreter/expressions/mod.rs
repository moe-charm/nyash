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
use crate::ast::UnaryOperator;
use crate::boxes::{buffer::BufferBox, JSONBox, HttpClientBox, StreamBox, RegexBox, IntentBox, SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox};
use crate::boxes::{FloatBox, MathBox, ConsoleBox, TimeBox, DateTimeBox, RandomBox, SoundBox, DebugBox, file::FileBox, MapBox};
use crate::box_trait::{BoolBox, SharedNyashBox};
// Direct implementation approach to avoid import issues
use crate::operator_traits::{DynamicAdd, DynamicSub, DynamicMul, DynamicDiv, OperatorError};

use std::sync::Arc;
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
                Ok((*shared_result).clone_box())  // Convert Arc to Box for external interface
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
                Ok((*shared_this).clone_box())  // Convert for external interface
            }
            
            ASTNode::Me { .. } => {
                
                // 🌍 革命的me解決：local変数から取得（thisと同じ）
                let shared_me = self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'me' is only available inside methods".to_string(),
                    })?;
                    
                Ok((*shared_me).clone_box())  // Convert for external interface
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
                    Ok((*shared_field).clone_box())  // Convert for external interface
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
                    Ok((*shared_field).clone_box())  // Convert for external interface
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
    
    
    
    /// フィールドアクセスを実行 - Field access processing with weak reference support
    pub(super) fn execute_field_access(&mut self, object: &ASTNode, field: &str) 
        -> Result<SharedNyashBox, RuntimeError> {
        
        // 🔥 Static Boxアクセスチェック
        if let ASTNode::Variable { name, .. } = object {
            // Static boxの可能性をチェック
            if self.is_static_box(name) {
                let static_result = self.execute_static_field_access(name, field)?;
                return Ok(Arc::from(static_result));
            }
        }
        
        
        // オブジェクトを評価（通常のフィールドアクセス）  
        let obj_value = self.execute_expression(object);
        
        let obj_value = obj_value?;
        
        // InstanceBoxにキャスト
        if let Some(instance) = obj_value.as_any().downcast_ref::<InstanceBox>() {
            // 🔥 Usage prohibition guard - check if instance is finalized
            if instance.is_finalized() {
                return Err(RuntimeError::InvalidOperation {
                    message: "Instance was finalized; further use is prohibited".to_string(),
                });
            }
            
            // フィールドの値を取得
            let field_value = instance.get_field(field)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Field '{}' not found in {}", field, instance.class_name),
                })?;
            
            eprintln!("✅ FIELD ACCESS: Returning shared reference id={}", field_value.box_id());
            
            // 🔗 Weak Reference Check: Use unified accessor for weak fields
            let box_decls = self.shared.box_declarations.read().unwrap();
            if let Some(box_decl) = box_decls.get(&instance.class_name) {
                if box_decl.weak_fields.contains(&field.to_string()) {
                    eprintln!("🔗 DEBUG: Accessing weak field '{}' in class '{}'", field, instance.class_name);
                    
                    // 🎯 PHASE 2: Use unified accessor for auto-nil weak reference handling
                    if let Some(weak_value) = instance.get_weak_field(field, self) { // Pass self
                        match &weak_value {
                            crate::value::NyashValue::Null => {
                                eprintln!("🔗 DEBUG: Weak field '{}' is null (reference dropped)", field);
                                // Return null box for compatibility
                                return Ok(Arc::new(crate::boxes::null_box::NullBox::new()));
                            }
                            _ => {
                                eprintln!("🔗 DEBUG: Weak field '{}' still has valid reference", field);
                                // Convert back to Box<dyn NyashBox> for now
                                if let Ok(box_value) = weak_value.to_box() {
                                    if let Ok(inner_box) = box_value.try_lock() {
                                        return Ok(Arc::from(inner_box.clone_box()));
                                    }
                                }
                            }
                        }
                    }
                    // If weak field access failed, fall through to normal access
                }
            }
            
            // Return the shared Arc reference directly
            Ok(field_value)
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
        let shared_field = instance.get_field(field)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Field '{}' not found in static box '{}'", field, static_box_name),
            })?;
        
        // Convert Arc to Box for compatibility
        Ok((*shared_field).clone_box())
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
    
    /// 🔗 Convert NyashBox to NyashValue for weak reference operations
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
    
    
    /// 🔥 ビルトインBoxのメソッド呼び出し
    fn execute_builtin_box_method(&mut self, parent: &str, method: &str, mut current_instance: Box<dyn NyashBox>, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🌟 Phase 8.9: birth method support for builtin boxes
        if method == "birth" {
            return self.execute_builtin_birth_method(parent, current_instance, arguments);
        }
        
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
            "SocketBox" => {
                let socket_box = SocketBox::new();
                self.execute_socket_method(&socket_box, method, arguments)
            }
            "HTTPServerBox" => {
                let http_server_box = HTTPServerBox::new();
                self.execute_http_server_method(&http_server_box, method, arguments)
            }
            "HTTPRequestBox" => {
                let http_request_box = HTTPRequestBox::new();
                self.execute_http_request_method(&http_request_box, method, arguments)
            }
            "HTTPResponseBox" => {
                let http_response_box = HTTPResponseBox::new();
                self.execute_http_response_method(&http_response_box, method, arguments)
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown built-in Box type for delegation: {}", parent),
                })
            }
        }
    }
    
    /// 🌟 Phase 8.9: Execute birth method for builtin boxes
    /// Provides constructor functionality for builtin boxes through explicit birth() calls
    fn execute_builtin_birth_method(&mut self, builtin_name: &str, current_instance: Box<dyn NyashBox>, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // ビルトインBoxの種類に応じて適切なインスタンスを作成して返す
        match builtin_name {
            "StringBox" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StringBox.birth() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                
                let content = arg_values[0].to_string_box().value;
                eprintln!("🌟 DEBUG: StringBox.birth() created with content: '{}'", content);
                let string_box = StringBox::new(content);
                Ok(Box::new(VoidBox::new())) // Return void to indicate successful initialization
            }
            "IntegerBox" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntegerBox.birth() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                
                let value = if let Ok(int_val) = arg_values[0].to_string_box().value.parse::<i64>() {
                    int_val
                } else {
                    return Err(RuntimeError::TypeError {
                        message: format!("Cannot convert '{}' to integer", arg_values[0].to_string_box().value),
                    });
                };
                
                let integer_box = IntegerBox::new(value);
                eprintln!("🌟 DEBUG: IntegerBox.birth() created with value: {}", value);
                Ok(Box::new(VoidBox::new()))
            }
            "MathBox" => {
                // MathBoxは引数なしのコンストラクタ
                if arg_values.len() != 0 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MathBox.birth() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                let math_box = MathBox::new();
                eprintln!("🌟 DEBUG: MathBox.birth() created");
                Ok(Box::new(VoidBox::new()))
            }
            "ArrayBox" => {
                // ArrayBoxも引数なしのコンストラクタ
                if arg_values.len() != 0 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ArrayBox.birth() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                let array_box = ArrayBox::new();
                eprintln!("🌟 DEBUG: ArrayBox.birth() created");
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                // 他のビルトインBoxは今後追加
                Err(RuntimeError::InvalidOperation {
                    message: format!("birth() method not yet implemented for builtin box '{}'", builtin_name),
                })
            }
        }
    }
}