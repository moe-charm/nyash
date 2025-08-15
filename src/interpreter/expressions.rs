/*!
 * Expression Processing Module
 * 
 * Clean dispatcher for expression evaluation with modular delegation
 * Handles expression evaluation by delegating to specialized modules
 * Core philosophy: "Everything is Box" with clean expression evaluation
 */

use super::*;
use crate::ast::UnaryOperator;
use crate::box_trait::{BoolBox, SharedNyashBox};
use std::sync::Arc;

impl NyashInterpreter {
    /// 式を実行 - Expression evaluation engine (main dispatcher)
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
                self.execute_method_call(object, method, arguments)
            }
            
            ASTNode::FieldAccess { object, field, .. } => {
                let shared_result = self.execute_field_access(object, field)?;
                Ok((*shared_result).clone_box())  // Convert Arc to Box for external interface
            }
            
            ASTNode::New { class, arguments, type_arguments, .. } => {
                self.execute_new(class, arguments, type_arguments)
            }
            
            ASTNode::This { .. } => {
                // 'this'は現在のインスタンスを指す（クラス内でのみ有効）
                if let Ok(shared_field) = self.resolve_variable("this") {
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
                message: format!("Unsupported expression type: {:?}", expression),
            }),
        }
    }
    
    // ==================== Delegation Wrappers ====================
    // These functions delegate to the appropriate specialized modules
    
    /// 二項演算を実行 - Binary operation processing (delegated to operators module)
    pub(super) fn execute_binary_op(&mut self, op: &BinaryOperator, left: &ASTNode, right: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::operators::NyashInterpreter::execute_binary_op(self, op, left, right)
    }
    
    /// 単項演算を実行 - Unary operation processing (delegated to operators module)
    pub(super) fn execute_unary_op(&mut self, operator: &UnaryOperator, operand: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::operators::NyashInterpreter::execute_unary_op(self, operator, operand)
    }
    
    /// メソッド呼び出しを実行 - Method call processing (delegated to method_dispatch module)
    pub(super) fn execute_method_call(&mut self, object: &ASTNode, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::method_dispatch::NyashInterpreter::execute_method_call(self, object, method, arguments)
    }
    
    /// フィールドアクセスを実行 - Field access processing (delegated to field_access module)
    pub(super) fn execute_field_access(&mut self, object: &ASTNode, field: &str) 
        -> Result<SharedNyashBox, RuntimeError> {
        super::field_access::NyashInterpreter::execute_field_access(self, object, field)
    }
    
    /// await式を実行 - Await expression processing (delegated to async_ops module)
    pub(super) fn execute_await(&mut self, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::async_ops::NyashInterpreter::execute_await(self, expression)
    }
    
    /// 🔥 FromCall実行処理 - Delegation call processing (delegated to delegation module)
    pub(super) fn execute_from_call(&mut self, parent: &str, method: &str, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::delegation::NyashInterpreter::execute_from_call(self, parent, method, arguments)
    }
}