/*!
 * Delegation Module
 * 
 * Extracted from expressions.rs for modular organization
 * Handles 'from' calls and delegation logic, including birth methods
 * Core philosophy: "Everything is Box" with clean parent delegation
 */

use super::*;
use crate::box_trait::{BoolBox, SharedNyashBox};

impl NyashInterpreter {
    /// 🔥 FromCall実行処理 - from Parent.method(arguments) or from Parent.constructor(arguments)
    pub(super) fn execute_from_call(&mut self, parent: &str, method: &str, arguments: &[ASTNode])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 1. 現在のコンテキストで'me'変数を取得（現在のインスタンス）
        let current_instance_val = self.resolve_variable("me")
            .map_err(|_| RuntimeError::InvalidOperation {
                message: "'from' can only be used inside methods".to_string(),
            })?;
        
        let current_instance = (*current_instance_val).as_any().downcast_ref::<InstanceBox>()
            .ok_or(RuntimeError::TypeError {
                message: "'from' requires current instance to be InstanceBox".to_string(),
            })?;
        
        // 2. 継承チェックを行う
        if !current_instance.inherits_from(parent) {
            return Err(RuntimeError::TypeError {
                message: format!("Class '{}' does not extend '{}'", current_instance.class_name, parent),
            });
        }
        
        // 3. 親クラスのBox宣言を取得
        let parent_box_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.get(parent).cloned()
        };
        
        if let Some(parent_box_decl) = parent_box_decl {
            // 4. 一般的なメソッドの場合はコンストラクタチェック
            if method == "init" || parent_box_decl.constructors.contains_key(method) {
                // コンストラクタの場合
                return self.execute_from_parent_constructor(parent, &parent_box_decl, current_instance_val.clone_box(), arguments);
            }
            
            // 5. 通常のメソッドの場合
            if let Some(method_impl) = parent_box_decl.methods.get(method) {
                // 親クラスのメソッドを現在のインスタンスで実行
                return self.execute_instance_method(current_instance, method_impl, arguments);
            }
            
            // メソッドが見つからない場合エラー
            Err(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in parent class '{}'", method, parent),
            })
        } else {
            // 3.1 ビルトインBox処理（StringBox, IntegerBox, ArrayBox等）
            eprintln!("🌟 DEBUG: Attempting builtin box method for '{}' with method '{}'", parent, method);
            return self.execute_builtin_box_method(parent, method, current_instance_val.clone_box(), arguments);
        }
    }

    /// 親クラスのコンストラクタを実行
    fn execute_from_parent_constructor(&mut self, parent: &str, parent_box_decl: &super::BoxDeclaration, 
                                      current_instance: Box<dyn NyashBox>, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // initコンストラクタを使用（標準）
        let constructor_impl = parent_box_decl.constructors.get("init")
            .or_else(|| parent_box_decl.constructors.get("pack"))  // pack も試行
            .cloned();
        
        if let Some(constructor_node) = constructor_impl {
            if let ASTNode::FunctionDeclaration { params, body, .. } = constructor_node {
                // パラメータ数チェック
                if arg_values.len() != params.len() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Constructor expects {} arguments, got {}", params.len(), arg_values.len()),
                    });
                }
                
                // 引数をlocal変数として設定
                for (param, value) in params.iter().zip(arg_values.iter()) {
                    self.declare_local_variable(param, value.clone_box());
                }
                
                // 'me'変数を現在のインスタンスに設定
                if let Some(instance) = current_instance.as_any().downcast_ref::<InstanceBox>() {
                    self.declare_local_variable("me", current_instance.clone_box());
                }
                
                // コンストラクタ本体を実行
                for stmt in body {
                    match self.execute_statement(stmt)? {
                        ControlFlow::Return(value) => return Ok(value),
                        ControlFlow::Break => break,
                        ControlFlow::Throw(error) => {
                            return Err(RuntimeError::CustomException { value: error });
                        }
                        ControlFlow::None => {}
                    }
                }
                
                Ok(current_instance)
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: "Invalid constructor declaration".to_string(),
                })
            }
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Constructor not found in parent class '{}'", parent),
            })
        }
    }

    /// ビルトインBoxに対するメソッド呼び出し処理
    fn execute_builtin_box_method(&mut self, parent: &str, method: &str, mut current_instance: Box<dyn NyashBox>, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 🌟 Phase 8.9: builtin box birth method processing
        eprintln!("🌟 DEBUG: Processing builtin box method '{}' for '{}'", method, parent);
        
        // pack呼び出しの場合（ビルトインBox用コンストラクタ）
        if method == "pack" {
            eprintln!("🌟 DEBUG: Processing pack method for builtin box '{}'", parent);
            
            // 引数を評価
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            
            // ビルトインBox特有のpack処理
            match parent {
                "P2PBox" => {
                    // P2PBox pack(nodeId, transport)
                    if arg_values.len() != 2 {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("P2PBox.pack() expects 2 arguments (nodeId, transport), got {}", arg_values.len()),
                        });
                    }
                    
                    let node_id = arg_values[0].to_string_box().value;
                    let transport = arg_values[1].to_string_box().value;
                    
                    eprintln!("🌟 DEBUG: P2PBox.pack() called with nodeId='{}', transport='{}'", node_id, transport);
                    
                    // P2PBoxの初期化
                    // 注: ここでは実際のP2P接続は行わず、構造だけ作成
                    Ok(Box::new(VoidBox::new())) // Void return for pack method
                }
                _ => {
                    Err(RuntimeError::InvalidOperation {
                        message: format!("pack() method not implemented for builtin box '{}'", parent),
                    })
                }
            }
        } else if method == "birth" {
            // 🌟 Phase 8.9: birth method for builtin boxes
            return self.execute_builtin_birth_method(parent, current_instance, arguments);
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in builtin box '{}'", method, parent),
            })
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