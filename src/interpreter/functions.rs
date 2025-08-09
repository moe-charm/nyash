/*!
 * Function Processing Module
 * 
 * Extracted from core.rs - function call and definition handling
 * Handles function declarations, calls, and function-related operations
 * Core philosophy: "Everything is Box" with structured function processing
 */

use super::*;

impl NyashInterpreter {
    /// 関数呼び出しを実行 - 🌍 革命的実装：GlobalBoxのメソッド呼び出し
    pub(super) fn execute_function_call(&mut self, name: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // コンストラクタ内での親コンストラクタ呼び出しチェック
        if let Some(context) = self.current_constructor_context.clone() {
            if let Some(parent_class) = context.parent_class {
                if name == parent_class {
                    // 親コンストラクタ呼び出し
                    return self.execute_parent_constructor(&parent_class, arguments);
                }
            }
        }
        
        // 🌍 GlobalBoxのメソッドとして実行
        let global_box = self.shared.global_box.lock().unwrap();
        let method_ast = global_box.get_method(name)
            .ok_or(RuntimeError::UndefinedFunction { name: name.to_string() })?
            .clone();
        drop(global_box);
        
        // メソッド呼び出しとして実行（GlobalBoxインスタンス上で）
        if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast {
            // 引数を評価
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            
            // パラメータ数チェック
            if arg_values.len() != params.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Function {} expects {} arguments, got {}", 
                                   name, params.len(), arg_values.len()),
                });
            }
            
            // 🌍 local変数スタックを保存・クリア（関数呼び出し開始）
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // パラメータをlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // 関数本体を実行
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
            
            // 🌍 local変数スタックを復元（関数呼び出し終了）
            self.restore_local_vars(saved_locals);
            
            Ok(result)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Function '{}' is not a valid function declaration", name),
            })
        }
    }
    
    /// 関数宣言を登録 - 🌍 革命的実装：GlobalBoxのメソッドとして登録
    pub(super) fn register_function_declaration(&mut self, name: String, params: Vec<String>, body: Vec<ASTNode>) {
        // 🌍 GlobalBoxのメソッドとして登録
        let func_ast = ASTNode::FunctionDeclaration {
            name: name.clone(),
            params,
            body,
            is_static: false,  // 通常の関数は静的でない
            span: crate::ast::Span::unknown(), // デフォルトspan
        };
        
        self.register_global_function(name, func_ast).unwrap_or_else(|err| {
            eprintln!("Warning: Failed to register global function: {}", err);
        });
    }
}