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
// Direct implementation approach to avoid import issues

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