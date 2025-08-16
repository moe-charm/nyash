/*!
 * Object Processing Module
 * 
 * Extracted from core.rs - object creation, construction, and inheritance
 * Handles Box declarations, instantiation, constructors, and inheritance system
 * Core philosophy: "Everything is Box" with complete OOP support
 * 
 * 🔥 Modularized: This file now delegates to specialized modules for better maintainability:
 * - builtin_box_constructors.rs: Builtin Box instantiation
 * - type_validation.rs: Type checking and validation  
 * - inheritance.rs: Inheritance and delegation processing
 * - generics.rs: Generic type processing
 */

use super::*;
use std::sync::Arc;
use crate::box_trait::SharedNyashBox;

impl NyashInterpreter {
    /// new式を実行 - Object creation engine  
    pub(super) fn execute_new(&mut self, class: &str, arguments: &[ASTNode], type_arguments: &[String]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 🔥 Try builtin box constructors first (extracted to separate module)
        if let Some(builtin_box) = self.create_builtin_box_instance(class, arguments)? {
            return Ok(builtin_box);
        }
        
        // 🔥 Static Boxインスタンス化禁止チェック
        if self.is_static_box(class) {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Cannot instantiate static box '{}'. Static boxes cannot be instantiated.", class),
            });
        }
        
        // ユーザー定義Box宣言を探す
        let box_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.get(class)
                .ok_or(RuntimeError::UndefinedClass { name: class.to_string() })?
                .clone()
        };
        
        // 🔥 ジェネリクス型引数の検証
        if !box_decl.type_parameters.is_empty() || !type_arguments.is_empty() {
            self.validate_generic_arguments(&box_decl, type_arguments)?;
        }
        
        // インターフェースはインスタンス化できない
        if box_decl.is_interface {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Cannot instantiate interface '{}'", class),
            });
        }
        
        // 🚀 ジェネリクス型の特殊化処理
        let (final_box_decl, actual_class_name) = if !type_arguments.is_empty() {
            // ジェネリクス型を特殊化
            let specialized = self.specialize_generic_class(&box_decl, type_arguments)?;
            let specialized_name = specialized.name.clone();
            (specialized, specialized_name)
        } else {
            (box_decl.clone(), class.to_string())
        };
        
        // 継承チェーンを解決してフィールドとメソッドを収集（init_fieldsも含む）
        let (all_fields, all_methods) = self.resolve_inheritance(&final_box_decl)?;
        
        // 🔥 フィールド順序と weak フィールドを準備（finiシステム用）
        let init_field_order = final_box_decl.init_fields.clone();
        let weak_fields = final_box_decl.weak_fields.clone();
        
        // インスタンスを作成（Enhanced fini system対応）
        let instance = InstanceBox::new_with_box_info(
            actual_class_name.clone(),
            all_fields,
            all_methods,
            init_field_order,
            weak_fields
        );
        
        let instance_box = Box::new(instance) as Box<dyn NyashBox>;
        
        // 現在のスコープでBoxを追跡（自動解放のため）
        // 🌍 革命的実装：Environment tracking廃止
        
        // Create Arc outside if block so it's available in all scopes
        let instance_arc = Arc::from(instance_box);
        
        // コンストラクタを呼び出す
        // "birth/引数数"、"pack/引数数"、"init/引数数"、"Box名/引数数" の順で試す
        let birth_key = format!("birth/{}", arguments.len());
        let pack_key = format!("pack/{}", arguments.len());
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", actual_class_name, arguments.len());
        
        if let Some(constructor) = final_box_decl.constructors.get(&birth_key)
            .or_else(|| final_box_decl.constructors.get(&pack_key))
            .or_else(|| final_box_decl.constructors.get(&init_key))
            .or_else(|| final_box_decl.constructors.get(&box_name_key)) {
            // コンストラクタを実行
            self.execute_constructor(&instance_arc, constructor, arguments, &final_box_decl)?;
        } else if !arguments.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: format!("No constructor found for {} with {} arguments", class, arguments.len()),
            });
        }
        
        Ok((*instance_arc).clone_box())  // Convert Arc back to Box for external interface
    }
    
    /// コンストラクタを実行 - Constructor execution
    pub(super) fn execute_constructor(
        &mut self, 
        instance: &SharedNyashBox, 
        constructor: &ASTNode, 
        arguments: &[ASTNode],
        box_decl: &BoxDeclaration
    ) -> Result<(), RuntimeError> {
        if let ASTNode::FunctionDeclaration { name: _, params, body, .. } = constructor {
            // 引数を評価
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            
            // パラメータ数チェック
            if params.len() != arg_values.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Constructor expects {} arguments, got {}", params.len(), arg_values.len()),
                });
            }
            
            // 🌍 革命的コンストラクタ実行：local変数スタックを使用
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // パラメータをlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // this（me）をlocal変数として設定
            self.declare_local_variable("me", instance.clone_box());
            
            // コンストラクタコンテキストを設定
            let old_context = self.current_constructor_context.clone();
            self.current_constructor_context = Some(ConstructorContext {
                class_name: box_decl.name.clone(),
                parent_class: box_decl.extends.first().cloned(), // Use first parent for context
            });
            
            // コンストラクタを実行
            let mut result = Ok(());
            for statement in body.iter() {
                if let Err(e) = self.execute_statement(statement) {
                    result = Err(e);
                    break;
                }
            }
            
            // local変数スタックとコンテキストを復元
            self.restore_local_vars(saved_locals);
            self.current_constructor_context = old_context;
            
            result
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Invalid constructor node".to_string(),
            })
        }
    }
    
    /// Box宣言を登録 - 🔥 コンストラクタオーバーロード禁止対応
    pub(super) fn register_box_declaration(
        &mut self, 
        name: String, 
        fields: Vec<String>, 
        methods: HashMap<String, ASTNode>,
        constructors: HashMap<String, ASTNode>,
        init_fields: Vec<String>,
        weak_fields: Vec<String>,  // 🔗 weak修飾子が付いたフィールドのリスト
        is_interface: bool,
        extends: Vec<String>,  // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
        implements: Vec<String>,
        type_parameters: Vec<String>  // 🔥 ジェネリクス型パラメータ追加
    ) -> Result<(), RuntimeError> {
        
        // 🚨 コンストラクタオーバーロード禁止：複数コンストラクタ検出
        if constructors.len() > 1 {
            let constructor_names: Vec<String> = constructors.keys().cloned().collect();
            return Err(RuntimeError::InvalidOperation {
                message: format!(
                    "🚨 CONSTRUCTOR OVERLOAD FORBIDDEN: Box '{}' has {} constructors: [{}].\n\
                    🌟 Nyash's explicit philosophy: One Box, One Constructor!\n\
                    💡 Use different Box classes for different initialization patterns.\n\
                    📖 Example: UserBox, AdminUserBox, GuestUserBox instead of User(type)",
                    name, 
                    constructors.len(),
                    constructor_names.join(", ")
                )
            });
        }
        let box_decl = super::BoxDeclaration { 
            name: name.clone(), 
            fields, 
            methods,
            constructors,
            init_fields,
            weak_fields,  // 🔗 Add weak_fields to the construction
            is_interface,
            extends,
            implements,
            type_parameters, // 🔥 ジェネリクス型パラメータを正しく使用
        };
        
        {
            let mut box_decls = self.shared.box_declarations.write().unwrap();
            box_decls.insert(name, box_decl);
        }
        
        Ok(()) // 🔥 正常終了
    }
}