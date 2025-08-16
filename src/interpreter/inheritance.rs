/*!
 * Inheritance Module
 * 
 * Extracted from objects.rs - handles inheritance and delegation processing
 * Core responsibility: Parent constructor execution and inheritance chain resolution
 * Part of complete OOP support in "Everything is Box" philosophy
 */

use super::*;

impl NyashInterpreter {
    /// 親コンストラクタを実行 - Parent constructor execution
    pub(super) fn execute_parent_constructor(&mut self, parent_class: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 親クラスの宣言を取得
        let parent_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.get(parent_class)
                .ok_or(RuntimeError::UndefinedClass { name: parent_class.to_string() })?
                .clone()
        };
            
        // 親コンストラクタを探す
        // まず "init/引数数" を試し、なければ "Box名/引数数" を試す
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", parent_class, arguments.len());
        
        if let Some(parent_constructor) = parent_decl.constructors.get(&init_key)
            .or_else(|| parent_decl.constructors.get(&box_name_key)) {
            // 現在のthis参照を取得
            // 🌍 革命的this取得：local変数から
            let this_instance = self.resolve_variable("me")
                .map_err(|_| RuntimeError::InvalidOperation {
                    message: "'this' not available in parent constructor call".to_string(),
                })?;
                
            // 親コンストラクタを実行
            self.execute_constructor(&this_instance, parent_constructor, arguments, &parent_decl)?;
            
            // VoidBoxを返す（コンストラクタ呼び出しは値を返さない）
            Ok(Box::new(VoidBox::new()))
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("No constructor found for parent class {} with {} arguments", parent_class, arguments.len()),
            })
        }
    }
    
    /// 継承チェーンを解決してフィールドとメソッドを収集 - Inheritance resolution
    pub(super) fn resolve_inheritance(&self, box_decl: &BoxDeclaration) 
        -> Result<(Vec<String>, HashMap<String, ASTNode>), RuntimeError> {
        let mut all_fields = Vec::new();
        let mut all_methods = HashMap::new();
        
        // 親クラスの継承チェーンを再帰的に解決 (Multi-delegation) 🚀
        for parent_name in &box_decl.extends {
            // 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定
            use crate::box_trait::is_builtin_box;
            
            let mut is_builtin = is_builtin_box(parent_name);
            
            // GUI機能が有効な場合はEguiBoxも追加判定
            #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
            {
                if parent_name == "EguiBox" {
                    is_builtin = true;
                }
            }
            
            if is_builtin {
                // ビルトインBoxの場合、フィールドやメソッドは継承しない
                // （ビルトインBoxのメソッドはfrom構文でアクセス可能）
            } else {
                let parent_decl = {
                    let box_decls = self.shared.box_declarations.read().unwrap();
                    box_decls.get(parent_name)
                        .ok_or(RuntimeError::UndefinedClass { name: parent_name.clone() })?
                        .clone()
                };
                
                // インターフェースは継承できない
                if parent_decl.is_interface {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot extend interface '{}'. Use 'implements' instead.", parent_name),
                    });
                }
                
                // 親クラスの継承チェーンを再帰的に解決
                let (parent_fields, parent_methods) = self.resolve_inheritance(&parent_decl)?;
                
                // 親のフィールドとメソッドを追加
                all_fields.extend(parent_fields);
                all_methods.extend(parent_methods);
            }
        }
        
        // 現在のクラスのフィールドとメソッドを追加（オーバーライド可能）
        all_fields.extend(box_decl.fields.clone());
        
        // init_fieldsも追加（重複チェック）
        for init_field in &box_decl.init_fields {
            if !all_fields.contains(init_field) {
                all_fields.push(init_field.clone());
            }
        }
        
        for (method_name, method_ast) in &box_decl.methods {
            all_methods.insert(method_name.clone(), method_ast.clone());  // オーバーライド
        }
        
        // インターフェース実装の検証
        for interface_name in &box_decl.implements {
            let interface_decl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                box_decls.get(interface_name)
                    .ok_or(RuntimeError::UndefinedClass { name: interface_name.clone() })?
                    .clone()
            };
            
            if !interface_decl.is_interface {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("'{}' is not an interface", interface_name),
                });
            }
            
            // インターフェースの全メソッドが実装されているかチェック
            for (required_method, _) in &interface_decl.methods {
                if !all_methods.contains_key(required_method) {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Class '{}' must implement method '{}' from interface '{}'", 
                                       box_decl.name, required_method, interface_name),
                    });
                }
            }
        }
        
        Ok((all_fields, all_methods))
    }
}