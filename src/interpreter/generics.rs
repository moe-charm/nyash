/*!
 * Generics Module
 * 
 * Extracted from objects.rs - handles generic type processing and specialization
 * Core responsibility: Generic class specialization and type substitution
 * Part of advanced type system in "Everything is Box" philosophy
 */

use super::*;

impl NyashInterpreter {
    /// 🚀 ジェネリクス型を特殊化してBoxDeclarationを生成
    pub(super) fn specialize_generic_class(
        &self, 
        generic_decl: &BoxDeclaration, 
        type_arguments: &[String]
    ) -> Result<BoxDeclaration, RuntimeError> {
        use std::collections::HashMap;
        
        // 特殊化されたクラス名を生成
        let specialized_name = format!(
            "{}_{}",
            generic_decl.name,
            type_arguments.join("_")
        );
        
        // 型パラメータ → 具体型のマッピングを作成
        let mut type_mapping = HashMap::new();
        for (i, param) in generic_decl.type_parameters.iter().enumerate() {
            type_mapping.insert(param.clone(), type_arguments[i].clone());
        }
        
        // 特殊化されたBoxDeclarationを作成
        let mut specialized = generic_decl.clone();
        specialized.name = specialized_name.clone();
        specialized.type_parameters.clear(); // 特殊化後は型パラメータなし
        
        // 🔄 フィールドの型を置換
        specialized.init_fields = self.substitute_types_in_fields(
            &specialized.init_fields, 
            &type_mapping
        );
        
        // 🔧 コンストラクタキーを新しいクラス名で更新
        let mut updated_constructors = HashMap::new();
        for (old_key, constructor_node) in &generic_decl.constructors {
            // "Container/1" -> "Container_IntegerBox/1" に変更
            if let Some(args_count) = old_key.split('/').nth(1) {
                let new_key = format!("{}/{}", specialized_name, args_count);
                updated_constructors.insert(new_key, constructor_node.clone());
            }
        }
        specialized.constructors = updated_constructors;
        
        // 🔄 メソッドの型を置換（現在はプレースホルダー実装）
        // TODO: メソッド内部のコードも置換が必要
        
        Ok(specialized)
    }
    
    /// フィールドの型置換
    pub(super) fn substitute_types_in_fields(
        &self,
        fields: &[String],
        _type_mapping: &HashMap<String, String>
    ) -> Vec<String> {
        // TODO: フィールド型の置換実装
        // 現在はシンプルにコピー
        fields.to_vec()
    }
}