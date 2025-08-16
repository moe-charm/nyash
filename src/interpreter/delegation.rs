/*!
 * Delegation Processing Module
 * 
 * Extracted from expressions.rs lines 1086-1457 (~371 lines)
 * Handles 'from' calls, delegation validation, and builtin box method calls
 * Core philosophy: "Everything is Box" with explicit delegation
 */

use super::*;

impl NyashInterpreter {
    /// from呼び出しを実行 - 完全明示デリゲーション
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
        
        // 2. 現在のクラスのデリゲーション関係を検証
        let current_class = &current_instance.class_name;
        let box_declarations = self.shared.box_declarations.read().unwrap();
        
        let current_box_decl = box_declarations.get(current_class)
            .ok_or(RuntimeError::UndefinedClass { 
                name: current_class.clone() 
            })?;
        
        // extendsまたはimplementsでparentが指定されているか確認 (Multi-delegation) 🚀
        let is_valid_delegation = current_box_decl.extends.contains(&parent.to_string()) || 
                                 current_box_decl.implements.contains(&parent.to_string());
        
        if !is_valid_delegation {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Class '{}' does not delegate to '{}'. Use 'box {} from {}' to establish delegation.", 
                               current_class, parent, current_class, parent),
            });
        }
        
        // 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定
        use crate::box_trait::{is_builtin_box, BUILTIN_BOXES};
        
        let mut is_builtin = is_builtin_box(parent);
        
        // GUI機能が有効な場合はEguiBoxも追加判定
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        {
            if parent == "EguiBox" {
                is_builtin = true;
            }
        }
        
        // 🔥 Phase 8.9: Transparency system removed - all delegation must be explicit
        // ビルトインBoxの場合、専用メソッドで処理
        if is_builtin {
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
        
        // 4. constructorまたはinitまたはpackまたはbirthの場合の特別処理
        if method == "constructor" || method == "init" || method == "pack" || method == "birth" || method == parent {
            return self.execute_from_parent_constructor(parent, &parent_box_decl, current_instance_val.clone_box(), arguments);
        }
        
        // 5. 通常の親メソッド実行
        self.execute_parent_method(parent, method, &parent_box_decl, current_instance_val.clone_box(), arguments)
    }

    /// 親クラスのメソッドを実行
    fn execute_parent_method(
        &mut self, 
        parent: &str, 
        method: &str, 
        parent_box_decl: &super::BoxDeclaration, 
        current_instance_val: Box<dyn NyashBox>, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 親クラスのメソッドを取得
        let parent_method = parent_box_decl.methods.get(method)
            .ok_or(RuntimeError::InvalidOperation {
                message: format!("Method '{}' not found in parent class '{}'", method, parent),
            })?
            .clone();
        
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // 親メソッドを実行
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
        // "birth/引数数"、"pack/引数数"、"init/引数数"、"Box名/引数数" の順で試す
        let birth_key = format!("birth/{}", arguments.len());
        let pack_key = format!("pack/{}", arguments.len());
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", parent, arguments.len());
        
        let parent_constructor = parent_box_decl.constructors.get(&birth_key)
            .or_else(|| parent_box_decl.constructors.get(&pack_key))
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
        
        // 🌟 Phase 8.9: birth method support for builtin boxes
        if method == "birth" {
            return self.execute_builtin_birth_method(parent, current_instance, arguments);
        }
        
        // ビルトインBoxのインスタンスを作成または取得
        match parent {
            "StringBox" => {
                let string_box = StringBox::new("");
                self.execute_string_method(&string_box, method, arguments)
            }
            "IntegerBox" => {
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
            // 他のビルトインBoxは必要に応じて追加
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Builtin box '{}' method '{}' not implemented", parent, method),
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
            // 他のビルトインBoxは必要に応じて追加
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("birth() method not implemented for builtin box '{}'", builtin_name),
                })
            }
        }
    }
}