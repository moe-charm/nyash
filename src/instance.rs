/*!
 * Nyash Instance System - Box Instance Implementation
 * 
 * BoxインスタンスとClassBoxの実装
 * Everything is Box哲学に基づくオブジェクト指向システム
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, VoidBox};
use crate::ast::ASTNode;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Boxインスタンス - フィールドとメソッドを持つオブジェクト
#[derive(Debug, Clone)]
pub struct InstanceBox {
    /// クラス名
    pub class_name: String,
    
    /// フィールド値
    pub fields: Arc<Mutex<HashMap<String, Box<dyn NyashBox>>>>,
    
    /// メソッド定義（ClassBoxから共有）
    pub methods: Arc<HashMap<String, ASTNode>>,
    
    /// インスタンスID
    id: u64,
    
    /// 解放済みフラグ
    finalized: Arc<Mutex<bool>>,
}

impl InstanceBox {
    pub fn new(class_name: String, fields: Vec<String>, methods: HashMap<String, ASTNode>) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        // フィールドをVoidBoxで初期化
        let mut field_map = HashMap::new();
        for field in fields {
            field_map.insert(field, Box::new(VoidBox::new()) as Box<dyn NyashBox>);
        }
        
        Self {
            class_name,
            fields: Arc::new(Mutex::new(field_map)),
            methods: Arc::new(methods),
            id,
            finalized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// フィールドの値を取得
    pub fn get_field(&self, field_name: &str) -> Option<Box<dyn NyashBox>> {
        self.fields.lock().unwrap().get(field_name).map(|v| v.clone_box())
    }
    
    /// フィールドに値を設定
    pub fn set_field(&self, field_name: &str, value: Box<dyn NyashBox>) -> Result<(), String> {
        let mut fields = self.fields.lock().unwrap();
        if fields.contains_key(field_name) {
            fields.insert(field_name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Field '{}' does not exist in {}", field_name, self.class_name))
        }
    }
    
    /// 🌍 GlobalBox用：フィールドを動的に追加・設定
    pub fn set_field_dynamic(&mut self, field_name: String, value: Box<dyn NyashBox>) {
        let mut fields = self.fields.lock().unwrap();
        fields.insert(field_name, value);
    }
    
    /// メソッド定義を取得
    pub fn get_method(&self, method_name: &str) -> Option<&ASTNode> {
        self.methods.get(method_name)
    }
    
    /// メソッドが存在するかチェック
    pub fn has_method(&self, method_name: &str) -> bool {
        self.methods.contains_key(method_name)
    }
    
    /// 🌍 GlobalBox用：メソッドを動的に追加
    pub fn add_method(&mut self, method_name: String, method_ast: ASTNode) {
        // Arc<T>は不変なので、新しいHashMapを作成してArcで包む
        let mut new_methods = (*self.methods).clone();
        new_methods.insert(method_name, method_ast);
        self.methods = Arc::new(new_methods);
    }
    
    /// fini()メソッド - インスタンスの解放
    pub fn fini(&self) -> Result<(), String> {
        let mut finalized = self.finalized.lock().unwrap();
        if *finalized {
            // 既に解放済みなら何もしない
            return Ok(());
        }
        
        *finalized = true;
        
        // フィールドをクリア
        let mut fields = self.fields.lock().unwrap();
        fields.clear();
        
        Ok(())
    }
    
    /// 解放済みかチェック
    pub fn is_finalized(&self) -> bool {
        *self.finalized.lock().unwrap()
    }
}

impl NyashBox for InstanceBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<{} instance #{}>", self.class_name, self.id))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_instance) = other.as_any().downcast_ref::<InstanceBox>() {
            // 同じインスタンスIDなら等しい
            BoolBox::new(self.id == other_instance.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "InstanceBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // インスタンスは同じフィールドを共有
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for InstanceBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance>", self.class_name)
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::IntegerBox;
    
    #[test]
    fn test_instance_creation() {
        let fields = vec!["x".to_string(), "y".to_string()];
        let methods = HashMap::new();
        let instance = InstanceBox::new("Point".to_string(), fields, methods);
        
        assert_eq!(instance.class_name, "Point");
        assert!(instance.get_field("x").is_some());
        assert!(instance.get_field("y").is_some());
        assert!(instance.get_field("z").is_none());
    }
    
    #[test]
    fn test_field_access() {
        let fields = vec!["value".to_string()];
        let methods = HashMap::new();
        let instance = InstanceBox::new("TestBox".to_string(), fields, methods);
        
        // フィールドに値を設定
        let int_value = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
        instance.set_field("value", int_value).unwrap();
        
        // フィールドの値を取得
        let retrieved = instance.get_field("value").unwrap();
        let int_box = retrieved.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(int_box.value, 42);
    }
    
    #[test]
    fn test_instance_equality() {
        let instance1 = InstanceBox::new("Test".to_string(), vec![], HashMap::new());
        let instance2 = InstanceBox::new("Test".to_string(), vec![], HashMap::new());
        
        // 異なるインスタンスは等しくない
        assert!(!instance1.equals(&instance2).value);
        
        // 同じインスタンスは等しい
        assert!(instance1.equals(&instance1).value);
    }
}